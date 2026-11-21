// Compiles each way of drawing a shadow's shaders to DXIL at build time.
// Each includes Shaders/Common.hlsli.

use std::path::Path;

const WAYS: [&str; 9] = ["Linear", "Curved", "Smoothstep", "Exponential", "Bell", "Gaussian", "RoundGaussian", "BSpline", "Layered"];

fn main() {
    for way in WAYS {
        shader_compiler::build_vertex_and_pixel(&Path::new("Shaders").join(format!("{way}.hlsl")), &["-O3", "-WX"], way);
    }
}
