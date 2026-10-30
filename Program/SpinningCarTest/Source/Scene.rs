// The scene of Veehiicuul's 3dTestScene_CppDx12: a baked image of the
// backdrop, the green ground and the red cube, then a car that moves back
// and forth along X at 8 units per second while it turns at 90 degrees per
// second, and a sphere that stays put, both painted with SimplePaint, with a
// depth buffer between them. Camera, sizes and timing are the C++ scene's.

use std::path::Path;
use std::time::Instant;

use graphics::{BACK_BUFFER_SRGB_FORMAT, DEPTH_FORMAT, Renderer, UploadBuffer, bytecode, load_image, upload_texture};
use math::Matrix;
use simple_paint::direct3d12::{Mesh, OBJECT_SIZE, Pipeline, Target, materials_size};
use simple_paint::orthographic::{build_object_transforms, make_projection};
use simple_paint::{GpuMaterial, Material};
use windows_api::*;
use windows_core::{Error, Result};

use crate::car_mesh;
use crate::settings::SceneSettings;

static PAINT_VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintVertex.cso"));
static PAINT_PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintPixel.cso"));
static BACKGROUND_VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/BackgroundVertex.cso"));
static BACKGROUND_PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/BackgroundPixel.cso"));

// The car's five materials, then the sphere's, as SIMPLE_PAINT_MATERIAL_COUNT
// in Build.rs says.
const MATERIAL_COUNT: usize = 6;
const SPHERE_MATERIAL: u32 = car_mesh::MATERIAL_COUNT;
// The car, then the sphere.
const OBJECT_COUNT: usize = 2;
// The baked background's aspect. A wider window gets the scene in a 32:9
// middle, with the background's corner color on the sides.
const BACKGROUND_ASPECT: f32 = 32.0 / 9.0;
// The camera looks down at the origin from 45 degrees above the horizon, and
// 45 degrees from +Z toward +X, 5 units away, at a view 5 units tall.
const CAMERA_PITCH: f32 = std::f32::consts::FRAC_PI_4;
const CAMERA_YAW: f32 = std::f32::consts::FRAC_PI_4;
const CAMERA_DISTANCE: f32 = 5.0;
const VIEW_HEIGHT: f32 = 5.0;
const CAR_SCALE: f32 = 0.25;
// Units per second, and how far from the middle the car turns back.
const SPEED: f64 = 8.0;
const TRAVEL_DISTANCE: f64 = 7.0;
// Radians per second.
const TURN_SPEED: f64 = std::f64::consts::FRAC_PI_2;
const SPHERE_RADIUS: f32 = 0.4;

pub struct Scene {
    paint: Pipeline,
    car: Mesh,
    sphere: Mesh,
    // Each object's transforms, then the materials, each at a constant
    // buffer's start. The GPU has finished with them when a frame is ready.
    constants: UploadBuffer,
    materials: [GpuMaterial; MATERIAL_COUNT],
    background_root_signature: ID3D12RootSignature,
    background_pipeline: ID3D12PipelineState,
    // Shader-visible, holding the background texture's view.
    srv_heap: ID3D12DescriptorHeap,
    // Kept alive for the view in srv_heap.
    _background: ID3D12Resource,
    start: Instant,
    // The C++ scene's S: the car stays in the middle, facing +X.
    pub stationary: bool,
}

impl Scene {
    pub unsafe fn new(renderer: &mut Renderer, settings: &SceneSettings, background: &Path) -> Result<Self> {
        unsafe {
            let device = renderer.device.clone();
            let paint = Pipeline::new(
                &device,
                PAINT_VERTEX_SHADER,
                PAINT_PIXEL_SHADER,
                &Target {
                    format: BACK_BUFFER_SRGB_FORMAT,
                    depth_format: Some(DEPTH_FORMAT),
                    sample_count: 1,
                    cull_mode: D3D12_CULL_MODE_NONE,
                },
            )?;
            let car = Mesh::new(&device, &car_mesh::VERTICES, &car_mesh::INDICES, MATERIAL_COUNT as u32)?;
            let [longitudes, latitudes] = settings.sphere_resolution;
            let sphere = meshes::uv_sphere(longitudes, latitudes, SPHERE_MATERIAL);
            let sphere = Mesh::new(&device, &sphere.vertices, &sphere.indices, MATERIAL_COUNT as u32)?;
            let mut materials = [GpuMaterial::default(); MATERIAL_COUNT];
            for (material, parameters) in materials.iter_mut().zip(&settings.materials) {
                *material = *Material::compile(parameters)
                    .map_err(|error| Error::new(E_INVALIDARG, error.0))?
                    .constants();
            }

            // The background is sRGB, so sampling it gives linear colors,
            // which the sRGB render target encodes again.
            let image = load_image(background)?;
            let texture = upload_texture(
                renderer,
                image.width,
                image.height,
                DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
                4,
                &image.pixels,
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
                Format: DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
                ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
                Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING as u32,
                ..Default::default()
            };
            view.Anonymous.Texture2D.MipLevels = 1;
            device.CreateShaderResourceView(&texture, Some(&view), srv_heap.GetCPUDescriptorHandleForHeapStart());
            let background_root_signature: ID3D12RootSignature =
                device.CreateRootSignature(0, BACKGROUND_VERTEX_SHADER)?;
            let mut description = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                // Borrow the root signature without touching its reference count.
                pRootSignature: std::mem::transmute_copy(&background_root_signature),
                VS: bytecode(BACKGROUND_VERTEX_SHADER),
                PS: bytecode(BACKGROUND_PIXEL_SHADER),
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
                    CullMode: D3D12_CULL_MODE_NONE,
                    DepthClipEnable: true.into(),
                    ..Default::default()
                },
                PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                NumRenderTargets: 1,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                ..Default::default()
            };
            description.RTVFormats[0] = BACK_BUFFER_SRGB_FORMAT;
            let background_pipeline = device.CreateGraphicsPipelineState(&description)?;

            Ok(Self {
                paint,
                car,
                sphere,
                constants: UploadBuffer::new(&device, OBJECT_SIZE * OBJECT_COUNT + materials_size(MATERIAL_COUNT))?,
                materials,
                background_root_signature,
                background_pipeline,
                srv_heap,
                _background: texture,
                start: Instant::now(),
                stationary: false,
            })
        }
    }

    // Where the car is along X, and how far it has turned, from the time
    // since the scene started. It goes from the middle to +7, back to -7,
    // and again, 28 units a cycle.
    fn car_placement(&self) -> (f32, f32) {
        if self.stationary {
            // The mesh faces +Z. Facing +X puts the headlights at the lower
            // right in the fixed 45-degree camera.
            return (0.0, std::f32::consts::FRAC_PI_2);
        }
        let elapsed = self.start.elapsed().as_secs_f64();
        let phase = (elapsed * SPEED + TRAVEL_DISTANCE) % (TRAVEL_DISTANCE * 4.0);
        let position = TRAVEL_DISTANCE - (phase - TRAVEL_DISTANCE * 2.0).abs();
        let turn = (elapsed * TURN_SPEED) % std::f64::consts::TAU;
        (position as f32, turn as f32)
    }

    // Records the scene into the frame, over the whole window.
    pub unsafe fn draw(&self, renderer: &Renderer) -> Result<()> {
        unsafe {
            let (width, height) = (renderer.width as f32, renderer.height as f32);
            let eye = [
                CAMERA_DISTANCE * CAMERA_PITCH.cos() * CAMERA_YAW.sin(),
                CAMERA_DISTANCE * CAMERA_PITCH.sin(),
                CAMERA_DISTANCE * CAMERA_PITCH.cos() * CAMERA_YAW.cos(),
            ];
            let view = Matrix::look_at_rh(eye, [0.0; 3], [0.0, 1.0, 0.0]);
            let aspect = (width / height).min(BACKGROUND_ASPECT);
            let projection = make_projection(VIEW_HEIGHT * aspect, VIEW_HEIGHT, 1.0, 20.0)
                .map_err(|error| Error::new(E_INVALIDARG, error.0))?;
            let (position, turn) = self.car_placement();
            let worlds = [
                Matrix::scaling(CAR_SCALE, CAR_SCALE, CAR_SCALE)
                    * Matrix::rotation_y(turn)
                    * Matrix::translation(position, 0.0, 0.0),
                Matrix::scaling(SPHERE_RADIUS, SPHERE_RADIUS, SPHERE_RADIUS)
                    * Matrix::translation(1.5, SPHERE_RADIUS, -1.5),
            ];
            for (index, world) in worlds.iter().enumerate() {
                let object = build_object_transforms(&(*world * view), &projection)
                    .map_err(|error| Error::new(E_INVALIDARG, error.0))?;
                self.constants.write(OBJECT_SIZE * index, &[object]);
            }
            let materials_offset = OBJECT_SIZE * OBJECT_COUNT;
            self.constants.write(materials_offset, &self.materials);

            // The background, over the whole window.
            renderer.set_render_target(true, false);
            let list = &renderer.command_list;
            list.SetPipelineState(&self.background_pipeline);
            list.SetGraphicsRootSignature(&self.background_root_signature);
            list.SetDescriptorHeaps(&[Some(self.srv_heap.clone())]);
            list.SetGraphicsRootDescriptorTable(0, self.srv_heap.GetGPUDescriptorHandleForHeapStart());
            let horizontal_uv_scale = width / height / BACKGROUND_ASPECT;
            list.SetGraphicsRoot32BitConstants(1, 1, (&raw const horizontal_uv_scale).cast(), 0);
            list.IASetPrimitiveTopology(D3D12_PRIMITIVE_TOPOLOGY(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST));
            list.DrawInstanced(3, 1, 0, 0);

            // The car and the sphere, in the scene's 32:9 middle of a wider
            // window, which the supported setups never are.
            renderer.clear_depth();
            renderer.set_render_target(true, true);
            let scene_width = width.min(height * BACKGROUND_ASPECT);
            list.RSSetViewports(&[D3D12_VIEWPORT {
                TopLeftX: (width - scene_width) * 0.5,
                TopLeftY: 0.0,
                Width: scene_width,
                Height: height,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            }]);
            self.paint.bind(list);
            let materials = self.constants.address(materials_offset);
            self.car.draw(list, self.constants.address(0), materials);
            self.sphere.draw(list, self.constants.address(OBJECT_SIZE), materials);
            Ok(())
        }
    }
}
