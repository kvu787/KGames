// Tests the program's window before its first frame. The window is made
// hidden and never shown, so nothing opens on a display, and nothing takes
// focus.

use user_interface::{Theme, ThemeKind, Window, WindowMode};
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
