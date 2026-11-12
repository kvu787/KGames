// Library/GameInput: that KGames starts GameInput with GameInputInitialize
// from the NuGet package's GameInput.lib, whose loader takes GameInput
// 4.0.306's installed runtime, from System32, unless one is beside the exe,
// and that it says how to install it where it isn't installed. KGames doesn't
// build without it, so these run where it's installed.

mod modules;

use core::ffi::c_void;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use game_input::installation::{self, INSTALL_COMMAND, RUNTIME, VERSION};
use game_input::{
    GameInputCallbackToken, GameInputDeviceEvent, GameInputKindGamepad, GameInputKindKeyboard, GameInputKindMouse,
};

// The command that the user installs GameInput with, which is theirs to run.
const USERS_COMMAND: &str = r#"winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306""#;

#[test]
fn the_runtime_is_4_0_306_in_system32() {
    assert_eq!(VERSION, [4, 0, 306, 0]);
    assert_eq!(RUNTIME, r"C:\Windows\System32\GameInputRedist.dll");
    assert_eq!(installation::installed_runtime().unwrap(), Path::new(RUNTIME));

    // With nothing beside the exe, the loader takes it.
    let (_game_input, runtime) = game_input::create().unwrap();
    assert!(modules::same(&runtime.path, Path::new(RUNTIME)), "{:?}", runtime.path);
    assert_eq!(runtime.version, VERSION);
}

// The version is read from the DLL's bytes, with the standard library alone,
// which rs has: from its VS_FIXEDFILEINFO, which starts with the
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

// The check reads the DLL without letting others write it, so a DLL that's
// being written isn't checked. System32 takes an administrator to change at
// all, so this checks a copy.
#[test]
fn a_dll_thats_being_written_isnt_checked() {
    let folder = std::env::current_exe().unwrap().with_file_name(format!("CheckedRuntime{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let _removed = RemovedOnDrop(folder.clone());
    let dll = folder.join("GameInputRedist.dll");
    std::fs::copy(RUNTIME, &dll).unwrap();
    assert_eq!(installation::check(&dll).unwrap(), dll);

    let writing = OpenOptions::new().write(true).open(&dll).unwrap();
    assert!(installation::check(&dll).is_err());
    drop(writing);
    std::fs::remove_file(&dll).unwrap();
    assert!(installation::check(&dll).unwrap_err().contains("isn't installed"));
}

// GameInput.lib's loader, which create starts GameInput with, takes a
// GameInputRedist.dll beside the exe over the installed one of the same
// version, as Microsoft's documentation of sideloading says. So nothing in
// the repo may put one beside an exe.
//
// The loader picks a runtime once a process, and a loaded DLL can't be
// deleted, so a copy of this test's exe, in a folder of its own, with a copy
// of the runtime beside it, runs this test again, and says which runtime it
// loaded.
#[test]
fn a_runtime_beside_the_exe_of_the_same_version_is_loaded() {
    const IN_THE_COPY: &str = "KGAMES_GAME_INPUT_TEST_IN_THE_COPY";
    if std::env::var_os(IN_THE_COPY).is_some() {
        let (_game_input, runtime) = game_input::create().unwrap();
        println!("Loaded {}", runtime.path.display());
        return;
    }

    let exe = std::env::current_exe().unwrap();
    let folder = exe.with_file_name(format!("SideloadedRuntime{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let _removed = RemovedOnDrop(folder.clone());
    let copy = folder.join(exe.file_name().unwrap());
    std::fs::copy(&exe, &copy).unwrap();
    let beside = folder.join("GameInputRedist.dll");
    std::fs::copy(RUNTIME, &beside).unwrap();

    let output = Command::new(&copy)
        .args(["--exact", "a_runtime_beside_the_exe_of_the_same_version_is_loaded", "--nocapture"])
        .env(IN_THE_COPY, "1")
        .output()
        .unwrap();
    let printed = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{printed}{}", String::from_utf8_lossy(&output.stderr));
    let loaded = printed.lines().find_map(|line| line.strip_prefix("Loaded ")).expect(&printed);
    assert!(modules::same(Path::new(loaded), &beside), "{loaded}");
}

// The runtime's DLL is also what COM loads to carry calls to GameInput's
// service, from System32, where the installer registers it. The runtime
// connects to its service on a thread of its own as it starts, which loads
// that copy, about 5 ms in, and reports devices after that, about 600 ms in,
// so this waits for the first device. With nothing beside the exe,
// GameInput.lib's loader takes System32's copy too, so the process runs one.
#[test]
fn one_copy_of_the_runtime_runs() {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    unsafe extern "system" fn reported(_: GameInputCallbackToken, _: *mut c_void, _: GameInputDeviceEvent, _: *mut c_void) {
        REPORTED.store(true, Ordering::Release);
    }

    let (game_input, _runtime) = game_input::create().unwrap();
    let kinds = GameInputKindGamepad | GameInputKindKeyboard | GameInputKindMouse;
    let token = unsafe { game_input.RegisterDeviceCallback(None, kinds, None, Some(reported)).unwrap() };
    let deadline = Instant::now() + Duration::from_secs(10);
    while !REPORTED.load(Ordering::Acquire) {
        assert!(Instant::now() < deadline, "GameInput reported no keyboard, mouse or gamepad");
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(unsafe { game_input.UnregisterCallback(token) });

    let loaded = modules::named("GameInputRedist.dll");
    assert_eq!(loaded.len(), 1, "{loaded:?}");
    assert!(modules::same(&loaded[0], Path::new(RUNTIME)), "{loaded:?}");
}

// The reading callback, which Library/UserInterface queues every reading
// with, registers with GameInput 4.0.306's signature, in its slot, and
// unregisters. A test doesn't have focus, so with background input off it
// may get no readings, but the callback is only ever given one.
#[test]
fn a_reading_callback_registers_and_unregisters() {
    static NULL_READING: AtomicBool = AtomicBool::new(false);
    unsafe extern "system" fn read(_: GameInputCallbackToken, _: *mut c_void, reading: *mut c_void) {
        if reading.is_null() {
            NULL_READING.store(true, Ordering::Release);
        }
    }
    let (game_input, _runtime) = game_input::create().unwrap();
    let kinds = GameInputKindGamepad | GameInputKindKeyboard | GameInputKindMouse;
    let token = unsafe { game_input.RegisterReadingCallback(None, kinds, None, Some(read)).unwrap() };
    assert_ne!(token, 0);
    std::thread::sleep(Duration::from_millis(50));
    assert!(unsafe { game_input.UnregisterCallback(token) });
    assert!(!NULL_READING.load(Ordering::Acquire));
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

    let runtime = Path::new(RUNTIME);
    let message = installation::not_installed_message(runtime);
    assert!(message.contains("GameInput 4.0.306 isn't installed") && message.contains(USERS_COMMAND), "{message}");

    // A newer version, like one that another program's installer put in,
    // which GameInputRedist.msi 4.0.306 doesn't replace.
    let message = installation::wrong_version_message(runtime, Some([4, 1, 0, 0]));
    assert!(message.contains("is GameInput 4.1.0.0") && message.contains("uninstall Microsoft GameInput first"), "{message}");
    assert!(message.contains(USERS_COMMAND), "{message}");

    // An older one, which it does replace.
    let message = installation::wrong_version_message(runtime, Some([4, 0, 300, 0]));
    assert!(message.contains("is GameInput 4.0.300.0") && !message.contains("uninstall"), "{message}");
    assert!(message.contains(USERS_COMMAND), "{message}");
}

// NuGetPackages.csproj asks for exactly the version of GameInput whose runtime
// the user installs, and packages.lock.json pins that version.
#[test]
fn the_package_is_the_runtimes_version() {
    let version = VERSION[..3].iter().map(|part| part.to_string()).collect::<Vec<_>>().join(".");
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Library/GameInput");
    let project = std::fs::read_to_string(folder.join("NuGetPackages.csproj")).unwrap();
    let reference = format!(r#"<PackageReference Include="Microsoft.GameInput" Version="[{version}]""#);
    assert!(project.contains(&reference), "{project}");
    let lock = std::fs::read_to_string(folder.join("packages.lock.json")).unwrap();
    assert!(lock.contains(&format!(r#""requested": "[{version}, {version}]""#)), "{lock}");
    assert!(lock.contains(&format!(r#""resolved": "{version}""#)), "{lock}");
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

// A file or folder that a test made, removed as the test ends.
struct RemovedOnDrop(PathBuf);

impl Drop for RemovedOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
