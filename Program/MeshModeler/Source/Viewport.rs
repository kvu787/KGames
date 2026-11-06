// Draws the view: the mesh, painted with SimplePaint or flat-lit, into the
// view's part of the frame through its sRGB view, with a depth buffer; then
// the overlay over it, the cage's edges and vertices, selected faces and the
// floor's grid, through the frame's plain view, tested against the mesh's
// depth, and the overlay that shows through the mesh, untested.
//
// The mesh's triangles go into an upload buffer only when they change; the
// constants and the overlay go into another every frame. The main loop starts
// a frame only once the GPU has finished the last, so rewriting them before
// drawing is safe.

use graphics::{DEPTH_FORMAT, DescriptorHeap, Frame, Gpu, TARGET_FORMAT, TARGET_SRGB_FORMAT, UploadBuffer, bytecode, create_texture};
use math::Matrix;
use polygon_mesh::Triangles;
use simple_paint::GpuMaterial;
use simple_paint::direct3d12::{OBJECT_SIZE, Pipeline, Target, materials_size};
use simple_paint::orthographic::ObjectTransforms;
use user_interface::Rect;
use windows_api::*;
use windows_core::Result;

use crate::paint::MAXIMUM_PAINTS;

const PAINT_VERTEX: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintVertex.cso"));
const PAINT_PIXEL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintPixel.cso"));
const FLAT_VERTEX: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/FlatVertex.cso"));
const FLAT_PIXEL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/FlatPixel.cso"));
const OVERLAY_VERTEX: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/OverlayVertex.cso"));
const OVERLAY_PIXEL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/OverlayPixel.cso"));

const ALIGNMENT: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Shader {
    #[default]
    SimplePaint,
    Flat,
}

impl Shader {
    pub fn name(self) -> &'static str {
        match self {
            Shader::SimplePaint => "SimplePaint",
            Shader::Flat => "Flat",
        }
    }
}

// What the overlay draws, laid out as Overlay.hlsl's Item: a line from a to
// b, a dot at a, or a triangle, `width` pixels wide, in a premultiplied sRGB
// color.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OverlayItem {
    pub a: [f32; 3],
    pub width: f32,
    pub b: [f32; 3],
    pub color: u32,
    pub c: [f32; 3],
    pub kind: u32,
}

const _: () = assert!(size_of::<OverlayItem>() == 48);

pub const KIND_LINE: u32 = 0;
pub const KIND_POINT: u32 = 1;
pub const KIND_TRIANGLE: u32 = 2;

// The flat shader's constants at b1.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct FlatConstants {
    light: [f32; 4],
    colors: [[f32; 4]; MAXIMUM_PAINTS],
}

// What a frame of the view draws, which the program updates as it builds the
// interface.
pub struct Scene {
    // Where the view is in the window.
    pub view: Rect,
    // Its background, sRGB.
    pub background: [f32; 4],
    pub transforms: ObjectTransforms,
    pub world_to_clip: Matrix,
    pub shader: Shader,
    // The mesh's triangles, and the document's version that they're of.
    pub triangles: Triangles,
    pub version: u64,
    pub materials: [GpuMaterial; MAXIMUM_PAINTS],
    // The flat shader's: the paints' linear colors, and the light toward the
    // camera's upper left, with how much of the color is ambient as w.
    pub colors: [[f32; 4]; MAXIMUM_PAINTS],
    pub light: [f32; 4],
    pub overlay: Vec<OverlayItem>,
    pub xray: Vec<OverlayItem>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            view: Rect::ZERO,
            background: [0.0; 4],
            transforms: ObjectTransforms::default(),
            world_to_clip: Matrix::IDENTITY,
            shader: Shader::SimplePaint,
            triangles: Triangles {
                vertices: Vec::new(),
                indices: Vec::new(),
                faces: Vec::new(),
            },
            version: 0,
            materials: [GpuMaterial::default(); MAXIMUM_PAINTS],
            colors: [[0.0; 4]; MAXIMUM_PAINTS],
            light: [0.0; 4],
            overlay: Vec::new(),
            xray: Vec::new(),
        }
    }
}

struct OverlayPipeline {
    root_signature: ID3D12RootSignature,
    tested: ID3D12PipelineState,
    untested: ID3D12PipelineState,
}

pub struct Viewport {
    paint: Pipeline,
    flat: Pipeline,
    overlay: OverlayPipeline,
    depth: Option<(ID3D12Resource, u32, u32)>,
    depth_views: DescriptorHeap,
    geometry: Option<UploadBuffer>,
    geometry_version: u64,
    index_count: u32,
    vertex_bytes: usize,
    frame_data: Option<UploadBuffer>,
}

impl Viewport {
    pub fn new(gpu: &Gpu) -> Result<Self> {
        unsafe {
            let target = Target {
                format: TARGET_SRGB_FORMAT,
                depth_format: Some(DEPTH_FORMAT),
                sample_count: 1,
                // Open meshes show their insides, which SimplePaint shades
                // as facing away.
                cull_mode: D3D12_CULL_MODE_NONE,
            };
            Ok(Self {
                paint: Pipeline::new(&gpu.device, PAINT_VERTEX, PAINT_PIXEL, &target)?,
                flat: Pipeline::new(&gpu.device, FLAT_VERTEX, FLAT_PIXEL, &target)?,
                overlay: overlay_pipeline(&gpu.device)?,
                depth: None,
                depth_views: DescriptorHeap::new(&gpu.device, D3D12_DESCRIPTOR_HEAP_TYPE_DSV, 1, false)?,
                geometry: None,
                geometry_version: 0,
                index_count: 0,
                vertex_bytes: 0,
                frame_data: None,
            })
        }
    }

    fn depth_view(&mut self, gpu: &Gpu, frame: &Frame) -> Result<D3D12_CPU_DESCRIPTOR_HANDLE> {
        let view = self.depth_views.cpu(0);
        if !matches!(self.depth, Some((_, width, height)) if (width, height) == (frame.width, frame.height)) {
            let clear = D3D12_CLEAR_VALUE {
                Format: DEPTH_FORMAT,
                Anonymous: D3D12_CLEAR_VALUE_0 {
                    DepthStencil: D3D12_DEPTH_STENCIL_VALUE { Depth: 1.0, Stencil: 0 },
                },
            };
            unsafe {
                let texture = create_texture(
                    &gpu.device,
                    frame.width,
                    frame.height,
                    DEPTH_FORMAT,
                    D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
                    D3D12_RESOURCE_STATE_DEPTH_WRITE,
                    Some(&clear),
                    1,
                )?;
                gpu.device.CreateDepthStencilView(&texture, None, view);
                self.depth = Some((texture, frame.width, frame.height));
            }
        }
        Ok(view)
    }

    // Copies the mesh's triangles into the geometry buffer, if they changed.
    fn upload_geometry(&mut self, gpu: &Gpu, scene: &Scene) -> Result<()> {
        if self.geometry_version == scene.version && self.geometry.is_some() {
            return Ok(());
        }
        let vertex_bytes = size_of_val(scene.triangles.vertices.as_slice());
        let index_bytes = size_of_val(scene.triangles.indices.as_slice());
        let needed = vertex_bytes.next_multiple_of(ALIGNMENT) + index_bytes.max(4);
        if self.geometry.as_ref().is_none_or(|buffer| buffer.size < needed) {
            self.geometry = Some(unsafe { UploadBuffer::new(&gpu.device, (needed * 3 / 2).max(1 << 16))? });
        }
        let buffer = self.geometry.as_ref().unwrap();
        buffer.write(0, &scene.triangles.vertices);
        buffer.write(vertex_bytes.next_multiple_of(ALIGNMENT), &scene.triangles.indices);
        self.vertex_bytes = vertex_bytes;
        self.index_count = scene.triangles.indices.len() as u32;
        self.geometry_version = scene.version;
        Ok(())
    }

    // Draws the scene into its part of the frame.
    pub fn draw(&mut self, gpu: &Gpu, frame: &Frame, scene: &Scene) -> Result<()> {
        let view = scene.view.round();
        if view.is_empty() {
            return Ok(());
        }
        self.upload_geometry(gpu, scene)?;
        let depth = self.depth_view(gpu, frame)?;

        // This frame's constants and overlay.
        let flat = FlatConstants {
            light: scene.light,
            colors: scene.colors,
        };
        let transforms_at = 0;
        let materials_at = OBJECT_SIZE;
        let flat_at = materials_at + materials_size(MAXIMUM_PAINTS);
        let matrix_at = flat_at + size_of::<FlatConstants>().next_multiple_of(ALIGNMENT);
        let overlay_at = matrix_at + ALIGNMENT;
        let overlay_bytes = size_of_val(scene.overlay.as_slice()).next_multiple_of(ALIGNMENT);
        let xray_at = overlay_at + overlay_bytes.max(ALIGNMENT);
        let needed = xray_at + size_of_val(scene.xray.as_slice()).max(ALIGNMENT);
        if self.frame_data.as_ref().is_none_or(|buffer| buffer.size < needed) {
            self.frame_data = Some(unsafe { UploadBuffer::new(&gpu.device, (needed * 3 / 2).max(1 << 16))? });
        }
        let data = self.frame_data.as_ref().unwrap();
        data.write(transforms_at, &[scene.transforms]);
        data.write(materials_at, &scene.materials);
        data.write(flat_at, &[flat]);
        data.write(matrix_at, &[scene.world_to_clip.0]);
        data.write(overlay_at, &scene.overlay);
        data.write(xray_at, &scene.xray);

        let commands = &frame.list;
        let rect = RECT {
            left: view.x as i32,
            top: view.y as i32,
            right: view.right() as i32,
            bottom: view.bottom() as i32,
        };
        let viewport = D3D12_VIEWPORT {
            TopLeftX: view.x,
            TopLeftY: view.y,
            Width: view.width,
            Height: view.height,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };
        unsafe {
            commands.ClearRenderTargetView(frame.view, &scene.background, &[rect]);
            commands.ClearDepthStencilView(depth, D3D12_CLEAR_FLAG_DEPTH, 1.0, 0, &[rect]);
            commands.RSSetViewports(&[viewport]);
            commands.RSSetScissorRects(&[rect]);

            // The mesh.
            if self.index_count > 0
                && let Some(geometry) = &self.geometry
            {
                commands.OMSetRenderTargets(1, Some(&frame.srgb_view), false, Some(&depth));
                let (pipeline, shading) = match scene.shader {
                    Shader::SimplePaint => (&self.paint, materials_at),
                    Shader::Flat => (&self.flat, flat_at),
                };
                pipeline.bind(commands);
                commands.SetGraphicsRootConstantBufferView(0, data.gpu_address(transforms_at));
                commands.SetGraphicsRootConstantBufferView(1, data.gpu_address(shading));
                let start = geometry.gpu_address(0);
                commands.IASetVertexBuffers(
                    0,
                    Some(&[D3D12_VERTEX_BUFFER_VIEW {
                        BufferLocation: start,
                        SizeInBytes: self.vertex_bytes as u32,
                        StrideInBytes: size_of::<simple_paint::Vertex>() as u32,
                    }]),
                );
                commands.IASetIndexBuffer(Some(&D3D12_INDEX_BUFFER_VIEW {
                    BufferLocation: geometry.gpu_address(self.vertex_bytes.next_multiple_of(ALIGNMENT)),
                    SizeInBytes: self.index_count * 4,
                    Format: DXGI_FORMAT_R32_UINT,
                }));
                commands.DrawIndexedInstanced(self.index_count, 1, 0, 0, 0);
            }

            // The overlay, tested against the mesh's depth, then the X-ray's.
            commands.OMSetRenderTargets(1, Some(&frame.view), false, Some(&depth));
            commands.SetGraphicsRootSignature(&self.overlay.root_signature);
            commands.IASetPrimitiveTopology(D3D12_PRIMITIVE_TOPOLOGY(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP));
            let constants = [view.width, view.height, 0.0004, 0.0];
            for (state, items, at) in [(&self.overlay.tested, &scene.overlay, overlay_at), (&self.overlay.untested, &scene.xray, xray_at)] {
                if items.is_empty() {
                    continue;
                }
                commands.SetPipelineState(state);
                commands.SetGraphicsRoot32BitConstants(0, 4, constants.as_ptr() as *const core::ffi::c_void, 0);
                commands.SetGraphicsRootConstantBufferView(1, data.gpu_address(matrix_at));
                commands.SetGraphicsRootShaderResourceView(2, data.gpu_address(at));
                commands.DrawInstanced(4, items.len() as u32, 0, 0);
            }
        }
        Ok(())
    }
}

unsafe fn overlay_pipeline(device: &ID3D12Device) -> Result<OverlayPipeline> {
    unsafe {
        let root_signature: ID3D12RootSignature = device.CreateRootSignature(0, OVERLAY_VERTEX)?;
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
            VS: bytecode(OVERLAY_VERTEX),
            PS: bytecode(OVERLAY_PIXEL),
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
            DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
                DepthEnable: true.into(),
                DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ZERO,
                DepthFunc: D3D12_COMPARISON_FUNC_LESS_EQUAL,
                ..Default::default()
            },
            PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            NumRenderTargets: 1,
            DSVFormat: DEPTH_FORMAT,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            ..Default::default()
        };
        description.RTVFormats[0] = TARGET_FORMAT;
        let tested: ID3D12PipelineState = device.CreateGraphicsPipelineState(&description)?;
        description.DepthStencilState.DepthEnable = false.into();
        let untested: ID3D12PipelineState = device.CreateGraphicsPipelineState(&description)?;
        Ok(OverlayPipeline {
            root_signature,
            tested,
            untested,
        })
    }
}
