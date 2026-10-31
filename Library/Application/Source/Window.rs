// A program's window: one per process, of the class "MainWindow", which is
// how RunInBackground finds it. It opens on the display that the settings or
// the command line name, maximized or in borderless fullscreen, and
// toggle_fullscreen switches between the two, for the program's F11.
//
// The window procedure only records what happened, and the main loop reacts.
// This keeps program state out of the callback.
//
// Each frame starts with begin_frame, which says the frame's time and input.
// Programs ignore all input while the window doesn't have focus: then a
// frame has no events and no cursor, and a click that gives the window focus
// does only that.
//
// With a script (see Script.rs), the script is the only input, the window
// acts as if it had focus, and time is the script's fixed clock.

use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::Instant;

use displays::Display;
use json::Value;
use windows_api::*;
use windows_core::{Error, PCWSTR, Result, w};

use crate::script::{Checkpoint, Script, compare};
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
    // while the key is held, with `repeat`, which keys that switch something
    // on and off ignore, so that holding them doesn't flip it back and forth.
    Key { key: u32, control: bool, shift: bool, repeat: bool },
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
                // Bit 30 says whether the key was already down.
                repeat: lparam.0 & (1 << 30) != 0,
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

// What a frame starts with: its time and its input.
pub struct Frame {
    // Counting from 0.
    pub number: u64,
    // Since the window opened, or the script's fixed clock.
    pub seconds: f64,
    // Whether the window has focus, which a script stands in for.
    pub focused: bool,
    // The mouse and keyboard input since the last frame, oldest first.
    pub events: Vec<Event>,
    // Where the cursor is in the window, if it's over it, with focus.
    pub cursor: Option<[f32; 2]>,
    // A script's "Program" steps for this frame, for input that doesn't
    // come from the window. See Script.rs.
    pub program_steps: Vec<Value>,
    // What the program saves or checks at the end of this frame. See
    // handle_checkpoints.
    pub checkpoints: Vec<Checkpoint>,
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
    opened: Instant,
    frame: u64,
    script: Option<Script>,
    // How a scripted run's state differed from what it expected.
    failures: Vec<String>,
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
        let script = match (&arguments.script, &arguments.output) {
            (Some(path), Some(output)) => Some(
                Script::load(path, output).map_err(|problem| Error::new(E_INVALIDARG, problem))?,
            ),
            (Some(_), None) => {
                return Err(Error::new(
                    E_INVALIDARG,
                    "--script needs --output and the folder to save what it captures in",
                ));
            }
            _ => None,
        };
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
                    if arguments.display.is_some() {
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
                opened: Instant::now(),
                frame: 0,
                script,
                failures: Vec::new(),
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

    // Shows the window in its mode. Without `activate`, it doesn't take focus
    // from the window that has it, though it opens on top of the others.
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

    // Handles every pending message. Returns false once the window has
    // closed, or a script has finished.
    pub fn handle_messages(&mut self) -> bool {
        if self.script.as_ref().is_some_and(Script::finished) {
            return false;
        }
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

    // Whether the window covers its monitor, without a frame.
    pub fn fullscreen(&self) -> bool {
        self.windowed.is_some()
    }

    // Whether a script is the program's input.
    pub fn scripted(&self) -> bool {
        self.script.is_some()
    }

    // Starts a frame: its time, and the input since the last one. Programs
    // call it once the renderer is ready, right before drawing.
    pub fn begin_frame(&mut self) -> Frame {
        let number = self.frame;
        self.frame += 1;
        // The window's own input, dropped without focus, or with a script.
        let events = std::mem::take(&mut *EVENTS.lock().unwrap());
        if let Some(script) = &mut self.script {
            let frame = script.next_frame();
            return Frame {
                number,
                seconds: frame.seconds,
                focused: true,
                events: frame.events,
                cursor: frame.cursor,
                program_steps: frame.program_steps,
                checkpoints: frame.checkpoints,
            };
        }
        let focused = unsafe { GetForegroundWindow() == self.handle };
        Frame {
            number,
            seconds: self.opened.elapsed().as_secs_f64(),
            focused,
            events: if focused { events } else { Vec::new() },
            cursor: if focused {
                unpack_point(CURSOR.load(Ordering::Relaxed))
            } else {
                None
            },
            program_steps: Vec::new(),
            checkpoints: Vec::new(),
        }
    }

    // Does what a script asks at the end of a frame, after the program has
    // recorded its draws and before it presents them: `capture` saves the
    // frame as a PNG at a path, and `state` is the program's state, as an
    // object, to save or check. The state gets the window's own, as
    // "Window".
    pub fn handle_checkpoints(
        &mut self,
        frame: &Frame,
        mut capture: impl FnMut(&Path) -> Result<()>,
        state: impl FnOnce() -> Value,
    ) -> Result<()> {
        if frame.checkpoints.is_empty() {
            return Ok(());
        }
        let mut state = state();
        let window = json::object([
            ("Fullscreen", self.fullscreen().into()),
            ("Problems", self.problems.clone().into()),
            ("Frame", frame.number.into()),
        ]);
        match &mut state {
            Value::Object(members) => members.push(("Window".to_string(), window)),
            other => *other = json::object([("Program", other.clone()), ("Window", window)]),
        }
        for checkpoint in &frame.checkpoints {
            match checkpoint {
                Checkpoint::Capture { image, state: path } => {
                    if let Some(folder) = path.parent() {
                        std::fs::create_dir_all(folder).map_err(|error| {
                            Error::new(E_FAIL, format!("Couldn't make {}: {error}", folder.display()))
                        })?;
                    }
                    capture(image)?;
                    std::fs::write(path, state.to_json() + "\n").map_err(|error| {
                        Error::new(E_FAIL, format!("Couldn't write {}: {error}", path.display()))
                    })?;
                }
                Checkpoint::Expect(expected) => {
                    for difference in compare(expected, &state) {
                        self.failures.push(format!("Frame {}: {difference}", frame.number));
                    }
                }
            }
        }
        Ok(())
    }

    // Once the main loop ends: fails if a script's expectations weren't
    // met, after writing each difference to standard error, where
    // RunInBackground logs it.
    pub fn finish(&self) -> Result<()> {
        if self.failures.is_empty() {
            return Ok(());
        }
        for failure in &self.failures {
            eprintln!("{failure}");
        }
        Err(Error::new(
            E_FAIL,
            format!("{} of the script's expectations weren't met", self.failures.len()),
        ))
    }

    // Switches between the maximized window and a borderless one that
    // covers its monitor. Covering the monitor lets Windows flip the
    // program's frames straight to the screen, instead of the desktop
    // compositor copying them into its own frame, which saves up to a
    // refresh of latency and lets frames tear. A hidden window stays
    // hidden, so it can start in fullscreen. Neither way activates a window
    // that doesn't have focus, so a script's F11 doesn't take focus from the
    // window that the user works in.
    pub fn toggle_fullscreen(&mut self) -> Result<()> {
        unsafe {
            let window = self.handle;
            let visible = GetWindowLongPtrW(window, GWL_STYLE) & WS_VISIBLE as isize;
            if let Some(mut placement) = self.windowed.take() {
                SetWindowLongPtrW(window, GWL_STYLE, WS_OVERLAPPEDWINDOW as isize | visible);
                if GetForegroundWindow() != window {
                    return self.maximize_without_activating(placement, visible);
                }
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

    // Maximizes the window, with a frame, without activating it, which
    // SW_SHOWMAXIMIZED would. `placement` has the size it restores to. It
    // shows the window restored there, without activating it, then gives it
    // the maximized style and puts it where Windows puts a maximized window:
    // over the work area, with its borders past the edges.
    unsafe fn maximize_without_activating(&self, mut placement: WINDOWPLACEMENT, visible: isize) -> Result<()> {
        unsafe {
            let window = self.handle;
            placement.showCmd = if visible != 0 { SW_SHOWNOACTIVATE } else { SW_HIDE } as u32;
            SetWindowPlacement(window, &placement).ok()?;
            SetWindowLongPtrW(
                window,
                GWL_STYLE,
                (WS_OVERLAPPEDWINDOW | WS_MAXIMIZE) as isize | visible,
            );
            let mut monitor = MONITORINFO {
                cbSize: size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            GetMonitorInfoW(
                MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST as u32),
                &mut monitor,
            )
            .ok()?;
            // A frame around an empty client area: its right edge is the
            // border's thickness.
            let mut frame = RECT::default();
            AdjustWindowRectExForDpi(
                &mut frame,
                WS_OVERLAPPEDWINDOW as u32,
                false,
                0,
                GetDpiForWindow(window),
            )
            .ok()?;
            let border = frame.right;
            let work = monitor.rcWork;
            SetWindowPos(
                window,
                None,
                work.left - border,
                work.top - border,
                work.right - work.left + border * 2,
                work.bottom - work.top + border * 2,
                (SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_NOACTIVATE) as u32,
            )
            .ok()
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
