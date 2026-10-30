// VsyncStutterTest's Godot scene, drawn with SimplePaint: a sphere that moves
// back and forth along X, above a ground plane, past a cube, seen by an
// orthographic camera from above and to the side. Positions and sizes are
// Main.tscn's, in Godot's units.

use graphics::{DEPTH_FORMAT, Renderer, UploadBuffer, BACK_BUFFER_SRGB_FORMAT};
use math::Matrix;
use simple_paint::direct3d12::{Mesh, OBJECT_SIZE, Pipeline, Target, materials_size};
use simple_paint::orthographic::{Projection, build_object_transforms, make_projection};
use simple_paint::{GpuMaterial, Material, Parameters};
use windows_api::*;
use windows_core::{Error, Result};

use crate::settings::SceneSettings;

static VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintVertex.cso"));
static PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintPixel.cso"));

// The shaders' SIMPLE_PAINT_MATERIAL_COUNT, from Build.rs: the sphere's,
// the ground's and the cube's, in that order.
const MATERIAL_COUNT: usize = 3;
const OBJECT_COUNT: usize = 3;

// Sphere.gd: units per second, and how far from the middle it turns back.
const SPEED: f32 = 8.0;
const TRAVEL_DISTANCE: f32 = 7.0;
// Godot's default SphereMesh: radius 0.5, with 64 segments around and 32 rings.
const SPHERE_RADIUS: f32 = 0.5;

// The camera, from CameraPivot's basis in Main.tscn and the camera 5 units
// along the pivot's Z: an orthographic view 5 units tall, from 1 to 20 units
// in front of it.
const CAMERA_BACKWARD: [f32; 3] = [0.3377951, 0.7372773, 0.5850783];
const CAMERA_UP: [f32; 3] = [-0.36863866, 0.6755902, -0.63850087];
const CAMERA_DISTANCE: f32 = 5.0;
const VIEW_HEIGHT: f32 = 5.0;

pub struct Scene {
    pipeline: Pipeline,
    // The sphere, the ground and the cube.
    meshes: [Mesh; OBJECT_COUNT],
    // Each object's transforms, then the materials, each at a constant
    // buffer's start. The GPU has finished with them when a frame is ready.
    constants: UploadBuffer,
    materials: [GpuMaterial; MATERIAL_COUNT],
    // The sphere's X, and which way it's moving along X.
    pub position: f32,
    direction: f32,
}

impl Scene {
    pub unsafe fn new(renderer: &Renderer, settings: &SceneSettings) -> Result<Self> {
        unsafe {
            let device = &renderer.device;
            let pipeline = Pipeline::new(
                device,
                VERTEX_SHADER,
                PIXEL_SHADER,
                &Target {
                    format: BACK_BUFFER_SRGB_FORMAT,
                    depth_format: Some(DEPTH_FORMAT),
                    sample_count: 1,
                    cull_mode: D3D12_CULL_MODE_NONE,
                },
            )?;
            let upload = |mesh: meshes::Mesh| {
                Mesh::new(device, &mesh.vertices, &mesh.indices, MATERIAL_COUNT as u32)
            };
            let meshes = [
                upload(meshes::uv_sphere(64, 32, 0))?,
                upload(meshes::plane([10.0, 5.0], 1))?,
                upload(meshes::cuboid([1.0; 3], 2))?,
            ];
            let compile = |parameters: &Parameters| {
                Material::compile(parameters)
                    .map(|material| *material.constants())
                    .map_err(|error| Error::new(E_INVALIDARG, error.0))
            };
            Ok(Self {
                pipeline,
                meshes,
                constants: UploadBuffer::new(
                    device,
                    OBJECT_SIZE * OBJECT_COUNT + materials_size(MATERIAL_COUNT),
                )?,
                materials: [
                    compile(&settings.sphere)?,
                    compile(&settings.ground)?,
                    compile(&settings.cube)?,
                ],
                position: 0.0,
                direction: 1.0,
            })
        }
    }

    // Moves the sphere by `seconds` since the last frame, like Sphere.gd's
    // _process: it turns back once it's past the travel distance.
    pub fn advance(&mut self, seconds: f32) {
        self.position += self.direction * SPEED * seconds;
        if self.position.abs() > TRAVEL_DISTANCE {
            self.direction = if self.position > 0.0 { -1.0 } else { 1.0 };
        }
    }

    // Records the scene into the frame, over the whole window.
    pub unsafe fn draw(&self, renderer: &Renderer) -> Result<()> {
        unsafe {
            let aspect = renderer.width as f32 / renderer.height as f32;
            let projection = make_projection(VIEW_HEIGHT * aspect, VIEW_HEIGHT, 1.0, 20.0)
                .map_err(|error| Error::new(E_INVALIDARG, error.0))?;
            let eye = CAMERA_BACKWARD.map(|component| component * CAMERA_DISTANCE);
            let view = Matrix::look_at_rh(eye, [0.0; 3], CAMERA_UP);
            let worlds = [
                Matrix::scaling(SPHERE_RADIUS, SPHERE_RADIUS, SPHERE_RADIUS)
                    * Matrix::translation(self.position, SPHERE_RADIUS, 0.0),
                Matrix::IDENTITY,
                Matrix::translation(0.0, 0.5, -1.5),
            ];
            for (index, world) in worlds.iter().enumerate() {
                self.write_object(index, &(*world * view), &projection)?;
            }
            let materials_offset = OBJECT_SIZE * OBJECT_COUNT;
            self.constants.write(materials_offset, &self.materials);

            renderer.clear_depth();
            renderer.set_render_target(true, true);
            let list = &renderer.command_list;
            self.pipeline.bind(list);
            for (index, mesh) in self.meshes.iter().enumerate() {
                mesh.draw(
                    list,
                    self.constants.address(OBJECT_SIZE * index),
                    self.constants.address(materials_offset),
                );
            }
            Ok(())
        }
    }

    unsafe fn write_object(&self, index: usize, world_view: &Matrix, projection: &Projection) -> Result<()> {
        let object = build_object_transforms(world_view, projection)
            .map_err(|error| Error::new(E_INVALIDARG, error.0))?;
        unsafe { self.constants.write(OBJECT_SIZE * index, &[object]) };
        Ok(())
    }
}
