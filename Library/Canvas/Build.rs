// Compiles the canvas's shaders to DXIL at build time.

use std::path::{Path, PathBuf};

fn main() {
    let shader = Path::new("Shaders/Canvas.hlsl");
    shader_compiler::rerun_if_changed(shader);
    let output_directory = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    for (entry_point, profile, output) in [
        ("VSMain", "vs_6_0", "CanvasVertex.cso"),
        ("PSMain", "ps_6_0", "CanvasPixel.cso"),
    ] {
        shader_compiler::compile(
            shader,
            entry_point,
            profile,
            &[],
            &output_directory.join(output),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    }
}
