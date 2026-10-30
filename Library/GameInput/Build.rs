// Puts the GameInput runtime from the Microsoft.GameInput NuGet package next
// to the exe of every program that depends on this crate, where
// game_input::create finds it. It doesn't need a Rust crate.

use std::path::{Path, PathBuf};
use std::process::Command;

// The hash is NuGet's own SHA-512 of the .nupkg file (the "packageHash" in its
// catalog entry), so a tampered or truncated download is rejected.
// Change both together.
const GAME_INPUT_VERSION: &str = "4.0.306";
const GAME_INPUT_SHA512: &str = "f2865d53418bc4135180370d3439b063c166186b9479b150413bbafe7daffbb7\
                                 7f8c7ee3fffc8fa4b73161b0729c8f4bfcde5967490167a93ce60f010be6104c";

// The runtime DLL, and the helper process it starts to read keyboards and mice
// when it's loaded from the app's folder instead of being installed.
const GAME_INPUT_FILES: [&str; 2] = ["GameInputRedist.dll", "GameInputRawInputProxy.exe"];

fn main() {
    deploy_game_input();
}

fn deploy_game_input() {
    // Packages is outside target, so `cargo clean` doesn't download it again,
    // and every program shares it.
    let manifest_directory = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let package = manifest_directory
        .join("Packages")
        .join(format!("Microsoft.GameInput.{GAME_INPUT_VERSION}"));
    if !package.exists() {
        download_package(&package);
    }

    // OUT_DIR is target\<profile>\build\game_input-<hash>\out, and the
    // programs' exes go in target\<profile>.
    let output_directory = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let exe_directory = output_directory.ancestors().nth(3).unwrap();
    for file in GAME_INPUT_FILES {
        let source = package.join(r"native\bin\x64").join(file);
        let destination = exe_directory.join(file);
        std::fs::copy(&source, &destination).expect("failed to copy the GameInput runtime");
        // Cargo runs this script again if either copy goes missing.
        println!("cargo:rerun-if-changed={}", source.display());
        println!("cargo:rerun-if-changed={}", destination.display());
    }
}

fn download_package(package: &Path) {
    let url = format!(
        "https://api.nuget.org/v3-flatcontainer/microsoft.gameinput/{0}/microsoft.gameinput.{0}.nupkg",
        GAME_INPUT_VERSION
    );
    let archive =
        PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("Microsoft.GameInput.nupkg");
    run(Command::new(system_tool("curl.exe"))
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--output",
        ])
        .arg(&archive)
        .arg(&url));

    let hash = sha512(&archive);
    assert!(
        hash == GAME_INPUT_SHA512,
        "{url} has SHA-512 {hash}, expected {GAME_INPUT_SHA512}"
    );

    // A .nupkg file is a zip file, which Windows' own tar.exe can extract.
    // Extract beside the final folder and rename it into place, so an
    // interrupted build never leaves a half-extracted package behind.
    let staging =
        package.with_file_name(format!("Microsoft.GameInput.{GAME_INPUT_VERSION}.Partial"));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).unwrap();
    run(Command::new(system_tool("tar.exe"))
        .arg("-xf")
        .arg(&archive)
        .arg("-C")
        .arg(&staging));
    std::fs::rename(&staging, package).unwrap();
}

fn sha512(file: &Path) -> String {
    let output = Command::new(system_tool("certutil.exe"))
        .arg("-hashfile")
        .arg(file)
        .arg("SHA512")
        .output()
        .expect("failed to run certutil.exe");
    assert!(
        output.status.success(),
        "certutil.exe failed to hash {}",
        file.display()
    );
    // The hash is the second line of the output.
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().nth(1).unwrap_or_default();
    line.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
}

// Windows' own tools, so a curl.exe or tar.exe from Git or MSYS on the PATH
// can't be picked up instead.
fn system_tool(name: &str) -> PathBuf {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    PathBuf::from(root).join("System32").join(name)
}

fn run(command: &mut Command) {
    let status = command
        .status()
        .unwrap_or_else(|error| panic!("failed to run {command:?}: {error}"));
    assert!(status.success(), "{command:?} failed");
}
