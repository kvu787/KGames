// Compiles the tester's shader to DXIL at build time.

use std::path::Path;

fn main() {
    shader_compiler::build_vertex_and_pixel(Path::new("Shaders/PixelFont.hlsl"), &["-O3", "-WX"], "PixelFont");
}
