// Compiles SimplePaint's shaders, for the scene's three materials, to DXIL at
// build time. Library/SimplePaint says where they are.

use std::path::PathBuf;

fn main() {
    let shaders = PathBuf::from(std::env::var("DEP_SIMPLE_PAINT_SHADERS").unwrap());
    shader_compiler::build_vertex_and_pixel(
        &shaders.join("SimplePaint.hlsl"),
        &["-O3", "-Ges", "-WX", "-D", "SIMPLE_PAINT_MATERIAL_COUNT=3"],
        "SimplePaint",
    );
}
