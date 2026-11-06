// A program's Win32 window, and the events that its messages bring.
//
// - It's maximized or borderless fullscreen, the two supported setups, and
//   switches between them without taking focus.
// - Without focus it sends no input. The click that gives it focus does only
//   that, and keys held as focus comes count once they're let go of.
// - It captures the mouse while a button is held, so drags go on past its
//   edges.
// - It says when its setup may have changed: when it moves or changes size,
//   or the displays, their scale or the taskbar change.
// - Its title bar is dark in the dark theme, as Windows 11's apps' are.

use windows_api::*;
use windows_core::{BOOL, Error, PCWSTR, Result, w};

use crate::application::settings::WindowMode;
use crate::core::ui::Cursor;
use crate::drawing::geometry::Point;
use crate::input::devices::{Key, MouseButton};
use crate::input::events::Event;

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

// What the window procedure shares with the window, by GWLP_USERDATA.
struct Shared {
    events: Vec<Event>,
    focused: bool,
    minimized: bool,
    close_requested: bool,
    setup_changed: bool,
    tracking_leave: bool,
    buttons_down: u32,
    high_surrogate: Option<u16>,
    cursor: HCURSOR,
    // Overlays close on any click, or Esc.
    overlay: bool,
}

pub struct Window {
    hwnd: HWND,
    shared: Box<Shared>,
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
            let mut window = Window::wrap(hwnd, false);
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
            Ok(Window::wrap(hwnd, true))
        }
    }

    unsafe fn wrap(hwnd: HWND, overlay: bool) -> Window {
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
            let mut shared = Box::new(Shared {
                events: Vec::new(),
                focused: false,
                minimized: false,
                close_requested: false,
                setup_changed: true,
                tracking_leave: false,
                buttons_down: 0,
                high_surrogate: None,
                cursor: cursors[0],
                overlay,
            });
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, &mut *shared as *mut Shared as isize);
            Window {
                hwnd,
                shared,
                mode: WindowMode::Maximized,
                cursors,
                dark_title_bar: None,
            }
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    // Shows the window. Without `activate`, it opens over the others without
    // taking focus from the window that has it.
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
            let (style, area) = match mode {
                WindowMode::BorderlessFullscreen => (WS_POPUP | visible, info.rcMonitor),
                WindowMode::Maximized => {
                    // Maximized as Windows would, but without activating: the
                    // work area, with the frame's borders past its edges, which
                    // leaves the title bar at its top.
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
        self.shared.setup_changed = true;
    }

    // Dark or light, as the theme is, like the title bars of Windows 11's
    // apps.
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
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        !self.shared.close_requested
    }

    // Waits for a message, like while the window is minimized.
    pub fn wait(&self) {
        unsafe {
            let _ = WaitMessage();
        }
    }

    // The input since the last call.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.shared.events)
    }

    pub fn minimized(&self) -> bool {
        self.shared.minimized
    }

    // Whether it was asked to close, like by its close button or Alt+F4.
    pub fn close_requested(&self) -> bool {
        self.shared.close_requested
    }

    // Keeps the window open after all, when the program says it may not
    // close yet.
    pub fn cancel_close(&mut self) {
        self.shared.close_requested = false;
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
        if self.shared.cursor != cursor {
            self.shared.cursor = cursor;
            // The pointer may be still, so it's set now, not at its next move.
            if self.shared.focused && self.shared.tracking_leave {
                unsafe {
                    SetCursor(Some(cursor));
                }
            }
        }
    }

    // Whether the setup may have changed since the last call.
    pub fn take_setup_changed(&mut self) -> bool {
        std::mem::replace(&mut self.shared.setup_changed, false)
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
                WindowMode::Maximized => {
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
                            "The window's client area is {client_width} × {client_height}. Maximized, it must be {DISPLAY_WIDTH} × {MAXIMIZED_CLIENT_HEIGHT}."
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
// window, and no power throttling, which would slow the main loop on
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
            &state as *const PROCESS_POWER_THROTTLING_STATE as *const core::ffi::c_void,
            size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
    });
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

fn high_word(value: isize) -> i32 {
    ((value >> 16) & 0xFFFF) as u16 as i16 as i32
}

fn point_of(lparam: LPARAM) -> Point {
    Point::new(low_word(lparam.0) as f32, high_word(lparam.0) as f32)
}

unsafe extern "system" fn window_procedure(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let shared = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Shared;
        if shared.is_null() {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        let shared = &mut *shared;
        let message = message as i32;
        let button_of = |message: i32| match message {
            WM_LBUTTONDOWN | WM_LBUTTONUP => MouseButton::Left,
            WM_RBUTTONDOWN | WM_RBUTTONUP => MouseButton::Right,
            _ => MouseButton::Middle,
        };
        match message {
            WM_SETFOCUS => {
                shared.focused = true;
                // Keys held now, like the Alt of an Alt+Tab, count once
                // they're let go of.
                let mut keys = [0u8; 256];
                let _ = GetKeyboardState(keys.as_mut_ptr());
                let held = (1..=254u8).filter(|code| keys[*code as usize] & 0x80 != 0).map(Key).collect();
                shared.events.push(Event::FocusGained(held));
                LRESULT(0)
            }
            WM_KILLFOCUS => {
                shared.focused = false;
                shared.buttons_down = 0;
                let _ = ReleaseCapture();
                shared.events.push(Event::FocusLost);
                LRESULT(0)
            }
            WM_MOUSEACTIVATE => {
                // The click that gives the window focus does only that.
                if low_word(lparam.0) == HTCLIENT {
                    LRESULT(MA_ACTIVATEANDEAT as isize)
                } else {
                    DefWindowProcW(hwnd, message as u32, wparam, lparam)
                }
            }
            WM_MOUSEMOVE => {
                if shared.focused {
                    if !shared.tracking_leave {
                        let mut track = TRACKMOUSEEVENT {
                            cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                            dwFlags: TME_LEAVE as u32,
                            hwndTrack: hwnd,
                            dwHoverTime: 0,
                        };
                        let _ = TrackMouseEvent(&mut track);
                        shared.tracking_leave = true;
                    }
                    shared.events.push(Event::PointerMoved(point_of(lparam)));
                }
                LRESULT(0)
            }
            WM_MOUSELEAVE => {
                shared.tracking_leave = false;
                shared.events.push(Event::PointerLeft);
                LRESULT(0)
            }
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
                if shared.overlay {
                    shared.close_requested = true;
                }
                if shared.focused {
                    let button = button_of(message);
                    shared.events.push(Event::PointerMoved(point_of(lparam)));
                    shared.events.push(Event::ButtonPressed(button));
                    if shared.buttons_down == 0 {
                        SetCapture(hwnd);
                    }
                    shared.buttons_down |= 1 << button.index();
                }
                LRESULT(0)
            }
            WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => {
                let button = button_of(message);
                if shared.buttons_down & (1 << button.index()) != 0 {
                    shared.events.push(Event::PointerMoved(point_of(lparam)));
                    shared.events.push(Event::ButtonReleased(button));
                    shared.buttons_down &= !(1 << button.index());
                    if shared.buttons_down == 0 {
                        let _ = ReleaseCapture();
                    }
                }
                LRESULT(0)
            }
            WM_CAPTURECHANGED => {
                // Another window took the mouse, which ends every press.
                for button in MouseButton::ALL {
                    if shared.buttons_down & (1 << button.index()) != 0 {
                        shared.events.push(Event::ButtonReleased(button));
                    }
                }
                shared.buttons_down = 0;
                LRESULT(0)
            }
            WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                if shared.focused {
                    let notches = high_word(wparam.0 as isize) as f32 / WHEEL_DELTA as f32;
                    let notches = if message == WM_MOUSEWHEEL { Point::new(0.0, notches) } else { Point::new(notches, 0.0) };
                    shared.events.push(Event::Wheel(notches));
                }
                LRESULT(0)
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                let key = Key(wparam.0 as u8);
                if shared.overlay && key == Key::ESCAPE {
                    shared.close_requested = true;
                }
                if shared.focused {
                    let repeat = (lparam.0 >> 30) & 1 == 1;
                    shared.events.push(Event::KeyPressed { key, repeat });
                }
                // Alt+F4 still closes the window.
                if message == WM_SYSKEYDOWN && key == Key::F4 {
                    return DefWindowProcW(hwnd, message as u32, wparam, lparam);
                }
                LRESULT(0)
            }
            WM_KEYUP | WM_SYSKEYUP => {
                shared.events.push(Event::KeyReleased(Key(wparam.0 as u8)));
                LRESULT(0)
            }
            WM_CHAR => {
                if shared.focused {
                    let unit = wparam.0 as u16;
                    if (0xD800..0xDC00).contains(&unit) {
                        shared.high_surrogate = Some(unit);
                    } else {
                        let units = match shared.high_surrogate.take() {
                            Some(high) => vec![high, unit],
                            None => vec![unit],
                        };
                        for character in char::decode_utf16(units).flatten() {
                            shared.events.push(Event::Text(character));
                        }
                    }
                }
                LRESULT(0)
            }
            WM_SYSCHAR => LRESULT(0),
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
                    SetCursor(Some(shared.cursor));
                    LRESULT(1)
                } else {
                    DefWindowProcW(hwnd, message as u32, wparam, lparam)
                }
            }
            WM_SIZE => {
                shared.minimized = wparam.0 as i32 == SIZE_MINIMIZED;
                shared.setup_changed = true;
                LRESULT(0)
            }
            WM_MOVE | WM_DISPLAYCHANGE | WM_SETTINGCHANGE => {
                shared.setup_changed = true;
                DefWindowProcW(hwnd, message as u32, wparam, lparam)
            }
            WM_DPICHANGED => {
                // It keeps its size: the check says what's wrong instead.
                shared.setup_changed = true;
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_PAINT => {
                let _ = ValidateRect(Some(hwnd), None);
                LRESULT(0)
            }
            WM_CLOSE => {
                shared.close_requested = true;
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message as u32, wparam, lparam),
        }
    }
}
