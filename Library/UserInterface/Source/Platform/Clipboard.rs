// Copy, cut and paste: Windows' clipboard for the user, and one of the
// interface's own for scripts and tests, which must never read or change
// what the user copied.

use windows_api::*;

pub enum Clipboard {
    Windows,
    Own(String),
}

impl Clipboard {
    pub fn text(&self) -> Option<String> {
        match self {
            Clipboard::Own(text) => Some(text.clone()),
            Clipboard::Windows => unsafe { read_windows() },
        }
    }

    pub fn set_text(&mut self, text: &str) {
        match self {
            Clipboard::Own(own) => *own = text.to_string(),
            Clipboard::Windows => unsafe { write_windows(text) },
        }
    }
}

unsafe fn read_windows() -> Option<String> {
    unsafe {
        if !IsClipboardFormatAvailable(CF_UNICODETEXT as u32).as_bool() || !OpenClipboard(None).as_bool() {
            return None;
        }
        let mut text = None;
        let handle = GetClipboardData(CF_UNICODETEXT as u32);
        if !handle.0.is_null() {
            let units = GlobalLock(handle) as *const u16;
            if !units.is_null() {
                let mut length = 0;
                while *units.add(length) != 0 {
                    length += 1;
                }
                text = Some(String::from_utf16_lossy(std::slice::from_raw_parts(units, length)));
                let _ = GlobalUnlock(handle);
            }
        }
        let _ = CloseClipboard();
        text
    }
}

unsafe fn write_windows(text: &str) {
    unsafe {
        if !OpenClipboard(None).as_bool() {
            return;
        }
        let _ = EmptyClipboard();
        let units: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let memory = GlobalAlloc(GMEM_MOVEABLE as u32, units.len() * 2);
        if !memory.0.is_null() {
            let destination = GlobalLock(memory) as *mut u16;
            if !destination.is_null() {
                std::ptr::copy_nonoverlapping(units.as_ptr(), destination, units.len());
                let _ = GlobalUnlock(memory);
                // Once the clipboard takes the memory, it's the clipboard's.
                if SetClipboardData(CF_UNICODETEXT as u32, Some(memory)).0.is_null() {
                    let _ = GlobalFree(memory);
                }
            }
        }
        let _ = CloseClipboard();
    }
}
