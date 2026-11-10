// The DLLs that this process has loaded, from a snapshot of its modules.

use std::path::{Path, PathBuf};

use windows_api::{CloseHandle, CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, TH32CS_SNAPMODULE};

// Windows' ERROR_BAD_LENGTH, which a snapshot of the modules fails with while
// a DLL loads or unloads. Then it's taken again.
const ERROR_BAD_LENGTH: i32 = 24;

// The full path of every DLL that the process has loaded, and of its exe.
pub fn loaded() -> Vec<PathBuf> {
    unsafe {
        let snapshot = loop {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE as u32, 0);
            // INVALID_HANDLE_VALUE.
            if snapshot.0 as isize != -1 {
                break snapshot;
            }
            let error = std::io::Error::last_os_error();
            assert_eq!(error.raw_os_error(), Some(ERROR_BAD_LENGTH), "Couldn't list the modules: {error}");
        };
        let mut entry = MODULEENTRY32W {
            dwSize: size_of::<MODULEENTRY32W>() as u32,
            ..Default::default()
        };
        let mut paths = Vec::new();
        let mut more = Module32FirstW(snapshot, &mut entry).as_bool();
        while more {
            let path = &entry.szExePath;
            let length = path.iter().position(|&character| character == 0).unwrap_or(path.len());
            paths.push(PathBuf::from(String::from_utf16_lossy(&path[..length])));
            more = Module32NextW(snapshot, &mut entry).as_bool();
        }
        let _ = CloseHandle(snapshot);
        paths
    }
}

// The loaded DLLs with this file name, regardless of case.
pub fn named(name: &str) -> Vec<PathBuf> {
    loaded()
        .into_iter()
        .filter(|path| path.file_name().is_some_and(|file| file.eq_ignore_ascii_case(name)))
        .collect()
}

// Whether two paths are the same, regardless of case, as Windows' are.
pub fn same(left: &Path, right: &Path) -> bool {
    left.as_os_str().eq_ignore_ascii_case(right.as_os_str())
}
