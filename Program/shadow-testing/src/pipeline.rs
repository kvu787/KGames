// Draws a way's box and shadow into a frame with the way's shaders: one quad
// over the area that the shadow can reach, only inside the way's stage, with
// the box's rectangle, its color and the way's settings as root constants,
// laid out as Common.hlsli's Constants.

use graphics::{Frame, Gpu, TARGET_FORMAT, bytecode};
use user_interface::{Color, Rect};
use windows_api_raw::*;
use windows_core::Result;

pub struct ShadowPipeline {
    root_signature: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
}

impl ShadowPipeline {
    pub fn new(gpu: &Gpu, vertex_shader: &[u8], pixel_shader: &[u8]) -> Result<Self> {
        unsafe {
            let device = &gpu.device;
            let root_signature: ID3D12RootSignature = device.CreateRootSignature(0, vertex_shader)?;
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
                VS: bytecode(vertex_shader),
                PS: bytecode(pixel_shader),
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
            // Through the view that stores colors as they are, so that the
            // shadow blends in sRGB, as the interface's colors do.
            description.RTVFormats[0] = TARGET_FORMAT;
            let pipeline: ID3D12PipelineState = device.CreateGraphicsPipelineState(&description)?;
            Ok(Self { root_signature, pipeline })
        }
    }

    // Records the box, in its color, over its shadow into the frame, over
    // what the frame has: a quad over `area`, cut to `stage`.
    pub fn draw(&self, frame: &Frame, stage: Rect, area: Rect, rect: Rect, color: Color, params: [f32; 8]) {
        let constants: [f32; 24] = [
            frame.width as f32,
            frame.height as f32,
            0.0,
            0.0,
            rect.left(),
            rect.top(),
            rect.right(),
            rect.bottom(),
            color.r * color.a,
            color.g * color.a,
            color.b * color.a,
            color.a,
            area.left(),
            area.top(),
            area.right(),
            area.bottom(),
            params[0],
            params[1],
            params[2],
            params[3],
            params[4],
            params[5],
            params[6],
            params[7],
        ];
        let stage = stage.round();
        unsafe {
            let commands = &frame.list;
            frame.bind(false, None);
            commands.RSSetScissorRects(&[RECT {
                left: stage.left() as i32,
                top: stage.top() as i32,
                right: stage.right() as i32,
                bottom: stage.bottom() as i32,
            }]);
            commands.SetPipelineState(&self.pipeline);
            commands.SetGraphicsRootSignature(&self.root_signature);
            commands.SetGraphicsRoot32BitConstants(0, constants.len() as u32, constants.as_ptr() as *const core::ffi::c_void, 0);
            commands.IASetPrimitiveTopology(D3D12_PRIMITIVE_TOPOLOGY(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP));
            commands.DrawInstanced(4, 1, 0, 0);
        }
    }
}
