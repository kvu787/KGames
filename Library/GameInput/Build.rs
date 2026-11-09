// Fails the build unless GameInput 4.0.306's installed runtime DLL is in
// System32, the only GameInput runtime that KGames uses, and that every
// program needs to start (see Source/Installation.rs).
// The user installs it, once. The build never does, and never fetches or
// copies a runtime of its own.

#[path = "Source/Installation.rs"]
mod installation;

fn main() {
    println!("cargo::rerun-if-changed=Build.rs");
    println!("cargo::rerun-if-changed=Source/Installation.rs");
    match installation::installed_runtime() {
        // Cargo runs this again when the DLL goes, or when its modified time
        // is newer than this script's last run. GameInput's installer and its
        // service keep the time that Microsoft built the DLL, so another
        // version installed after a build may look older, and pass. Programs
        // check again as they start, which catches it. A build that fails runs
        // this again anyway.
        Ok(checked) => println!("cargo::rerun-if-changed={}", checked.into_path().display()),
        Err(message) => println!("cargo::error={message}"),
    }
}
