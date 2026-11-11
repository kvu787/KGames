// A program's Win32 window. Its input comes from GameInput
// (platform/input_devices.rs and input/reader.rs), not from its messages,
// which it handles only to be a window: its focus, its cursor, its size and
// its close button.
//
// - It's maximized or borderless fullscreen, the two supported setups, and
//   switches between them without taking focus.
// - It says when it gets and loses focus, on the clock that input is timed
//   on, as input without focus is ignored. The time is when it handles the
//   message, which Windows sends it once the focus has moved, so it can be
//   a little after.
// - Drags keep going past its edges, as it captures the mouse while a button
//   is held.
// - It says when its setup may have changed: when it moves or changes size,
//   or the displays, their scale or the taskbar change.
// - It asks for a dark title bar in the dark theme, as Windows 11's apps
//   have, with DWMWA_USE_IMMERSIVE_DARK_MODE, which Microsoft's documentation
//   describes as drawing the frame dark while Windows' own dark mode is on.

use std::cell::Cell;
use std::ptr::NonNull;

use windows_api::*;
use windows_core::{BOOL, Error, PCWSTR, Result, w};

use crate::application::settings::WindowMode;
use crate::core::interaction::Cursor;
use crate::drawing::geometry::Point;
use crate::platform::clock;

// RunInBackground finds programs' windows by this class.
const CLASS: PCWSTR = w!("MainWindow");
const OVERLAY_CLASS: PCWSTR = w!("OverlayWindow");

// The one supported display and its taskbar.
pub const DISPLAY_WIDTH: i32 = 2560;
pub const DISPLAY_HEIGHT: i32 = 1440;
pub const TASKBAR_HEIGHT: i32 = 48;
pub const MAXIMIZED_CLIENT_HEIGHT: i32 = 1369;

const CURSORS: [Cursor; 10] = [
    Cursor::Arrow,
    Cursor::Text,
    Cursor::Hand,
    Cursor::Move,
    Cursor::ResizeHorizontal,
    Cursor::ResizeVertical,
    Cursor::ResizeDiagonal,
    Cursor::ResizeAntidiagonal,
    Cursor::Crosshair,
    Cursor::NotAllowed,
];

// The window got focus, or lost it, at a time on the counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusChange {
    pub time: i64,
    pub gained: bool,
}

// What the window procedure shares with the window, by GWLP_USERDATA.
//
// The window procedure runs inside the Windows calls that send the window
// messages, while one of the window's methods is running, like
// DispatchMessageW in `pump`, SetWindowPos in `apply_mode` and `show`,
// ShowWindow, SetCapture and ReleaseCapture. It runs inside its own calls
// too, like ReleaseCapture and DefWindowProcW. So both only ever hold
// `&Shared`, which may overlap, and change it through its cells. A
// `&mut Shared` on either side would alias the other's reference, which is
// undefined behavior even when it works.
struct Shared {
    focus_changes: Cell<Vec<FocusChange>>,
    focused: Cell<bool>,
    minimized: Cell<bool>,
    close_requested: Cell<bool>,
    setup_changed: Cell<bool>,
    // Whether the cursor was on the client area when last asked.
    cursor_inside: Cell<bool>,
    cursor: Cell<HCURSOR>,
}

impl Shared {
    // The window got focus, or lost it, now.
    fn focus_changed(&self, gained: bool) {
        let time = clock::now();
        self.focused.set(gained);
        let mut changes = self.focus_changes.take();
        changes.push(FocusChange { time, gained });
        self.focus_changes.set(changes);
    }
}

pub struct Window {
    hwnd: HWND,
    // From Box::into_raw, not a Box, which claims its contents as uniquely as
    // a `&mut` does: writing through it, or moving the Window, would
    // invalidate GWLP_USERDATA's copy, which the window procedure goes on
    // using. Drop frees it, after the window is destroyed.
    shared: NonNull<Shared>,
    mode: WindowMode,
    cursors: [HCURSOR; CURSORS.len()],
    dark_title_bar: Option<bool>,
}

impl Window {
    // A program's window on the monitor, hidden until shown.
    pub fn create(title: &str, monitor: HMONITOR, mode: WindowMode) -> Result<Window> {
        unsafe {
            prepare_process();
            register_class(CLASS)?;
            // Where it would be if it were ever restored: half the work area,
            // in its middle.
            let work = monitor_info(monitor).rcWork;
            let (width, height) = ((work.right - work.left) / 2, (work.bottom - work.top) / 2);
            let title = wide(title);
            let hwnd = CreateWindowExW(
                WS_EX_APPWINDOW as u32,
                CLASS,
                PCWSTR(title.as_ptr()),
                WS_OVERLAPPEDWINDOW as u32,
                work.left + width / 2,
                work.top + height / 2,
                width,
                height,
                None,
                None,
                Some(GetModuleHandleW(None)),
                None,
            );
            if hwnd.0.is_null() {
                return Err(Error::from_thread());
            }
            let mut window = Window::wrap(hwnd);
            window.apply_mode(mode, false);
            Ok(window)
        }
    }

    // A window without a frame that floats over the others, like one of
    // IdentifyDisplays' boxes, in screen pixels.
    pub fn create_overlay(title: &str, rect: RECT) -> Result<Window> {
        unsafe {
            prepare_process();
            register_class(OVERLAY_CLASS)?;
            let title = wide(title);
            let hwnd = CreateWindowExW(
                (WS_EX_TOPMOST | WS_EX_TOOLWINDOW) as u32,
                OVERLAY_CLASS,
                PCWSTR(title.as_ptr()),
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
            if hwnd.0.is_null() {
                return Err(Error::from_thread());
            }
            Ok(Window::wrap(hwnd))
        }
    }

    unsafe fn wrap(hwnd: HWND) -> Window {
        unsafe {
            let cursors = [
                IDC_ARROW,
                IDC_IBEAM,
                IDC_HAND,
                IDC_SIZEALL,
                IDC_SIZEWE,
                IDC_SIZENS,
                IDC_SIZENWSE,
                IDC_SIZENESW,
                IDC_CROSS,
                IDC_NO,
            ]
            .map(|cursor| LoadCursorW(None, cursor));
            let shared = Box::new(Shared {
                focus_changes: Cell::new(Vec::new()),
                focused: Cell::new(false),
                minimized: Cell::new(false),
                close_requested: Cell::new(false),
                setup_changed: Cell::new(true),
                cursor_inside: Cell::new(false),
                cursor: Cell::new(cursors[0]),
            });
            let shared = NonNull::new_unchecked(Box::into_raw(shared));
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, shared.as_ptr() as isize);
            Window {
                hwnd,
                shared,
                mode: WindowMode::MaximizedWindowed,
                cursors,
                dark_title_bar: None,
            }
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    fn shared(&self) -> &Shared {
        // Freed only by Drop.
        unsafe { self.shared.as_ref() }
    }

    // Shows the window. Without `activate`, it opens beneath the others,
    // without taking focus from the window that has it.
    pub fn show(&mut self, activate: bool) {
        unsafe {
            if activate {
                set_power_throttling(false);
                let _ = ShowWindow(self.hwnd, SW_SHOW);
                return;
            }
            // A window that mustn't take focus, like a test run's while
            // someone works or plays a game, goes beneath the other windows,
            // where it never covers theirs or takes their pointer, and a click
            // on it never activates it. Screenshots of it work all the same.
            set_power_throttling(true);
            let style = GetWindowLongPtrW(self.hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(self.hwnd, GWL_EXSTYLE, style | WS_EX_NOACTIVATE as isize);
            let _ = ShowWindow(self.hwnd, SW_SHOWNA);
            let flags = (SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER) as u32;
            let _ = SetWindowPos(self.hwnd, Some(HWND_BOTTOM), 0, 0, 0, 0, flags);
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
            let (style, area) = match mode {
                WindowMode::BorderlessFullscreen => (WS_POPUP | visible, info.rcMonitor),
                WindowMode::MaximizedWindowed => {
                    // Maximized as Windows would, but without activating, which
                    // Windows has no documented way to do: ShowWindow's
                    // SW_SHOWMAXIMIZED activates the window. So the style says
                    // maximized, though the documentation doesn't list
                    // WS_MAXIMIZE among the styles that may change after the
                    // window is made, and the window takes the work area, with
                    // the frame's borders past its edges, which leaves the
                    // title bar at its top.
                    let dpi = GetDpiForWindow(self.hwnd).max(USER_DEFAULT_SCREEN_DPI as u32);
                    let mut frame = RECT::default();
                    let _ = AdjustWindowRectExForDpi(&mut frame, WS_OVERLAPPEDWINDOW as u32, false, WS_EX_APPWINDOW as u32, dpi);
                    let border = frame.right;
                    let work = info.rcWork;
                    let area = RECT {
                        left: work.left - border,
                        top: work.top - border,
                        right: work.right + border,
                        bottom: work.bottom + border,
                    };
                    (WS_OVERLAPPEDWINDOW as u32 | WS_MAXIMIZE as u32 | visible, area)
                }
            };
            SetWindowLongPtrW(self.hwnd, GWL_STYLE, style as isize);
            let _ = SetWindowPos(self.hwnd, None, area.left, area.top, area.right - area.left, area.bottom - area.top, flags);
        }
        self.shared().setup_changed.set(true);
    }

    // Dark or light, as the theme is, like the title bars of Windows 11's
    // apps. DWM draws the dark one; its documentation promises it while
    // Windows' own dark mode is on.
    pub fn set_dark_title_bar(&mut self, dark: bool) {
        if self.dark_title_bar == Some(dark) {
            return;
        }
        self.dark_title_bar = Some(dark);
        let value = BOOL::from(dark);
        unsafe {
            let _ = DwmSetWindowAttribute(
                self.hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
                &value as *const BOOL as *const core::ffi::c_void,
                size_of::<BOOL>() as u32,
            );
        }
    }

    // Handles the window's messages without waiting. False once it should
    // close.
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
        !self.shared().close_requested.get()
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
        self.shared().focus_changes.take()
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
            self.shared().cursor_inside.set(inside);
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
        self.shared().focused.get()
    }

    pub fn minimized(&self) -> bool {
        self.shared().minimized.get()
    }

    // Whether it was asked to close, like by its close button or Alt+F4.
    pub fn close_requested(&self) -> bool {
        self.shared().close_requested.get()
    }

    // Keeps the window open after all, when the program says it may not
    // close yet.
    pub fn cancel_close(&mut self) {
        self.shared().close_requested.set(false);
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
            let _ = SetWindowTextW(self.hwnd, PCWSTR(title.as_ptr()));
        }
    }

    pub fn set_cursor(&mut self, cursor: Cursor) {
        let index = CURSORS.iter().position(|known| *known == cursor).unwrap_or(0);
        let cursor = self.cursors[index];
        let shared = self.shared();
        if shared.cursor.get() != cursor {
            shared.cursor.set(cursor);
            // The pointer may be still, so set it now, not at its next move.
            if shared.focused.get() && shared.cursor_inside.get() {
                unsafe {
                    SetCursor(Some(cursor));
                }
            }
        }
    }

    // Whether the setup may have changed since the last call.
    pub fn take_setup_changed(&mut self) -> bool {
        self.shared().setup_changed.replace(false)
    }

    // What's wrong with the setup, if anything. Programs support only a
    // 2560 × 1440 display at 100% scale with a 48-pixel taskbar at its
    // bottom, and the window maximized or borderless fullscreen.
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
            if dpi != USER_DEFAULT_SCREEN_DPI as u32 {
                problems.push(format!("The display's scale is {}%. It must be 100%.", dpi * 100 / USER_DEFAULT_SCREEN_DPI as u32));
            }
            let (client_width, client_height) = self.client_size();
            let client = (client_width as i32, client_height as i32);
            match self.mode {
                WindowMode::MaximizedWindowed => {
                    let work = info.rcWork;
                    let taskbar_right = work.left == monitor.left
                        && work.top == monitor.top
                        && work.right == monitor.right
                        && monitor.bottom - work.bottom == TASKBAR_HEIGHT;
                    if !taskbar_right {
                        problems.push(format!("The taskbar must be {TASKBAR_HEIGHT} pixels tall, at the bottom of the display."));
                    }
                    if !IsZoomed(self.hwnd).as_bool() {
                        problems.push("The window must be maximized, or borderless fullscreen with F11.".to_string());
                    } else if client != (DISPLAY_WIDTH, MAXIMIZED_CLIENT_HEIGHT) {
                        problems.push(format!(
                            "The window's client area is {client_width} × {client_height}. Maximized windowed, it must be {DISPLAY_WIDTH} × {MAXIMIZED_CLIENT_HEIGHT}."
                        ));
                    }
                }
                WindowMode::BorderlessFullscreen => {
                    if client != (DISPLAY_WIDTH, DISPLAY_HEIGHT) {
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
            // Nothing can read it now: the window procedure finds no pointer,
            // and the window is gone.
            drop(Box::from_raw(self.shared.as_ptr()));
        }
    }
}

pub(crate) fn wide(text: &str) -> Vec<u16> {
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

// Once a process: per-monitor DPI awareness, so that Windows never scales the
// window.
unsafe fn prepare_process() {
    static PREPARED: std::sync::Once = std::sync::Once::new();
    PREPARED.call_once(|| unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    });
}

// Whether Windows may throttle the process's execution speed, as it does on
// efficiency cores while a process's window is in the background. A window
// that someone uses asks it not to, so that the main loop stays fast. One
// that opens without focus, like a coding agent's test run while someone
// plays a game, asks for efficiency mode instead, so that it runs at
// efficient clocks and the chip's power budget stays with the game.
unsafe fn set_power_throttling(throttle: bool) {
    let state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION as u32,
        ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED as u32,
        StateMask: if throttle { PROCESS_POWER_THROTTLING_EXECUTION_SPEED as u32 } else { 0 },
    };
    unsafe {
        let _ = SetProcessInformation(
            GetCurrentProcess(),
            ProcessPowerThrottling,
            &state as *const PROCESS_POWER_THROTTLING_STATE as *const core::ffi::c_void,
            size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
    }
}

unsafe fn register_class(name: PCWSTR) -> Result<()> {
    unsafe {
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(window_procedure),
            hInstance: GetModuleHandleW(None),
            lpszClassName: name,
            ..Default::default()
        };
        // ERROR_CLASS_ALREADY_EXISTS: a second window of the class.
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
        let shared = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Shared;
        if shared.is_null() {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        // Shared, never unique: see Shared.
        let shared = &*shared;
        let message = message as i32;
        match message {
            WM_SETFOCUS => {
                shared.focus_changed(true);
                LRESULT(0)
            }
            WM_KILLFOCUS => {
                shared.focus_changed(false);
                let _ = ReleaseCapture();
                LRESULT(0)
            }
            WM_SYSCOMMAND => {
                // Alt alone would open the window's menu, which takes the keys.
                if (wparam.0 & 0xFFF0) as i32 == SC_KEYMENU {
                    LRESULT(0)
                } else {
                    DefWindowProcW(hwnd, message as u32, wparam, lparam)
                }
            }
            WM_SETCURSOR => {
                if low_word(lparam.0) == HTCLIENT {
                    SetCursor(Some(shared.cursor.get()));
                    LRESULT(1)
                } else {
                    DefWindowProcW(hwnd, message as u32, wparam, lparam)
                }
            }
            WM_SIZE => {
                shared.minimized.set(wparam.0 as i32 == SIZE_MINIMIZED);
                shared.setup_changed.set(true);
                LRESULT(0)
            }
            WM_MOVE | WM_DISPLAYCHANGE | WM_SETTINGCHANGE => {
                shared.setup_changed.set(true);
                DefWindowProcW(hwnd, message as u32, wparam, lparam)
            }
            WM_DPICHANGED => {
                // It keeps its size: the check says what's wrong instead.
                shared.setup_changed.set(true);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_PAINT => {
                let _ = ValidateRect(Some(hwnd), None);
                LRESULT(0)
            }
            WM_CLOSE => {
                shared.close_requested.set(true);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message as u32, wparam, lparam),
        }
    }
}
