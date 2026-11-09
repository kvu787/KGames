// Library/GameInput: that the DLLs that the GameInput runtime imports load
// from System32, and never from beside the exe. GameInputRedist.dll imports
// HID.DLL, which isn't a known DLL, so LoadLibraryW, which Microsoft's loader
// uses, looks for it beside the exe first.
//
// This is a test binary of its own, with this one test, so that nothing loads
// the runtime, or HID.DLL, before the file beside the exe is there.

#[path = "Modules.rs"]
mod modules;

use std::path::{Path, PathBuf};

#[test]
fn the_runtimes_imports_load_from_system32() {
    // Not a DLL at all, so the runtime can't load if it's taken.
    let beside = std::env::current_exe().unwrap().with_file_name("hid.dll");
    std::fs::write(&beside, b"Not a DLL").unwrap();
    let _removed = RemovedOnDrop(beside);
    assert_eq!(modules::named("hid.dll"), Vec::<PathBuf>::new());

    let (_game_input, _runtime) = game_input::create().unwrap();
    let loaded = modules::named("hid.dll");
    assert_eq!(loaded.len(), 1, "{loaded:?}");
    assert!(modules::same(&loaded[0], Path::new(r"C:\Windows\System32\hid.dll")), "{loaded:?}");
}

// A file that a test made, removed as the test ends.
struct RemovedOnDrop(PathBuf);

impl Drop for RemovedOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
