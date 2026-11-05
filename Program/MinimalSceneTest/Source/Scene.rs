// The scene, drawn with SimplePaint: a sphere that moves back and forth along
// X, above a ground plane, past a cube, seen by an orthographic camera from
// above and to the side.

use graphics::{DEPTH_FORMAT, Frame, Gpu, TARGET_SRGB_FORMAT, UploadBuffer};
use math::Matrix;
use simple_paint::direct3d12::{Mesh, OBJECT_SIZE, Pipeline, Target, materials_size};
use simple_paint::orthographic::{Projection, build_object_transforms, make_projection};
use simple_paint::{GpuMaterial, Material, Parameters};
use windows_api::*;
use windows_core::{Error, Result};

use crate::depth::DepthBuffer;
use crate::settings::SceneSettings;

static VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintVertex.cso"));
static PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintPixel.cso"));

// The shaders' SIMPLE_PAINT_MATERIAL_COUNT, from Build.rs: the sphere's,
// the ground's and the cube's, in that order.
const MATERIAL_COUNT: usize = 3;
const OBJECT_COUNT: usize = 3;

// The sphere's units per second, and how far from the middle it turns back.
const SPEED: f32 = 8.0;
pub const TRAVEL_DISTANCE: f32 = 7.0;
// Of a frame's time, at most this many seconds count, so that the sphere
// doesn't jump ahead when frames stopped for a while, like while the window
// was minimized, or showed what's wrong with its setup instead of the scene.
pub const LONGEST_FRAME: f32 = 0.1;
// Its mesh has 64 vertices around each ring, and 32 bands from pole to pole.
pub const SPHERE_RADIUS: f32 = 0.5;

// The camera looks at the origin from 5 units away, 47.5 degrees above the
// horizon and 30 degrees from +Z toward +X: an orthographic view 5 units
// tall, from 1 to 20 units in front of it.
const CAMERA_BACKWARD: [f32; 3] = [0.3377951, 0.7372773, 0.5850783];
const CAMERA_UP: [f32; 3] = [-0.36863866, 0.6755902, -0.63850087];
const CAMERA_DISTANCE: f32 = 5.0;
pub const VIEW_HEIGHT: f32 = 5.0;

// #4c4c4c, stored as it is.
pub const BACKGROUND: [f32; 4] = [76.0 / 255.0, 76.0 / 255.0, 76.0 / 255.0, 1.0];

// Where the sphere is along X, and which way it's moving: +1 or -1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    pub position: f32,
    pub direction: f32,
}

impl Default for Motion {
    // The sphere starts in the middle, moving toward +X.
    fn default() -> Self {
        Self {
            position: 0.0,
            direction: 1.0,
        }
    }
}

impl Motion {
    // Moves the sphere by a frame's time since the last frame, of which
    // LONGEST_FRAME at most counts.
    pub fn advance_frame(&mut self, seconds: f32) {
        self.advance(seconds.min(LONGEST_FRAME));
    }

    // Moves the sphere by `seconds`. Past the travel distance, it bounces
    // back by as much as it went past, as many times as the time takes it,
    // so that no time takes it off its path.
    pub fn advance(&mut self, seconds: f32) {
        self.position += self.direction * SPEED * seconds;
        while self.position.abs() > TRAVEL_DISTANCE {
            self.position = 2.0 * TRAVEL_DISTANCE.copysign(self.position) - self.position;
            self.direction = -self.direction;
        }
    }
}

// The camera's view and projection, for a frame of the size.
fn camera(width: f32, height: f32) -> Result<(Matrix, Projection)> {
    let projection = make_projection(VIEW_HEIGHT * width / height, VIEW_HEIGHT, 1.0, 20.0)
        .map_err(|error| Error::new(E_INVALIDARG, error.0))?;
    let eye = CAMERA_BACKWARD.map(|component| component * CAMERA_DISTANCE);
    Ok((Matrix::look_at_rh(eye, [0.0; 3], CAMERA_UP), projection))
}

// Where a point of the world shows in a frame of the size, in pixels from
// its top left corner.
pub fn to_screen(point: [f32; 3], width: f32, height: f32) -> [f32; 2] {
    let (view, projection) = camera(width, height).expect("the camera fits any frame");
    let [x, y, _] = view.transform_point(point);
    [
        (1.0 + x * projection.scale[0]) * width / 2.0,
        (1.0 - y * projection.scale[1]) * height / 2.0,
    ]
}

// Where the sphere's middle is, with it at `x` along X.
pub fn sphere_center(x: f32) -> [f32; 3] {
    [x, SPHERE_RADIUS, 0.0]
}

pub struct Scene {
    pipeline: Pipeline,
    // The sphere, the ground and the cube.
    meshes: [Mesh; OBJECT_COUNT],
    // Each object's transforms, then the materials, each at a constant
    // buffer's start. The GPU has finished with them when a frame begins.
    constants: UploadBuffer,
    materials: [GpuMaterial; MATERIAL_COUNT],
    depth: DepthBuffer,
}

impl Scene {
    pub fn new(gpu: &mut Gpu, settings: &SceneSettings) -> Result<Self> {
        unsafe {
            let pipeline = Pipeline::new(
                &gpu.device,
                VERTEX_SHADER,
                PIXEL_SHADER,
                &Target {
                    format: TARGET_SRGB_FORMAT,
                    depth_format: Some(DEPTH_FORMAT),
                    sample_count: 1,
                    cull_mode: D3D12_CULL_MODE_NONE,
                },
            )?;
            let mut upload = |mesh: meshes::Mesh| Mesh::new(gpu, &mesh.vertices, &mesh.indices, MATERIAL_COUNT as u32);
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
                constants: UploadBuffer::new(&gpu.device, OBJECT_SIZE * OBJECT_COUNT + materials_size(MATERIAL_COUNT))?,
                materials: [compile(&settings.sphere)?, compile(&settings.ground)?, compile(&settings.cube)?],
                depth: DepthBuffer::new(gpu)?,
            })
        }
    }

    // Records the scene into the frame, over all of it, with the sphere at
    // `sphere_x` along X.
    pub fn draw(&mut self, gpu: &Gpu, frame: &Frame, sphere_x: f32) -> Result<()> {
        let (view, projection) = camera(frame.width as f32, frame.height as f32)?;
        let worlds = [
            Matrix::scaling(SPHERE_RADIUS, SPHERE_RADIUS, SPHERE_RADIUS) * Matrix::translation(sphere_x, SPHERE_RADIUS, 0.0),
            Matrix::IDENTITY,
            Matrix::translation(0.0, 0.5, -1.5),
        ];
        for (index, world) in worlds.iter().enumerate() {
            let object = build_object_transforms(&(*world * view), &projection)
                .map_err(|error| Error::new(E_INVALIDARG, error.0))?;
            self.constants.write(OBJECT_SIZE * index, &[object]);
        }
        let materials_offset = OBJECT_SIZE * OBJECT_COUNT;
        self.constants.write(materials_offset, &self.materials);

        unsafe {
            frame.clear(BACKGROUND);
            let depth = self.depth.clear(gpu, frame)?;
            // SimplePaint's colors are linear, which the sRGB view encodes.
            frame.bind(true, Some(depth));
            self.pipeline.bind(&frame.list);
            for (index, mesh) in self.meshes.iter().enumerate() {
                mesh.draw(
                    &frame.list,
                    self.constants.gpu_address(OBJECT_SIZE * index),
                    self.constants.gpu_address(materials_offset),
                );
            }
        }
        Ok(())
    }
}
