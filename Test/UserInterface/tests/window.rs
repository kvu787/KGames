// Tests the program's window before its first frame, and where it keeps
// Windows' cursor. The window is made hidden and never shown, so nothing
// opens on a display, and nothing takes focus.

use user_interface::{ClipHolder, Theme, ThemeKind, Window, clip_holder, keeps_cursor_in};
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
            let window = Window::create("Background", monitor, background).unwrap();
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
        let window = Window::create("Cursor", monitor, Theme::light().background).unwrap();
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
// focus. A button held while it keeps the cursor, like for a drag, doesn't
// let it go, and a minimized window keeps none.
#[test]
fn the_click_that_gives_the_window_focus_is_let_go_of_before_the_window_keeps_the_cursor() {
    let area = RECT { left: 0, top: 0, right: 2560, bottom: 1440 };
    assert_eq!(keeps_cursor_in(true, true, false, Some(area), true), None);
    assert_eq!(keeps_cursor_in(true, true, false, Some(area), false), Some(area));
    assert_eq!(keeps_cursor_in(true, true, true, Some(area), true), Some(area));
    assert_eq!(keeps_cursor_in(false, true, true, Some(area), false), None);
    assert_eq!(keeps_cursor_in(true, true, true, None, false), None);
}

// A window can get focus without being the foreground window, when Windows
// won't let it take the foreground from the program that has it, and it
// mustn't keep the cursor of someone using that program then.
#[test]
fn a_window_with_focus_that_isn_t_the_foreground_window_keeps_no_cursor() {
    let area = RECT { left: 0, top: 0, right: 2560, bottom: 1440 };
    assert_eq!(keeps_cursor_in(true, false, false, Some(area), false), None);
    assert_eq!(keeps_cursor_in(true, false, true, Some(area), false), None);
}

// The cursor's confinement is shared by every program, and the window and
// the program that gets focus hear of it in no fixed order: as the window
// lets the cursor go, it releases the confinement and puts the cursor in its
// middle only if it still holds it, or nobody does, and leaves another
// program's alone.
#[test]
fn the_window_lets_go_only_of_the_cursor_s_confinement_that_it_holds() {
    let area = RECT { left: 0, top: 0, right: 2560, bottom: 1440 };
    let free = RECT { left: -2560, top: 0, right: 5120, bottom: 1440 };
    let other = RECT { left: 2560, top: 0, right: 5120, bottom: 1440 };
    assert_eq!(clip_holder(area, area, free), ClipHolder::Window);
    assert_eq!(clip_holder(free, area, free), ClipHolder::Nobody);
    assert_eq!(clip_holder(other, area, free), ClipHolder::Other);
}
