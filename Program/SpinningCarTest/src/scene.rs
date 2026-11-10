// The scene: the green ground and the red cube, in flat colors, then a car
// that moves back and forth along X at 8 units per second while it turns at
// 90 degrees per second, and a sphere that stays put, both painted with
// SimplePaint, all with one depth buffer. Every frame draws all of it.

use graphics::{DEPTH_FORMAT, TARGET_FORMAT, TARGET_SRGB_FORMAT, DepthBuffer, Frame, Gpu, UploadBuffer};
use math::Matrix;
use meshes::car as car_mesh;
use simple_paint::direct3d12::{CONSTANT_BUFFER_ALIGNMENT, Mesh, OBJECT_SIZE, Pipeline, Target, materials_size};
use simple_paint::orthographic::{Projection, build_object_transforms, make_projection};
use simple_paint::{GpuMaterial, Material};
use user_interface::Rect;
use windows_api::*;
use windows_core::{Error, Result};

use crate::settings::SceneSettings;

static PAINT_VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintVertex.cso"));
static PAINT_PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintPixel.cso"));
static FLAT_VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/FlatShadingVertex.cso"));
static FLAT_PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/FlatShadingPixel.cso"));

// The car's five materials, then the sphere's, as SIMPLE_PAINT_MATERIAL_COUNT
// in build.rs says.
const MATERIAL_COUNT: usize = 6;
const SPHERE_MATERIAL: u32 = car_mesh::MATERIAL_COUNT;
// The ground's color, then the cube's, as FlatShading.hlsl's MaterialCount
// says.
const FLAT_MATERIAL_COUNT: usize = 2;
// The car, the sphere, the ground and the cube.
const OBJECT_COUNT: usize = 4;
// Where the constants have the materials, after each object's transforms,
// and the flat shading, after the materials.
const MATERIALS_OFFSET: usize = OBJECT_SIZE * OBJECT_COUNT;
const SHADING_OFFSET: usize = MATERIALS_OFFSET + materials_size(MATERIAL_COUNT);
const SHADING_SIZE: usize = size_of::<Shading>().next_multiple_of(CONSTANT_BUFFER_ALIGNMENT);
// The camera looks down at the origin from 45 degrees above the horizon, and
// 45 degrees from +Z toward +X, 5 units away, at a view 5 units tall.
const CAMERA_PITCH: f32 = std::f32::consts::FRAC_PI_4;
const CAMERA_YAW: f32 = std::f32::consts::FRAC_PI_4;
const CAMERA_DISTANCE: f32 = 5.0;
pub const VIEW_HEIGHT: f32 = 5.0;
const CAR_SCALE: f32 = 0.25;
// Units per second, and how far from the middle the car turns back.
const SPEED: f64 = 8.0;
const TRAVEL_DISTANCE: f64 = 7.0;
// Radians per second.
const TURN_SPEED: f64 = std::f64::consts::FRAC_PI_2;
pub const SPHERE_RADIUS: f32 = 0.4;
// Where the sphere's middle is.
pub const SPHERE_CENTER: [f32; 3] = [1.5, SPHERE_RADIUS, -1.5];
// Along X and Z.
const GROUND_SIZE: [f32; 2] = [10.0, 5.0];
const CUBE_SIZE: f32 = 1.0;
// sRGB, like the C++ scene's.
const GROUND_COLOR: [f32; 4] = [0.08900002, 0.89, 0.48950002, 1.0];
const CUBE_COLOR: [f32; 4] = [0.88, 0.1672, 0.17907982, 1.0];
// Toward the light, which is above, between +X and +Z, as a unit vector.
const LIGHT: [f32; 3] = [0.5, std::f32::consts::FRAC_1_SQRT_2, 0.5];
// #4c4c4c, stored as it is.
pub const BACKGROUND: [f32; 4] = [76.0 / 255.0, 76.0 / 255.0, 76.0 / 255.0, 1.0];

// What FlatShading.hlsl binds at b1.
#[repr(C)]
#[derive(Clone, Copy)]
struct Shading {
    // Toward the light, in view space. W isn't used.
    light: [f32; 4],
    colors: [[f32; 4]; FLAT_MATERIAL_COUNT],
}

// Where the car is along X, and how far it has turned, `seconds` after the
// program started. It goes from the middle to +7, back to -7, and again, 28
// units a cycle. S makes it `stationary`.
pub fn car_placement(seconds: f64, stationary: bool) -> (f32, f32) {
    if stationary {
        // The mesh faces +Z. Facing +X puts the headlights at the lower
        // right in the fixed 45-degree camera.
        return (0.0, std::f32::consts::FRAC_PI_2);
    }
    let phase = (seconds * SPEED + TRAVEL_DISTANCE) % (TRAVEL_DISTANCE * 4.0);
    let position = TRAVEL_DISTANCE - (phase - TRAVEL_DISTANCE * 2.0).abs();
    let turn = (seconds * TURN_SPEED) % std::f64::consts::TAU;
    (position as f32, turn as f32)
}

fn car_world((position, turn): (f32, f32)) -> Matrix {
    Matrix::scaling(CAR_SCALE, CAR_SCALE, CAR_SCALE) * Matrix::rotation_y(turn) * Matrix::translation(position, 0.0, 0.0)
}

// The camera's view and projection, for a frame of the size.
fn camera(width: f32, height: f32) -> Result<(Matrix, Projection)> {
    let eye = [
        CAMERA_DISTANCE * CAMERA_PITCH.cos() * CAMERA_YAW.sin(),
        CAMERA_DISTANCE * CAMERA_PITCH.sin(),
        CAMERA_DISTANCE * CAMERA_PITCH.cos() * CAMERA_YAW.cos(),
    ];
    let view = Matrix::look_at_rh(eye, [0.0; 3], [0.0, 1.0, 0.0]);
    let projection = make_projection(VIEW_HEIGHT * width / height, VIEW_HEIGHT, 1.0, 20.0)
        .map_err(|error| Error::new(E_INVALIDARG, error.0))?;
    Ok((view, projection))
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

// The rectangle that the car covers in a frame of the size, where
// car_placement put it.
pub fn car_on_screen(placement: (f32, f32), width: f32, height: f32) -> Rect {
    let world = car_world(placement);
    let (mut left, mut top, mut right, mut bottom) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for vertex in &car_mesh::VERTICES {
        let [x, y] = to_screen(world.transform_point(vertex.position), width, height);
        (left, top, right, bottom) = (left.min(x), top.min(y), right.max(x), bottom.max(y));
    }
    Rect::new(left, top, right - left, bottom - top)
}

pub struct Scene {
    paint: Pipeline,
    flat: Pipeline,
    car: Mesh,
    sphere: Mesh,
    ground: Mesh,
    cube: Mesh,
    // Each object's transforms, then the materials, then the flat shading,
    // each at a constant buffer's start. The GPU has finished with them when
    // a frame begins.
    constants: UploadBuffer,
    materials: [GpuMaterial; MATERIAL_COUNT],
    depth: DepthBuffer,
}

impl Scene {
    pub fn new(gpu: &mut Gpu, settings: &SceneSettings) -> Result<Self> {
        unsafe {
            let target = |format| Target {
                format,
                depth_format: Some(DEPTH_FORMAT),
                sample_count: 1,
                cull_mode: D3D12_CULL_MODE_NONE,
            };
            let paint = Pipeline::new(&gpu.device, PAINT_VERTEX_SHADER, PAINT_PIXEL_SHADER, &target(TARGET_SRGB_FORMAT))?;
            // The flat colors are already sRGB, so they're stored as they are.
            let flat = Pipeline::new(&gpu.device, FLAT_VERTEX_SHADER, FLAT_PIXEL_SHADER, &target(TARGET_FORMAT))?;
            let car = Mesh::new(gpu, &car_mesh::VERTICES, &car_mesh::INDICES, MATERIAL_COUNT as u32)?;
            let mut upload = |mesh: meshes::Mesh, material_count: usize| {
                Mesh::new(gpu, &mesh.vertices, &mesh.indices, material_count as u32)
            };
            let [longitudes, latitudes] = settings.sphere_resolution;
            let sphere = upload(meshes::uv_sphere(longitudes, latitudes, SPHERE_MATERIAL), MATERIAL_COUNT)?;
            let ground = upload(meshes::plane(GROUND_SIZE, 0), FLAT_MATERIAL_COUNT)?;
            let cube = upload(meshes::cuboid([CUBE_SIZE; 3], 1), FLAT_MATERIAL_COUNT)?;
            let mut materials = [GpuMaterial::default(); MATERIAL_COUNT];
            for (material, parameters) in materials.iter_mut().zip(&settings.materials) {
                *material = *Material::compile(parameters)
                    .map_err(|error| Error::new(E_INVALIDARG, error.0))?
                    .constants();
            }

            Ok(Self {
                paint,
                flat,
                car,
                sphere,
                ground,
                cube,
                constants: UploadBuffer::new(&gpu.device, SHADING_OFFSET + SHADING_SIZE)?,
                materials,
                depth: DepthBuffer::new(gpu)?,
            })
        }
    }

    // Records the scene into the frame, over all of it, with the car where
    // car_placement put it.
    pub fn draw(&mut self, gpu: &Gpu, frame: &Frame, placement: (f32, f32)) -> Result<()> {
        let (view, projection) = camera(frame.width as f32, frame.height as f32)?;
        let worlds = [
            car_world(placement),
            Matrix::scaling(SPHERE_RADIUS, SPHERE_RADIUS, SPHERE_RADIUS)
                * Matrix::translation(SPHERE_CENTER[0], SPHERE_CENTER[1], SPHERE_CENTER[2]),
            Matrix::IDENTITY,
            Matrix::translation(0.0, CUBE_SIZE * 0.5, -1.5),
        ];
        for (index, world) in worlds.iter().enumerate() {
            let object = build_object_transforms(&(*world * view), &projection)
                .map_err(|error| Error::new(E_INVALIDARG, error.0))?;
            self.constants.write(OBJECT_SIZE * index, &[object]);
        }
        self.constants.write(MATERIALS_OFFSET, &self.materials);
        let [x, y, z] = view.transform_direction(LIGHT);
        let shading = Shading {
            light: [x, y, z, 0.0],
            colors: [GROUND_COLOR, CUBE_COLOR],
        };
        self.constants.write(SHADING_OFFSET, &[shading]);

        unsafe {
            frame.clear(BACKGROUND);
            let depth = self.depth.clear(gpu, frame)?;
            let list = &frame.list;
            let object = |index| self.constants.gpu_address(OBJECT_SIZE * index);
            // The ground and the cube, in their flat colors.
            frame.bind(false, Some(depth));
            self.flat.bind(list);
            let shading = self.constants.gpu_address(SHADING_OFFSET);
            self.ground.draw(list, object(2), shading);
            self.cube.draw(list, object(3), shading);
            // The car and the sphere, whose linear colors are encoded to
            // sRGB, in front of or behind them.
            frame.bind(true, Some(depth));
            self.paint.bind(list);
            let materials = self.constants.gpu_address(MATERIALS_OFFSET);
            self.car.draw(list, object(0), materials);
            self.sphere.draw(list, object(1), materials);
        }
        Ok(())
    }
}
