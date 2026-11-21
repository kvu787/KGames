// Compiles the shaders of the ways of drawing a shadow to DXIL at build time.

use std::path::Path;

fn main() {
    shader_compiler::build_vertex_and_pixel(Path::new("Shaders/Linear.hlsl"), &["-O3", "-WX"], "Linear");
}
