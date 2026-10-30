// The Direct3D 12 renderer. Its setup follows HelloDirect3D12, and adds what
// a 2D overlay needs: a vertex buffer that the CPU refills every frame, the
// font atlas texture, and alpha blending.
//
// It's set up so that input reaches the screen as soon as possible: a frame
// only reads input once the previous one has left the GPU and DXGI's queue,
// so no finished frame waits behind another, and frames are presented
// without waiting for vertical sync.

use std::ffi::c_void;
use std::mem::ManuallyDrop;

use crate::bindings::*;
use crate::canvas::Vertex;
use crate::font::Font;
use windows_core::{Error, Interface, PCWSTR, Result, s};

// The fewest a flip model swap chain allows: one on screen, one being drawn.
const BUFFER_COUNT: u32 = 2;
const BACK_BUFFER_FORMAT: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_UNORM;
// Vertices past this are dropped.
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
    // One frame is in flight at a time, so one allocator and one vertex
    // buffer are enough.
    allocator: ID3D12CommandAllocator,
    vertex_buffer: VertexBuffer,
    command_list: ID3D12GraphicsCommandList,
    root_signature: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
    atlas: Option<ID3D12Resource>,
    fence: ID3D12Fence,
    fence_event: HANDLE,
    // Marks when the last submitted commands are finished.
    last_fence_value: u64,
    pub adapter_name: String,
    // Whether the adapter is the one the window's monitor is connected to.
    pub adapter_drives_monitor: bool,
    // Whether frames may tear, which lets them reach the screen mid-refresh.
    pub tearing: bool,
    // ResizeBuffers must be given the flags the swap chain was created with.
    swap_chain_flags: u32,
    // Signaled when DXGI's queue of presented frames has room for another.
    frame_latency_event: HANDLE,
    // Whether frame_latency_event has been waited on since the last Present.
    queue_has_room: bool,
    // The back buffer that is_ready started recording the next frame for.
    recording: Option<usize>,
    pub width: u32,
    pub height: u32,
}

impl Renderer {
    // Frames may tear if `allow_tearing` and the GPU's driver allow it.
    pub unsafe fn new(window: HWND, font: &Font, allow_tearing: bool) -> Result<Self> {
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
            // Draw with the GPU that the window's monitor is connected to.
            // Drawing with another, like the discrete GPU of a laptop whose
            // screen is wired to the integrated one, adds a copy between GPUs
            // to every frame, and keeps Windows from flipping frames straight
            // to the screen. Any GPU is fast enough for this UI. If none
            // reports the monitor, like over Remote Desktop, use the
            // high-performance one. The choice isn't revisited if the window
            // moves to a monitor on another GPU.
            let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST as u32);
            let mut adapter: IDXGIAdapter1 =
                factory.EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE)?;
            let mut adapter_drives_monitor = false;
            for index in 0.. {
                let Ok(candidate) = factory.EnumAdapterByGpuPreference::<IDXGIAdapter1>(
                    index,
                    DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
                ) else {
                    break;
                };
                if drives_monitor(&candidate, monitor) {
                    adapter = candidate;
                    adapter_drives_monitor = true;
                    break;
                }
            }
            let mut adapter_description = DXGI_ADAPTER_DESC1::default();
            adapter.GetDesc1(&mut adapter_description).ok()?;
            let name = &adapter_description.Description;
            let length = name
                .iter()
                .position(|&unit| unit == 0)
                .unwrap_or(name.len());
            let adapter_name = String::from_utf16_lossy(&name[..length]);

            let mut device: Option<ID3D12Device> = None;
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_0, &mut device)?;
            let device = device.unwrap();

            let queue: ID3D12CommandQueue =
                device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                    Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                    ..Default::default()
                })?;

            // Tearing needs Windows 10 1809 or later, and a driver that allows it.
            let mut supported = windows_core::BOOL(0);
            let tearing = allow_tearing
                && factory
                    .cast::<IDXGIFactory5>()?
                    .CheckFeatureSupport(
                        DXGI_FEATURE_PRESENT_ALLOW_TEARING,
                        (&raw mut supported).cast(),
                        size_of::<windows_core::BOOL>() as u32,
                    )
                    .is_ok()
                && supported.as_bool();
            let mut swap_chain_flags = DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT as u32;
            if tearing {
                swap_chain_flags |= DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING as u32;
            }

            // Width and height 0 means "use the window's client size".
            let swap_chain_description = DXGI_SWAP_CHAIN_DESC1 {
                Format: BACK_BUFFER_FORMAT,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE(DXGI_USAGE_RENDER_TARGET_OUTPUT),
                BufferCount: BUFFER_COUNT,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                Flags: swap_chain_flags,
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
            // DXGI otherwise lets 3 presented frames queue up before Present
            // blocks, and each one queued delays the newest by a refresh.
            swap_chain.SetMaximumFrameLatency(1).ok()?;
            let frame_latency_event = swap_chain.GetFrameLatencyWaitableObject();
            check_handle(frame_latency_event.0)?;

            let rtv_heap: ID3D12DescriptorHeap =
                device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                    NumDescriptors: BUFFER_COUNT,
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

            let allocator: ID3D12CommandAllocator =
                device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;

            let resource =
                create_upload_buffer(&device, (MAX_VERTICES * size_of::<Vertex>()) as u64)?;
            let mut mapped = std::ptr::null_mut();
            // An empty read range: the CPU never reads it back.
            resource
                .Map(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }), &mut mapped)
                .ok()?;
            let vertex_buffer = VertexBuffer {
                resource,
                mapped: mapped.cast(),
            };

            let root_signature: ID3D12RootSignature =
                device.CreateRootSignature(0, VERTEX_SHADER)?;
            let pipeline = create_pipeline(&device, &root_signature)?;

            let command_list: ID3D12GraphicsCommandList = device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &allocator,
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
                allocator,
                vertex_buffer,
                command_list,
                root_signature,
                pipeline,
                atlas: None,
                fence,
                fence_event,
                last_fence_value: 0,
                adapter_name,
                adapter_drives_monitor,
                tearing,
                swap_chain_flags,
                frame_latency_event,
                queue_has_room: false,
                recording: None,
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
            for index in 0..BUFFER_COUNT as usize {
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

    // Drops the frame that is_ready started recording, if any, so that the
    // command allocator can be reset and the back buffers released.
    unsafe fn discard_frame(&mut self) -> Result<()> {
        if self.recording.take().is_some() {
            unsafe { self.command_list.Close() }.ok()?;
        }
        Ok(())
    }

    pub unsafe fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        unsafe {
            if width == 0 || height == 0 || (width == self.width && height == self.height) {
                return Ok(());
            }
            // All references to the old back buffers must be gone before ResizeBuffers.
            self.discard_frame()?;
            self.wait_for_idle()?;
            self.back_buffers.clear();
            self.swap_chain
                .ResizeBuffers(
                    BUFFER_COUNT,
                    width,
                    height,
                    DXGI_FORMAT_UNKNOWN,
                    self.swap_chain_flags,
                )
                .ok()?;
            self.width = width;
            self.height = height;
            self.create_render_target_views()
        }
    }

    // Uploads the font's atlas into a new texture and points the shader at it.
    unsafe fn set_font(&mut self, font: &Font) -> Result<()> {
        unsafe {
            // The GPU might still be drawing with the old atlas.
            self.discard_frame()?;
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

            let list = &self.command_list;
            self.allocator.Reset().ok()?;
            list.Reset(&self.allocator, None).ok()?;
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

    // Whether a frame drawn now would reach the screen without waiting behind
    // an older one: the GPU has finished the previous frame, and DXGI's queue
    // has room. It doesn't block, so the caller can keep reading the newest
    // input until it's true, then draw with it.
    //
    // The GPU usually finishes tens of microseconds before DXGI's queue has
    // room. In between, this records the commands that don't depend on input,
    // so they aren't left for render, between reading input and presenting.
    pub unsafe fn is_ready(&mut self) -> Result<bool> {
        unsafe {
            if self.recording.is_none() {
                if self.fence.GetCompletedValue() < self.last_fence_value {
                    return Ok(false);
                }
                self.begin_frame()?;
            }
            if !self.queue_has_room {
                match WaitForSingleObject(self.frame_latency_event, 0) as i32 {
                    WAIT_OBJECT_0 => self.queue_has_room = true,
                    WAIT_TIMEOUT => {}
                    _ => return Err(Error::from_thread()),
                }
            }
            Ok(self.queue_has_room)
        }
    }

    // Records the start of a frame, up to where it needs the frame's
    // vertices. The GPU must have finished with the command allocator.
    unsafe fn begin_frame(&mut self) -> Result<()> {
        unsafe {
            let index = self.swap_chain.GetCurrentBackBufferIndex() as usize;
            let list = &self.command_list;
            self.allocator.Reset().ok()?;
            list.Reset(&self.allocator, &self.pipeline).ok()?;

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

            list.ResourceBarrier(&[transition(
                &self.back_buffers[index],
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )]);

            let rtv = self.rtv_handle(index);
            list.OMSetRenderTargets(1, Some(&raw const rtv), false, None);
            list.IASetPrimitiveTopology(D3D12_PRIMITIVE_TOPOLOGY(
                D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            ));
            self.recording = Some(index);
            Ok(())
        }
    }

    // Finishes the frame that is_ready started, and presents it. Call it once
    // is_ready is true.
    pub unsafe fn render(&mut self, vertices: &[Vertex], clear_color: [f32; 4]) -> Result<()> {
        unsafe {
            let index = self
                .recording
                .take()
                .expect("render is called once is_ready is true");

            // The GPU has finished the previous frame, so it's done reading
            // the vertex buffer.
            let vertex_count = vertices.len().min(MAX_VERTICES);
            let vertex_buffer = &self.vertex_buffer;
            std::ptr::copy_nonoverlapping(vertices.as_ptr(), vertex_buffer.mapped, vertex_count);

            let list = &self.command_list;
            list.ClearRenderTargetView(self.rtv_handle(index), &clear_color, &[]);
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
                &self.back_buffers[index],
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_PRESENT,
            )]);
            list.Close().ok()?;

            self.queue.ExecuteCommandLists(&[Some(list.cast()?)]);
            // Sync interval 0: don't wait for vertical sync. With tearing
            // allowed, a frame can replace the one being scanned out mid-refresh
            // when Windows gives the window its own flips, like in fullscreen.
            // Otherwise the desktop compositor shows the newest frame at its
            // next refresh, and drops the ones it didn't get to.
            let flags = if self.tearing {
                DXGI_PRESENT_ALLOW_TEARING
            } else {
                0
            };
            self.swap_chain.Present(0, flags).ok()?;
            self.queue_has_room = false;

            self.last_fence_value += 1;
            self.queue.Signal(&self.fence, self.last_fence_value).ok()?;
            Ok(())
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.wait_for_idle();
            let _ = CloseHandle(self.fence_event);
            let _ = CloseHandle(self.frame_latency_event);
        }
    }
}

// Whether one of the adapter's outputs is this monitor.
unsafe fn drives_monitor(adapter: &IDXGIAdapter1, monitor: HMONITOR) -> bool {
    (0..)
        .map_while(|index| {
            let mut output = None;
            unsafe { adapter.EnumOutputs(index, &mut output) }
                .ok()
                .ok()?;
            output
        })
        .any(|output| {
            let mut description = DXGI_OUTPUT_DESC::default();
            unsafe { output.GetDesc(&mut description) }.is_ok() && description.Monitor == monitor
        })
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
