// The Direct3D 12 renderer. Its setup follows HelloDirect3D12, and adds what
// a 2D overlay needs: a vertex buffer that the CPU refills every frame, the
// font atlas texture, and alpha blending.

use std::ffi::c_void;
use std::mem::ManuallyDrop;

use crate::bindings::*;
use crate::canvas::Vertex;
use crate::font::Font;
use windows_core::{Error, Interface, PCWSTR, Result, s};

const FRAME_COUNT: usize = 2;
const BACK_BUFFER_FORMAT: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_UNORM;
// Per frame in flight. Vertices past this are dropped.
const MAX_VERTICES: usize = 256 * 1024;

static VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Vertex.cso"));
static PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Pixel.cso"));

// Win32 functions report failure with a null handle and set the thread's last error.
pub fn check_handle(handle: *mut c_void) -> Result<()> {
    if handle.is_null() {
        Err(Error::from_thread())
    } else {
        Ok(())
    }
}

// An upload heap buffer that stays mapped, so the CPU can write it directly.
struct VertexBuffer {
    resource: ID3D12Resource,
    mapped: *mut Vertex,
}

pub struct Renderer {
    device: ID3D12Device,
    queue: ID3D12CommandQueue,
    swap_chain: IDXGISwapChain3,
    rtv_heap: ID3D12DescriptorHeap,
    rtv_increment: usize,
    // Shader-visible, holding the font atlas's shader resource view.
    srv_heap: ID3D12DescriptorHeap,
    back_buffers: Vec<ID3D12Resource>,
    allocators: Vec<ID3D12CommandAllocator>,
    vertex_buffers: Vec<VertexBuffer>,
    command_list: ID3D12GraphicsCommandList,
    root_signature: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
    atlas: Option<ID3D12Resource>,
    fence: ID3D12Fence,
    fence_event: HANDLE,
    // Fence value that marks when each back buffer's commands are finished.
    frame_fence_values: [u64; FRAME_COUNT],
    last_fence_value: u64,
    pub width: u32,
    pub height: u32,
}

impl Renderer {
    pub unsafe fn new(window: HWND, font: &Font) -> Result<Self> {
        unsafe {
            // The debug layer needs the "Graphics Tools" optional Windows feature.
            let mut factory_flags = 0;
            if cfg!(debug_assertions) {
                let mut debug: Option<ID3D12Debug> = None;
                if D3D12GetDebugInterface(&mut debug).is_ok() {
                    debug.unwrap().EnableDebugLayer();
                    factory_flags = DXGI_CREATE_FACTORY_DEBUG as u32;
                }
            }

            let factory: IDXGIFactory6 = CreateDXGIFactory2(factory_flags)?;
            let adapter: IDXGIAdapter1 =
                factory.EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE)?;

            let mut device: Option<ID3D12Device> = None;
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_0, &mut device)?;
            let device = device.unwrap();

            let queue: ID3D12CommandQueue =
                device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                    Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                    ..Default::default()
                })?;

            // Width and height 0 means "use the window's client size".
            let swap_chain_description = DXGI_SWAP_CHAIN_DESC1 {
                Format: BACK_BUFFER_FORMAT,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE(DXGI_USAGE_RENDER_TARGET_OUTPUT),
                BufferCount: FRAME_COUNT as u32,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                ..Default::default()
            };
            let swap_chain: IDXGISwapChain3 = factory
                .CreateSwapChainForHwnd(&queue, window, &swap_chain_description, None, None)?
                .cast()?;
            factory
                .MakeWindowAssociation(window, DXGI_MWA_NO_ALT_ENTER as u32)
                .ok()?;
            let mut actual = DXGI_SWAP_CHAIN_DESC1::default();
            swap_chain.GetDesc1(&mut actual).ok()?;

            let rtv_heap: ID3D12DescriptorHeap =
                device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                    NumDescriptors: FRAME_COUNT as u32,
                    ..Default::default()
                })?;
            let rtv_increment =
                device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) as usize;
            let srv_heap: ID3D12DescriptorHeap =
                device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                    NumDescriptors: 1,
                    Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                    ..Default::default()
                })?;

            let allocators = (0..FRAME_COUNT)
                .map(|_| device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT))
                .collect::<Result<Vec<ID3D12CommandAllocator>>>()?;

            let vertex_buffers = (0..FRAME_COUNT)
                .map(|_| {
                    let resource =
                        create_upload_buffer(&device, (MAX_VERTICES * size_of::<Vertex>()) as u64)?;
                    let mut mapped = std::ptr::null_mut();
                    // An empty read range: the CPU never reads it back.
                    resource
                        .Map(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }), &mut mapped)
                        .ok()?;
                    Ok(VertexBuffer {
                        resource,
                        mapped: mapped.cast(),
                    })
                })
                .collect::<Result<Vec<VertexBuffer>>>()?;

            let root_signature: ID3D12RootSignature =
                device.CreateRootSignature(0, VERTEX_SHADER)?;
            let pipeline = create_pipeline(&device, &root_signature)?;

            let command_list: ID3D12GraphicsCommandList = device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &allocators[0],
                &pipeline,
            )?;
            command_list.Close().ok()?;

            let fence: ID3D12Fence = device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
            let fence_event = CreateEventW(None, false, false, PCWSTR::null());
            check_handle(fence_event.0)?;

            let mut renderer = Self {
                device,
                queue,
                swap_chain,
                rtv_heap,
                rtv_increment,
                srv_heap,
                back_buffers: Vec::new(),
                allocators,
                vertex_buffers,
                command_list,
                root_signature,
                pipeline,
                atlas: None,
                fence,
                fence_event,
                frame_fence_values: [0; FRAME_COUNT],
                last_fence_value: 0,
                width: actual.Width,
                height: actual.Height,
            };
            renderer.create_render_target_views()?;
            renderer.set_font(font)?;
            Ok(renderer)
        }
    }

    unsafe fn create_render_target_views(&mut self) -> Result<()> {
        unsafe {
            for index in 0..FRAME_COUNT {
                let buffer: ID3D12Resource = self.swap_chain.GetBuffer(index as u32)?;
                self.device
                    .CreateRenderTargetView(&buffer, None, self.rtv_handle(index));
                self.back_buffers.push(buffer);
            }
            Ok(())
        }
    }

    unsafe fn rtv_handle(&self, index: usize) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        let start = unsafe { self.rtv_heap.GetCPUDescriptorHandleForHeapStart() };
        D3D12_CPU_DESCRIPTOR_HANDLE {
            ptr: start.ptr + index * self.rtv_increment,
        }
    }

    unsafe fn wait_for_fence(&self, value: u64) -> Result<()> {
        unsafe {
            if self.fence.GetCompletedValue() < value {
                self.fence
                    .SetEventOnCompletion(value, self.fence_event)
                    .ok()?;
                WaitForSingleObject(self.fence_event, INFINITE);
            }
            Ok(())
        }
    }

    unsafe fn wait_for_idle(&mut self) -> Result<()> {
        unsafe {
            self.last_fence_value += 1;
            self.queue.Signal(&self.fence, self.last_fence_value).ok()?;
            self.wait_for_fence(self.last_fence_value)
        }
    }

    pub unsafe fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        unsafe {
            if width == 0 || height == 0 || (width == self.width && height == self.height) {
                return Ok(());
            }
            // All references to the old back buffers must be gone before ResizeBuffers.
            self.wait_for_idle()?;
            self.back_buffers.clear();
            self.swap_chain
                .ResizeBuffers(FRAME_COUNT as u32, width, height, DXGI_FORMAT_UNKNOWN, 0)
                .ok()?;
            self.width = width;
            self.height = height;
            self.create_render_target_views()
        }
    }

    // Uploads the font's atlas into a new texture and points the shader at it.
    pub unsafe fn set_font(&mut self, font: &Font) -> Result<()> {
        unsafe {
            // The GPU might still be drawing with the old atlas.
            self.wait_for_idle()?;

            let description = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Width: font.atlas_width as u64,
                Height: font.atlas_height,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_R8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                ..Default::default()
            };
            let mut atlas: Option<ID3D12Resource> = None;
            self.device.CreateCommittedResource(
                &D3D12_HEAP_PROPERTIES {
                    Type: D3D12_HEAP_TYPE_DEFAULT,
                    ..Default::default()
                },
                D3D12_HEAP_FLAG_NONE,
                &description,
                D3D12_RESOURCE_STATE_COPY_DEST,
                None,
                &mut atlas,
            )?;
            let atlas = atlas.unwrap();

            // Rows in an upload buffer must start at 256-byte boundaries, so
            // ask D3D12 for the layout instead of assuming a tight one.
            let mut footprint = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
            let mut total_bytes = 0u64;
            self.device.GetCopyableFootprints(
                &description,
                0,
                1,
                0,
                Some(&mut footprint),
                None,
                None,
                Some(&mut total_bytes),
            );
            let upload = create_upload_buffer(&self.device, total_bytes)?;
            let mut mapped = std::ptr::null_mut();
            upload.Map(0, None, &mut mapped).ok()?;
            let row_pitch = footprint.Footprint.RowPitch as usize;
            let width = font.atlas_width as usize;
            for row in 0..font.atlas_height as usize {
                std::ptr::copy_nonoverlapping(
                    font.pixels[row * width..].as_ptr(),
                    (mapped as *mut u8).add(footprint.Offset as usize + row * row_pitch),
                    width,
                );
            }
            upload.Unmap(0, None);

            let allocator = &self.allocators[0];
            let list = &self.command_list;
            allocator.Reset().ok()?;
            list.Reset(allocator, None).ok()?;
            let destination = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&atlas),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: 0,
                },
            };
            let source = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&upload),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    PlacedFootprint: footprint,
                },
            };
            list.CopyTextureRegion(&destination, 0, 0, 0, &source, None);
            list.ResourceBarrier(&[transition(
                &atlas,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
            )]);
            list.Close().ok()?;
            self.queue.ExecuteCommandLists(&[Some(list.cast()?)]);
            // The upload buffer must outlive the copy.
            self.wait_for_idle()?;

            let mut view = D3D12_SHADER_RESOURCE_VIEW_DESC {
                Format: DXGI_FORMAT_R8_UNORM,
                ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
                Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING as u32,
                ..Default::default()
            };
            view.Anonymous.Texture2D.MipLevels = 1;
            self.device.CreateShaderResourceView(
                &atlas,
                Some(&view),
                self.srv_heap.GetCPUDescriptorHandleForHeapStart(),
            );
            self.atlas = Some(atlas);
            Ok(())
        }
    }

    pub unsafe fn render(&mut self, vertices: &[Vertex], clear_color: [f32; 4]) -> Result<()> {
        unsafe {
            let index = self.swap_chain.GetCurrentBackBufferIndex() as usize;

            // The GPU might still be using this frame's allocator and vertex buffer.
            self.wait_for_fence(self.frame_fence_values[index])?;

            let vertex_count = vertices.len().min(MAX_VERTICES);
            let vertex_buffer = &self.vertex_buffers[index];
            std::ptr::copy_nonoverlapping(vertices.as_ptr(), vertex_buffer.mapped, vertex_count);

            let allocator = &self.allocators[index];
            let list = &self.command_list;
            allocator.Reset().ok()?;
            list.Reset(allocator, &self.pipeline).ok()?;

            list.SetGraphicsRootSignature(&self.root_signature);
            list.SetDescriptorHeaps(&[Some(self.srv_heap.clone())]);
            let pixel_to_clip = [2.0 / self.width as f32, 2.0 / self.height as f32];
            list.SetGraphicsRoot32BitConstants(0, 2, pixel_to_clip.as_ptr().cast(), 0);
            list.SetGraphicsRootDescriptorTable(
                1,
                self.srv_heap.GetGPUDescriptorHandleForHeapStart(),
            );
            list.RSSetViewports(&[D3D12_VIEWPORT {
                Width: self.width as f32,
                Height: self.height as f32,
                MaxDepth: 1.0,
                ..Default::default()
            }]);
            list.RSSetScissorRects(&[D3D12_RECT {
                left: 0,
                top: 0,
                right: self.width as i32,
                bottom: self.height as i32,
            }]);

            let back_buffer = &self.back_buffers[index];
            list.ResourceBarrier(&[transition(
                back_buffer,
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )]);

            let rtv = self.rtv_handle(index);
            list.OMSetRenderTargets(1, Some(&raw const rtv), false, None);
            list.ClearRenderTargetView(rtv, &clear_color, &[]);
            list.IASetPrimitiveTopology(D3D12_PRIMITIVE_TOPOLOGY(
                D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            ));
            list.IASetVertexBuffers(
                0,
                Some(&[D3D12_VERTEX_BUFFER_VIEW {
                    BufferLocation: vertex_buffer.resource.GetGPUVirtualAddress(),
                    SizeInBytes: (vertex_count * size_of::<Vertex>()) as u32,
                    StrideInBytes: size_of::<Vertex>() as u32,
                }]),
            );
            list.DrawInstanced(vertex_count as u32, 1, 0, 0);

            list.ResourceBarrier(&[transition(
                back_buffer,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_PRESENT,
            )]);
            list.Close().ok()?;

            self.queue.ExecuteCommandLists(&[Some(list.cast()?)]);
            // Sync interval 1 = vsync. Present blocks when the queue of frames is full.
            self.swap_chain.Present(1, 0).ok()?;

            self.last_fence_value += 1;
            self.queue.Signal(&self.fence, self.last_fence_value).ok()?;
            self.frame_fence_values[index] = self.last_fence_value;
            Ok(())
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.wait_for_idle();
            let _ = CloseHandle(self.fence_event);
        }
    }
}

unsafe fn create_upload_buffer(device: &ID3D12Device, size: u64) -> Result<ID3D12Resource> {
    let mut buffer: Option<ID3D12Resource> = None;
    unsafe {
        device.CreateCommittedResource(
            &D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_UPLOAD,
                ..Default::default()
            },
            D3D12_HEAP_FLAG_NONE,
            &D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Width: size,
                Height: 1,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_UNKNOWN,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                ..Default::default()
            },
            D3D12_RESOURCE_STATE_GENERIC_READ,
            None,
            &mut buffer,
        )?;
    }
    Ok(buffer.unwrap())
}

unsafe fn create_pipeline(
    device: &ID3D12Device,
    root_signature: &ID3D12RootSignature,
) -> Result<ID3D12PipelineState> {
    // Matches canvas::Vertex.
    let input_elements = [
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: s!("POSITION"),
            Format: DXGI_FORMAT_R32G32_FLOAT,
            AlignedByteOffset: 0,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            ..Default::default()
        },
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: s!("TEXCOORD"),
            Format: DXGI_FORMAT_R32G32_FLOAT,
            AlignedByteOffset: 8,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            ..Default::default()
        },
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: s!("COLOR"),
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            AlignedByteOffset: 16,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            ..Default::default()
        },
    ];
    let mut description = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
        // Borrow the root signature without touching its reference count.
        pRootSignature: unsafe { std::mem::transmute_copy(root_signature) },
        VS: bytecode(VERTEX_SHADER),
        PS: bytecode(PIXEL_SHADER),
        // Standard "over" blending, for antialiased edges and text.
        BlendState: D3D12_BLEND_DESC {
            RenderTarget: [D3D12_RENDER_TARGET_BLEND_DESC {
                BlendEnable: true.into(),
                SrcBlend: D3D12_BLEND_SRC_ALPHA,
                DestBlend: D3D12_BLEND_INV_SRC_ALPHA,
                BlendOp: D3D12_BLEND_OP_ADD,
                SrcBlendAlpha: D3D12_BLEND_ONE,
                DestBlendAlpha: D3D12_BLEND_INV_SRC_ALPHA,
                BlendOpAlpha: D3D12_BLEND_OP_ADD,
                RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL as u8,
                ..Default::default()
            }; 8],
            ..Default::default()
        },
        SampleMask: u32::MAX,
        RasterizerState: D3D12_RASTERIZER_DESC {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_NONE,
            DepthClipEnable: true.into(),
            ..Default::default()
        },
        InputLayout: D3D12_INPUT_LAYOUT_DESC {
            pInputElementDescs: input_elements.as_ptr(),
            NumElements: input_elements.len() as u32,
        },
        PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        NumRenderTargets: 1,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        ..Default::default()
    };
    description.RTVFormats[0] = BACK_BUFFER_FORMAT;
    unsafe { device.CreateGraphicsPipelineState(&description) }
}

fn bytecode(bytes: &[u8]) -> D3D12_SHADER_BYTECODE {
    D3D12_SHADER_BYTECODE {
        pShaderBytecode: bytes.as_ptr().cast(),
        BytecodeLength: bytes.len(),
    }
}

fn transition(
    resource: &ID3D12Resource,
    before: D3D12_RESOURCE_STATES,
    after: D3D12_RESOURCE_STATES,
) -> D3D12_RESOURCE_BARRIER {
    D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                pResource: unsafe { std::mem::transmute_copy(resource) },
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: before,
                StateAfter: after,
            }),
        },
    }
}
