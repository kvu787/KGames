// Compiles SimplePaint's production shaders for the GPU test, with six
// materials, so that the test exercises the material array's stride.

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
            &[
                "-O3",
                "-Ges",
                "-WX",
                "-all_resources_bound",
                "-Qstrip_debug",
                "-Qstrip_reflect",
                "-D",
                "SIMPLE_PAINT_MATERIAL_COUNT=6",
            ],
            &output_directory.join(output),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    }
}
