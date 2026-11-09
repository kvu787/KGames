// The one GameInput runtime that KGames uses: version 4.0.306 of Microsoft's
// GameInputRedist.msi, which the user installs, once, with INSTALL_COMMAND.
// Nothing in KGames installs it, and nothing uses another runtime: not the
// GameInput.dll that comes with Windows, nor a GameInputRedist.dll beside the
// exe, which would be sideloaded, with less than an installed runtime can do.
//
// Windows Installer says whether the MSI's 4.0.306 product is installed, and
// where it put the runtime's DLL, whose own version this checks too. Build.rs
// includes this file, so that KGames doesn't build without the runtime, and
// the library loads the runtime from where this finds it, so that no program
// starts without it.

use std::path::{Path, PathBuf};

use windows_api::*;
use windows_core::{PCWSTR, PWSTR, w};

// What the user runs, from an elevated prompt, to install the runtime.
pub const INSTALL_COMMAND: &str = r#"winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306""#;

// The runtime's version, as its DLL's file version says it.
pub const VERSION: [u16; 4] = [4, 0, 306, 0];

// The product code of GameInputRedist.msi 4.0.306, which is that version's
// own, as winget's manifest of Microsoft.GameInput 4.0.306 says too.
const PRODUCT_CODE: PCWSTR = w!("{DE540882-68F6-4A5E-888D-7E7902BB4CE7}");
// The upgrade code that every version of GameInputRedist.msi shares, which
// finds the version that's installed instead, if any.
const UPGRADE_CODE: PCWSTR = w!("{37042BD3-CED5-451B-818A-3E9454BDF060}");
// The MSI's component that installs the x64 GameInputRedist.dll, into
// C:\Program Files\Microsoft GameInput\x64, from its Component table.
const RUNTIME_COMPONENT: PCWSTR = w!("{C5BD8EE2-7E6B-DDDF-6A70-6701B409AFE5}");

// The path of GameInput 4.0.306's runtime DLL, where GameInputRedist.msi
// installed it, or what's wrong, and how to install it.
pub fn runtime_path() -> Result<PathBuf, String> {
    if unsafe { MsiQueryProductStateW(PRODUCT_CODE) } != INSTALLSTATE_DEFAULT {
        return Err(not_installed_message(&installed_versions()));
    }
    let Some(path) = component_path() else {
        return Err(format!(
            "GameInput 4.0.306 is installed, but its GameInputRedist.dll is missing. KGames needs it. \
             Uninstall Microsoft GameInput, then install it again, from an elevated prompt, with: {INSTALL_COMMAND}"
        ));
    };
    match file_version(&path) {
        Some(version) if version == VERSION => Ok(path),
        version => Err(format!(
            "GameInput 4.0.306 is installed, but {} is version {}, where KGames needs 4.0.306.0. \
             Uninstall Microsoft GameInput, then install it again, from an elevated prompt, with: {INSTALL_COMMAND}",
            path.display(),
            version.map_or("unknown".to_string(), |version| version.map(|part| part.to_string()).join(".")),
        )),
    }
}

// What a build or a program says when GameInput 4.0.306 isn't installed,
// given the other versions of GameInputRedist.msi that are.
pub fn not_installed_message(other_versions: &[String]) -> String {
    if other_versions.is_empty() {
        format!(
            "GameInput 4.0.306 isn't installed, and KGames needs it. It never installs it itself. \
             Install it, from an elevated prompt, with: {INSTALL_COMMAND}"
        )
    } else {
        format!(
            "GameInput {} is installed, but KGames needs exactly 4.0.306. \
             Install it, from an elevated prompt, with: {INSTALL_COMMAND}. \
             GameInputRedist.msi doesn't replace a newer version, so uninstall a newer one first.",
            other_versions.join(" and ")
        )
    }
}

// The versions of GameInputRedist.msi that are installed, like a newer one
// that another program's installer put in, in place of 4.0.306.
fn installed_versions() -> Vec<String> {
    let mut versions = Vec::new();
    for index in 0u32.. {
        // A product code, in braces, and its terminating zero.
        let mut code = [0u16; 39];
        if unsafe { MsiEnumRelatedProductsW(UPGRADE_CODE, None, index, PWSTR(code.as_mut_ptr())) } != ERROR_SUCCESS as u32 {
            break;
        }
        if let Some(version) = product_version(PCWSTR(code.as_ptr())) {
            versions.push(version);
        }
    }
    versions
}

// An installed product's version, like "4.0.306.0".
fn product_version(code: PCWSTR) -> Option<String> {
    // INSTALLPROPERTY_VERSIONSTRING, which isn't in the Windows metadata.
    let property = w!("VersionString");
    let mut buffer = vec![0u16; 32];
    loop {
        // The buffer's size goes in, and the value's length, without its
        // terminating zero, comes out.
        let mut length = buffer.len() as u32;
        let result = unsafe { MsiGetProductInfoW(code, property, Some(PWSTR(buffer.as_mut_ptr())), Some(&mut length)) };
        match result as i32 {
            ERROR_SUCCESS => return Some(String::from_utf16_lossy(&buffer[..length as usize])),
            ERROR_MORE_DATA => buffer.resize(length as usize + 1, 0),
            _ => return None,
        }
    }
}

// Where GameInputRedist.msi 4.0.306 installed GameInputRedist.dll, if it's
// there.
fn component_path() -> Option<PathBuf> {
    let mut buffer = vec![0u16; 260];
    loop {
        // Like MsiGetProductInfoW's size and length.
        let mut length = buffer.len() as u32;
        let state = unsafe { MsiGetComponentPathW(PRODUCT_CODE, RUNTIME_COMPONENT, Some(PWSTR(buffer.as_mut_ptr())), Some(&mut length)) };
        match state {
            INSTALLSTATE_LOCAL => return Some(PathBuf::from(String::from_utf16_lossy(&buffer[..length as usize]))),
            INSTALLSTATE_MOREDATA => buffer.resize(length as usize + 1, 0),
            _ => return None,
        }
    }
}

// The file version from a DLL's version resource, as major, minor, build and
// revision.
fn file_version(path: &Path) -> Option<[u16; 4]> {
    let path = wide(path);
    let path = PCWSTR(path.as_ptr());
    unsafe {
        let size = GetFileVersionInfoSizeW(path, None);
        if size == 0 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        if !GetFileVersionInfoW(path, None, size, data.as_mut_ptr().cast()).as_bool() {
            return None;
        }
        let mut info: *mut core::ffi::c_void = core::ptr::null_mut();
        let mut length = 0u32;
        if !VerQueryValueW(data.as_ptr().cast(), w!(r"\"), &mut info, &mut length).as_bool() {
            return None;
        }
        let info = &*(info as *const VS_FIXEDFILEINFO);
        Some([
            (info.dwFileVersionMS >> 16) as u16,
            info.dwFileVersionMS as u16,
            (info.dwFileVersionLS >> 16) as u16,
            info.dwFileVersionLS as u16,
        ])
    }
}

// A path as a zero-terminated wide string, for Windows' functions.
pub(crate) fn wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str().encode_wide().chain([0]).collect()
}
