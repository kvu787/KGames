// Draws a scene with Direct3D 12 in one draw call: the font's glyphs are in a
// texture of one row, which goes to the GPU once, the frame's quads go into an
// upload buffer, and one instanced draw of a 4-vertex strip draws them all.
// No sampler reads the texture: the pixel shader loads each glyph's pixels by
// their integer coordinates, so nothing is filtered.

use graphics::{DescriptorHeap, Frame, Gpu, TARGET_FORMAT, UploadBuffer, bytecode, create_texture, transition};
use windows_api_raw::*;
use windows_core::Result;

use crate::font::Font;
use crate::scene::Scene;

const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/PixelFontVertex.cso"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/PixelFontPixel.cso"));

pub struct Renderer {
    root_signature: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
    // Kept for the descriptor of it, which the heap has.
    _atlas: ID3D12Resource,
    descriptors: DescriptorHeap,
    glyph_width: u32,
    upload: Option<UploadBuffer>,
}

impl Renderer {
    // Makes the pipeline, and copies the font's glyphs to the GPU, and waits
    // for the copy.
    pub fn new(gpu: &mut Gpu, font: &Font) -> Result<Self> {
        unsafe {
            let device = gpu.device.clone();
            let root_signature: ID3D12RootSignature = device.CreateRootSignature(0, VERTEX_SHADER)?;
            // Premultiplied alpha.
            let blend = D3D12_RENDER_TARGET_BLEND_DESC {
                BlendEnable: true.into(),
                SrcBlend: D3D12_BLEND_ONE,
                DestBlend: D3D12_BLEND_INV_SRC_ALPHA,
                BlendOp: D3D12_BLEND_OP_ADD,
                SrcBlendAlpha: D3D12_BLEND_ONE,
                DestBlendAlpha: D3D12_BLEND_INV_SRC_ALPHA,
                BlendOpAlpha: D3D12_BLEND_OP_ADD,
                RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL as u8,
                ..Default::default()
            };
            let mut description = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                pRootSignature: std::mem::transmute_copy(&root_signature),
                VS: bytecode(VERTEX_SHADER),
                PS: bytecode(PIXEL_SHADER),
                BlendState: D3D12_BLEND_DESC {
                    RenderTarget: [blend; 8],
                    ..Default::default()
                },
                SampleMask: u32::MAX,
                RasterizerState: D3D12_RASTERIZER_DESC {
                    FillMode: D3D12_FILL_MODE_SOLID,
                    CullMode: D3D12_CULL_MODE_NONE,
                    DepthClipEnable: true.into(),
                    ..Default::default()
                },
                PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                NumRenderTargets: 1,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                ..Default::default()
            };
            description.RTVFormats[0] = TARGET_FORMAT;
            let pipeline: ID3D12PipelineState = device.CreateGraphicsPipelineState(&description)?;

            let (width, height, pixels) = font.atlas();
            let atlas = create_texture(
                &device,
                width,
                height,
                DXGI_FORMAT_R8_UNORM,
                D3D12_RESOURCE_FLAG_NONE,
                D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                None,
                1,
            )?;
            upload_atlas(gpu, &atlas, width, height, &pixels)?;
            let descriptors = DescriptorHeap::new(&device, D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV, 1, true)?;
            let view = D3D12_SHADER_RESOURCE_VIEW_DESC {
                Format: DXGI_FORMAT_R8_UNORM,
                ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
                Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING as u32,
                Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Texture2D: D3D12_TEX2D_SRV {
                        MipLevels: 1,
                        ..Default::default()
                    },
                },
            };
            device.CreateShaderResourceView(&atlas, Some(&view), descriptors.cpu(0));
            Ok(Self {
                root_signature,
                pipeline,
                _atlas: atlas,
                descriptors,
                glyph_width: font.glyph_width,
                upload: None,
            })
        }
    }

    // Records the scene's drawing into the frame: its background over all
    // of it, then its quads. The GPU must be done with the last frame that
    // this renderer drew, whose quads the upload buffer has, which the main
    // loop's pacing and Offscreen::finish make sure of.
    pub fn draw(&mut self, gpu: &Gpu, frame: &Frame, scene: &Scene) -> Result<()> {
        let [r, g, b] = scene.background.map(|channel| f32::from(channel) / 255.0);
        unsafe { frame.clear([r, g, b, 1.0]) };
        let count = scene.quads.len() as u32;
        if count == 0 {
            return Ok(());
        }
        let bytes = size_of_val(scene.quads.as_slice());
        if self.upload.as_ref().is_none_or(|upload| upload.size < bytes) {
            // Twice as big, so that a growing text doesn't make a new buffer
            // every frame.
            self.upload = Some(unsafe { UploadBuffer::new(&gpu.device, (bytes * 2).max(64 * 1024))? });
        }
        let upload = self.upload.as_ref().unwrap();
        upload.write(0, &scene.quads);
        unsafe {
            let commands = &frame.list;
            frame.bind(false, None);
            commands.SetPipelineState(&self.pipeline);
            commands.SetGraphicsRootSignature(&self.root_signature);
            commands.SetDescriptorHeaps(&[Some(self.descriptors.heap.clone())]);
            let constants = [(frame.width as f32).to_bits(), (frame.height as f32).to_bits(), self.glyph_width, 0];
            commands.SetGraphicsRoot32BitConstants(0, 4, constants.as_ptr() as *const core::ffi::c_void, 0);
            commands.SetGraphicsRootShaderResourceView(1, upload.gpu_address(0));
            commands.SetGraphicsRootDescriptorTable(2, self.descriptors.gpu(0));
            commands.IASetPrimitiveTopology(D3D12_PRIMITIVE_TOPOLOGY(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP));
            commands.DrawInstanced(4, count, 0, 0);
        }
        Ok(())
    }
}

// Copies the atlas's pixels into its texture, through an upload buffer, and
// waits for the copy, after which the buffer can go.
unsafe fn upload_atlas(gpu: &mut Gpu, atlas: &ID3D12Resource, width: u32, height: u32, pixels: &[u8]) -> Result<()> {
    let pitch = (width as usize).next_multiple_of(D3D12_TEXTURE_DATA_PITCH_ALIGNMENT as usize);
    let mut upload = unsafe { UploadBuffer::new(&gpu.device, pitch * height as usize)? };
    let bytes = upload.bytes_mut();
    for (row, source) in pixels.chunks(width as usize).enumerate() {
        bytes[row * pitch..row * pitch + source.len()].copy_from_slice(source);
    }
    gpu.run(|commands| unsafe {
        commands.ResourceBarrier(&[transition(atlas, D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE, D3D12_RESOURCE_STATE_COPY_DEST)]);
        let source = D3D12_TEXTURE_COPY_LOCATION {
            pResource: std::mem::transmute_copy(&upload.resource),
            Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                    Offset: 0,
                    Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                        Format: DXGI_FORMAT_R8_UNORM,
                        Width: width,
                        Height: height,
                        Depth: 1,
                        RowPitch: pitch as u32,
                    },
                },
            },
        };
        let destination = D3D12_TEXTURE_COPY_LOCATION {
            pResource: std::mem::transmute_copy(atlas),
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
        };
        commands.CopyTextureRegion(&destination, 0, 0, 0, &source, None);
        commands.ResourceBarrier(&[transition(atlas, D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE)]);
    })
}
