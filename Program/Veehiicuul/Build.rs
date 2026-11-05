// Compiles the shaders to DXIL at build time: SimplePaint's, for the 16
// materials that a model has at most, from where Library/SimplePaint says
// they are, and the ground's.

use std::path::{Path, PathBuf};

fn main() {
    let shaders = PathBuf::from(std::env::var("DEP_SIMPLE_PAINT_SHADERS").unwrap());
    shader_compiler::build_vertex_and_pixel(
        &shaders.join("SimplePaint.hlsl"),
        &["-O3", "-Ges", "-WX", "-D", "SIMPLE_PAINT_MATERIAL_COUNT=16"],
        "SimplePaint",
    );
    shader_compiler::build_vertex_and_pixel(Path::new("Shaders/Ground.hlsl"), &["-O3", "-WX"], "Ground");
}
