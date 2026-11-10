// Library/GameInput: that the DLLs that the GameInput runtime imports are
// looked for beside the exe first. GameInput.lib's loader, which create starts
// GameInput with, loads the runtime with LoadLibraryW, and GameInputRedist.dll
// imports HID.DLL, which isn't a known DLL, so Windows takes a hid.dll beside
// the exe over System32's: here, one that isn't a DLL at all, which fails to
// load, so GameInput doesn't start.
//
// This is a test binary of its own, with this one test, so that nothing loads
// the runtime, or HID.DLL, before the file beside the exe is there.

// It needs only `named`.
#[allow(dead_code)]
mod modules;

use std::path::PathBuf;

#[test]
fn the_runtimes_imports_are_looked_for_beside_the_exe_first() {
    let beside = std::env::current_exe().unwrap().with_file_name("hid.dll");
    std::fs::write(&beside, b"Not a DLL").unwrap();
    let _removed = RemovedOnDrop(beside);
    assert_eq!(modules::named("hid.dll"), Vec::<PathBuf>::new());

    // Windows' ERROR_BAD_EXE_FORMAT, as an HRESULT.
    let error = game_input::create().err().unwrap();
    assert!(error.contains("0x800700C1"), "{error}");
    assert_eq!(modules::named("hid.dll"), Vec::<PathBuf>::new());
}

// A file that a test made, removed as the test ends.
struct RemovedOnDrop(PathBuf);

impl Drop for RemovedOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
