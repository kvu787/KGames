// Library/GameInput: that KGames uses GameInput 4.0.306's runtime from where
// GameInputRedist.msi installs it, and no other, and that it says how to
// install it where it isn't installed. KGames doesn't build without it, so
// these run where it's installed.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};
use std::time::Duration;

use game_input::installation::{self, INSTALL_COMMAND, RUNTIME, VERSION};

// The command that the user installs GameInput with, which is theirs to run.
const USERS_COMMAND: &str = r#"winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306""#;

#[test]
fn the_runtime_is_4_0_306_where_the_msi_installs_it() {
    assert_eq!(VERSION, [4, 0, 306, 0]);
    assert_eq!(RUNTIME, r"C:\Program Files\Microsoft GameInput\x64\GameInputRedist.dll");
    let path = installation::installed_runtime().unwrap();
    assert_eq!(path, Path::new(RUNTIME));

    let (_game_input, runtime) = game_input::create().unwrap();
    assert_eq!(runtime.path, path);
    assert_eq!(runtime.version, VERSION);
}

// The version is read from the DLL's bytes, with the standard library alone,
// which Build.rs has: from its VS_FIXEDFILEINFO, which starts with the
// signature 0xFEEF04BD and the structure's version, 1.0.
#[test]
fn the_version_is_read_from_the_dll_itself() {
    assert_eq!(installation::version_in(&std::fs::read(RUNTIME).unwrap()), Some(VERSION));
    assert_eq!(installation::version_in(&[]), None);

    // The signature alone, without the structure's version, isn't one.
    let mut bytes = vec![0u8; 64];
    bytes[4..12].copy_from_slice(&[0xBD, 0x04, 0xEF, 0xFE, 0x00, 0x00, 0x02, 0x00]);
    assert_eq!(installation::version_in(&bytes), None);

    // 4.1.2.3, as its most significant half, 4 and 1, and its least, 2 and 3.
    bytes[32..48].copy_from_slice(&[
        0xBD, 0x04, 0xEF, 0xFE, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x04, 0x00, 0x03, 0x00, 0x02, 0x00,
    ]);
    assert_eq!(installation::version_in(&bytes), Some([4, 1, 2, 3]));
}

// Microsoft's loader in GameInput.lib takes a GameInputRedist.dll beside the
// exe over an installed one of the same version. KGames' never does.
#[test]
fn a_runtime_beside_the_exe_is_never_loaded() {
    let beside = std::env::current_exe().unwrap().with_file_name("GameInputRedist.dll");
    std::fs::copy(RUNTIME, &beside).unwrap();
    let _removed = RemovedOnDrop(beside);

    let (_game_input, runtime) = game_input::create().unwrap();
    assert_eq!(runtime.path, Path::new(RUNTIME));
}

// GameInput 4.0.306's runtime crashes the process when one thread releases
// the last IGameInput while another creates one. create keeps the first one
// that it creates, so threads can create and release them at once.
#[test]
fn threads_can_create_and_release_game_input_at_once() {
    for _ in 0..20 {
        let barrier = Arc::new(Barrier::new(2));
        let threads: Vec<_> = (0..2)
            .map(|index| {
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    // The second thread creates while the first releases.
                    if index == 1 {
                        std::thread::sleep(Duration::from_micros(200));
                    }
                    drop(game_input::create().unwrap());
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
    }
}

#[test]
fn without_4_0_306_it_says_how_to_install_it() {
    assert_eq!(INSTALL_COMMAND, USERS_COMMAND);

    let message = installation::not_installed_message();
    assert!(message.contains("GameInput 4.0.306 isn't installed") && message.contains(USERS_COMMAND), "{message}");

    // A newer version, like one that another program's installer put in,
    // which GameInputRedist.msi 4.0.306 doesn't replace.
    let message = installation::wrong_version_message(Some([4, 1, 0, 0]));
    assert!(message.contains("is GameInput 4.1.0.0") && message.contains("uninstall Microsoft GameInput first"), "{message}");
    assert!(message.contains(USERS_COMMAND), "{message}");

    // An older one, which it does replace.
    let message = installation::wrong_version_message(Some([4, 0, 300, 0]));
    assert!(message.contains("is GameInput 4.0.300.0") && !message.contains("uninstall"), "{message}");
    assert!(message.contains(USERS_COMMAND), "{message}");
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
