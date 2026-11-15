// Tests the program's window before its first frame, and where it keeps
// Windows' cursor. The window is made hidden and never shown, so nothing
// opens on a display, and nothing takes focus.

use user_interface::{Theme, ThemeKind, Window};
use windows_api::*;

// Until its first frame is presented, a window shows what it paints with GDI.
// One that paints nothing shows white, which flashed before the dark theme's
// first frame. So it paints the theme's background wherever Windows asks it
// to: printed into a bitmap, as WM_PRINT and PrintWindow ask, its client area
// is the background, in both themes.
#[test]
fn a_window_shows_its_themes_background_before_its_first_frame() {
    for kind in [ThemeKind::Light, ThemeKind::Dark] {
        let background = Theme::new(kind).background;
        let [r, g, b, _] = background.to_bytes();
        let expected = r as u32 | (g as u32) << 8 | (b as u32) << 16;
        unsafe {
            let monitor = MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY as u32);
            let window = Window::create("Background", monitor, background, true).unwrap();
            let (width, height) = window.client_size();
            let screen = GetDC(None);
            let memory = CreateCompatibleDC(Some(screen));
            let bitmap = CreateCompatibleBitmap(screen, width as i32, height as i32);
            let previous = SelectObject(memory, HGDIOBJ(bitmap.0));
            let flags = (PRF_CLIENT | PRF_ERASEBKGND) as isize;
            SendMessageW(window.hwnd(), WM_PRINT as u32, WPARAM(memory.0 as usize), LPARAM(flags));
            let points = [(0, 0), (width / 2, height / 2), (width - 1, height - 1)];
            let pixels: Vec<u32> = points.iter().map(|&(x, y)| GetPixel(memory, x as i32, y as i32).0).collect();
            SelectObject(memory, previous);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(memory);
            ReleaseDC(None, screen);
            assert_eq!(pixels, vec![expected; points.len()], "{kind:?}: the client area at {points:?}, as 0x00BBGGRR");
        }
    }
}

// Windows' cursor is kept in, and placed in, the client area on the screen,
// and a minimized window has none: its client area is off the screen
// then, where a cursor kept in it would be lost, and Windows can give a
// minimized window focus, as it does as one is restored. Minimized here by
// its style, as showing it would put it on the taskbar.
#[test]
fn a_minimized_window_keeps_no_cursor() {
    unsafe {
        let monitor = MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY as u32);
        let window = Window::create("Cursor", monitor, Theme::light().background, true).unwrap();
        let (width, height) = window.client_size();
        let area = window.cursor_area().expect("the client area");
        assert_eq!((area.right - area.left, area.bottom - area.top), (width as i32, height as i32));
        let style = GetWindowLongPtrW(window.hwnd(), GWL_STYLE);
        SetWindowLongPtrW(window.hwnd(), GWL_STYLE, style | WS_MINIMIZE as isize);
        assert!(IsIconic(window.hwnd()).as_bool());
        assert!(window.cursor_area().is_none());
    }
}

// The window covers its display, and its client area must be the whole
// display, where it is too, not only its size: one that's moved, or whose
// displays were arranged anew under it, says so. Moved here while hidden,
// without activating it.
#[test]
fn a_window_moved_off_its_display_says_so() {
    unsafe {
        let monitor = MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY as u32);
        let window = Window::create("Moved", monitor, Theme::light().background, true).unwrap();
        let about_the_client_area = |problems: &[String]| problems.iter().any(|problem| problem.starts_with("The window's client area"));
        assert!(!about_the_client_area(&window.setup_problems()), "{:?}", window.setup_problems());
        let area = window.cursor_area().unwrap();
        let flags = (SWP_NOSIZE | SWP_NOACTIVATE) as u32;
        assert!(SetWindowPos(window.hwnd(), None, area.left + 10, area.top, 0, 0, flags).as_bool());
        assert!(about_the_client_area(&window.setup_problems()), "{:?}", window.setup_problems());
    }
}
