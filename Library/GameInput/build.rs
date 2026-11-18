// Fails the build unless GameInput 4.0.306's installed runtime DLL is in
// System32, which every program needs to start (see src/installation.rs).
// The user installs it, once. The build never does.

#[path = "src/installation.rs"]
mod installation;

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
}
