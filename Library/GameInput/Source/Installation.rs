// Where KGames finds the one GameInput runtime that it uses: version 4.0.306,
// which GameInputRedist.msi installs into C:\Program Files\Microsoft
// GameInput\x64, and which the user installs, once, with INSTALL_COMMAND.
// Nothing in KGames installs it, and nothing uses another runtime: not the
// GameInput.dll that comes with Windows, nor a GameInputRedist.dll beside the
// exe, which would be sideloaded, with less than an installed runtime can do.
//
// This only checks that the DLL is there, with the standard library, so that
// Build.rs can include it, and fail the build without it. Programs also check
// the DLL's version as they start, with game_input::installed_runtime.

use std::path::PathBuf;

// What the user runs, from an elevated prompt, to install the runtime.
pub const INSTALL_COMMAND: &str = r#"winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306""#;

// Where GameInputRedist.msi installs the x64 runtime.
pub const RUNTIME: &str = r"C:\Program Files\Microsoft GameInput\x64\GameInputRedist.dll";

// The runtime's DLL, if it's there, or what's wrong, and how to install it.
pub fn runtime_path() -> Result<PathBuf, String> {
    let path = PathBuf::from(RUNTIME);
    if path.is_file() { Ok(path) } else { Err(not_installed_message()) }
}

// What a build or a program says when the runtime isn't there.
pub fn not_installed_message() -> String {
    format!(
        "GameInput 4.0.306 isn't installed: {RUNTIME} isn't there, and KGames needs it. \
         It never installs it itself. Install it, from an elevated prompt, with: {INSTALL_COMMAND}"
    )
}
