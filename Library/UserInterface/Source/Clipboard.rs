// Where text fields copy to and paste from: Windows' clipboard while the user
// is the input, or the interface's own while a script or a test is, so that
// they never touch what the user copied.

use windows_api::*;

pub enum Clipboard {
    System,
    Memory(String),
}

impl Clipboard {
    pub fn memory() -> Self {
        Clipboard::Memory(String::new())
    }

    // The text on the clipboard, if it has any.
    pub fn get(&self) -> Option<String> {
        match self {
            Clipboard::Memory(text) => Some(text.clone()),
            Clipboard::System => unsafe { system_text() },
        }
    }

    pub fn set(&mut self, text: &str) {
        match self {
            Clipboard::Memory(memory) => *memory = text.to_string(),
            Clipboard::System => unsafe { set_system_text(text) },
        }
    }
}

unsafe fn system_text() -> Option<String> {
    unsafe {
        if !IsClipboardFormatAvailable(CF_UNICODETEXT as u32).as_bool() || !OpenClipboard(None).as_bool() {
            return None;
        }
        let handle = GetClipboardData(CF_UNICODETEXT as u32);
        let mut text = None;
        if !handle.0.is_null() {
            let memory = handle;
            let pointer = GlobalLock(memory) as *const u16;
            if !pointer.is_null() {
                let mut length = 0;
                while *pointer.add(length) != 0 {
                    length += 1;
                }
                text = Some(String::from_utf16_lossy(std::slice::from_raw_parts(pointer, length)));
                let _ = GlobalUnlock(memory);
            }
        }
        let _ = CloseClipboard();
        text
    }
}

unsafe fn set_system_text(text: &str) {
    unsafe {
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        if !OpenClipboard(None).as_bool() {
            return;
        }
        let _ = EmptyClipboard();
        let memory = GlobalAlloc(GMEM_MOVEABLE as u32, wide.len() * 2);
        if !memory.0.is_null() {
            let pointer = GlobalLock(memory) as *mut u16;
            if !pointer.is_null() {
                std::ptr::copy_nonoverlapping(wide.as_ptr(), pointer, wide.len());
                let _ = GlobalUnlock(memory);
                // The clipboard owns the memory once it takes it.
                if SetClipboardData(CF_UNICODETEXT as u32, Some(memory)).0.is_null() {
                    GlobalFree(memory);
                }
            } else {
                GlobalFree(memory);
            }
        }
        let _ = CloseClipboard();
    }
}
