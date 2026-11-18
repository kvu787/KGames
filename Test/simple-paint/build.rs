// Compiles SimplePaint's production shaders for the GPU tests: SimplePaint's
// with six materials, so that the test exercises the material array's
// stride, and the unlit one with three colors, for the same reason.

use std::path::PathBuf;

fn main() {
    let shaders = PathBuf::from(std::env::var("DEP_SIMPLE_PAINT_SHADERS").unwrap());
    shader_compiler::build_vertex_and_pixel(
        &shaders.join("SimplePaint.hlsl"),
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
        "SimplePaint",
    );
    shader_compiler::build_vertex_and_pixel(&shaders.join("Unlit.hlsl"), &["-O3", "-WX", "-D", "UNLIT_COLOR_COUNT=3"], "Unlit");
}
