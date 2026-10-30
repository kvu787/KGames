// Compiles HLSL to DXIL at build time with dxc.exe from the Windows SDK, which
// doesn't need a Rust crate.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    compile_shaders();
}

fn compile_shaders() {
    let shader = "Shaders/Canvas.hlsl";
    println!("cargo:rerun-if-changed={shader}");

    let output_directory = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let dxc = find_dxc();

    for (entry_point, profile, output) in [
        ("VSMain", "vs_6_0", "Vertex.cso"),
        ("PSMain", "ps_6_0", "Pixel.cso"),
    ] {
        let status = Command::new(&dxc)
            .args(["-T", profile, "-E", entry_point, "-Fo"])
            .arg(output_directory.join(output))
            .arg(shader)
            .status()
            .expect("failed to run dxc.exe");
        assert!(status.success(), "dxc.exe failed to compile {entry_point}");
    }
}

// Picks dxc.exe from the newest installed Windows SDK.
fn find_dxc() -> PathBuf {
    let bin = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin");
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(&bin)
        .expect("Windows SDK not found")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().join(r"x64\dxc.exe"))
        .filter(|path| path.exists())
        .collect();
    candidates.sort();
    candidates
        .pop()
        .expect("dxc.exe not found; install the Windows SDK")
}
