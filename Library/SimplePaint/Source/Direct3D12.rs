// SimplePaint on the GPU, for hosts that draw with Library/Graphics: the
// pipeline of SimplePaint.hlsl's adapter, meshes in GPU memory, and the
// sizes of the constant buffers that the adapter binds. The host compiles
// the adapter with its material count (see Usage.md), owns the constant
// buffers, and keeps them from changing while the GPU reads them.

use graphics::{Renderer, UploadBuffer, bytecode, create_buffer, transition};
use windows_api::*;
use windows_core::{Error, Result, s};

use crate::orthographic::ObjectTransforms;
use crate::{GpuMaterial, Vertex, validate_mesh};

// Where constant buffers may start.
pub const CONSTANT_BUFFER_ALIGNMENT: usize = 256;
// The bytes an object's transforms take at b0, rounded up to a whole
// constant buffer.
pub const OBJECT_SIZE: usize = size_of::<ObjectTransforms>().next_multiple_of(CONSTANT_BUFFER_ALIGNMENT);

// The bytes that `count` materials take at b1. They're packed 80 bytes
// apart, and the buffer is rounded up to a whole constant buffer.
pub const fn materials_size(count: usize) -> usize {
    (count * size_of::<GpuMaterial>()).next_multiple_of(CONSTANT_BUFFER_ALIGNMENT)
}

// What a pipeline draws into, and how.
pub struct Target {
    // The shader outputs linear colors, so this should be an sRGB format,
    // which encodes them once for display.
    pub format: DXGI_FORMAT,
    // With a depth buffer, nearer surfaces hide farther ones.
    pub depth_format: Option<DXGI_FORMAT>,
    pub sample_count: u32,
    // Meshes are wound counterclockwise when seen from their front, the
    // right-handed convention, so D3D12_CULL_MODE_BACK hides their insides.
    pub cull_mode: D3D12_CULL_MODE,
}

pub struct Pipeline {
    root_signature: ID3D12RootSignature,
    state: ID3D12PipelineState,
}

impl Pipeline {
    // From the adapter's two stages, compiled with the same material count.
    // The vertex stage holds the root signature.
    pub unsafe fn new(
        device: &ID3D12Device,
        vertex_shader: &[u8],
        pixel_shader: &[u8],
        target: &Target,
    ) -> Result<Self> {
        unsafe {
            let root_signature: ID3D12RootSignature =
                device.CreateRootSignature(0, vertex_shader)?;
            // Matches Vertex.
            let input_elements = [
                D3D12_INPUT_ELEMENT_DESC {
                    SemanticName: s!("POSITION"),
                    Format: DXGI_FORMAT_R32G32B32_FLOAT,
                    AlignedByteOffset: 0,
                    InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                    ..Default::default()
                },
                D3D12_INPUT_ELEMENT_DESC {
                    SemanticName: s!("NORMAL"),
                    Format: DXGI_FORMAT_R32G32B32_FLOAT,
                    AlignedByteOffset: 12,
                    InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                    ..Default::default()
                },
                D3D12_INPUT_ELEMENT_DESC {
                    SemanticName: s!("MATERIAL"),
                    Format: DXGI_FORMAT_R32_UINT,
                    AlignedByteOffset: 24,
                    InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                    ..Default::default()
                },
            ];
            let mut description = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                // Borrow the root signature without touching its reference count.
                pRootSignature: std::mem::transmute_copy(&root_signature),
                VS: bytecode(vertex_shader),
                PS: bytecode(pixel_shader),
                BlendState: D3D12_BLEND_DESC {
                    RenderTarget: [D3D12_RENDER_TARGET_BLEND_DESC {
                        RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL as u8,
                        ..Default::default()
                    }; 8],
                    ..Default::default()
                },
                SampleMask: u32::MAX,
                RasterizerState: D3D12_RASTERIZER_DESC {
                    FillMode: D3D12_FILL_MODE_SOLID,
                    CullMode: target.cull_mode,
                    FrontCounterClockwise: true.into(),
                    DepthClipEnable: true.into(),
                    MultisampleEnable: (target.sample_count > 1).into(),
                    ..Default::default()
                },
                DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
                    DepthEnable: target.depth_format.is_some().into(),
                    DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
                    DepthFunc: D3D12_COMPARISON_FUNC_LESS,
                    ..Default::default()
                },
                InputLayout: D3D12_INPUT_LAYOUT_DESC {
                    pInputElementDescs: input_elements.as_ptr(),
                    NumElements: input_elements.len() as u32,
                },
                PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                NumRenderTargets: 1,
                DSVFormat: target.depth_format.unwrap_or(DXGI_FORMAT_UNKNOWN),
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: target.sample_count,
                    Quality: 0,
                },
                ..Default::default()
            };
            description.RTVFormats[0] = target.format;
            let state = device.CreateGraphicsPipelineState(&description)?;
            Ok(Self {
                root_signature,
                state,
            })
        }
    }

    // Sets the pipeline, its root signature, and the triangle list topology.
    pub unsafe fn bind(&self, list: &ID3D12GraphicsCommandList) {
        unsafe {
            list.SetPipelineState(&self.state);
            list.SetGraphicsRootSignature(&self.root_signature);
            list.IASetPrimitiveTopology(D3D12_PRIMITIVE_TOPOLOGY(
                D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            ));
        }
    }
}

// An indexed triangle mesh in the GPU's own memory. Meshes never change, so
// it's copied there once, instead of the GPU reading it from an upload heap
// in system memory, across PCIe, every frame.
pub struct Mesh {
    _buffer: ID3D12Resource,
    vertex_view: D3D12_VERTEX_BUFFER_VIEW,
    index_view: D3D12_INDEX_BUFFER_VIEW,
    pub index_count: u32,
}

impl Mesh {
    // Validates the mesh with validate_mesh for `material_count` materials
    // first, and fails with its reason. The copy waits for the GPU, so it's
    // for setup, not for frames.
    pub unsafe fn new(
        renderer: &mut Renderer,
        vertices: &[Vertex],
        indices: &[u32],
        material_count: u32,
    ) -> Result<Self> {
        validate_mesh(vertices, indices, material_count)
            .map_err(|error| Error::new(E_INVALIDARG, error.0))?;
        unsafe {
            let device = renderer.device.clone();
            let vertex_bytes = size_of_val(vertices);
            let index_bytes = size_of_val(indices);
            let size = vertex_bytes + index_bytes;
            let upload = UploadBuffer::new(&device, size)?;
            upload.write(0, vertices);
            upload.write(vertex_bytes, indices);
            // Buffers are created in the common state, and the copy promotes
            // it to COPY_DEST.
            let buffer = create_buffer(
                &device,
                size as u64,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_COMMON,
            )?;
            // The upload buffer must outlive the copy, which run waits for.
            renderer.run(|list| {
                list.CopyBufferRegion(&buffer, 0, &upload.resource, 0, size as u64);
                list.ResourceBarrier(&[transition(
                    &buffer,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER
                        | D3D12_RESOURCE_STATE_INDEX_BUFFER,
                )]);
            })?;
            let start = buffer.GetGPUVirtualAddress().0;
            Ok(Self {
                vertex_view: D3D12_VERTEX_BUFFER_VIEW {
                    BufferLocation: D3D12_GPU_VIRTUAL_ADDRESS(start),
                    SizeInBytes: vertex_bytes as u32,
                    StrideInBytes: size_of::<Vertex>() as u32,
                },
                index_view: D3D12_INDEX_BUFFER_VIEW {
                    BufferLocation: D3D12_GPU_VIRTUAL_ADDRESS(start + vertex_bytes as u64),
                    SizeInBytes: index_bytes as u32,
                    Format: DXGI_FORMAT_R32_UINT,
                },
                index_count: indices.len() as u32,
                _buffer: buffer,
            })
        }
    }

    // Draws the mesh with the pipeline that's bound, with its object
    // transforms and its materials at these addresses, each at the start of
    // a constant buffer.
    pub unsafe fn draw(
        &self,
        list: &ID3D12GraphicsCommandList,
        object: D3D12_GPU_VIRTUAL_ADDRESS,
        materials: D3D12_GPU_VIRTUAL_ADDRESS,
    ) {
        unsafe {
            list.SetGraphicsRootConstantBufferView(0, object);
            list.SetGraphicsRootConstantBufferView(1, materials);
            list.IASetVertexBuffers(0, Some(&[self.vertex_view]));
            list.IASetIndexBuffer(Some(&self.index_view));
            list.DrawIndexedInstanced(self.index_count, 1, 0, 0, 0);
        }
    }
}
