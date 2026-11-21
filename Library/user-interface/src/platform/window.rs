// A program's Win32 window. Its input comes from GameInput
// (platform/input_devices.rs and input/reader.rs), not from its messages,
// which it handles only to be a window: Windows' cursor over it, its size,
// WM_CLOSE, which quits, the system keys that it swallows, and its
// background.
//
// - It's borderless fullscreen on its display, the one supported setup.
// - Windows' own window shortcuts do nothing in it: it swallows the system
//   keys' messages, which Windows' default handling would turn into the
//   menu's keyboard mode for Alt alone and F10, the window's menu for
//   Alt+Space, and a close for Alt+F4, which is only two keys to KGames.
//   Ctrl+Shift+Alt+F12 quits instead (see input/reader.rs), from GameInput's
//   keys, not from these messages. The shortcuts that Windows handles
//   before any window sees them, like Alt+Tab and the Windows key, stay
//   Windows'.
// - It has focus while it's the foreground window and isn't minimized,
//   which the main loop checks on every pass, with check_focus, and not
//   from Windows' focus messages, as a window can get those without being
//   the foreground window. It says when that changes, on the clock that
//   input is timed on, as input without focus is ignored, at the time the
//   check finds it, so it can be a little after.
// - While it has focus, if its frames read GameInput's mice, it keeps
//   Windows' cursor inside its client area, hidden in GameInput's cursor
//   mode, and lets it go as focus goes and as the window closes, as
//   platform/cursor.rs decides on the facts that the window gathers from
//   Windows as it checks its focus, like whether a click on it gave it
//   focus, which it hears of in WM_MOUSEACTIVATE.
// - In Windows' cursor mode, which Ctrl+Shift+Alt+F11 switches to, it keeps
//   the cursor shown, in the shape of the frame's pointer, and finds where
//   it went as it checks its focus, on the clock that input is timed on, for
//   the interface's pointer to follow it there (see input/reader.rs), so it
//   can be found a little after it moved, as focus changes are. The loop
//   checks on every pass, which a move of the cursor over the window wakes,
//   as Windows' messages do.
// - It says when its setup may have changed: when it moves or changes size,
//   or the displays, their scale or Windows' settings change.
// - Until its first frame is presented, it shows the theme's background,
//   which it fills with GDI whenever Windows asks it to paint, and as soon as
//   it shows. A window that paints nothing shows white, which would flash
//   before a dark theme's first frame. GDI shows only until the first
//   present of the window's flip-model swap chain, as DXGI_SWAP_EFFECT's
//   documentation says, so after that the fills change nothing.

use std::cell::Cell;
use std::ptr::NonNull;

use windows_api_raw::*;
use windows_core::{Error, PCWSTR, Result, w};

use crate::core::interaction::Cursor;
use crate::drawing::geometry::{Color, Point};
use crate::platform::clock;
use crate::platform::cursor::{CursorAction, CursorFacts, CursorKeeper, CursorMode};

// RunInBackground finds programs' windows by this class.
const CLASS: PCWSTR = w!("MainWindow");

// The one supported display.
pub const DISPLAY_WIDTH: i32 = 2560;
pub const DISPLAY_HEIGHT: i32 = 1440;

// The window got focus, or lost it, at a time on the counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusChange {
    pub time: i64,
    pub gained: bool,
}

// Windows' cursor went to a point in the client area, as the window found at
// a time on the counter, in Windows' cursor mode, while the window keeps it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorMove {
    pub time: i64,
    pub at: Point,
}

// What the window procedure shares with the window, by GWLP_USERDATA.
//
// The window procedure runs inside the Windows calls that send the window
// messages, while one of the window's methods is running, like
// DispatchMessageW in `pump`, SetWindowPos in `show`, and ShowWindow. It
// runs inside its own calls too, like DefWindowProcW. So both only ever hold
// `&Shared`, which may overlap, and change it through its cells. A
// `&mut Shared` on either side would alias the other's reference, which is
// undefined behavior even when it works.
struct Shared {
    focus_changes: Cell<Vec<FocusChange>>,
    focused: Cell<bool>,
    close_requested: Cell<bool>,
    setup_changed: Cell<bool>,
    // Whether a click on the window, without focus, activated it since the
    // last check of its focus: Windows sends WM_MOUSEACTIVATE as the button
    // goes down, inside the PeekMessageW that activates the window, so the
    // check after it finds the focus that the click gave.
    clicked: Cell<bool>,
    // Whether Windows' cursor is hidden over the client area.
    cursor_hidden: Cell<bool>,
    // The shape that Windows' cursor shows in over the client area while it
    // isn't hidden: the frame's pointer's while the window keeps it in
    // Windows' cursor mode, and the arrow otherwise.
    cursor_shape: Cell<HCURSOR>,
    // Windows' arrow.
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
    cursor: CursorKeeper,
    // Where the interface's pointer is placed, in the client area, as the
    // window last started to keep Windows' cursor since the last frame read
    // its input.
    pointer_placed: Option<POINT>,
    // In Windows' cursor mode, while the window keeps the cursor: where it
    // went since the last frame read its input, and where it was last found,
    // or put, in the client area.
    cursor_moves: Vec<CursorMove>,
    followed: Option<POINT>,
    // Windows' cursors in the shapes of the interface's pointer, by
    // Cursor::ALL, and the frame's pointer's.
    shapes: [HCURSOR; Cursor::ALL.len()],
    pointer_shape: HCURSOR,
}

impl Window {
    // A program's window, borderless over the whole monitor, hidden until
    // shown, which shows `background`, the theme's, until its first frame is
    // presented, and keeps Windows' cursor while it has focus if its frames
    // read GameInput's mice: `mice`.
    pub fn create(title: &str, monitor: HMONITOR, background: Color, mice: bool) -> Result<Window> {
        unsafe {
            prepare_process();
            register_class()?;
            let area = monitor_info(monitor).rcMonitor;
            let title = wide(title);
            let hwnd = CreateWindowExW(
                WS_EX_APPWINDOW as u32,
                CLASS,
                PCWSTR(title.as_ptr()),
                WS_POPUP,
                area.left,
                area.top,
                area.right - area.left,
                area.bottom - area.top,
                None,
                None,
                Some(GetModuleHandleW(None)),
                None,
            );
            if hwnd.0.is_null() {
                return Err(Error::from_thread());
            }
            Ok(Window::wrap(hwnd, background, mice))
        }
    }

    unsafe fn wrap(hwnd: HWND, background: Color, mice: bool) -> Window {
        unsafe {
            let shapes = Cursor::ALL.map(|cursor| LoadCursorW(None, windows_cursor(cursor)));
            let arrow = shapes[Cursor::Arrow as usize];
            let shared = Box::new(Shared {
                focus_changes: Cell::new(Vec::new()),
                focused: Cell::new(false),
                close_requested: Cell::new(false),
                setup_changed: Cell::new(true),
                clicked: Cell::new(false),
                cursor_hidden: Cell::new(false),
                cursor_shape: Cell::new(arrow),
                arrow,
                background: colorref(background),
            });
            let shared = Box::into_non_null(shared);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, shared.as_ptr() as isize);
            Window {
                hwnd,
                shared,
                cursor: CursorKeeper::new(mice),
                pointer_placed: None,
                cursor_moves: Vec::new(),
                followed: None,
                shapes,
                pointer_shape: arrow,
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

    // Handles the window's messages without waiting. False once it should
    // close.
    pub fn pump(&mut self) -> bool {
        unsafe {
            let mut message = MSG::default();
            while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE as u32).as_bool() {
                if message.message == WM_QUIT as u32 {
                    return false;
                }
                // No TranslateMessage: typed text comes from GameInput's keys
                // (see input/typing.rs), not from WM_CHAR.
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

    // Where the interface's pointer is placed, in the client area, as the
    // window last started to keep Windows' cursor since the last call, if it
    // did: where the cursor was, or the middle (see platform/cursor.rs).
    pub fn take_pointer_placed(&mut self) -> Option<Point> {
        self.pointer_placed.take().map(|at| Point::new(at.x as f32, at.y as f32))
    }

    // Where the interface's pointer is, in the client area, where Windows'
    // cursor shows as it's let go of.
    pub fn set_pointer(&mut self, at: Point) {
        self.cursor.set_pointer(POINT {
            x: at.x.floor() as i32,
            y: at.y.floor() as i32,
        });
    }

    // Whether the window has focus: whether it's the foreground window and
    // isn't minimized, which the main loop asks on every pass, so that
    // losing it, like with Alt+Tab, the Windows key or minimizing, is found
    // at once, without waiting for a frame. Says when that changes, at the
    // time it finds it. Windows' cursor goes with it, as platform/cursor.rs
    // decides, on every pass while the window keeps it, as something else
    // may have changed its confinement, and in Windows' cursor mode, where
    // the cursor went. A click on the window that didn't give it focus by
    // this check, like one on a window that mustn't take focus, counts for
    // nothing after it.
    pub fn check_focus(&mut self) {
        unsafe {
            let focused = GetForegroundWindow() == self.hwnd && !IsIconic(self.hwnd).as_bool();
            let shared = self.shared();
            let clicked = shared.clicked.replace(false);
            if focused != shared.focused.get() {
                shared.focus_changed(focused);
            }
            if focused || self.cursor.keeps() {
                let facts = CursorFacts {
                    focused,
                    button_held: focused && !self.cursor.keeps() && mouse_button_held(),
                    clicked,
                    ..cursor_facts(self.hwnd)
                };
                let actions = self.cursor.update(facts);
                self.carry_out(&actions);
            }
        }
        self.follow_cursor();
    }

    // In Windows' cursor mode, while the window keeps the cursor: where it
    // went, in the client area, if it moved since it was last found, or put,
    // timed now, for the pointer to follow (see input/reader.rs).
    fn follow_cursor(&mut self) {
        if !self.cursor.keeps() || self.cursor.mode() != CursorMode::Windows {
            self.followed = None;
            return;
        }
        let Some(area) = self.cursor_area() else {
            return;
        };
        let mut cursor = POINT::default();
        unsafe {
            let _ = GetCursorPos(&mut cursor);
        }
        let at = POINT {
            x: cursor.x - area.left,
            y: cursor.y - area.top,
        };
        if self.followed.replace(at) != Some(at) {
            self.cursor_moves.push(CursorMove {
                time: clock::now(),
                at: Point::new(at.x as f32, at.y as f32),
            });
        }
    }

    // Does what platform/cursor.rs decided with Windows' cursor, and the
    // pointer, and gives the cursor the shape that goes with what it
    // decided.
    fn carry_out(&mut self, actions: &[CursorAction]) {
        let mut shown = false;
        for action in actions {
            let shared = self.shared();
            unsafe {
                match *action {
                    CursorAction::Confine(area) => {
                        let _ = ClipCursor(Some(&area));
                    }
                    CursorAction::Release => {
                        let _ = ClipCursor(None);
                    }
                    // Hidden at once, as Windows asks for its shape only as
                    // it moves.
                    CursorAction::Hide => {
                        shared.cursor_hidden.set(true);
                        SetCursor(None);
                    }
                    CursorAction::Show => {
                        shared.cursor_hidden.set(false);
                        shown = true;
                    }
                    // Where Windows' cursor is from then on, which the
                    // pointer follows in Windows' cursor mode.
                    CursorAction::Place(at) => {
                        let _ = SetCursorPos(at.x, at.y);
                        if let Some(area) = cursor_area(self.hwnd) {
                            self.followed = Some(POINT {
                                x: at.x - area.left,
                                y: at.y - area.top,
                            });
                        }
                    }
                    // For the next frame's input (see input/reader.rs),
                    // where Windows' cursor is too.
                    CursorAction::PlacePointer(at) => {
                        self.pointer_placed = Some(at);
                        self.followed = Some(at);
                    }
                }
            }
        }
        self.update_shape(shown);
    }

    // The shape that Windows' cursor shows in over the client area: the
    // frame's pointer's while the window keeps it in Windows' cursor mode,
    // and the arrow otherwise. Windows asks for it with WM_SETCURSOR only as
    // the cursor moves over the window, so while the window keeps the
    // cursor, which is over its client area then, it's set at once as it
    // changes, or as the cursor is `shown` again, unless it's hidden.
    fn update_shape(&mut self, shown: bool) {
        let keeps = self.cursor.keeps();
        let shape = if keeps && self.cursor.mode() == CursorMode::Windows { self.pointer_shape } else { self.shared().arrow };
        let shared = self.shared();
        let changed = shared.cursor_shape.replace(shape) != shape;
        if (changed || shown) && keeps && !shared.cursor_hidden.get() {
            unsafe {
                SetCursor(Some(shape));
            }
        }
    }

    // The cursor's mode, which the next frame's input starts in.
    pub fn cursor_mode(&self) -> CursorMode {
        self.cursor.mode()
    }

    // Switches the cursor's mode, as a frame's input left it, once the
    // frame's input has placed the pointer, with set_pointer: while the
    // window keeps the cursor, it shows there, in Windows' cursor mode, or
    // hides, in GameInput's (see platform/cursor.rs).
    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        if mode == self.cursor.mode() {
            return;
        }
        let actions = self.cursor.set_mode(mode, unsafe { cursor_facts(self.hwnd) });
        self.carry_out(&actions);
    }

    // The shape of the frame's pointer, which Windows' cursor takes while the
    // window keeps it in Windows' cursor mode.
    pub fn set_pointer_shape(&mut self, cursor: Cursor) {
        self.pointer_shape = self.shapes[cursor as usize];
        self.update_shape(false);
    }

    // Where Windows' cursor went, in Windows' cursor mode, since the last
    // call, oldest first.
    pub fn take_cursor_moves(&mut self) -> Vec<CursorMove> {
        std::mem::take(&mut self.cursor_moves)
    }

    // Whether the window keeps Windows' cursor: hidden in GameInput's cursor
    // mode, so that its frames draw the pointer instead, and shown in
    // Windows'.
    pub fn keeps_cursor(&self) -> bool {
        self.cursor.keeps()
    }

    // Where Windows' cursor is kept, and placed as it's let go of: the client
    // area, in the screen's pixels, or none while the window is minimized,
    // whose client area is off the screen then, where a cursor kept in it
    // would be lost.
    pub fn cursor_area(&self) -> Option<RECT> {
        unsafe { cursor_area(self.hwnd) }
    }

    pub fn focused(&self) -> bool {
        self.shared().focused.get()
    }

    pub fn minimized(&self) -> bool {
        unsafe { IsIconic(self.hwnd).as_bool() }
    }

    // Whether it was asked to close with WM_CLOSE, like by the taskbar's
    // Close, which quits.
    pub fn close_requested(&self) -> bool {
        self.shared().close_requested.get()
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

    // What's wrong with the setup, if anything. Programs support only
    // borderless fullscreen on a 2560 × 1440 display at 100% scale: the
    // client area must be the whole display, where it is and its size, as a
    // window can be moved, or the displays arranged anew under it.
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
            let area = cursor_area(self.hwnd).unwrap_or_default();
            if area != monitor {
                problems.push(format!(
                    "The window's client area is {} × {} at ({}, {}). It must be the whole display, {width} × {height} at ({}, {}).",
                    area.right - area.left,
                    area.bottom - area.top,
                    area.left,
                    area.top,
                    monitor.left,
                    monitor.top
                ));
            }
        }
        problems
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            // Windows' cursor, let go of as the window closes, like with
            // Ctrl+Shift+Alt+F12 or the taskbar's Close, or a panic's
            // unwinding, which Windows doesn't do for the window, while it
            // can still be let go of where the pointer was.
            let actions = self.cursor.let_go(cursor_facts(self.hwnd));
            self.carry_out(&actions);
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

// What Windows says about the window's client area, the cursor and its
// confinement, and the screens, for platform/cursor.rs, without focus.
unsafe fn cursor_facts(hwnd: HWND) -> CursorFacts {
    unsafe {
        let mut clip = RECT::default();
        let _ = GetClipCursor(&mut clip);
        let mut cursor = POINT::default();
        let _ = GetCursorPos(&mut cursor);
        let (left, top) = (GetSystemMetrics(SM_XVIRTUALSCREEN), GetSystemMetrics(SM_YVIRTUALSCREEN));
        CursorFacts {
            focused: false,
            area: cursor_area(hwnd),
            button_held: false,
            clicked: false,
            cursor,
            clip,
            screens: RECT {
                left,
                top,
                right: left + GetSystemMetrics(SM_CXVIRTUALSCREEN),
                bottom: top + GetSystemMetrics(SM_CYVIRTUALSCREEN),
            },
        }
    }
}

// Windows' own cursor in the shape of the interface's pointer.
fn windows_cursor(cursor: Cursor) -> PCWSTR {
    match cursor {
        Cursor::Arrow => IDC_ARROW,
        Cursor::Text => IDC_IBEAM,
        Cursor::Hand => IDC_HAND,
        Cursor::Move => IDC_SIZEALL,
        Cursor::ResizeHorizontal => IDC_SIZEWE,
        Cursor::ResizeVertical => IDC_SIZENS,
        Cursor::ResizeDiagonal => IDC_SIZENWSE,
        Cursor::ResizeAntidiagonal => IDC_SIZENESW,
        Cursor::Crosshair => IDC_CROSS,
        Cursor::NotAllowed => IDC_NO,
    }
}

// Whether a mouse button is held, by Windows' account of the mouse, which has
// a click that was going on as the window got focus, as GameInput's doesn't.
fn mouse_button_held() -> bool {
    [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, VK_XBUTTON1, VK_XBUTTON2]
        .into_iter()
        .any(|button| unsafe { GetAsyncKeyState(button) } < 0)
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
            // Windows' own window shortcuts do nothing: its default handling
            // of the system keys' messages would turn Alt alone and F10 into
            // the menu's keyboard mode, Alt+Space into the window's menu, and
            // Alt+F4 into a close. Alt+F4 is only two keys to KGames, and
            // Ctrl+Shift+Alt+F12 quits instead (see input/reader.rs).
            WM_SYSKEYDOWN | WM_SYSKEYUP | WM_SYSCHAR => LRESULT(0),
            // Hidden over the client area while the window keeps it in
            // GameInput's cursor mode, where the interface draws its own
            // pointer, in the pointer's shape while it keeps it in Windows',
            // and the arrow otherwise.
            WM_SETCURSOR => {
                if low_word(lparam.0) == HTCLIENT {
                    let cursor = if shared.cursor_hidden.get() { None } else { Some(shared.cursor_shape.get()) };
                    SetCursor(cursor);
                    LRESULT(1)
                } else {
                    DefWindowProcW(hwnd, message as u32, wparam, lparam)
                }
            }
            WM_SIZE => {
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
            // A click on the window without focus, which activates it, as
            // Windows' default handling says, unless the window mustn't take
            // focus: the cursor is kept at once, while it's held.
            WM_MOUSEACTIVATE => {
                shared.clicked.set(true);
                DefWindowProcW(hwnd, message as u32, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, message as u32, wparam, lparam),
        }
    }
}
