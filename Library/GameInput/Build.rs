// Fails the build unless GameInput 4.0.306 is installed, from
// GameInputRedist.msi, the only GameInput runtime that KGames uses, and that
// every program needs to start (see Source/Installation.rs). The user
// installs it, once. The build never does, and never fetches or copies a
// runtime of its own.

#[path = "Source/Installation.rs"]
mod installation;

fn main() {
    println!("cargo::rerun-if-changed=Build.rs");
    println!("cargo::rerun-if-changed=Source/Installation.rs");
    match installation::runtime_path() {
        // Cargo runs this again if the installed DLL changes or goes, like
        // when GameInput is uninstalled. Whatever it misses, programs check
        // again as they start.
        Ok(path) => println!("cargo::rerun-if-changed={}", path.display()),
        Err(message) => println!("cargo::error={message}"),
    }
}
