// Unlit.hlsl on the GPU, through SimplePaint's Pipeline and Mesh: a sphere,
// and a cube turned to show three faces, in two of three materials, on the
// preferred hardware adapter and on WARP. Unlit means a pixel shader that's
// equivalent to returning one value per material, so every pixel that they
// cover must be its material's color, the same whichever way its triangle
// faces. They're drawn through a frame's view that stores colors as they are,
// with sRGB colors, which must come out exactly, and through its sRGB view,
// with the same colors as linear, which it encodes back to within one step.
// It fails on D3D12 debug-layer errors when the debug layer is installed.

use graphics::{AdapterChoice, Gpu, Image, Offscreen, Readback, TARGET_FORMAT, TARGET_SRGB_FORMAT, UploadBuffer};
use math::Matrix;
use simple_paint::direct3d12::{Mesh, OBJECT_SIZE, Pipeline, Target, UnlitColor, unlit_colors_size};
use simple_paint::orthographic::{build_object_transforms, make_projection};
use windows_api::*;
use windows_core::Result;

static VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/UnlitVertex.cso"));
static PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/UnlitPixel.cso"));
// The shaders' UNLIT_COLOR_COUNT, from build.rs.
const COLOR_COUNT: usize = 3;
// sRGB. The sphere is in the first and the cube in the last, so that the
// colors' stride matters. The middle one is drawn nowhere.
const COLORS: [[u8; 3]; COLOR_COUNT] = [[0x33, 0x99, 0xCC], [0x12, 0x34, 0x56], [0xE0, 0x40, 0x20]];
const SPHERE: u32 = 0;
const CUBE: u32 = 2;
const BACKGROUND: [u8; 3] = [0, 0, 0];
// 64 pixels a unit: 4 units by 2, with the sphere in the left half and the
// cube in the right.
const WIDTH: u32 = 256;
const HEIGHT: u32 = 128;
const PIXELS_PER_UNIT: f32 = 64.0;
const SPHERE_RADIUS: f32 = 0.8;

// An sRGB channel, as linear.
fn linear(channel: u8) -> f32 {
    let value = channel as f64 / 255.0;
    let linear = if value <= 0.04045 { value / 12.92 } else { ((value + 0.055) / 1.055).powf(2.4) };
    linear as f32
}

// Draws the sphere and the cube into a new offscreen frame, through its sRGB
// view or the one that stores colors as they are, and reads it back.
fn draw(gpu: &mut Gpu, srgb: bool) -> Result<Image> {
    unsafe {
        let target = Target {
            format: if srgb { TARGET_SRGB_FORMAT } else { TARGET_FORMAT },
            depth_format: None,
            sample_count: 1,
            // The models are convex, so without their back faces, nothing
            // overlaps, and they need no depth buffer.
            cull_mode: D3D12_CULL_MODE_BACK,
        };
        let pipeline = Pipeline::new(&gpu.device, VERTEX_SHADER, PIXEL_SHADER, &target)?;
        let sphere = meshes::uv_sphere(32, 16, SPHERE);
        let cube = meshes::cuboid([1.0; 3], CUBE);
        let sphere = Mesh::new(gpu, &sphere.vertices, &sphere.indices, COLOR_COUNT as u32)?;
        let cube = Mesh::new(gpu, &cube.vertices, &cube.indices, COLOR_COUNT as u32)?;

        let view = Matrix::look_at_rh([0.0, 0.0, 5.0], [0.0; 3], [0.0, 1.0, 0.0]);
        let projection = make_projection(WIDTH as f32 / PIXELS_PER_UNIT, HEIGHT as f32 / PIXELS_PER_UNIT, 1.0, 10.0).unwrap();
        // The cube turns about Y to show its +Z and +X faces, then tips its
        // top toward the camera.
        let worlds = [
            Matrix::scaling(SPHERE_RADIUS, SPHERE_RADIUS, SPHERE_RADIUS) * Matrix::translation(-1.0, 0.0, 0.0),
            Matrix::rotation_y(0.7) * Matrix::rotation_x(0.5) * Matrix::translation(1.0, 0.0, 0.0),
        ];
        let colors_at = OBJECT_SIZE * worlds.len();
        let constants = UploadBuffer::new(&gpu.device, colors_at + unlit_colors_size(COLOR_COUNT))?;
        for (index, world) in worlds.iter().enumerate() {
            constants.write(OBJECT_SIZE * index, &[build_object_transforms(&(*world * view), &projection).unwrap()]);
        }
        let colors: [UnlitColor; COLOR_COUNT] = COLORS.map(|color| {
            let [red, green, blue] = color.map(|channel| if srgb { linear(channel) } else { channel as f32 / 255.0 });
            [red, green, blue, 1.0]
        });
        constants.write(colors_at, &colors);

        let mut offscreen = Offscreen::new(gpu, WIDTH, HEIGHT, [0.0, 0.0, 0.0, 1.0])?;
        let frame = offscreen.begin()?;
        frame.clear([0.0, 0.0, 0.0, 1.0]);
        frame.bind(srgb, None);
        pipeline.bind(&frame.list);
        sphere.draw(&frame.list, constants.gpu_address(0), constants.gpu_address(colors_at));
        cube.draw(&frame.list, constants.gpu_address(OBJECT_SIZE), constants.gpu_address(colors_at));
        let readback = Readback::record(&gpu.device, &frame)?;
        let fence = offscreen.submit(gpu, frame)?;
        let image = readback.read(gpu, fence)?;
        assert_eq!(gpu.report_messages(), 0, "the D3D12 debug layer reported errors");
        Ok(image)
    }
}

// Checks that every pixel is the background or, in the sphere's half or the
// cube's, one color, which is its material's, and returns how many pixels
// each covers.
fn check(image: &Image, srgb: bool) -> [usize; 2] {
    let names = ["sphere", "cube"];
    let mut counts = [0; 2];
    let mut drawn: [Option<[u8; 3]>; 2] = [None; 2];
    for y in 0..image.height {
        for x in 0..image.width {
            let [red, green, blue, alpha] = image.pixel(x, y);
            assert_eq!(alpha, 255, "at ({x}, {y})");
            let pixel = [red, green, blue];
            if pixel == BACKGROUND {
                continue;
            }
            let half = usize::from(x >= image.width / 2);
            let first = *drawn[half].get_or_insert(pixel);
            assert_eq!(pixel, first, "the {} has two colors, the second at ({x}, {y})", names[half]);
            let expected = COLORS[[SPHERE, CUBE][half] as usize];
            let off = pixel.iter().zip(expected).map(|(drawn, expected)| drawn.abs_diff(expected)).max().unwrap();
            assert!(off <= u8::from(srgb), "the {} is {pixel:?}, not {expected:?}", names[half]);
            counts[half] += 1;
        }
    }
    counts
}

fn main() {
    for adapter in [AdapterChoice::HighPerformance, AdapterChoice::Warp] {
        let mut gpu = Gpu::new(adapter).unwrap();
        for srgb in [false, true] {
            let image = draw(&mut gpu, srgb).unwrap();
            let [sphere, cube] = check(&image, srgb);
            // A disc of the sphere's radius, which its 32 sides cut a little
            // short of.
            let disc = std::f32::consts::PI * (SPHERE_RADIUS * PIXELS_PER_UNIT).powi(2);
            assert!((disc * 0.97..=disc * 1.01).contains(&(sphere as f32)), "the sphere covers {sphere} pixels, not about {disc}");
            // More than one face's worth.
            assert!(cube > (PIXELS_PER_UNIT * PIXELS_PER_UNIT) as usize * 5 / 4, "the cube covers {cube} pixels");
            let view = if srgb { "sRGB view" } else { "view that stores colors as they are" };
            println!("{}: through the {view}, the sphere's {sphere} pixels and the cube's {cube} are each one color, its material's.", gpu.name);
        }
    }
}
