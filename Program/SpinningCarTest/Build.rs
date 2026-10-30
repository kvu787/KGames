// Compiles the shaders to DXIL at build time: SimplePaint's, for the car's
// five materials and the sphere's, from where Library/SimplePaint says they
// are, and the background's.

use std::path::{Path, PathBuf};

fn main() {
    let output_directory = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let paint = PathBuf::from(std::env::var("DEP_SIMPLE_PAINT_SHADERS").unwrap())
        .join("SimplePaint.hlsl");
    let background = Path::new("Shaders/Background.hlsl");
    let paint_arguments = ["-O3", "-Ges", "-WX", "-D", "SIMPLE_PAINT_MATERIAL_COUNT=6"];
    let background_arguments = ["-O3", "-WX"];
    for (source, arguments, prefix) in [
        (paint.as_path(), &paint_arguments[..], "SimplePaint"),
        (background, &background_arguments[..], "Background"),
    ] {
        shader_compiler::rerun_if_changed(source);
        for (entry_point, profile, stage) in [("VSMain", "vs_6_0", "Vertex"), ("PSMain", "ps_6_0", "Pixel")] {
            shader_compiler::compile(
                source,
                entry_point,
                profile,
                arguments,
                &output_directory.join(format!("{prefix}{stage}.cso")),
            )
            .unwrap_or_else(|error| panic!("{error}"));
        }
    }
}
