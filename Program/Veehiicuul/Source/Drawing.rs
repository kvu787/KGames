// Draws the 3D view every frame: the background's color, then what's flat on
// the ground, without a depth buffer, as each part goes over the ones before
// (the terrain, then the track), and then the models, painted with
// SimplePaint, with one: the car, the parked vehicles and the decorations.
//
// Each model is copied to the GPU once, at the start. The terrain and the
// track change as the editors change them, so their meshes are in upload
// buffers, which are written again when they change.

use graphics::{BACK_BUFFER_SRGB_FORMAT, DEPTH_FORMAT, Renderer, UploadBuffer};
use math::Matrix;
use meshes::Mesh as Triangles;
use simple_paint::direct3d12::{CONSTANT_BUFFER_ALIGNMENT, Mesh, OBJECT_SIZE, Pipeline, Target, materials_size};
use simple_paint::orthographic::{build_object_transforms, make_projection};
use simple_paint::{GpuMaterial, Material, Parameters, Vertex};
use windows_api::*;
use windows_core::{Error, Result};

use crate::camera::{self, Eye};
use crate::ground;
use crate::models::{MOST_SLOTS, Models};
use crate::terrain::Terrain;
use crate::track::{Shape, Track};

static PAINT_VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintVertex.cso"));
static PAINT_PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintPixel.cso"));
static GROUND_VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/GroundVertex.cso"));
static GROUND_PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/GroundPixel.cso"));

// A model's materials, as SIMPLE_PAINT_MATERIAL_COUNT in Build.rs says.
const MATERIALS_SIZE: usize = materials_size(MOST_SLOTS);
// The ground's colors, as Ground.hlsl takes them.
const COLORS_SIZE: usize = size_of::<[[f32; 4]; ground::COLOR_COUNT]>().next_multiple_of(CONSTANT_BUFFER_ALIGNMENT);
// The constants start with the ground's transforms and colors, then each
// model drawn's transforms and materials.
const COLORS_OFFSET: usize = OBJECT_SIZE;
const INSTANCES_OFFSET: usize = COLORS_OFFSET + COLORS_SIZE;
const INSTANCE_SIZE: usize = OBJECT_SIZE + MATERIALS_SIZE;
// The most models a frame draws.
pub const MOST_INSTANCES: usize = 512;

// A model drawn somewhere: turned to `heading`, in radians from +Z toward
// +X, and scaled the same along every axis, with its slots' materials.
pub struct Instance {
    pub model: usize,
    pub position: [f64; 2],
    pub heading: f64,
    pub scale: f64,
    pub materials: Vec<Parameters>,
}

// What's flat on the ground.
pub struct Ground<'a> {
    pub terrain: &'a Terrain,
    pub track: &'a Track,
    pub shape: &'a Shape,
}

// What a frame's 3D view shows.
pub struct View<'a> {
    pub eye: Eye,
    // As sRGB.
    pub background: [f32; 3],
    pub ground: Option<Ground<'a>>,
    pub instances: &'a [Instance],
}

// A mesh that changes, in an upload buffer, which the GPU reads where it is.
#[derive(Default)]
struct Changing {
    buffer: Option<UploadBuffer>,
    vertex_bytes: usize,
    index_count: u32,
}

impl Changing {
    unsafe fn set(&mut self, device: &ID3D12Device, mesh: &Triangles) -> Result<()> {
        let vertex_bytes = size_of_val(mesh.vertices.as_slice());
        let size = vertex_bytes + size_of_val(mesh.indices.as_slice());
        unsafe {
            if self.buffer.as_ref().is_none_or(|buffer| buffer.size < size) {
                // Room to grow, so that painting doesn't make one each time.
                self.buffer = Some(UploadBuffer::new(device, (size * 2).max(65536))?);
            }
            let buffer = self.buffer.as_ref().unwrap();
            buffer.write(0, &mesh.vertices);
            buffer.write(vertex_bytes, &mesh.indices);
        }
        self.vertex_bytes = vertex_bytes;
        self.index_count = mesh.indices.len() as u32;
        Ok(())
    }

    unsafe fn draw(&self, list: &ID3D12GraphicsCommandList, object: D3D12_GPU_VIRTUAL_ADDRESS, colors: D3D12_GPU_VIRTUAL_ADDRESS) {
        let Some(buffer) = &self.buffer else {
            return;
        };
        if self.index_count == 0 {
            return;
        }
        unsafe {
            list.SetGraphicsRootConstantBufferView(0, object);
            list.SetGraphicsRootConstantBufferView(1, colors);
            list.IASetVertexBuffers(
                0,
                Some(&[D3D12_VERTEX_BUFFER_VIEW {
                    BufferLocation: buffer.address(0),
                    SizeInBytes: self.vertex_bytes as u32,
                    StrideInBytes: size_of::<Vertex>() as u32,
                }]),
            );
            list.IASetIndexBuffer(Some(&D3D12_INDEX_BUFFER_VIEW {
                BufferLocation: buffer.address(self.vertex_bytes),
                SizeInBytes: self.index_count * 4,
                Format: DXGI_FORMAT_R32_UINT,
            }));
            list.DrawIndexedInstanced(self.index_count, 1, 0, 0, 0);
        }
    }
}

pub struct Drawing {
    paint: Pipeline,
    ground: Pipeline,
    // Each model's mesh, in the models' order.
    models: Vec<Mesh>,
    terrain: Changing,
    track: Changing,
    // What the terrain's and the track's meshes were made from.
    terrain_of: Option<Terrain>,
    track_of: Option<Track>,
    constants: UploadBuffer,
    // Each instance's materials as of the last frame, and as the GPU takes
    // them, so that they're compiled again only when they change.
    compiled: Vec<(Vec<Parameters>, [GpuMaterial; MOST_SLOTS])>,
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

impl Drawing {
    pub unsafe fn new(renderer: &mut Renderer, models: &Models) -> Result<Self> {
        unsafe {
            let device = renderer.device.clone();
            // The models' linear colors, and the ground's, are encoded to
            // sRGB.
            let target = |depth_format| Target {
                format: BACK_BUFFER_SRGB_FORMAT,
                depth_format,
                sample_count: 1,
                cull_mode: D3D12_CULL_MODE_NONE,
            };
            let paint = Pipeline::new(&device, PAINT_VERTEX_SHADER, PAINT_PIXEL_SHADER, &target(Some(DEPTH_FORMAT)))?;
            let ground = Pipeline::new(&device, GROUND_VERTEX_SHADER, GROUND_PIXEL_SHADER, &target(None))?;
            let mut meshes = Vec::with_capacity(models.list.len());
            for model in &models.list {
                meshes.push(Mesh::new(renderer, &model.vertices, &model.indices, MOST_SLOTS as u32)?);
            }
            Ok(Drawing {
                paint,
                ground,
                models: meshes,
                terrain: Changing::default(),
                track: Changing::default(),
                terrain_of: None,
                track_of: None,
                constants: UploadBuffer::new(&device, INSTANCES_OFFSET + INSTANCE_SIZE * MOST_INSTANCES)?,
                compiled: Vec::new(),
            })
        }
    }

    // Records the view's draws into the frame, over the whole window.
    pub unsafe fn draw(&mut self, renderer: &Renderer, view: &View) -> Result<()> {
        unsafe {
            let device = renderer.device.clone();
            let eye_view = view.eye.view();
            let aspect = renderer.width as f32 / renderer.height as f32;
            let height = view.eye.height as f32;
            let projection = make_projection(height * aspect, height, camera::NEAR, camera::FAR).map_err(invalid)?;
            let transforms = |world: Matrix| build_object_transforms(&(world * eye_view), &projection).map_err(invalid);

            let [red, green, blue] = view.background;
            renderer.clear([red, green, blue, 1.0]);
            renderer.clear_depth();
            let list = &renderer.command_list;
            if let Some(ground) = &view.ground {
                if self.terrain_of.as_ref() != Some(ground.terrain) {
                    self.terrain.set(&device, &ground::terrain_mesh(ground.terrain))?;
                    self.terrain_of = Some(ground.terrain.clone());
                }
                if self.track_of.as_ref() != Some(ground.track) {
                    self.track.set(&device, &ground::track_mesh(ground.track, ground.shape))?;
                    self.track_of = Some(ground.track.clone());
                }
                self.constants.write(0, &[transforms(Matrix::IDENTITY)?]);
                let colors = ground::colors(ground.terrain, ground.track).map(|color| {
                    let [red, green, blue] = color.map(linear);
                    [red, green, blue, 1.0]
                });
                self.constants.write(COLORS_OFFSET, &colors);
                renderer.set_render_target(true, false);
                self.ground.bind(list);
                let [object, colors] = [self.constants.address(0), self.constants.address(COLORS_OFFSET)];
                self.terrain.draw(list, object, colors);
                self.track.draw(list, object, colors);
            }

            renderer.set_render_target(true, true);
            self.paint.bind(list);
            for (index, instance) in view.instances.iter().take(MOST_INSTANCES).enumerate() {
                let Some(mesh) = self.models.get(instance.model) else {
                    continue;
                };
                let scale = instance.scale as f32;
                let [x, z] = instance.position.map(|value| value as f32);
                let world = Matrix::scaling(scale, scale, scale)
                    * Matrix::rotation_y(instance.heading as f32)
                    * Matrix::translation(x, 0.0, z);
                let offset = INSTANCES_OFFSET + INSTANCE_SIZE * index;
                self.constants.write(offset, &[transforms(world)?]);
                if self.compiled.len() <= index {
                    self.compiled.push((Vec::new(), [GpuMaterial::default(); MOST_SLOTS]));
                }
                let (parameters, materials) = &mut self.compiled[index];
                if *parameters != instance.materials {
                    for (material, parameters) in materials.iter_mut().zip(&instance.materials) {
                        *material = *Material::compile(parameters).map_err(invalid)?.constants();
                    }
                    *parameters = instance.materials.clone();
                }
                self.constants.write(offset + OBJECT_SIZE, materials.as_slice());
                mesh.draw(list, self.constants.address(offset), self.constants.address(offset + OBJECT_SIZE));
            }
            Ok(())
        }
    }
}
