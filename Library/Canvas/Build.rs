// Compiles the canvas's shaders to DXIL at build time.

use std::path::Path;

fn main() {
    shader_compiler::build_vertex_and_pixel(Path::new("Shaders/Canvas.hlsl"), &[], "Canvas");
}
