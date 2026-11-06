// A program's Win32 window. Its input comes from GameInput (Devices.rs and
// InputReader.rs), not from its messages, which it handles only to be a
// window: its focus, its cursor, its size and its close button.
//
// - It's maximized or borderless fullscreen, the two supported setups, and
//   switches between them without taking focus.
// - It says when it gets and loses focus, on the clock that input is timed
//   on, as input without focus is ignored.
// - Drags keep going past its edges, as it captures the mouse while a button
//   is held.
// - It checks the setup whenever the window, the displays or the taskbar
//   change.

use windows_api::*;
use windows_core::{Error, Result, w};

use crate::clock;
use crate::geometry::Point;
use crate::settings::WindowMode;
use crate::ui::Cursor;

// The window class that RunInBackground finds programs' windows by.
const CLASS: windows_core::PCWSTR = w!("MainWindow");
const OVERLAY_CLASS: windows_core::PCWSTR = w!("OverlayWindow");

// The one supported display, and the taskbar on it.
pub const DISPLAY_WIDTH: i32 = 2560;
pub const DISPLAY_HEIGHT: i32 = 1440;
pub const TASKBAR_HEIGHT: i32 = 48;
pub const MAXIMIZED_CLIENT_HEIGHT: i32 = 1369;

// The window got focus, or lost it, at a time on the counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusChange {
    pub time: i64,
    pub gained: bool,
}

struct State {
    focus_changes: Vec<FocusChange>,
    focused: bool,
    minimized: bool,
    close_requested: bool,
    setup_changed: bool,
    // Whether the cursor was on the client area when last asked.
    cursor_inside: bool,
    cursor: HCURSOR,
}

pub struct Window {
    hwnd: HWND,
    state: Box<State>,
    mode: WindowMode,
    cursors: [HCURSOR; 7],
}

impl Window {
    // A program's window on the monitor, hidden until shown.
    pub fn create(title: &str, monitor: HMONITOR, mode: WindowMode) -> Result<Window> {
        unsafe {
            prepare_process();
            register_class(CLASS)?;
            let info = monitor_info(monitor);
            let work = info.rcWork;
            // Its restored size and place, if it's ever restored: a quarter
            // of the work area, in its middle.
            let width = (work.right - work.left) / 2;
            let height = (work.bottom - work.top) / 2;
            let x = work.left + width / 2;
            let y = work.top + height / 2;
            let title = wide(title);
            let hwnd = CreateWindowExW(
                WS_EX_APPWINDOW as u32,
                CLASS,
                windows_core::PCWSTR(title.as_ptr()),
                WS_OVERLAPPEDWINDOW as u32,
                x,
                y,
                width,
                height,
                None,
                None,
                Some(GetModuleHandleW(None)),
                None,
            );
            check_handle(hwnd.0)?;
            let mut window = Window::wrap(hwnd);
            window.apply_mode(mode, false);
            Ok(window)
        }
    }

    // A window without a frame that floats over the others, like
    // IdentifyDisplays' boxes, in screen pixels.
    pub fn create_overlay(title: &str, rect: RECT) -> Result<Window> {
        unsafe {
            prepare_process();
            register_class(OVERLAY_CLASS)?;
            let title = wide(title);
            let hwnd = CreateWindowExW(
                (WS_EX_TOPMOST | WS_EX_TOOLWINDOW) as u32,
                OVERLAY_CLASS,
                windows_core::PCWSTR(title.as_ptr()),
                WS_POPUP,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                None,
                None,
                Some(GetModuleHandleW(None)),
                None,
            );
            check_handle(hwnd.0)?;
            Ok(Window::wrap(hwnd))
        }
    }

    unsafe fn wrap(hwnd: HWND) -> Window {
        unsafe {
            let cursors = [IDC_ARROW, IDC_IBEAM, IDC_HAND, IDC_SIZEALL, IDC_SIZEWE, IDC_SIZENS, IDC_CROSS]
                .map(|cursor| LoadCursorW(None, cursor));
            let mut state = Box::new(State {
                focus_changes: Vec::new(),
                focused: false,
                minimized: false,
                close_requested: false,
                setup_changed: true,
                cursor_inside: false,
                cursor: cursors[0],
            });
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, &mut *state as *mut State as isize);
            Window {
                hwnd,
                state,
                mode: WindowMode::Maximized,
                cursors,
            }
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    // Shows the window, and gives it focus unless `activate` is false, in
    // which case it opens on top of the others without taking focus from the
    // window that has it.
    pub fn show(&mut self, activate: bool) {
        unsafe {
            let _ = ShowWindow(self.hwnd, if activate { SW_SHOW } else { SW_SHOWNA });
        }
    }

    pub fn mode(&self) -> WindowMode {
        self.mode
    }

    // Switches between maximized and borderless fullscreen, on the monitor
    // that the window is on, without taking focus.
    pub fn set_mode(&mut self, mode: WindowMode) {
        if mode != self.mode {
            self.apply_mode(mode, true);
        }
    }

    fn apply_mode(&mut self, mode: WindowMode, visible: bool) {
        self.mode = mode;
        unsafe {
            let info = monitor_info(MonitorFromWindow(self.hwnd, MONITOR_DEFAULTTONEAREST as u32));
            let visible = if visible { WS_VISIBLE as u32 } else { 0 };
            let flags = (SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOOWNERZORDER) as u32;
            match mode {
                WindowMode::BorderlessFullscreen => {
                    let area = info.rcMonitor;
                    SetWindowLongPtrW(self.hwnd, GWL_STYLE, (WS_POPUP | visible) as isize);
                    let _ = SetWindowPos(
                        self.hwnd,
                        None,
                        area.left,
                        area.top,
                        area.right - area.left,
                        area.bottom - area.top,
                        flags,
                    );
                }
                WindowMode::Maximized => {
                    // Maximized, as Windows would, but without activating:
                    // the work area, with the frame's borders past its edges,
                    // which leaves the title bar at its top.
                    let style = WS_OVERLAPPEDWINDOW as u32 | WS_MAXIMIZE as u32 | visible;
                    SetWindowLongPtrW(self.hwnd, GWL_STYLE, style as isize);
                    let dpi = GetDpiForWindow(self.hwnd).max(96);
                    let mut frame = RECT::default();
                    let _ = AdjustWindowRectExForDpi(&mut frame, WS_OVERLAPPEDWINDOW as u32, false, WS_EX_APPWINDOW as u32, dpi);
                    let border = frame.right;
                    let work = info.rcWork;
                    let _ = SetWindowPos(
                        self.hwnd,
                        None,
                        work.left - border,
                        work.top - border,
                        work.right - work.left + border * 2,
                        work.bottom - work.top + border * 2,
                        flags,
                    );
                }
            }
        }
        self.state.setup_changed = true;
    }

    // Handles the window's messages without waiting. Returns false once the
    // window should close.
    pub fn pump(&mut self) -> bool {
        unsafe {
            let mut message = MSG::default();
            while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE as u32).as_bool() {
                if message.message == WM_QUIT as u32 {
                    return false;
                }
                // No TranslateMessage: typed text comes from GameInput's keys,
                // and translating the messages too would use up dead keys.
                DispatchMessageW(&message);
            }
        }
        !self.state.close_requested
    }

    // Waits for a message, like while the window is minimized.
    pub fn wait(&self) {
        unsafe {
            let _ = WaitMessage();
        }
    }

    // Waits for a message, or for the milliseconds to pass.
    pub fn wait_for(&self, milliseconds: u32) {
        unsafe {
            let _ = MsgWaitForMultipleObjects(None, false, milliseconds, QS_ALLINPUT as u32);
        }
    }

    // The focus changes since the last call, oldest first.
    pub fn take_focus_changes(&mut self) -> Vec<FocusChange> {
        std::mem::take(&mut self.state.focus_changes)
    }

    // Where Windows' cursor is, in the client area's pixels, and whether it's
    // on the client area, with no other window over it there.
    pub fn cursor(&mut self) -> (Point, bool) {
        let mut point = POINT::default();
        unsafe {
            let _ = GetCursorPos(&mut point);
            let over = WindowFromPoint(point) == self.hwnd;
            let _ = ScreenToClient(self.hwnd, &mut point);
            let (width, height) = self.client_size();
            let inside = over && point.x >= 0 && point.y >= 0 && (point.x as u32) < width && (point.y as u32) < height;
            self.state.cursor_inside = inside;
            (Point::new(point.x as f32, point.y as f32), inside)
        }
    }

    // Whether Windows' cursor is on the client area.
    pub fn cursor_over(&mut self) -> bool {
        self.cursor().1
    }

    // Captures the mouse, so that a drag keeps going past the window's edges,
    // or lets it go.
    pub fn set_capture(&mut self, capture: bool) {
        unsafe {
            let captured = GetCapture() == self.hwnd;
            if capture && !captured {
                SetCapture(self.hwnd);
            } else if !capture && captured {
                let _ = ReleaseCapture();
            }
        }
    }

    pub fn focused(&self) -> bool {
        self.state.focused
    }

    pub fn minimized(&self) -> bool {
        self.state.minimized
    }

    // Whether the window was asked to close, like by its close button.
    pub fn close_requested(&self) -> bool {
        self.state.close_requested
    }

    pub fn request_close(&mut self) {
        self.state.close_requested = true;
    }

    pub fn client_size(&self) -> (u32, u32) {
        let mut rect = RECT::default();
        unsafe {
            let _ = GetClientRect(self.hwnd, &mut rect);
        }
        ((rect.right - rect.left).max(0) as u32, (rect.bottom - rect.top).max(0) as u32)
    }

    pub fn monitor(&self) -> HMONITOR {
        unsafe { MonitorFromWindow(self.hwnd, MONITOR_DEFAULTTONEAREST as u32) }
    }

    pub fn set_title(&self, title: &str) {
        let title = wide(title);
        unsafe {
            let _ = SetWindowTextW(self.hwnd, windows_core::PCWSTR(title.as_ptr()));
        }
    }

    pub fn set_cursor(&mut self, cursor: Cursor) {
        let index = match cursor {
            Cursor::Arrow => 0,
            Cursor::Text => 1,
            Cursor::Hand => 2,
            Cursor::Move => 3,
            Cursor::ResizeHorizontal => 4,
            Cursor::ResizeVertical => 5,
            Cursor::Crosshair => 6,
        };
        let cursor = self.cursors[index];
        if self.state.cursor != cursor {
            self.state.cursor = cursor;
            // The pointer may be still, so set it now, not at its next move.
            if self.state.focused && self.state.cursor_inside {
                unsafe {
                    SetCursor(Some(cursor));
                }
            }
        }
    }

    // Whether the setup may have changed since the last call.
    pub fn take_setup_changed(&mut self) -> bool {
        std::mem::replace(&mut self.state.setup_changed, false)
    }

    // What's wrong with the window's setup, if anything: the programs only
    // support a 2560 × 1440 display at 100% scale, with a 48-pixel taskbar at
    // its bottom, and the window maximized or borderless fullscreen.
    pub fn setup_problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        unsafe {
            let info = monitor_info(self.monitor());
            let monitor = info.rcMonitor;
            let (width, height) = (monitor.right - monitor.left, monitor.bottom - monitor.top);
            if (width, height) != (DISPLAY_WIDTH, DISPLAY_HEIGHT) {
                problems.push(format!("The display is {width} × {height} pixels. It must be {DISPLAY_WIDTH} × {DISPLAY_HEIGHT}."));
            }
            let dpi = GetDpiForWindow(self.hwnd);
            if dpi != 96 {
                problems.push(format!("The display's scale is {}%. It must be 100%.", dpi * 100 / 96));
            }
            let (client_width, client_height) = self.client_size();
            match self.mode {
                WindowMode::Maximized => {
                    let work = info.rcWork;
                    let taskbar_ok = work.left == monitor.left
                        && work.top == monitor.top
                        && work.right == monitor.right
                        && monitor.bottom - work.bottom == TASKBAR_HEIGHT;
                    if !taskbar_ok {
                        problems.push(format!("The taskbar must be {TASKBAR_HEIGHT} pixels tall, at the bottom of the display."));
                    }
                    if !IsZoomed(self.hwnd).as_bool() {
                        problems.push("The window must be maximized, or borderless fullscreen with F11.".to_string());
                    } else if (client_width as i32, client_height as i32) != (DISPLAY_WIDTH, MAXIMIZED_CLIENT_HEIGHT) {
                        problems.push(format!(
                            "The window's client area is {client_width} × {client_height}. Maximized, it must be {DISPLAY_WIDTH} × {MAXIMIZED_CLIENT_HEIGHT}."
                        ));
                    }
                }
                WindowMode::BorderlessFullscreen => {
                    if (client_width as i32, client_height as i32) != (DISPLAY_WIDTH, DISPLAY_HEIGHT) {
                        problems.push(format!(
                            "The window's client area is {client_width} × {client_height}. In borderless fullscreen, it must be {DISPLAY_WIDTH} × {DISPLAY_HEIGHT}."
                        ));
                    }
                }
            }
        }
        problems
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) unsafe fn monitor_info(monitor: HMONITOR) -> MONITORINFO {
    unsafe {
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut info);
        info
    }
}

// Once per process: per-monitor DPI awareness, so that Windows never scales
// the window, and no power throttling, which would slow the main loop on
// efficiency cores while the window is in the background.
unsafe fn prepare_process() {
    static PREPARED: std::sync::Once = std::sync::Once::new();
    PREPARED.call_once(|| unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let state = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION as u32,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED as u32,
            StateMask: 0,
        };
        let _ = SetProcessInformation(
            GetCurrentProcess(),
            ProcessPowerThrottling,
            &state as *const _ as *const _,
            size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
    });
}

unsafe fn register_class(name: windows_core::PCWSTR) -> Result<()> {
    unsafe {
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(window_procedure),
            hInstance: GetModuleHandleW(None),
            lpszClassName: name,
            ..Default::default()
        };
        // A second window of the class finds it registered already.
        if RegisterClassExW(&class).0 == 0 {
            let error = Error::from_thread();
            if error.code().0 as u32 != 0x8007_0582 {
                return Err(error);
            }
        }
        Ok(())
    }
}

fn low_word(value: isize) -> i32 {
    (value & 0xFFFF) as u16 as i16 as i32
}

unsafe extern "system" fn window_procedure(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut State;
        if state.is_null() {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        let state = &mut *state;
        let message_i = message as i32;
        match message_i {
            WM_SETFOCUS => {
                state.focused = true;
                state.focus_changes.push(FocusChange {
                    time: clock::now(),
                    gained: true,
                });
                LRESULT(0)
            }
            WM_KILLFOCUS => {
                state.focused = false;
                state.focus_changes.push(FocusChange {
                    time: clock::now(),
                    gained: false,
                });
                let _ = ReleaseCapture();
                LRESULT(0)
            }
            WM_SYSCOMMAND => {
                // Alt alone would open the window's menu, and take the keys.
                if (wparam.0 & 0xFFF0) as i32 == SC_KEYMENU {
                    LRESULT(0)
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
            WM_SETCURSOR => {
                if low_word(lparam.0) == HTCLIENT {
                    SetCursor(Some(state.cursor));
                    LRESULT(1)
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
            WM_SIZE => {
                state.minimized = wparam.0 as i32 == SIZE_MINIMIZED;
                state.setup_changed = true;
                LRESULT(0)
            }
            WM_MOVE | WM_DISPLAYCHANGE | WM_SETTINGCHANGE => {
                state.setup_changed = true;
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
            WM_DPICHANGED => {
                // The window keeps its size: it's a setup that the check says
                // isn't supported.
                state.setup_changed = true;
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_PAINT => {
                let _ = ValidateRect(Some(hwnd), None);
                LRESULT(0)
            }
            WM_CLOSE => {
                state.close_requested = true;
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}
