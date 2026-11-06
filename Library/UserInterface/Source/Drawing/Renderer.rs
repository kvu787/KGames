// Draws a frame's draw list with Direct3D 12 in one draw call: the instances,
// clips and polygon corners go into an upload buffer, the glyphs rasterized
// since the last frame go into the atlas texture, and one instanced draw of a
// 4-vertex strip draws everything, layer by layer.
//
// It binds the frame's target, viewport and scissor itself, so a program's
// 3D may leave anything bound, or draw into a target of its own and copy it
// into the frame.

use graphics::{DescriptorHeap, Frame, Gpu, TARGET_FORMAT, UploadBuffer, bytecode, create_texture, transition};
use windows_api::*;
use windows_core::Result;

use crate::drawing::painter::{DrawList, Instance, Layer};
use crate::drawing::text::{ATLAS_SIZE, Fonts};

const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/InterfaceVertex.cso"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/InterfacePixel.cso"));

// Where each part of the upload buffer starts: copies of texture rows need
// 512-byte alignment.
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
    // GPU must be done with the last frame this renderer drew, which the main
    // loop's pacing makes sure of.
    pub fn draw(&mut self, gpu: &Gpu, frame: &Frame, list: &DrawList, fonts: &mut Fonts) -> Result<()> {
        let uploads = fonts.take_uploads();
        let instance_bytes = (list.instance_count() * size_of::<Instance>()).max(size_of::<Instance>());
        let clip_bytes = list.clips.len() * size_of::<[f32; 4]>();
        let point_bytes = list.points.len().max(1) * size_of::<[f32; 2]>();
        let glyph_bytes: usize = uploads.iter().map(|glyph| (pitch(glyph.width) * glyph.height as usize).next_multiple_of(ALIGNMENT)).sum();
        let needed = [instance_bytes, clip_bytes, point_bytes].iter().map(|bytes| bytes.next_multiple_of(ALIGNMENT)).sum::<usize>() + glyph_bytes;
        if self.upload.as_ref().is_none_or(|upload| upload.size < needed) {
            // Half again as big, so that a growing interface doesn't make a
            // new buffer every frame.
            self.upload = Some(unsafe { UploadBuffer::new(&gpu.device, (needed * 3 / 2).max(1 << 20))? });
        }
        let upload = self.upload.as_mut().unwrap();

        // The instances, layer by layer, then the clips, then the corners.
        let mut offset = 0;
        for layer in Layer::ALL {
            let instances = list.layer_instances(layer);
            upload.write(offset, instances);
            offset += size_of_val(instances);
        }
        let clips_at = instance_bytes.next_multiple_of(ALIGNMENT);
        upload.write(clips_at, &list.clips);
        let points_at = clips_at + clip_bytes.next_multiple_of(ALIGNMENT);
        if list.points.is_empty() {
            upload.write(points_at, &[[0.0f32; 2]]);
        } else {
            upload.write(points_at, &list.points);
        }
        offset = points_at + point_bytes.next_multiple_of(ALIGNMENT);

        unsafe {
            let commands = &frame.list;
            if !uploads.is_empty() {
                commands.ResourceBarrier(&[transition(&self.atlas, D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE, D3D12_RESOURCE_STATE_COPY_DEST)]);
                for glyph in &uploads {
                    let row_pitch = pitch(glyph.width);
                    let bytes = upload.bytes_mut();
                    for row in 0..glyph.height as usize {
                        let source = &glyph.pixels[row * glyph.width as usize..(row + 1) * glyph.width as usize];
                        let at = offset + row * row_pitch;
                        bytes[at..at + source.len()].copy_from_slice(source);
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
                    commands.CopyTextureRegion(&destination, glyph.x, glyph.y, 0, &source, None);
                    offset += (row_pitch * glyph.height as usize).next_multiple_of(ALIGNMENT);
                }
                commands.ResourceBarrier(&[transition(&self.atlas, D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE)]);
            }

            let count = list.instance_count() as u32;
            if count == 0 {
                return Ok(());
            }
            frame.bind(false, None);
            commands.SetPipelineState(&self.pipeline);
            commands.SetGraphicsRootSignature(&self.root_signature);
            commands.SetDescriptorHeaps(&[Some(self.descriptors.heap.clone())]);
            let constants = [frame.width as f32, frame.height as f32, 0.0, 0.0];
            commands.SetGraphicsRoot32BitConstants(0, 4, constants.as_ptr() as *const core::ffi::c_void, 0);
            commands.SetGraphicsRootShaderResourceView(1, upload.gpu_address(0));
            commands.SetGraphicsRootShaderResourceView(2, upload.gpu_address(clips_at));
            commands.SetGraphicsRootShaderResourceView(3, upload.gpu_address(points_at));
            commands.SetGraphicsRootDescriptorTable(4, self.descriptors.gpu(0));
            commands.IASetPrimitiveTopology(D3D12_PRIMITIVE_TOPOLOGY(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP));
            commands.DrawInstanced(4, count, 0, 0);
        }
        Ok(())
    }
}

fn pitch(width: u32) -> usize {
    (width as usize).next_multiple_of(D3D12_TEXTURE_DATA_PITCH_ALIGNMENT as usize)
}
