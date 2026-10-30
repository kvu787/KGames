// A program's window: one per process, of the class "MainWindow", which is
// how RunInBackground finds it. It opens on the display that the settings or
// the command line name, maximized or in borderless fullscreen, and
// toggle_fullscreen switches between the two, for the program's F11.
//
// The window procedure only records what happened, and the main loop reacts.
// This keeps program state out of the callback.
//
// Programs ignore all input while the window doesn't have focus: then
// take_events returns nothing, cursor returns None, and a click that gives
// the window focus does only that.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use displays::Display;
use windows_api::*;
use windows_core::{Error, PCWSTR, Result, w};

use crate::{Arguments, Settings, WindowMode, window_check};

static CLIENT_WIDTH: AtomicU32 = AtomicU32::new(0);
static CLIENT_HEIGHT: AtomicU32 = AtomicU32::new(0);
static RESIZED: AtomicBool = AtomicBool::new(false);
// The window may have moved, been resized, or changed DPI, or the displays or
// the work area changed, so it may no longer be in a supported setup.
static WINDOW_CHANGED: AtomicBool = AtomicBool::new(true);
static DISPLAYS_CHANGED: AtomicBool = AtomicBool::new(false);
// The cursor's position in the window, packed by pack_point.
static CURSOR: AtomicU64 = AtomicU64::new(NO_POINT);
// Whether Windows will send WM_MOUSELEAVE when the cursor leaves the window.
static TRACKING_MOUSE: AtomicBool = AtomicBool::new(false);
// Whether the left button went down in the window and hasn't come up.
static BUTTON_DOWN: AtomicBool = AtomicBool::new(false);
static EVENTS: Mutex<Vec<Event>> = Mutex::new(Vec::new());

const NO_POINT: u64 = u64::MAX;

// Mouse and keyboard input from the window's messages, for what GameInput
// doesn't say, like where the cursor is in the window. Points are in the
// client area, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    // The left mouse button went down. Until it comes up, the window gets
    // the mouse's messages even when the cursor is outside it.
    Press([f32; 2]),
    // The left mouse button came up, or the window lost the mouse while it
    // was down.
    Release([f32; 2]),
    // The mouse wheel turned, by notches, positive away from the user.
    Wheel([f32; 2], f32),
    // A key went down, by its virtual-key code, like VK_F11. It repeats
    // while the key is held.
    Key { key: u32, control: bool, shift: bool },
}

// A mouse message's position in the window, from the low and high words of
// its LPARAM, as one value for an atomic.
fn pack_point(lparam: LPARAM) -> u64 {
    (lparam.0 as u64) & 0xFFFF_FFFF
}

fn unpack_point(packed: u64) -> Option<[f32; 2]> {
    (packed != NO_POINT).then_some([
        (packed as u16 as i16) as f32,
        ((packed >> 16) as u16 as i16) as f32,
    ])
}

fn push(event: Event) {
    EVENTS.lock().unwrap().push(event);
}

extern "system" fn window_procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message as i32 {
        WM_SIZE => {
            CLIENT_WIDTH.store((lparam.0 & 0xFFFF) as u32, Ordering::Relaxed);
            CLIENT_HEIGHT.store(((lparam.0 >> 16) & 0xFFFF) as u32, Ordering::Relaxed);
            RESIZED.store(true, Ordering::Relaxed);
            WINDOW_CHANGED.store(true, Ordering::Relaxed);
            LRESULT(0)
        }
        // A change to the taskbar sends WM_SETTINGCHANGE, as it changes the
        // work area. The window keeps its size when its DPI changes, as
        // programs only support 96 DPI, and says what's wrong instead.
        WM_MOVE | WM_DPICHANGED | WM_SETTINGCHANGE => {
            WINDOW_CHANGED.store(true, Ordering::Relaxed);
            LRESULT(0)
        }
        WM_DISPLAYCHANGE => {
            DISPLAYS_CHANGED.store(true, Ordering::Relaxed);
            WINDOW_CHANGED.store(true, Ordering::Relaxed);
            LRESULT(0)
        }
        // A tap of Alt or F10 would otherwise enter the window menu's modal
        // loop, which stalls the frame loop until the next key press.
        WM_SYSCOMMAND if (wparam.0 & 0xFFF0) as i32 == SC_KEYMENU => LRESULT(0),
        // A click in the client area of a window without focus gives it
        // focus, and nothing else, as programs ignore input without focus.
        // A click on the title bar still drags it.
        WM_MOUSEACTIVATE if (lparam.0 & 0xFFFF) as i32 == HTCLIENT => {
            LRESULT(MA_ACTIVATEANDEAT as isize)
        }
        // Windows only reports that the cursor left the window if asked, and
        // the request ends with the WM_MOUSELEAVE it sends.
        WM_MOUSEMOVE => {
            CURSOR.store(pack_point(lparam), Ordering::Relaxed);
            if !TRACKING_MOUSE.swap(true, Ordering::Relaxed) {
                let mut track = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE as u32,
                    hwndTrack: window,
                    dwHoverTime: 0,
                };
                unsafe {
                    let _ = TrackMouseEvent(&mut track);
                }
            }
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            CURSOR.store(NO_POINT, Ordering::Relaxed);
            TRACKING_MOUSE.store(false, Ordering::Relaxed);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            CURSOR.store(pack_point(lparam), Ordering::Relaxed);
            BUTTON_DOWN.store(true, Ordering::Relaxed);
            // Keeps the mouse's messages coming while the button is down,
            // like for dragging a slider past the window's edge.
            unsafe { SetCapture(window) };
            push(Event::Press(unpack_point(pack_point(lparam)).unwrap()));
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            if BUTTON_DOWN.swap(false, Ordering::Relaxed) {
                push(Event::Release(unpack_point(pack_point(lparam)).unwrap()));
            }
            unsafe {
                let _ = ReleaseCapture();
            }
            LRESULT(0)
        }
        // Like when another window takes focus while the button is down.
        WM_CAPTURECHANGED => {
            if BUTTON_DOWN.swap(false, Ordering::Relaxed) {
                let cursor = unpack_point(CURSOR.load(Ordering::Relaxed));
                push(Event::Release(cursor.unwrap_or([-1.0, -1.0])));
            }
            LRESULT(0)
        }
        // Unlike the other mouse messages, the position is on the screen.
        WM_MOUSEWHEEL => {
            let mut point = POINT {
                x: (lparam.0 & 0xFFFF) as u16 as i16 as i32,
                y: ((lparam.0 >> 16) & 0xFFFF) as u16 as i16 as i32,
            };
            unsafe {
                let _ = ScreenToClient(window, &mut point);
            }
            let notches = ((wparam.0 >> 16) & 0xFFFF) as u16 as i16 as f32 / WHEEL_DELTA as f32;
            push(Event::Wheel([point.x as f32, point.y as f32], notches));
            LRESULT(0)
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            let held = |key| unsafe { GetKeyState(key) } < 0;
            push(Event::Key {
                key: wparam.0 as u32,
                control: held(VK_CONTROL),
                shift: held(VK_SHIFT),
            });
            // Alt+F4 is a system key, which closes the window.
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

pub struct Window {
    pub handle: HWND,
    window_mode: WindowMode,
    // The maximized window's placement, kept while in borderless fullscreen.
    windowed: Option<WINDOWPLACEMENT>,
    displays: Vec<Display>,
    // The display that the window is mostly on, like "KXGR16 (PA278QGV)".
    pub display: String,
    // What's wrong with the window's setup, if it isn't one of the two that
    // programs support. Then programs show only that, with `supported`.
    pub problems: Vec<String>,
    pub supported: String,
}

impl Window {
    // Makes the window for the program called `name`, hidden until `show`.
    // A display named on the command line must be connected, and one named
    // in the settings falls back to the primary display, with a notice.
    pub fn open(
        name: &str,
        settings: &Settings,
        arguments: &Arguments,
        notices: &mut Vec<String>,
    ) -> Result<Self> {
        unsafe {
            let instance = GetModuleHandleW(PCWSTR::null());
            check_handle(instance.0)?;
            let cursor = LoadCursorW(None, IDC_ARROW);
            check_handle(cursor.0)?;

            let class_name = w!("MainWindow");
            let class = WNDCLASSEXW {
                cbSize: size_of::<WNDCLASSEXW>() as u32,
                style: (CS_HREDRAW | CS_VREDRAW) as u32,
                lpfnWndProc: Some(window_procedure),
                hInstance: instance,
                hCursor: cursor,
                lpszClassName: class_name,
                ..Default::default()
            };
            if RegisterClassExW(&class).0 == 0 {
                return Err(Error::from_thread());
            }

            let displays = displays::enumerate()?;
            let mut display = None;
            if let Some(identifier) = settings.display.as_deref() {
                display = displays::find(&displays, identifier);
                if display.is_none() {
                    // A display from the command line is a request to open
                    // there and nowhere else.
                    if arguments.display_required {
                        return Err(Error::new(
                            E_FAIL,
                            format!(
                                "Display {identifier} isn't connected. IdentifyDisplays --list \
                                 shows the displays' identifiers."
                            ),
                        ));
                    }
                    notices.push(format!(
                        "Display {identifier} isn't connected, so the window opened on the \
                         primary display. IdentifyDisplays shows the displays' identifiers."
                    ));
                }
            }
            let display = display
                .or_else(|| displays::primary(&displays))
                .ok_or_else(|| Error::new(E_FAIL, "Windows reports no displays"))?;

            // The window stays hidden until it's in its place, and the
            // renderer is ready to draw into it. It's maximized while it's
            // hidden, so that showing it doesn't have to activate it.
            // Restored, it's half the work area's size in its middle, which
            // programs don't support, so they say so.
            let work = display.work_area;
            let (width, height) = ((work.right - work.left) / 2, (work.bottom - work.top) / 2);
            let title: Vec<u16> = name.encode_utf16().chain([0]).collect();
            let handle = CreateWindowExW(
                0,
                class_name,
                PCWSTR(title.as_ptr()),
                (WS_OVERLAPPEDWINDOW | WS_MAXIMIZE) as u32,
                work.left + width / 2,
                work.top + height / 2,
                width,
                height,
                None,
                None,
                Some(instance),
                None,
            );
            check_handle(handle.0)?;
            let mut window = Self {
                handle,
                window_mode: settings.window_mode,
                windowed: None,
                displays,
                display: String::new(),
                problems: Vec::new(),
                supported: window_check::supported(name),
            };
            // Before the renderer, so its swap chain starts at the display's size.
            if settings.window_mode == WindowMode::BorderlessFullscreen {
                window.toggle_fullscreen()?;
            }
            Ok(window)
        }
    }

    pub fn set_title(&self, title: &str) -> Result<()> {
        let title: Vec<u16> = title.encode_utf16().chain([0]).collect();
        unsafe { SetWindowTextW(self.handle, PCWSTR(title.as_ptr())) }.ok()
    }

    // Shows the window in its mode. Without `activate`, it opens behind the
    // window that has focus, and doesn't take it.
    pub fn show(&self, activate: bool) {
        // SW_SHOWNA shows the window as it is, maximized or not, without
        // activating it. The others activate it.
        let show = if !activate {
            SW_SHOWNA
        } else if self.window_mode == WindowMode::Maximized {
            SW_SHOWMAXIMIZED
        } else {
            SW_SHOW
        };
        unsafe {
            let _ = ShowWindow(self.handle, show);
        }
    }

    // Handles every pending message. Returns false once the window has closed.
    pub fn handle_messages(&mut self) -> bool {
        unsafe {
            let mut message = MSG::default();
            while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE as u32).as_bool() {
                if message.message == WM_QUIT as u32 {
                    return false;
                }
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            if DISPLAYS_CHANGED.swap(false, Ordering::Relaxed) {
                // If this fails, the old list only makes `display` stale.
                if let Ok(list) = displays::enumerate() {
                    self.displays = list;
                }
            }
            if WINDOW_CHANGED.swap(false, Ordering::Relaxed) {
                self.display = self.display_label();
                self.problems = window_check::problems(self.handle);
            }
            true
        }
    }

    // The client area's size, if it changed since the last call.
    pub fn take_resize(&self) -> Option<[u32; 2]> {
        RESIZED.swap(false, Ordering::Relaxed).then(|| {
            [
                CLIENT_WIDTH.load(Ordering::Relaxed),
                CLIENT_HEIGHT.load(Ordering::Relaxed),
            ]
        })
    }

    pub fn minimized(&self) -> bool {
        unsafe { IsIconic(self.handle).as_bool() }
    }

    // Sleeps until a message comes, like while the window is minimized.
    pub fn wait_for_message(&self) -> Result<()> {
        unsafe { WaitMessage() }.ok()
    }

    pub fn focused(&self) -> bool {
        unsafe { GetForegroundWindow() == self.handle }
    }

    // Where the cursor is in the window, if it's over the window, and the
    // window has focus.
    pub fn cursor(&self) -> Option<[f32; 2]> {
        if !self.focused() {
            return None;
        }
        unpack_point(CURSOR.load(Ordering::Relaxed))
    }

    // The mouse and keyboard input since the last call, oldest first, or
    // none if the window doesn't have focus.
    pub fn take_events(&self) -> Vec<Event> {
        let events = std::mem::take(&mut *EVENTS.lock().unwrap());
        if self.focused() { events } else { Vec::new() }
    }

    // Switches between the maximized window and a borderless one that
    // covers its monitor. Covering the monitor lets Windows flip the
    // program's frames straight to the screen, instead of the desktop
    // compositor copying them into its own frame, which saves up to a
    // refresh of latency and lets frames tear. A hidden window stays
    // hidden, so it can start in fullscreen. Entering fullscreen doesn't
    // activate the window: F11 only reaches a program while its window is
    // active, and activating it before it's shown would take focus.
    pub fn toggle_fullscreen(&mut self) -> Result<()> {
        unsafe {
            let window = self.handle;
            let visible = GetWindowLongPtrW(window, GWL_STYLE) & WS_VISIBLE as isize;
            if let Some(mut placement) = self.windowed.take() {
                SetWindowLongPtrW(window, GWL_STYLE, WS_OVERLAPPEDWINDOW as isize | visible);
                // Maximized is the only other supported setup.
                placement.showCmd = SW_SHOWMAXIMIZED as u32;
                SetWindowPlacement(window, &placement).ok()?;
                // Makes the new style take effect.
                SetWindowPos(
                    window,
                    None,
                    0,
                    0,
                    0,
                    0,
                    (SWP_FRAMECHANGED
                        | SWP_NOMOVE
                        | SWP_NOSIZE
                        | SWP_NOZORDER
                        | SWP_NOOWNERZORDER) as u32,
                )
                .ok()
            } else {
                let mut placement = WINDOWPLACEMENT {
                    length: size_of::<WINDOWPLACEMENT>() as u32,
                    ..Default::default()
                };
                GetWindowPlacement(window, &mut placement).ok()?;
                let mut monitor = MONITORINFO {
                    cbSize: size_of::<MONITORINFO>() as u32,
                    ..Default::default()
                };
                GetMonitorInfoW(
                    MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST as u32),
                    &mut monitor,
                )
                .ok()?;
                SetWindowLongPtrW(window, GWL_STYLE, WS_POPUP as isize | visible);
                let area = monitor.rcMonitor;
                SetWindowPos(
                    window,
                    None,
                    area.left,
                    area.top,
                    area.right - area.left,
                    area.bottom - area.top,
                    (SWP_FRAMECHANGED | SWP_NOOWNERZORDER | SWP_NOACTIVATE) as u32,
                )
                .ok()?;
                self.windowed = Some(placement);
                Ok(())
            }
        }
    }

    // The display that the window is mostly on, like "KXGR16 (PA278QGV)".
    // Clones share a monitor, so this names the first.
    fn display_label(&self) -> String {
        let monitor = unsafe { MonitorFromWindow(self.handle, MONITOR_DEFAULTTONEAREST as u32) };
        match self
            .displays
            .iter()
            .find(|display| display.monitor == monitor)
        {
            Some(display) => format!("{} ({})", display.identifier, display.name),
            None => "unknown".to_string(),
        }
    }
}
