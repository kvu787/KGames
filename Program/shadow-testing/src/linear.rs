// The linear shadow: pure black all around the box, `length` pixels from its
// edge, whose opacity goes in a straight line from `start` at the edge to
// `end` at the length, by the distance from the box, so that it's round
// around the box's square corners. Shaders/Linear.hlsl draws it, with the
// box, in one quad.

use graphics::{Frame, Gpu, TARGET_FORMAT, bytecode};
use user_interface::{Color, Rect};
use windows_api_raw::*;
use windows_core::Result;

const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/LinearVertex.cso"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/LinearPixel.cso"));

// What the sliders allow.
pub const MAX_LENGTH: f32 = 256.0;

// The shadow's settings: its length in pixels, and its opacity at the box's
// edge and at its length, from 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Linear {
    pub length: f32,
    pub start: f32,
    pub end: f32,
}

impl Default for Linear {
    fn default() -> Self {
        Self {
            length: 32.0,
            start: 0.5,
            end: 0.0,
        }
    }
}

// One of the settings, for its slider's message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    Length,
    Start,
    End,
}

impl Linear {
    // Sets a setting to the value, inside what it allows.
    pub fn set(&mut self, setting: Setting, value: f64) {
        let value = value as f32;
        match setting {
            Setting::Length => self.length = value.clamp(0.0, MAX_LENGTH),
            Setting::Start => self.start = value.clamp(0.0, 1.0),
            Setting::End => self.end = value.clamp(0.0, 1.0),
        }
    }

    // The shadow's opacity at a pixel whose center is at a distance from the
    // box, less than 0 inside it, as the shader works it out: the part of
    // the pixel that's outside the box and inside the length.
    pub fn opacity(&self, distance: f32) -> f32 {
        if self.length <= 0.0 {
            return 0.0;
        }
        let along = self.start + (self.end - self.start) * (distance / self.length).clamp(0.0, 1.0);
        let outside = 1.0 - coverage(distance);
        let within = coverage(distance - self.length);
        along * outside * within
    }
}

// How much of a pixel is on the side of an edge where the distance is less
// than 0, as the shader has it.
fn coverage(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

// The signed distance from a point to the rectangle, as the shader works it
// out: 0 on its edge, and less than 0 inside it.
pub fn box_distance(rect: Rect, x: f32, y: f32) -> f32 {
    let center = rect.center();
    let qx = (x - center.x).abs() - rect.width / 2.0;
    let qy = (y - center.y).abs() - rect.height / 2.0;
    qx.max(qy).min(0.0) + qx.max(0.0).hypot(qy.max(0.0))
}

// The pipeline that draws the box and its shadow into a frame.
pub struct LinearPipeline {
    root_signature: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
}

impl LinearPipeline {
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
            // Through the view that stores colors as they are, so that the
            // shadow blends in sRGB, as the interface's colors do.
            description.RTVFormats[0] = TARGET_FORMAT;
            let pipeline: ID3D12PipelineState = device.CreateGraphicsPipelineState(&description)?;
            Ok(Self { root_signature, pipeline })
        }
    }

    // Records the box, in its color, and its shadow into the frame, over
    // what the frame has.
    pub fn draw(&self, frame: &Frame, rect: Rect, color: Color, shadow: &Linear) {
        let constants: [f32; 16] = [
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
            shadow.length,
            shadow.start,
            shadow.end,
            0.0,
        ];
        unsafe {
            let commands = &frame.list;
            frame.bind(false, None);
            commands.SetPipelineState(&self.pipeline);
            commands.SetGraphicsRootSignature(&self.root_signature);
            commands.SetGraphicsRoot32BitConstants(0, constants.len() as u32, constants.as_ptr() as *const core::ffi::c_void, 0);
            commands.IASetPrimitiveTopology(D3D12_PRIMITIVE_TOPOLOGY(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP));
            commands.DrawInstanced(4, 1, 0, 0);
        }
    }
}
