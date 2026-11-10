// Compiles the shaders to DXIL at build time: SimplePaint's, for the
// modeler's materials, from where Library/SimplePaint says they are; the flat
// shader, for the same materials' colors; and the overlay's.

use std::path::{Path, PathBuf};

// Matches paint.rs' MAXIMUM_PAINTS.
const MATERIALS: &str = "16";

fn main() {
    let shaders = PathBuf::from(std::env::var("DEP_SIMPLE_PAINT_SHADERS").unwrap());
    let count = format!("SIMPLE_PAINT_MATERIAL_COUNT={MATERIALS}");
    shader_compiler::build_vertex_and_pixel(&shaders.join("SimplePaint.hlsl"), &["-O3", "-Ges", "-WX", "-D", &count], "SimplePaint");
    let count = format!("MATERIAL_COUNT={MATERIALS}");
    shader_compiler::build_vertex_and_pixel(Path::new("Shaders/Flat.hlsl"), &["-O3", "-WX", "-D", &count], "Flat");
    shader_compiler::build_vertex_and_pixel(Path::new("Shaders/Overlay.hlsl"), &["-O3", "-WX"], "Overlay");
}
