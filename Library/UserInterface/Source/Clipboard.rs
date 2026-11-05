// Text fields' copy, cut and paste: Windows' clipboard for the user, and a
// clipboard of the interface's own for scripts and tests, which must never
// read or change what the user copied.

use windows_api::*;

pub enum Clipboard {
    Windows,
    Own(String),
}

impl Clipboard {
    pub fn get(&self) -> Option<String> {
        match self {
            Clipboard::Own(text) => Some(text.clone()),
            Clipboard::Windows => unsafe { read_windows() },
        }
    }

    pub fn set(&mut self, text: &str) {
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
            let pointer = GlobalLock(handle) as *const u16;
            if !pointer.is_null() {
                let mut length = 0;
                while *pointer.add(length) != 0 {
                    length += 1;
                }
                text = Some(String::from_utf16_lossy(std::slice::from_raw_parts(pointer, length)));
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
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let memory = GlobalAlloc(GMEM_MOVEABLE as u32, wide.len() * 2);
        if !memory.0.is_null() {
            let pointer = GlobalLock(memory) as *mut u16;
            if !pointer.is_null() {
                std::ptr::copy_nonoverlapping(wide.as_ptr(), pointer, wide.len());
                let _ = GlobalUnlock(memory);
                // The clipboard owns the memory once it takes it.
                if SetClipboardData(CF_UNICODETEXT as u32, Some(memory)).0.is_null() {
                    let _ = GlobalFree(memory);
                }
            }
        }
        let _ = CloseClipboard();
    }
}
