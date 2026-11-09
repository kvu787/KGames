// Where KGames finds the one GameInput runtime that it uses: version 4.0.306,
// which GameInputRedist.msi installs into C:\Program Files\Microsoft
// GameInput\x64, and which the user installs, once, with INSTALL_COMMAND.
// Nothing in KGames installs it, and nothing uses another runtime: not the
// GameInput.dll that comes with Windows, nor a GameInputRedist.dll beside the
// exe, which would be sideloaded, with less than an installed runtime can do.
//
// installed_runtime checks that the DLL is there, and that its version is
// VERSION, with the standard library alone, so that Build.rs can include this
// file and fail the build without it, and every program checks it again as
// it starts. It holds the DLL open as it checks, and hands it back held, so
// that game_input::create loads the DLL that it checked.

use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read};
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

// What the user runs, from an elevated prompt, to install the runtime.
pub const INSTALL_COMMAND: &str = r#"winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306""#;

// Where GameInputRedist.msi installs the x64 runtime.
pub const RUNTIME: &str = r"C:\Program Files\Microsoft GameInput\x64\GameInputRedist.dll";

// The runtime's version, as its DLL's file version says it.
pub const VERSION: [u16; 4] = [4, 0, 306, 0];

// Windows' FILE_SHARE_READ: others may read the file, but not write, rename
// or delete it, nor rename its folder.
const SHARE_READ_ONLY: u32 = 0x00000001;

// A DLL whose version was checked, held open, so that it can't change until
// this is dropped, or into_path lets go of it.
#[derive(Debug)]
pub struct Checked {
    pub path: PathBuf,
    _file: File,
}

impl Checked {
    // Lets go of the DLL, once it's loaded, which keeps it from changing
    // from then on.
    pub fn into_path(self) -> PathBuf {
        self.path
    }
}

// The runtime's DLL, held, once it's there and its version is VERSION, or
// what's wrong, and how to install it.
pub fn installed_runtime() -> Result<Checked, String> {
    check(Path::new(RUNTIME))
}

// The DLL at `path`, held, once its version is VERSION. The version is read
// through the handle that holds it, so it's the version of what's held.
pub fn check(path: &Path) -> Result<Checked, String> {
    let mut file = match OpenOptions::new().read(true).share_mode(SHARE_READ_ONLY).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Err(not_installed_message(path)),
        Err(error) => return Err(format!("Couldn't read {}: {error}", path.display())),
    };
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("Couldn't read {}: {error}", path.display()))?;
    match version_in(&bytes) {
        Some(version) if version == VERSION => Ok(Checked { path: path.to_path_buf(), _file: file }),
        version => Err(wrong_version_message(path, version)),
    }
}

// The file version in a DLL's version resource: its VS_FIXEDFILEINFO starts
// with the signature 0xFEEF04BD and the structure's version, 1.0, then has the
// file version's most and least significant halves, all little-endian. It's
// searched for from the end, where a DLL's resources are.
pub fn version_in(bytes: &[u8]) -> Option<[u16; 4]> {
    const START: [u8; 8] = [0xBD, 0x04, 0xEF, 0xFE, 0x00, 0x00, 0x01, 0x00];
    let info = bytes.windows(16).rev().find(|info| info[..8] == START)?;
    let half = |at: usize| u32::from_le_bytes([info[at], info[at + 1], info[at + 2], info[at + 3]]);
    let (most, least) = (half(8), half(12));
    Some([(most >> 16) as u16, most as u16, (least >> 16) as u16, least as u16])
}

// What a build or a program says when the runtime's DLL isn't there.
pub fn not_installed_message(path: &Path) -> String {
    format!(
        "GameInput 4.0.306 isn't installed: {} isn't there, and KGames needs it. \
         It never installs it itself. Install it, from an elevated prompt, with: {INSTALL_COMMAND}",
        path.display()
    )
}

// What a build or a program says when the runtime's version isn't VERSION,
// given its version, if it has one.
pub fn wrong_version_message(path: &Path, version: Option<[u16; 4]>) -> String {
    let path = path.display();
    let dotted = |version: [u16; 4]| version.map(|part| part.to_string()).join(".");
    let needed = dotted(VERSION);
    match version {
        Some(version) if version > VERSION => format!(
            "{path} is GameInput {}, but KGames needs exactly {needed}. GameInputRedist.msi doesn't \
             replace a newer version, so uninstall Microsoft GameInput first, then install {needed}, from an \
             elevated prompt, with: {INSTALL_COMMAND}",
            dotted(version)
        ),
        Some(version) => format!(
            "{path} is GameInput {}, but KGames needs exactly {needed}. Install it, from an elevated \
             prompt, with: {INSTALL_COMMAND}",
            dotted(version)
        ),
        None => format!(
            "{path} has no version, but KGames needs GameInput {needed}. Uninstall Microsoft GameInput, \
             then install it again, from an elevated prompt, with: {INSTALL_COMMAND}"
        ),
    }
}
