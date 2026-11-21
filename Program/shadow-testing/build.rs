// Compiles each way of drawing a shadow's shaders to DXIL at build time.
// Each includes Shaders/Common.hlsli.

use std::path::Path;

const WAYS: [&str; 3] = ["Linear", "Curved", "Gaussian"];

fn main() {
    for way in WAYS {
        shader_compiler::build_vertex_and_pixel(&Path::new("Shaders").join(format!("{way}.hlsl")), &["-O3", "-WX"], way);
    }
}
