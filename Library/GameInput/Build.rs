// Fails the build unless GameInput 4.0.306's runtime DLL is where
// GameInputRedist.msi installs it, the only GameInput runtime that KGames
// uses, and that every program needs to start (see Source/Installation.rs).
// The user installs it, once. The build never does, and never fetches or
// copies a runtime of its own.

#[path = "Source/Installation.rs"]
mod installation;

fn main() {
    println!("cargo::rerun-if-changed=Build.rs");
    println!("cargo::rerun-if-changed=Source/Installation.rs");
    // Cargo runs this again when the DLL goes, like when GameInput is
    // uninstalled, or when its modified time is newer than this script's
    // last run. GameInputRedist.msi keeps the time that Microsoft built the
    // DLL, so another version installed after a build may look older, and
    // pass. Programs check again as they start, which catches it.
    println!("cargo::rerun-if-changed={}", installation::RUNTIME);
    if let Err(message) = installation::installed_runtime() {
        println!("cargo::error={message}");
    }
}
