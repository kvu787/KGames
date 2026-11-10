// Fails the build unless GameInput 4.0.306's installed runtime DLL is in
// System32, which every program needs to start (see src/installation.rs).
// The user installs it, once. The build never does.
//
// And restores the NuGet packages that NuGetPackages.proj declares, with
// dotnet, as packages.lock.json pins them: Microsoft.GameInput, whose
// GameInput.lib this links, for its loader, GameInputInitialize, which
// game_input::create starts GameInput with (see README.md).

#[path = "src/installation.rs"]
mod installation;

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

// What a build says when there's no dotnet to restore the packages with.
const NO_DOTNET_MESSAGE: &str = "Building KGames needs the .NET SDK, 10 or later, whose dotnet restores \
    the Microsoft.GameInput NuGet package that Library/GameInput/NuGetPackages.proj declares. Visual Studio \
    installs it with its .NET desktop development workload, or get it from https://dotnet.microsoft.com/download";

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=src/installation.rs");
    match installation::installed_runtime() {
        // Cargo runs this again when the DLL goes, or when its modified time
        // is newer than this script's last run. GameInput's installer and its
        // service keep the time that Microsoft built the DLL, so another
        // version installed after a build may look older, and pass. Programs
        // check again as they start, which catches it. A build that fails runs
        // this again anyway.
        Ok(path) => println!("cargo::rerun-if-changed={}", path.display()),
        Err(message) => println!("cargo::error={message}"),
    }
    match restore() {
        Ok(package) => {
            // Cargo runs this again if the package leaves NuGet's folder.
            let library = package.join(LIBRARY);
            println!("cargo::rerun-if-changed={}", library.display());
            // A static library, which goes into this crate's rlib, so every
            // program that uses the crate links it.
            println!("cargo::rustc-link-search=native={}", library.parent().unwrap().display());
            println!("cargo::rustc-link-lib=static=GameInput");
        }
        Err(message) => println!("cargo::error={message}"),
    }
}

// GameInput.lib, for x64, in the package.
const LIBRARY: &str = r"native\lib\x64\GameInput.lib";

// Restores the packages, and returns Microsoft.GameInput's folder.
fn restore() -> Result<PathBuf, String> {
    let directory = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let project = directory.join("NuGetPackages.proj");
    for file in ["NuGetPackages.proj", "packages.lock.json", "Directory.Build.props", r"..\..\NuGet.config", r"..\..\global.json"] {
        println!("cargo::rerun-if-changed={}", directory.join(file).display());
    }

    // Locked mode fails, instead of changing packages.lock.json, when the
    // project asks for other packages than the lock file pins, and a package
    // that it downloads must have the hash that the lock file gives. Without
    // build servers, nothing that it starts outlives it.
    dotnet(&project, &["restore", "--locked-mode", "--disable-build-servers"])?;

    // Where NuGet put the package, which can be another folder on another
    // machine.
    let output = dotnet(&project, &["msbuild", "-getProperty:PkgMicrosoft_GameInput"])?;
    let package = PathBuf::from(output.trim());
    if !package.join(LIBRARY).is_file() {
        return Err(format!(
            "dotnet restored Microsoft.GameInput, but {} has no {LIBRARY}",
            package.display()
        ));
    }
    Ok(package)
}

// Runs `dotnet <command> <project> <options>`, and returns what it printed.
fn dotnet(project: &Path, arguments: &[&str]) -> Result<String, String> {
    let (command, options) = arguments.split_first().unwrap();
    let output = Command::new("dotnet")
        .arg(command)
        .arg(project)
        .args(options)
        .arg("--nologo")
        // A build that restores in passing doesn't send dotnet's usage data.
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .env("DOTNET_NOLOGO", "1")
        .output();
    let output = match output {
        Ok(output) => output,
        Err(error) if error.kind() == ErrorKind::NotFound => return Err(NO_DOTNET_MESSAGE.to_string()),
        Err(error) => return Err(format!("Couldn't run dotnet: {error}")),
    };
    let printed = String::from_utf8_lossy(&output.stdout).into_owned();
    if !output.status.success() {
        // Cargo shows one line of an error, so this keeps dotnet's errors,
        // each once, on one.
        let printed = printed + &String::from_utf8_lossy(&output.stderr);
        let mut errors: Vec<&str> = printed.lines().map(str::trim).filter(|line| line.contains("error")).collect();
        errors.dedup();
        let errors = if errors.is_empty() { printed.split_whitespace().collect::<Vec<_>>().join(" ") } else { errors.join(" ") };
        return Err(format!("dotnet {command} {} failed: {errors}", project.display()));
    }
    Ok(printed)
}
