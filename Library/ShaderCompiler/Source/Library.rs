// Compiles HLSL to DXIL with dxc.exe from the newest installed Windows SDK,
// for build scripts and tests, so that no crate is needed for it.

use std::path::{Path, PathBuf};
use std::process::Command;

// Picks dxc.exe from the newest installed Windows SDK.
pub fn find_dxc() -> PathBuf {
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

// Compiles one entry point of `source` into `output`. `arguments` go to dxc
// before the source, like ["-D", "COUNT=3", "-O3"]. Fails with what dxc
// wrote, like the line of a mistake in the shader.
pub fn compile(
    source: &Path,
    entry_point: &str,
    profile: &str,
    arguments: &[&str],
    output: &Path,
) -> Result<(), String> {
    let result = Command::new(find_dxc())
        .args(["-T", profile, "-E", entry_point])
        .args(arguments)
        .arg("-Fo")
        .arg(output)
        .arg(source)
        .output()
        .map_err(|error| format!("failed to run dxc.exe: {error}"))?;
    if result.status.success() {
        Ok(())
    } else {
        Err(format!(
            "dxc.exe failed to compile {entry_point} in {}:\n{}{}",
            source.display(),
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        ))
    }
}

// For build scripts: has Cargo run the build script again when anything in
// the shader's folder changes, which covers the files it includes from there.
pub fn rerun_if_changed(source: &Path) {
    let folder = source.parent().unwrap_or(source);
    println!("cargo::rerun-if-changed={}", folder.display());
}
