// Library/GameInput: that KGames uses GameInput 4.0.306's runtime from where
// GameInputRedist.msi installed it, and no other, and that it says how to
// install it where it isn't installed. KGames doesn't build without it, so
// these run where it's installed.

use std::path::{Path, PathBuf};

use game_input::installation::{self, INSTALL_COMMAND, VERSION};

// The command that the user installs GameInput with, which is theirs to run.
const USERS_COMMAND: &str = r#"winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306""#;

#[test]
fn the_runtime_is_4_0_306_where_the_msi_installed_it() {
    assert_eq!(VERSION, [4, 0, 306, 0]);
    let path = installation::runtime_path().unwrap();
    assert!(path.ends_with(r"Microsoft GameInput\x64\GameInputRedist.dll"), "{}", path.display());

    let (_game_input, runtime) = game_input::create().unwrap();
    assert_eq!(runtime.path, path);
    assert_eq!(runtime.version, VERSION);
}

// Microsoft's loader in GameInput.lib takes a GameInputRedist.dll beside the
// exe over an installed one of the same version. KGames' never does.
#[test]
fn a_runtime_beside_the_exe_is_never_loaded() {
    let installed = installation::runtime_path().unwrap();
    let beside = std::env::current_exe().unwrap().with_file_name("GameInputRedist.dll");
    std::fs::copy(&installed, &beside).unwrap();
    let _removed = RemovedOnDrop(beside);

    let (_game_input, runtime) = game_input::create().unwrap();
    assert_eq!(runtime.path, installed);
}

#[test]
fn without_4_0_306_it_says_how_to_install_it() {
    assert_eq!(INSTALL_COMMAND, USERS_COMMAND);

    let message = installation::not_installed_message(&[]);
    assert!(message.contains("GameInput 4.0.306 isn't installed") && message.contains(USERS_COMMAND), "{message}");

    // Another program's installer may have put in a newer version, which
    // GameInputRedist.msi 4.0.306 doesn't replace.
    let message = installation::not_installed_message(&["4.1.0.0".to_string()]);
    assert!(message.contains("GameInput 4.1.0.0 is installed"), "{message}");
    assert!(message.contains(USERS_COMMAND) && message.contains("uninstall a newer one first"), "{message}");
}

// The documentation that tells people and agents to install GameInput gives
// the command that the code checks for.
#[test]
fn the_documentation_gives_the_command() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for file in ["AGENTS.md", "README.md", "Library/GameInput/README.md", "Program/GameInputDemo/README.md"] {
        let text = std::fs::read_to_string(root.join(file)).unwrap();
        assert!(text.contains(USERS_COMMAND), "{file} doesn't give the command");
    }
}

struct RemovedOnDrop(PathBuf);

impl Drop for RemovedOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
