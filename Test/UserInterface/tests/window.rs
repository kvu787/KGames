// Tests the program's window before its first frame, and where it keeps
// Windows' cursor. The window is made hidden and never shown, so nothing
// opens on a display, and nothing takes focus.

use user_interface::{Theme, ThemeKind, Window, WindowMode, keeps_cursor_in};
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
            let window = Window::create("Background", monitor, WindowMode::WindowedMaximized, background).unwrap();
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

// Windows' cursor is kept in, and centered on, the client area on the
// screen, and a minimized window has none: its client area is off the screen
// then, where a cursor kept in it would be lost, and Windows can give a
// minimized window focus, as it does as one is restored. Minimized here by
// its style, as showing it would put it on the taskbar.
#[test]
fn a_minimized_window_keeps_no_cursor() {
    unsafe {
        let monitor = MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY as u32);
        let window = Window::create("Cursor", monitor, WindowMode::WindowedMaximized, Theme::light().background).unwrap();
        let (width, height) = window.client_size();
        let area = window.cursor_area().expect("the client area");
        assert_eq!((area.right - area.left, area.bottom - area.top), (width as i32, height as i32));
        let style = GetWindowLongPtrW(window.hwnd(), GWL_STYLE);
        SetWindowLongPtrW(window.hwnd(), GWL_STYLE, style | WS_MINIMIZE as isize);
        assert!(IsIconic(window.hwnd()).as_bool());
        assert!(window.cursor_area().is_none());
    }
}

// The window keeps Windows' cursor in its client area while it has focus, but
// starts to only once no mouse button is held, like the click that gave it
// focus on its title bar's minimize or close button, which Windows tracks
// until it's let go of there: kept in the client area, the cursor would never
// be let go of on the button, and the window would neither minimize nor
// close. A button held while it keeps the cursor, like for a drag, doesn't
// let it go, and a minimized window keeps none.
#[test]
fn the_click_that_gives_the_window_focus_is_let_go_of_before_the_window_keeps_the_cursor() {
    let area = RECT { left: 0, top: 23, right: 2560, bottom: 1392 };
    assert_eq!(keeps_cursor_in(true, false, Some(area), true), None);
    assert_eq!(keeps_cursor_in(true, false, Some(area), false), Some(area));
    assert_eq!(keeps_cursor_in(true, true, Some(area), true), Some(area));
    assert_eq!(keeps_cursor_in(false, true, Some(area), false), None);
    assert_eq!(keeps_cursor_in(true, true, None, false), None);
}
