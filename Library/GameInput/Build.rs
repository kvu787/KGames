// Fails the build unless GameInput's runtime DLL is where GameInputRedist.msi
// installs it, the only GameInput runtime that KGames uses, and that every
// program needs to start (see Source/Installation.rs). The user installs it,
// once. The build never does, and never fetches or copies a runtime of its
// own. Programs check the DLL's version as they start: checking it here would
// take Windows' API, which a build script would have to build a second time.

#[path = "Source/Installation.rs"]
mod installation;

fn main() {
    println!("cargo::rerun-if-changed=Build.rs");
    println!("cargo::rerun-if-changed=Source/Installation.rs");
    // Cargo runs this again if the DLL changes or goes, like when GameInput
    // is uninstalled.
    println!("cargo::rerun-if-changed={}", installation::RUNTIME);
    if let Err(message) = installation::runtime_path() {
        println!("cargo::error={message}");
    }
}
