// Copy, cut and paste: Windows' clipboard for the user, and one of the
// interface's own for scripts and tests, which must never read or change
// what the user copied.

use windows_api::*;

pub enum Clipboard {
    // Windows' clipboard, opened as the window that owns what it writes:
    // EmptyClipboard makes the window that has the clipboard open its owner,
    // and with no window, SetClipboardData fails.
    Windows(HWND),
    Own(String),
    // A clipboard that can't be read or written, like Windows' while another
    // program keeps it open, for tests.
    Unavailable,
}

impl Clipboard {
    pub fn text(&self) -> Option<String> {
        match self {
            Clipboard::Own(text) => Some(text.clone()),
            Clipboard::Windows(owner) => unsafe { read_windows(*owner) },
            Clipboard::Unavailable => None,
        }
    }

    // Puts the text on the clipboard, and says whether it's there, so that
    // a cut takes the text away only once it's on the clipboard.
    pub fn set_text(&mut self, text: &str) -> bool {
        match self {
            Clipboard::Own(own) => {
                *own = text.to_string();
                true
            }
            Clipboard::Windows(owner) => unsafe { write_windows(*owner, text) },
            Clipboard::Unavailable => false,
        }
    }
}

// Opens the clipboard, trying again for a few milliseconds while another
// program has it open, which it can only be for a moment.
unsafe fn open(owner: HWND) -> bool {
    for attempt in 0..5 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        if unsafe { OpenClipboard(Some(owner)) }.as_bool() {
            return true;
        }
    }
    false
}

unsafe fn read_windows(owner: HWND) -> Option<String> {
    unsafe {
        if !IsClipboardFormatAvailable(CF_UNICODETEXT as u32).as_bool() || !open(owner) {
            return None;
        }
        let mut text = None;
        let handle = GetClipboardData(CF_UNICODETEXT as u32);
        if !handle.0.is_null() {
            let units = GlobalLock(handle) as *const u16;
            if !units.is_null() {
                // The text ends at a null character, but no further than its
                // memory, in case another program left the null out.
                let most = GlobalSize(handle) / 2;
                let units = std::slice::from_raw_parts(units, most);
                let length = units.iter().position(|unit| *unit == 0).unwrap_or(most);
                text = Some(String::from_utf16_lossy(&units[..length]));
                let _ = GlobalUnlock(handle);
            }
        }
        let _ = CloseClipboard();
        text
    }
}

unsafe fn write_windows(owner: HWND, text: &str) -> bool {
    unsafe {
        if !open(owner) {
            return false;
        }
        let mut written = false;
        if EmptyClipboard().as_bool() {
            let units: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let memory = GlobalAlloc(GMEM_MOVEABLE as u32, units.len() * 2);
            if !memory.0.is_null() {
                let destination = GlobalLock(memory) as *mut u16;
                if !destination.is_null() {
                    std::ptr::copy_nonoverlapping(units.as_ptr(), destination, units.len());
                    let _ = GlobalUnlock(memory);
                    // Once the clipboard takes the memory, it's the clipboard's.
                    written = !SetClipboardData(CF_UNICODETEXT as u32, Some(memory)).0.is_null();
                }
                if !written {
                    let _ = GlobalFree(memory);
                }
            }
        }
        let _ = CloseClipboard();
        written
    }
}
