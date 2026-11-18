// Compiles the shaders to DXIL at build time: SimplePaint's and its unlit
// one, for the modeler's paints, from where Library/simple-paint says they
// are, and the overlay's.

use std::path::{Path, PathBuf};

// Matches paint.rs' MAXIMUM_PAINTS.
const MATERIALS: &str = "16";

fn main() {
    let shaders = PathBuf::from(std::env::var("DEP_SIMPLE_PAINT_SHADERS").unwrap());
    let count = format!("SIMPLE_PAINT_MATERIAL_COUNT={MATERIALS}");
    shader_compiler::build_vertex_and_pixel(&shaders.join("SimplePaint.hlsl"), &["-O3", "-Ges", "-WX", "-D", &count], "SimplePaint");
    let count = format!("UNLIT_COLOR_COUNT={MATERIALS}");
    shader_compiler::build_vertex_and_pixel(&shaders.join("Unlit.hlsl"), &["-O3", "-WX", "-D", &count], "Unlit");
    shader_compiler::build_vertex_and_pixel(Path::new("Shaders/Overlay.hlsl"), &["-O3", "-WX"], "Overlay");
}
