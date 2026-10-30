// Compiles SimplePaint's shaders, for the scene's three materials, to DXIL at
// build time. Library/SimplePaint says where they are.

use std::path::PathBuf;

fn main() {
    let shaders = PathBuf::from(std::env::var("DEP_SIMPLE_PAINT_SHADERS").unwrap());
    let source = shaders.join("SimplePaint.hlsl");
    shader_compiler::rerun_if_changed(&source);
    let output_directory = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    for (entry_point, profile, output) in [
        ("VSMain", "vs_6_0", "SimplePaintVertex.cso"),
        ("PSMain", "ps_6_0", "SimplePaintPixel.cso"),
    ] {
        shader_compiler::compile(
            &source,
            entry_point,
            profile,
            &["-O3", "-Ges", "-WX", "-D", "SIMPLE_PAINT_MATERIAL_COUNT=3"],
            &output_directory.join(output),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    }
}
