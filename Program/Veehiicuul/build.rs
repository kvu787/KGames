// Compiles the shaders to DXIL at build time, from where Library/SimplePaint
// says they are: SimplePaint's, for the 16 materials that a model has at
// most, and the unlit one, for the ground's colors, as many as ground.rs'
// COLOR_COUNT.

use std::path::PathBuf;

fn main() {
    let shaders = PathBuf::from(std::env::var("DEP_SIMPLE_PAINT_SHADERS").unwrap());
    shader_compiler::build_vertex_and_pixel(
        &shaders.join("SimplePaint.hlsl"),
        &["-O3", "-Ges", "-WX", "-D", "SIMPLE_PAINT_MATERIAL_COUNT=16"],
        "SimplePaint",
    );
    shader_compiler::build_vertex_and_pixel(&shaders.join("Unlit.hlsl"), &["-O3", "-WX", "-D", "UNLIT_COLOR_COUNT=72"], "Unlit");
}
