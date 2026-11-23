// The tester's Win32 window. It isn't built on Library/user-interface, so it
// does what that library's window does for every other program, as far as
// the tester needs it:
//
// - It's borderless fullscreen on its display, the one supported setup, of
//   the class that RunInBackground finds programs' windows by.
// - Its input is its keyboard messages, which only the window with focus
//   gets: WM_KEYDOWN for keys, and WM_CHAR for what they type, which
//   TranslateMessage makes with the keyboard's layout. The window procedure
//   keeps them, in order, for the main loop to hand to the tester.
// - Windows' own window shortcuts do nothing in it: it swallows the system
//   keys' messages, which Windows' default handling would turn into the
//   menu's keyboard mode for Alt alone and F10, the window's menu for
//   Alt+Space, and a close for Alt+F4. It reads F10, and Ctrl+Shift+Alt+F12,
//   from them first.
// - Without activating, it opens beneath the other windows, and a click on
//   it never activates it, so it never takes focus.
// - It says when its setup may have changed, and what's wrong with it.
// - Until its first frame is presented, it shows the theme's background,
//   which it fills with GDI whenever Windows asks it to paint.

use std::cell::Cell;
use std::ptr::NonNull;

use windows_api_raw::*;
use windows_core::{Error, PCWSTR, Result, w};

use crate::layout::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::tester::{Input, Key, Modifiers};

// RunInBackground finds programs' windows by this class, which
// Library/user-interface's windows have.
const CLASS: PCWSTR = w!("MainWindow");

// What the window procedure shares with the window, by GWLP_USERDATA. Both
// only ever hold `&Shared`, as the window procedure runs inside the window's
// own calls to Windows, like DispatchMessageW, and change it through its
// cells.
struct Shared {
    inputs: Cell<Vec<Input>>,
    close_requested: Cell<bool>,
    setup_changed: Cell<bool>,
    // What the window shows until its first frame is presented.
    background: Cell<COLORREF>,
}

pub struct Window {
    hwnd: HWND,
    // From Box::into_raw, freed by Drop after the window is destroyed.
    shared: NonNull<Shared>,
}

impl Window {
    // The window, borderless over the whole monitor, hidden until shown.
    pub fn create(title: &str, monitor: HMONITOR, background: [u8; 3]) -> Result<Window> {
        unsafe {
            let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
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
            let shared = Box::into_non_null(Box::new(Shared {
                inputs: Cell::new(Vec::new()),
                close_requested: Cell::new(false),
                setup_changed: Cell::new(true),
                background: Cell::new(colorref(background)),
            }));
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, shared.as_ptr() as isize);
            Ok(Window { hwnd, shared })
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
    // without taking focus from the window that has it, and a click on it
    // never activates it, and it asks Windows for efficiency mode, so that
    // a game that someone's playing keeps the chip's power.
    pub fn show(&mut self, activate: bool) {
        unsafe {
            set_power_throttling(!activate);
            if activate {
                let _ = ShowWindow(self.hwnd, SW_SHOW);
            } else {
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
                // WM_CHAR from WM_KEYDOWN, with the keyboard's layout.
                let _ = TranslateMessage(&message);
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

    // The input since the last call, oldest first.
    pub fn take_inputs(&mut self) -> Vec<Input> {
        self.shared().inputs.take()
    }

    // What the window shows until its first frame is presented.
    pub fn set_background(&mut self, background: [u8; 3]) {
        self.shared().background.set(colorref(background));
    }

    pub fn minimized(&self) -> bool {
        unsafe { IsIconic(self.hwnd).as_bool() }
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

    // Whether the setup may have changed since the last call.
    pub fn take_setup_changed(&mut self) -> bool {
        self.shared().setup_changed.replace(false)
    }

    // What's wrong with the setup, if anything, in ASCII, which the font
    // draws. The tester supports only borderless fullscreen on a 2560 × 1440
    // display at 100% scale: the client area must be the whole display.
    pub fn setup_problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        unsafe {
            let monitor = monitor_info(self.monitor()).rcMonitor;
            let (width, height) = (monitor.right - monitor.left, monitor.bottom - monitor.top);
            if (width, height) != (WINDOW_WIDTH, WINDOW_HEIGHT) {
                problems.push(format!("The display is {width} x {height} pixels. It must be {WINDOW_WIDTH} x {WINDOW_HEIGHT}."));
            }
            let dpi = GetDpiForWindow(self.hwnd);
            if dpi != USER_DEFAULT_SCREEN_DPI as u32 {
                problems.push(format!("The display's scale is {} percent. It must be 100 percent.", dpi * 100 / USER_DEFAULT_SCREEN_DPI as u32));
            }
            let mut client = RECT::default();
            let _ = GetClientRect(self.hwnd, &mut client);
            let mut origin = POINT::default();
            let _ = ClientToScreen(self.hwnd, &mut origin);
            let area = RECT {
                left: origin.x,
                top: origin.y,
                right: origin.x + client.right,
                bottom: origin.y + client.bottom,
            };
            if area != monitor {
                problems.push(format!(
                    "The window's client area is {} x {} at {} {}. It must be the whole display.",
                    area.right - area.left,
                    area.bottom - area.top,
                    area.left,
                    area.top
                ));
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

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe fn monitor_info(monitor: HMONITOR) -> MONITORINFO {
    unsafe {
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut info);
        info
    }
}

// Whether Windows may throttle the process's execution speed: a window that
// someone uses asks it not to, and one that opens without focus, like a
// coding agent's test run while someone plays a game, asks for efficiency
// mode.
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
            hCursor: LoadCursorW(None, IDC_ARROW),
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
fn colorref([r, g, b]: [u8; 3]) -> COLORREF {
    COLORREF(u32::from(r) | u32::from(g) << 8 | u32::from(b) << 16)
}

unsafe fn fill(hdc: HDC, rect: &RECT, color: COLORREF) {
    unsafe {
        SetDCBrushColor(hdc, color);
        FillRect(hdc, rect, HBRUSH(GetStockObject(DC_BRUSH).0));
    }
}

// The key of a virtual-key code, if the tester reads it.
pub fn key(virtual_key: i32) -> Option<Key> {
    Some(match virtual_key {
        VK_LEFT => Key::Left,
        VK_RIGHT => Key::Right,
        VK_UP => Key::Up,
        VK_DOWN => Key::Down,
        VK_HOME => Key::Home,
        VK_END => Key::End,
        VK_DELETE => Key::Delete,
        VK_F9 => Key::F9,
        VK_F10 => Key::F10,
        VK_F12 => Key::F12,
        // The top row's, whose codes are their digits' ASCII.
        0x30..=0x39 => Key::Digit((virtual_key - 0x30) as u8),
        VK_NUMPAD0..=VK_NUMPAD9 => Key::Digit((virtual_key - VK_NUMPAD0) as u8),
        _ => return None,
    })
}

// The modifiers held as the message that's being handled was sent.
unsafe fn modifiers() -> Modifiers {
    let held = |key| unsafe { GetKeyState(key) } < 0;
    Modifiers {
        control: held(VK_CONTROL),
        shift: held(VK_SHIFT),
        alt: held(VK_MENU),
    }
}

unsafe extern "system" fn window_procedure(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let shared = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Shared;
        if shared.is_null() {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        // Shared, never unique: see Shared.
        let shared = &*shared;
        let push = |input| {
            let mut inputs = shared.inputs.take();
            inputs.push(input);
            shared.inputs.set(inputs);
        };
        match message as i32 {
            // The system keys' messages too, which Alt and F10 send, and
            // which are swallowed: see the top.
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if let Some(key) = key(wparam.0 as i32) {
                    push(Input::Key(key, modifiers()));
                }
                LRESULT(0)
            }
            WM_CHAR => {
                push(Input::Character(wparam.0 as u16));
                LRESULT(0)
            }
            WM_SYSKEYUP | WM_SYSCHAR => LRESULT(0),
            WM_SIZE => {
                shared.setup_changed.set(true);
                LRESULT(0)
            }
            WM_MOVE | WM_DISPLAYCHANGE | WM_SETTINGCHANGE => {
                shared.setup_changed.set(true);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
            WM_DPICHANGED => {
                // It keeps its size: the check says what's wrong instead.
                shared.setup_changed.set(true);
                LRESULT(0)
            }
            // The background, until the first frame is presented. PrintWindow
            // asks for this one too, with WM_PRINT.
            WM_ERASEBKGND => {
                let mut client = RECT::default();
                let _ = GetClientRect(hwnd, &mut client);
                fill(HDC(wparam.0 as *mut core::ffi::c_void), &client, shared.background.get());
                LRESULT(1)
            }
            WM_PAINT => {
                let mut paint = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut paint);
                fill(hdc, &paint.rcPaint, shared.background.get());
                let _ = EndPaint(hwnd, &paint);
                LRESULT(0)
            }
            // Like the taskbar's Close, which quits.
            WM_CLOSE => {
                shared.close_requested.set(true);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}
