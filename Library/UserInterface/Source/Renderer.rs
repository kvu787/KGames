// Draws a frame's draw list with Direct3D 12, in one draw call: the
// instances, clip rectangles and polygon corners go into an upload buffer,
// the glyphs rasterized since the last frame go into the atlas texture, and
// one instanced draw of a 4-vertex strip draws everything, layer by layer.

use graphics::{
    DescriptorHeap, Frame, Gpu, TARGET_FORMAT, UploadBuffer, bytecode, create_texture, transition,
};
use windows_api::*;
use windows_core::Result;

use crate::painter::{DrawList, Instance};
use crate::text::{ATLAS_SIZE, Fonts};

const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/InterfaceVertex.cso"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/InterfacePixel.cso"));

const ALIGNMENT: usize = 512;

pub struct Renderer {
    root_signature: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
    atlas: ID3D12Resource,
    descriptors: DescriptorHeap,
    upload: Option<UploadBuffer>,
}

impl Renderer {
    pub fn new(gpu: &Gpu) -> Result<Self> {
        unsafe {
            let device = &gpu.device;
            let root_signature: ID3D12RootSignature = device.CreateRootSignature(0, VERTEX_SHADER)?;
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

            let atlas = create_texture(
                device,
                ATLAS_SIZE,
                ATLAS_SIZE,
                DXGI_FORMAT_R8_UNORM,
                D3D12_RESOURCE_FLAG_NONE,
                D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                None,
                1,
            )?;
            let descriptors = DescriptorHeap::new(device, D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV, 1, true)?;
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
                atlas,
                descriptors,
                upload: None,
            })
        }
    }

    // Records the draw list's drawing into the frame, over what it has. The
    // GPU must be done with the last frame that this renderer drew, which the
    // main loop's pacing makes sure of.
    pub fn draw(&mut self, gpu: &Gpu, frame: &Frame, list: &DrawList, fonts: &mut Fonts) -> Result<()> {
        let (_, uploads) = fonts.take_uploads();
        let instance_bytes = list.instance_count() * size_of::<Instance>();
        let clip_bytes = list.clips.len() * 16;
        let point_bytes = list.points.len().max(1) * 8;
        let glyph_bytes: usize = uploads
            .iter()
            .map(|glyph| pitch(glyph.width) * glyph.height as usize + ALIGNMENT)
            .sum();
        let needed = [instance_bytes.max(64), clip_bytes, point_bytes, glyph_bytes]
            .iter()
            .map(|bytes| bytes.next_multiple_of(ALIGNMENT))
            .sum::<usize>();
        if self.upload.as_ref().is_none_or(|upload| upload.size < needed) {
            // Grows by half again, so that a growing interface doesn't
            // reallocate every frame.
            self.upload = Some(unsafe { UploadBuffer::new(&gpu.device, (needed * 3 / 2).max(1 << 20))? });
        }
        let upload = self.upload.as_mut().unwrap();

        // Instances, layer by layer, then clips, then points.
        let mut offset = 0;
        let instances_at = offset;
        for layer in crate::painter::Layer::ALL {
            let instances = list.layer(layer);
            upload.write(offset, instances);
            offset += size_of_val(instances);
        }
        offset = offset.max(64).next_multiple_of(ALIGNMENT);
        let clips_at = offset;
        upload.write(offset, &list.clips);
        offset = (offset + clip_bytes).next_multiple_of(ALIGNMENT);
        let points_at = offset;
        if list.points.is_empty() {
            upload.write(offset, &[[0.0f32; 2]]);
        } else {
            upload.write(offset, &list.points);
        }
        offset = (offset + point_bytes).next_multiple_of(ALIGNMENT);

        unsafe {
            let command_list = &frame.list;
            if !uploads.is_empty() {
                command_list.ResourceBarrier(&[transition(
                    &self.atlas,
                    D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                )]);
                for glyph in &uploads {
                    let row_pitch = pitch(glyph.width);
                    let bytes = upload.bytes_mut();
                    for row in 0..glyph.height as usize {
                        let source = &glyph.pixels[row * glyph.width as usize..(row + 1) * glyph.width as usize];
                        bytes[offset + row * row_pitch..offset + row * row_pitch + source.len()].copy_from_slice(source);
                    }
                    let source = D3D12_TEXTURE_COPY_LOCATION {
                        pResource: std::mem::transmute_copy(&upload.resource),
                        Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                        Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                            PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                                Offset: offset as u64,
                                Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                                    Format: DXGI_FORMAT_R8_UNORM,
                                    Width: glyph.width,
                                    Height: glyph.height,
                                    Depth: 1,
                                    RowPitch: row_pitch as u32,
                                },
                            },
                        },
                    };
                    let destination = D3D12_TEXTURE_COPY_LOCATION {
                        pResource: std::mem::transmute_copy(&self.atlas),
                        Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                        Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
                    };
                    command_list.CopyTextureRegion(&destination, glyph.x, glyph.y, 0, &source, None);
                    offset = (offset + row_pitch * glyph.height as usize).next_multiple_of(ALIGNMENT);
                }
                command_list.ResourceBarrier(&[transition(
                    &self.atlas,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                )]);
            }

            let count = list.instance_count() as u32;
            if count == 0 {
                return Ok(());
            }
            frame.bind(false, None);
            command_list.SetPipelineState(&self.pipeline);
            command_list.SetGraphicsRootSignature(&self.root_signature);
            command_list.SetDescriptorHeaps(&[Some(self.descriptors.heap.clone())]);
            let constants = [frame.width as f32, frame.height as f32, 0.0, 0.0];
            command_list.SetGraphicsRoot32BitConstants(0, 4, constants.as_ptr() as *const _, 0);
            command_list.SetGraphicsRootShaderResourceView(1, upload.gpu_address(instances_at));
            command_list.SetGraphicsRootShaderResourceView(2, upload.gpu_address(clips_at));
            command_list.SetGraphicsRootShaderResourceView(3, upload.gpu_address(points_at));
            command_list.SetGraphicsRootDescriptorTable(4, self.descriptors.gpu(0));
            command_list.IASetPrimitiveTopology(D3D12_PRIMITIVE_TOPOLOGY(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP));
            command_list.DrawInstanced(4, count, 0, 0);
        }
        Ok(())
    }
}

fn pitch(width: u32) -> usize {
    (width as usize).next_multiple_of(D3D12_TEXTURE_DATA_PITCH_ALIGNMENT as usize)
}
