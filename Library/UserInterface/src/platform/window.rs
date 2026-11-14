// A program's Win32 window. Its input comes from GameInput
// (platform/input_devices.rs and input/reader.rs), not from its messages,
// which it handles only to be a window: its focus, Windows' cursor over it,
// its size and its close button.
//
// - It's maximized or borderless fullscreen, the two supported setups, and
//   switches between them without taking focus.
// - It says when it gets and loses focus, on the clock that input is timed
//   on, as input without focus is ignored. The time is when it handles the
//   message, which Windows sends it once the focus has moved, so it can be
//   a little after.
// - While it has focus and is the foreground window, it keeps Windows' cursor
//   inside its client area, hidden, as games do: the interface keeps a
//   pointer of its own, which GameInput's mice move, and draws it exactly
//   while the cursor is kept (see input/reader.rs). A window can get focus
//   without being the foreground window, whose cursor it mustn't keep. It
//   starts to keep it only once no mouse button is held, by Windows' account
//   of the mouse, which, unlike GameInput's, has the click that gave the
//   window focus: Windows tracks a click on the title bar's minimize or close
//   button until it's let go of there, which a cursor kept in the client area
//   never is. The cursor's confinement is shared by every program, so the
//   window releases it only if it still holds it, as the program that gets
//   focus may have taken it first. As the window loses focus, like with
//   Alt+Tab, it lets the cursor go where the interface's pointer was, where
//   it shows, as SDL does, unless that program has taken it.
// - It says when its setup may have changed: when it moves or changes size,
//   or the displays, their scale or the taskbar change.
// - It asks for a dark title bar in the dark theme, as Windows 11's apps
//   have, with DWMWA_USE_IMMERSIVE_DARK_MODE, which Microsoft's documentation
//   describes as drawing the frame dark while Windows' own dark mode is on.
// - Until its first frame is presented, it shows the theme's background,
//   which it fills with GDI whenever Windows asks it to paint, and as soon as
//   it shows. A window that paints nothing shows white, which would flash
//   before a dark theme's first frame. GDI shows only until the first
//   present of the window's flip-model swap chain, as DXGI_SWAP_EFFECT's
//   documentation says, so after that the fills change nothing.

use std::cell::Cell;
use std::ptr::NonNull;

use windows_api::*;
use windows_core::{BOOL, Error, PCWSTR, Result, w};

use crate::application::settings::WindowMode;
use crate::drawing::geometry::Color;
use crate::platform::clock;

// RunInBackground finds programs' windows by this class.
const CLASS: PCWSTR = w!("MainWindow");

// The one supported display and its taskbar.
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

// What the window procedure shares with the window, by GWLP_USERDATA.
//
// The window procedure runs inside the Windows calls that send the window
// messages, while one of the window's methods is running, like
// DispatchMessageW in `pump`, SetWindowPos in `apply_mode` and `show`, and
// ShowWindow. It runs inside its own calls too, like DefWindowProcW. So both
// only ever hold
// `&Shared`, which may overlap, and change it through its cells. A
// `&mut Shared` on either side would alias the other's reference, which is
// undefined behavior even when it works.
struct Shared {
    focus_changes: Cell<Vec<FocusChange>>,
    focused: Cell<bool>,
    minimized: Cell<bool>,
    close_requested: Cell<bool>,
    setup_changed: Cell<bool>,
    // Where the interface's pointer last was, in the client area's pixels.
    pointer: Cell<Option<POINT>>,
    // Where the window keeps Windows' cursor, hidden, if it does.
    kept: Cell<Option<Kept>>,
    // Windows' arrow, which its cursor is over the window while it isn't kept.
    arrow: HCURSOR,
    // What the window shows until its first frame is presented.
    background: COLORREF,
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
    dark_title_bar: Option<bool>,
}

impl Window {
    // A program's window on the monitor, hidden until shown, which shows
    // `background`, the theme's, until its first frame is presented.
    pub fn create(title: &str, monitor: HMONITOR, mode: WindowMode, background: Color) -> Result<Window> {
        unsafe {
            prepare_process();
            register_class()?;
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
            let mut window = Window::wrap(hwnd, background);
            window.apply_mode(mode, false);
            Ok(window)
        }
    }

    unsafe fn wrap(hwnd: HWND, background: Color) -> Window {
        unsafe {
            let shared = Box::new(Shared {
                focus_changes: Cell::new(Vec::new()),
                focused: Cell::new(false),
                minimized: Cell::new(false),
                close_requested: Cell::new(false),
                setup_changed: Cell::new(true),
                pointer: Cell::new(None),
                kept: Cell::new(None),
                arrow: LoadCursorW(None, IDC_ARROW),
                background: colorref(background),
            });
            let shared = NonNull::new_unchecked(Box::into_raw(shared));
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, shared.as_ptr() as isize);
            Window {
                hwnd,
                shared,
                mode: WindowMode::WindowedMaximized,
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

    // Shows the window, and paints the background at once, rather than when
    // the loop next handles messages. Without `activate`, it opens beneath
    // the others, without taking focus from the window that has it.
    pub fn show(&mut self, activate: bool) {
        unsafe {
            if activate {
                set_power_throttling(false);
                let _ = ShowWindow(self.hwnd, SW_SHOW);
            } else {
                // A window that mustn't take focus, like a test run's while
                // someone works or plays a game, goes beneath the other
                // windows, where it never covers theirs or takes their
                // pointer, and a click on it never activates it. Screenshots
                // of it work all the same.
                set_power_throttling(true);
                let style = GetWindowLongPtrW(self.hwnd, GWL_EXSTYLE);
                SetWindowLongPtrW(self.hwnd, GWL_EXSTYLE, style | WS_EX_NOACTIVATE as isize);
                let _ = ShowWindow(self.hwnd, SW_SHOWNA);
                let flags = (SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER) as u32;
                let _ = SetWindowPos(self.hwnd, Some(HWND_BOTTOM), 0, 0, 0, 0, flags);
            }
            let _ = UpdateWindow(self.hwnd);
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
                WindowMode::WindowedMaximized => {
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

    // The focus changes since the last call, oldest first.
    pub fn take_focus_changes(&mut self) -> Vec<FocusChange> {
        self.shared().focus_changes.take()
    }

    // Where the interface's pointer is, in the client area, where Windows'
    // cursor shows as the window loses focus.
    pub fn set_pointer(&mut self, at: crate::drawing::geometry::Point) {
        self.shared().pointer.set(Some(POINT {
            x: at.x.floor() as i32,
            y: at.y.floor() as i32,
        }));
    }

    // Keeps Windows' cursor inside the client area where keeps_cursor_in
    // says, every frame, as Windows lets go of a cursor's confinement when
    // other windows come and go, the client area moves when the window
    // switches between maximized and fullscreen, and a click that gave the
    // window focus is let go of.
    pub fn keep_cursor(&mut self) {
        unsafe { keep_cursor(self.hwnd, self.shared()) };
    }

    // Whether the window keeps Windows' cursor, hidden, so that its frames
    // draw the pointer instead.
    pub fn keeps_cursor(&self) -> bool {
        self.shared().kept.get().is_some()
    }

    // Where Windows' cursor is kept and centered: the client area, in the
    // screen's pixels, or none while the window is minimized, whose client
    // area is off the screen then, where a cursor kept in it would be lost.
    pub fn cursor_area(&self) -> Option<RECT> {
        unsafe { cursor_area(self.hwnd) }
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
                WindowMode::WindowedMaximized => {
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
                            "The window's client area is {client_width} × {client_height}. Windowed maximized, it must be {DISPLAY_WIDTH} × {MAXIMIZED_CLIENT_HEIGHT}."
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

unsafe fn register_class() -> Result<()> {
    unsafe {
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(window_procedure),
            hInstance: GetModuleHandleW(None),
            lpszClassName: CLASS,
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

// GDI's color: red in the low byte.
fn colorref(color: Color) -> COLORREF {
    let [r, g, b, _] = color.to_bytes();
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}

// Fills the rectangle with the color, with the DC's own brush, which needs no
// brush of its own to make and delete.
unsafe fn fill(hdc: HDC, rect: &RECT, color: COLORREF) {
    unsafe {
        SetDCBrushColor(hdc, color);
        FillRect(hdc, rect, HBRUSH(GetStockObject(DC_BRUSH).0));
    }
}

// See Window::cursor_area. Windows can give a minimized window focus, as it
// does as one is restored, before it's restored.
unsafe fn cursor_area(hwnd: HWND) -> Option<RECT> {
    unsafe {
        if IsIconic(hwnd).as_bool() {
            return None;
        }
        let mut client = RECT::default();
        let _ = GetClientRect(hwnd, &mut client);
        let mut origin = POINT::default();
        let _ = ClientToScreen(hwnd, &mut origin);
        Some(RECT {
            left: origin.x,
            top: origin.y,
            right: origin.x + client.right,
            bottom: origin.y + client.bottom,
        })
    }
}

// Where the window keeps Windows' cursor, if anywhere: in its client area,
// the area, which a minimized window has none of, while it has focus and is
// the foreground window, as a window can get focus without being it, when
// Windows won't let it take the foreground from the program that has it. It
// starts to keep it only once no mouse button is held, like the click that
// gave it focus on its title bar's minimize or close button, which Windows
// tracks until it's let go of there, where a cursor kept in the client area
// would never be let go of. A button held while it keeps it, like for a drag,
// doesn't let it go.
pub fn keeps_cursor_in(focused: bool, foreground: bool, keeps: bool, area: Option<RECT>, button_held: bool) -> Option<RECT> {
    if focused && foreground && (keeps || !button_held) { area } else { None }
}

// Where the window keeps the cursor, and the cursor's confinement as it was
// before, which was none, as Windows' documentation has a program save it.
#[derive(Clone, Copy, Debug)]
struct Kept {
    area: RECT,
    free: RECT,
}

// Who holds the cursor's confinement as the window lets the cursor go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipHolder {
    // The window, which releases it.
    Window,
    // Nobody: Windows released it, as it can as focus moves.
    Nobody,
    // Another program, which took it as it got focus, before the window
    // heard that it lost focus, as the two hear of it in no fixed order.
    Other,
}

// Who holds the confinement `clip`, which GetClipCursor gives, when the
// window kept the cursor in `area`, and the confinement was `free` before.
pub fn clip_holder(clip: RECT, area: RECT, free: RECT) -> ClipHolder {
    if clip == area {
        ClipHolder::Window
    } else if clip == free {
        ClipHolder::Nobody
    } else {
        ClipHolder::Other
    }
}

// The cursor's confinement, or the whole of the screens, without one.
unsafe fn clip() -> RECT {
    let mut clip = RECT::default();
    let _ = unsafe { GetClipCursor(&mut clip) };
    clip
}

// Whether a mouse button is held, by Windows' account of the mouse, which has
// the click that gave the window focus, as GameInput's doesn't.
fn mouse_button_held() -> bool {
    [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, VK_XBUTTON1, VK_XBUTTON2]
        .into_iter()
        .any(|button| unsafe { GetAsyncKeyState(button) } < 0)
}

// Keeps Windows' cursor inside the client area, hidden, where keeps_cursor_in
// says, and lets it go where it doesn't.
unsafe fn keep_cursor(hwnd: HWND, shared: &Shared) {
    unsafe {
        let (focused, kept) = (shared.focused.get(), shared.kept.get());
        // Without focus, there's nothing to keep, or let go of.
        if !focused && kept.is_none() {
            return;
        }
        let foreground = GetForegroundWindow() == hwnd;
        let held = focused && foreground && kept.is_none() && mouse_button_held();
        match keeps_cursor_in(focused, foreground, kept.is_some(), cursor_area(hwnd), held) {
            Some(area) => {
                let clip = clip();
                if clip != area {
                    let _ = ClipCursor(Some(&area));
                }
                let free = kept.map_or(clip, |kept| kept.free);
                shared.kept.set(Some(Kept { area, free }));
                // Hidden at once, as Windows asks for its shape only as it
                // moves.
                if kept.is_none() {
                    SetCursor(None);
                }
            }
            None => {
                let_go(shared);
            }
        }
    }
}

// Lets Windows' cursor go, if the window keeps it, and releases its
// confinement if the window still holds it. Says who held it.
unsafe fn let_go(shared: &Shared) -> Option<ClipHolder> {
    unsafe {
        let kept = shared.kept.take()?;
        let holder = clip_holder(clip(), kept.area, kept.free);
        if holder == ClipHolder::Window {
            let _ = ClipCursor(None);
        }
        Some(holder)
    }
}

// Puts Windows' cursor where the interface's pointer was, or in the middle of
// the client area if it was nowhere yet, unless the window is minimized.
unsafe fn place_cursor(hwnd: HWND, shared: &Shared) {
    unsafe {
        if let Some(area) = cursor_area(hwnd) {
            let middle = POINT {
                x: (area.right - area.left) / 2,
                y: (area.bottom - area.top) / 2,
            };
            let at = shared.pointer.get().unwrap_or(middle);
            let x = (area.left + at.x).clamp(area.left, (area.right - 1).max(area.left));
            let y = (area.top + at.y).clamp(area.top, (area.bottom - 1).max(area.top));
            let _ = SetCursorPos(x, y);
        }
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
                keep_cursor(hwnd, shared);
                LRESULT(0)
            }
            // Windows' cursor, if kept, is let go of where the interface's
            // pointer was, where it shows once the window is left, unless
            // the program that got focus has taken it already.
            WM_KILLFOCUS => {
                shared.focus_changed(false);
                if let_go(shared).is_some_and(|holder| holder != ClipHolder::Other) {
                    place_cursor(hwnd, shared);
                }
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
            // Hidden over the client area while the window keeps it, where
            // the interface draws its own pointer, and the arrow otherwise.
            WM_SETCURSOR => {
                if low_word(lparam.0) == HTCLIENT {
                    let cursor = if shared.kept.get().is_some() { None } else { Some(shared.arrow) };
                    SetCursor(cursor);
                    LRESULT(1)
                } else {
                    DefWindowProcW(hwnd, message as u32, wparam, lparam)
                }
            }
            WM_SIZE => {
                shared.minimized.set(wparam.0 as i32 == SIZE_MINIMIZED);
                // A minimized window keeps no cursor, even with focus.
                keep_cursor(hwnd, shared);
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
            // The background, until the first frame is presented: see the
            // top. WM_PAINT fills what it paints itself, and Windows asks for
            // this one too, like in BeginPaint and in WM_PRINT, as
            // PrintWindow sends it.
            WM_ERASEBKGND => {
                let mut client = RECT::default();
                let _ = GetClientRect(hwnd, &mut client);
                fill(HDC(wparam.0 as *mut core::ffi::c_void), &client, shared.background);
                LRESULT(1)
            }
            WM_PAINT => {
                let mut paint = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut paint);
                fill(hdc, &paint.rcPaint, shared.background);
                let _ = EndPaint(hwnd, &paint);
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
