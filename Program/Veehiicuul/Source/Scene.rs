// Draws the game every frame: the grass, as the frame's background, then
// the track's ground over it, without a depth buffer, as it's flat and each
// part of it goes over the ones before, and then the car, painted with
// SimplePaint, with one. Every track's ground and every vehicle's materials
// are made once, at the start.

use graphics::{BACK_BUFFER_SRGB_FORMAT, DEPTH_FORMAT, Renderer, UploadBuffer};
use math::Matrix;
use meshes::car;
use simple_paint::direct3d12::{CONSTANT_BUFFER_ALIGNMENT, Mesh, OBJECT_SIZE, Pipeline, Target, materials_size};
use simple_paint::orthographic::{build_object_transforms, make_projection};
use simple_paint::{GpuMaterial, Material};
use windows_api::*;
use windows_core::{Error, Result};

use crate::camera;
use crate::game::Game;
use crate::ground;

static PAINT_VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintVertex.cso"));
static PAINT_PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintPixel.cso"));
static GROUND_VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/GroundVertex.cso"));
static GROUND_PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/GroundPixel.cso"));

// The car's materials, as SIMPLE_PAINT_MATERIAL_COUNT in Build.rs says.
const CAR_MATERIALS: usize = car::MATERIAL_COUNT as usize;
const CAR_MATERIALS_SIZE: usize = materials_size(CAR_MATERIALS);
// A track's ground colors, as Ground.hlsl takes them.
const COLORS_SIZE: usize = size_of::<[[f32; 4]; ground::MATERIAL_COUNT]>().next_multiple_of(CONSTANT_BUFFER_ALIGNMENT);
// The constants start with the ground's transforms, then the car's.
const GROUND_OBJECT: usize = 0;
const CAR_OBJECT: usize = OBJECT_SIZE;
const COLORS_OFFSET: usize = OBJECT_SIZE * 2;

pub struct Scene {
    paint: Pipeline,
    ground: Pipeline,
    car: Mesh,
    // Each track's ground.
    grounds: Vec<Mesh>,
    // The ground's and the car's transforms, then each track's colors, then
    // each vehicle's materials, each at a constant buffer's start. The GPU
    // has finished with them when a frame is ready.
    constants: UploadBuffer,
    materials_offset: usize,
}

fn invalid(error: simple_paint::Error) -> Error {
    Error::new(E_INVALIDARG, error.0)
}

// An sRGB channel, as linear.
fn linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

impl Scene {
    pub unsafe fn new(renderer: &mut Renderer, game: &Game) -> Result<Self> {
        unsafe {
            let device = renderer.device.clone();
            // The car's linear colors, and the ground's, are encoded to sRGB.
            let target = |depth_format| Target {
                format: BACK_BUFFER_SRGB_FORMAT,
                depth_format,
                sample_count: 1,
                cull_mode: D3D12_CULL_MODE_NONE,
            };
            let paint = Pipeline::new(&device, PAINT_VERTEX_SHADER, PAINT_PIXEL_SHADER, &target(Some(DEPTH_FORMAT)))?;
            let ground = Pipeline::new(&device, GROUND_VERTEX_SHADER, GROUND_PIXEL_SHADER, &target(None))?;
            let car = Mesh::new(renderer, &car::VERTICES, &car::INDICES, car::MATERIAL_COUNT)?;
            let mut grounds = Vec::with_capacity(game.tracks.len());
            for track in &game.tracks {
                let mesh = ground::mesh(track);
                grounds.push(Mesh::new(renderer, &mesh.vertices, &mesh.indices, ground::MATERIAL_COUNT as u32)?);
            }
            let materials_offset = COLORS_OFFSET + COLORS_SIZE * game.tracks.len();
            let size = materials_offset + CAR_MATERIALS_SIZE * game.vehicles.list.len();
            let constants = UploadBuffer::new(&device, size)?;
            for (index, track) in game.tracks.iter().enumerate() {
                let colors = ground::colors(track).map(|color| {
                    let [red, green, blue] = color.map(linear);
                    [red, green, blue, 1.0]
                });
                constants.write(COLORS_OFFSET + COLORS_SIZE * index, &colors);
            }
            for index in 0..game.vehicles.list.len() {
                let mut materials = [GpuMaterial::default(); CAR_MATERIALS];
                for (material, paint) in materials.iter_mut().zip(game.vehicles.paints(index)) {
                    *material = *Material::compile(&paint).map_err(invalid)?.constants();
                }
                constants.write(materials_offset + CAR_MATERIALS_SIZE * index, &materials);
            }
            Ok(Scene {
                paint,
                ground,
                car,
                grounds,
                constants,
                materials_offset,
            })
        }
    }

    // Records the game's draws into the frame, over the whole window.
    pub unsafe fn draw(&self, renderer: &Renderer, game: &Game) -> Result<()> {
        unsafe {
            let view = camera::view(game.camera_center());
            let aspect = renderer.width as f32 / renderer.height as f32;
            let height = game.camera.height as f32;
            let projection = make_projection(height * aspect, height, camera::NEAR, camera::FAR).map_err(invalid)?;
            let [x, z] = game.car.position.map(|value| value as f32);
            let car = Matrix::rotation_y(game.car.heading as f32) * Matrix::translation(x, 0.0, z);
            for (offset, world) in [(GROUND_OBJECT, Matrix::IDENTITY), (CAR_OBJECT, car)] {
                let object = build_object_transforms(&(world * view), &projection).map_err(invalid)?;
                self.constants.write(offset, &[object]);
            }

            // The grass's sRGB color, stored as it is.
            let [red, green, blue] = game.current_track().colors.grass;
            renderer.clear([red, green, blue, 1.0]);
            renderer.clear_depth();
            let list = &renderer.command_list;
            renderer.set_render_target(true, false);
            self.ground.bind(list);
            let colors = self.constants.address(COLORS_OFFSET + COLORS_SIZE * game.track);
            self.grounds[game.track].draw(list, self.constants.address(GROUND_OBJECT), colors);
            renderer.set_render_target(true, true);
            self.paint.bind(list);
            let materials = self.constants.address(self.materials_offset + CAR_MATERIALS_SIZE * game.vehicle);
            self.car.draw(list, self.constants.address(CAR_OBJECT), materials);
            Ok(())
        }
    }
}
