// Draws a Canvas's triangles into a frame with Direct3D 12: a vertex buffer
// that the CPU refills every frame, the font atlas texture, and alpha
// blending, in one draw call.

use graphics::{Renderer, UploadBuffer, bytecode, upload_texture};
use windows_api::*;
use windows_core::{Result, s};

use crate::canvas::Vertex;
use crate::font::Font;

// Vertices past this are dropped.
const MAX_VERTICES: usize = 256 * 1024;

static VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/CanvasVertex.cso"));
static PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/CanvasPixel.cso"));

pub struct Pipeline {
    root_signature: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
    // Shader-visible, holding the font atlas's shader resource view.
    srv_heap: ID3D12DescriptorHeap,
    // Kept alive for the view in srv_heap.
    _atlas: ID3D12Resource,
    // One frame is in flight at a time, so one vertex buffer is enough.
    vertices: UploadBuffer,
}

impl Pipeline {
    // Uploads the font's atlas into a texture for the shader.
    pub unsafe fn new(renderer: &mut Renderer, font: &Font) -> Result<Self> {
        unsafe {
            let device = renderer.device.clone();
            let root_signature: ID3D12RootSignature =
                device.CreateRootSignature(0, VERTEX_SHADER)?;
            let pipeline = create_pipeline(&device, &root_signature)?;
            let vertices = UploadBuffer::new(&device, MAX_VERTICES * size_of::<Vertex>())?;

            let atlas = upload_texture(
                renderer,
                font.atlas_width,
                font.atlas_height,
                DXGI_FORMAT_R8_UNORM,
                1,
                &font.pixels,
                D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
            )?;
            let srv_heap: ID3D12DescriptorHeap =
                device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                    NumDescriptors: 1,
                    Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                    ..Default::default()
                })?;
            let mut view = D3D12_SHADER_RESOURCE_VIEW_DESC {
                Format: DXGI_FORMAT_R8_UNORM,
                ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
                Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING as u32,
                ..Default::default()
            };
            view.Anonymous.Texture2D.MipLevels = 1;
            device.CreateShaderResourceView(
                &atlas,
                Some(&view),
                srv_heap.GetCPUDescriptorHandleForHeapStart(),
            );

            Ok(Self {
                root_signature,
                pipeline,
                srv_heap,
                _atlas: atlas,
                vertices,
            })
        }
    }

    // Draws the triangles over the frame that the renderer is recording,
    // with the back buffer viewed as it's stored, as the canvas's colors are
    // sRGB already. Call it between is_ready and present.
    pub unsafe fn draw(&self, renderer: &Renderer, vertices: &[Vertex]) {
        unsafe {
            // The GPU has finished the previous frame, so it's done reading
            // the vertex buffer.
            let vertex_count = vertices.len().min(MAX_VERTICES);
            self.vertices.write(0, &vertices[..vertex_count]);

            renderer.set_render_target(false, false);
            let list = &renderer.command_list;
            list.SetPipelineState(&self.pipeline);
            list.SetGraphicsRootSignature(&self.root_signature);
            list.SetDescriptorHeaps(&[Some(self.srv_heap.clone())]);
            let pixel_to_clip = [2.0 / renderer.width as f32, 2.0 / renderer.height as f32];
            list.SetGraphicsRoot32BitConstants(0, 2, pixel_to_clip.as_ptr().cast(), 0);
            list.SetGraphicsRootDescriptorTable(
                1,
                self.srv_heap.GetGPUDescriptorHandleForHeapStart(),
            );
            list.IASetPrimitiveTopology(D3D12_PRIMITIVE_TOPOLOGY(
                D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            ));
            list.IASetVertexBuffers(
                0,
                Some(&[D3D12_VERTEX_BUFFER_VIEW {
                    BufferLocation: self.vertices.address(0),
                    SizeInBytes: (vertex_count * size_of::<Vertex>()) as u32,
                    StrideInBytes: size_of::<Vertex>() as u32,
                }]),
            );
            list.DrawInstanced(vertex_count as u32, 1, 0, 0);
        }
    }
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
    description.RTVFormats[0] = graphics::BACK_BUFFER_FORMAT;
    unsafe { device.CreateGraphicsPipelineState(&description) }
}
