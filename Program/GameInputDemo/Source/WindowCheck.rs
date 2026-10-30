// The demo supports exactly two window setups, both on a 2560 x 1440 monitor
// at 100% scaling (96 DPI), so that the panels can be laid out in fixed pixels:
//
// - Maximized, with a 48-pixel taskbar at the bottom of the monitor. That
//   leaves a 2560 x 1392 work area, and the title bar takes its top 23 pixels,
//   so the client area is 2560 x 1369.
// - Borderless fullscreen, where the client area covers the whole monitor:
//   2560 x 1440.
//
// Anything else gets an error in the window instead of the panels.

use windows_api::*;

pub const MONITOR_SIZE: [i32; 2] = [2560, 1440];
pub const WORK_AREA_SIZE: [i32; 2] = [2560, 1392];
pub const TITLE_BAR_HEIGHT: i32 = 23;

pub const SUPPORTED: &str = "GameInput Demo only supports two window setups, both on a \
    2560 x 1440 monitor at 100% scaling (96 DPI): maximized, with a 48-pixel taskbar at the \
    bottom, which leaves a 2560 x 1369 client area under the title bar; or borderless \
    fullscreen (F11), with a 2560 x 1440 client area that covers the monitor.";

fn size(rect: &RECT) -> [i32; 2] {
    [rect.right - rect.left, rect.bottom - rect.top]
}

fn describe(rect: &RECT) -> String {
    let [width, height] = size(rect);
    format!("{width} x {height} at ({}, {})", rect.left, rect.top)
}

// What's wrong with the window's setup, one problem per item, or nothing if
// it's one of the two that the demo supports.
pub unsafe fn problems(window: HWND) -> Vec<String> {
    let mut problems = Vec::new();
    unsafe {
        let dpi = GetDpiForWindow(window);
        if dpi != USER_DEFAULT_SCREEN_DPI as u32 {
            problems.push(format!(
                "The window's DPI is {dpi} ({}% scaling), not 96 (100%).",
                dpi * 100 / USER_DEFAULT_SCREEN_DPI as u32
            ));
        }

        let mut monitor = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(
            MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST as u32),
            &mut monitor,
        )
        .as_bool()
        {
            problems.push("Windows didn't say which monitor the window is on.".to_string());
            return problems;
        }
        let [width, height] = size(&monitor.rcMonitor);
        if [width, height] != MONITOR_SIZE {
            problems.push(format!(
                "The monitor is {width} x {height}, not 2560 x 1440."
            ));
        }

        // The client area, in screen coordinates.
        let mut client = RECT::default();
        let mut origin = POINT::default();
        if !GetClientRect(window, &mut client).as_bool()
            || !ClientToScreen(window, &mut origin).as_bool()
        {
            problems.push("Windows didn't say where the window's client area is.".to_string());
            return problems;
        }
        let client = RECT {
            left: origin.x,
            top: origin.y,
            right: origin.x + client.right,
            bottom: origin.y + client.bottom,
        };

        // Borderless fullscreen is the window without a title bar.
        let fullscreen = GetWindowLongPtrW(window, GWL_STYLE) & WS_CAPTION as isize == 0;
        if fullscreen {
            let expected = monitor.rcMonitor;
            if client != expected {
                problems.push(format!(
                    "In borderless fullscreen, the client area is {}, not {}, which covers \
                     the monitor.",
                    describe(&client),
                    describe(&expected)
                ));
            }
        } else {
            if !IsZoomed(window).as_bool() {
                problems
                    .push("The window isn't maximized or in borderless fullscreen.".to_string());
            }
            let work = monitor.rcWork;
            let [width, height] = size(&work);
            if [width, height] != WORK_AREA_SIZE {
                problems.push(format!(
                    "The monitor's work area, without the taskbar, is {width} x {height}, not \
                     2560 x 1392."
                ));
            }
            let expected = RECT {
                top: work.top + TITLE_BAR_HEIGHT,
                ..work
            };
            if client != expected {
                problems.push(format!(
                    "The client area is {}, not {}, which is the work area under a \
                     23-pixel title bar.",
                    describe(&client),
                    describe(&expected)
                ));
            }
        }
    }
    problems
}
