// What KGames adds to the Windows API bindings in Library/windows-api-raw,
// which are only what windows-bindgen generates. Bindings go there, never
// here.

// Win32 functions report failure with a null handle and set the thread's last error.
pub fn check_handle(handle: *mut core::ffi::c_void) -> windows_core::Result<()> {
    if handle.is_null() {
        Err(windows_core::Error::from_thread())
    } else {
        Ok(())
    }
}
