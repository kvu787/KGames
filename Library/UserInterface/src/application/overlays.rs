// Windows without a frame that float over the others, like IdentifyDisplays'
// boxes, one on each display. Each has its own interface, drawn with one
// GPU, and draws a frame at every refresh of its display, with vertical sync,
// like a program's main loop does, even though its contents rarely change: a
// display with a variable refresh rate then sees the overlays present like
// any other program, instead of a frame or two and then none. Each frame's
// `view` is told whether one of the overlays has focus. The loop reads the
// input once a frame, and waits between frames for any of the displays to
// take another, so one that stops taking them, like one that's asleep, holds
// up none of the others. A click on any of them, or Esc, while one has focus,
// closes them all. The input comes from GameInput, like every program's, so only while
// one of them has focus: GameInput doesn't report the click that gives it
// focus, so that click only does that.

use game_input::{GameInputKindKeyboard, GameInputKindMouse, GameInputMouseLeftButton};
use graphics::{AdapterChoice, Gpu, Presenter};
use windows_api::RECT;

use crate::core::interface::{Interface, Ui};
use crate::core::tree::Tree;
use crate::design::theme::ThemeKind;
use crate::drawing::geometry::Size;
use crate::drawing::renderer::Renderer;
use crate::drawing::text::Fonts;
use crate::input::devices::Key;
use crate::platform::clipboard::Clipboard;
use crate::platform::input_devices::Devices;
use crate::platform::window::Window;

// An overlay's title, and where it is on the desktop, in physical pixels.
#[derive(Clone, Debug)]
pub struct Overlay {
    pub title: String,
    pub rect: RECT,
}

struct Shown {
    window: Window,
    presenter: Presenter,
    renderer: Renderer,
    ui: Interface,
}

// How long the loop waits at most for a display to take another frame, or
// for a message, in milliseconds.
const WAIT_MILLISECONDS: u32 = 100;

// Whether a left click or Esc came since the last check. Each device's first
// reading only says what's held already.
#[derive(Default)]
struct Presses {
    mice: Vec<(Option<usize>, bool)>,
    keyboards: Vec<(Option<usize>, bool)>,
}

impl Presses {
    // Whether the left button went down while the cursor was on an overlay,
    // or Esc went down while one had focus.
    fn closing(&mut self, devices: &Devices, shown: &mut [Shown]) -> bool {
        let mut clicked = false;
        for reading in devices.readings(GameInputKindMouse) {
            let down = reading.mouse.buttons & GameInputMouseLeftButton != 0;
            clicked |= went_down(&mut self.mice, reading.device, down);
        }
        let mut escaped = false;
        for reading in devices.readings(GameInputKindKeyboard) {
            let down = reading.keys.iter().any(|key| key.virtualKey == Key::ESCAPE.0);
            escaped |= went_down(&mut self.keyboards, reading.device, down);
        }
        (clicked && shown.iter_mut().any(|overlay| overlay.window.cursor_over()))
            || (escaped && shown.iter().any(|overlay| overlay.window.focused()))
    }
}

// Whether the device's button or key went down, and remembers that it's down.
fn went_down(devices: &mut Vec<(Option<usize>, bool)>, device: Option<usize>, down: bool) -> bool {
    match devices.iter_mut().find(|(known, _)| *known == device) {
        Some((_, was_down)) => !std::mem::replace(was_down, down) && down,
        None => {
            devices.push((device, down));
            false
        }
    }
}

// Shows the overlays, with focus, and draws each with the tree that `view`
// describes, given its index and whether one of the overlays has focus, until
// one closes.
pub fn run_overlays(overlays: &[Overlay], theme: ThemeKind, mut view: impl FnMut(&mut Ui, usize, bool) -> Tree<()>) -> Result<(), String> {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).map_err(|error| format!("Couldn't start Direct3D 12: {error}"))?;
    let mut devices = Devices::new().map_err(|error| format!("Couldn't start GameInput, which reads the click or Esc that closes the overlays: {error}"))?;
    let mut presses = Presses::default();
    let mut shown = Vec::new();
    for overlay in overlays {
        let window = Window::create_overlay(&overlay.title, overlay.rect).map_err(|error| format!("Couldn't open a window: {error}"))?;
        let (width, height) = window.client_size();
        let presenter = Presenter::new(&gpu, window.hwnd(), width, height).map_err(|error| format!("Couldn't make a swap chain: {error}"))?;
        let renderer = Renderer::new(&gpu).map_err(|error| format!("Couldn't make a renderer: {error}"))?;
        let fonts = Fonts::new().map_err(|error| format!("Couldn't start DirectWrite: {error}"))?;
        shown.push(Shown {
            window,
            presenter,
            renderer,
            ui: Interface::new(fonts, theme, Clipboard::Own(String::new())),
        });
    }
    for overlay in &mut shown {
        overlay.window.show(true);
    }
    let start = std::time::Instant::now();
    loop {
        for overlay in &mut shown {
            overlay.window.pump();
        }
        devices.read();
        if shown.iter().any(|overlay| overlay.window.close_requested()) || presses.closing(&devices, &mut shown) {
            // The swap chains go as this returns, before the GPU does, so the
            // GPU must be done with their last frames first.
            gpu.flush().map_err(|error| format!("Couldn't wait for the GPU: {error}"))?;
            return Ok(());
        }
        let focused = shown.iter().any(|overlay| overlay.window.focused());
        let mut drew = false;
        for (index, overlay) in shown.iter_mut().enumerate() {
            // An overlay whose display hasn't taken its last frame yet draws
            // on a later turn of the loop.
            if !overlay.presenter.poll_ready(&gpu) {
                continue;
            }
            let (width, height) = overlay.presenter.size();
            let tree = view(&mut Ui { inner: &mut overlay.ui }, index, focused);
            overlay.ui.frame_of(&tree, &[], start.elapsed().as_secs_f64(), Size::new(width as f32, height as f32), false);
            let frame = overlay.presenter.begin().map_err(|error| format!("Couldn't begin a frame: {error}"))?;
            let background = overlay.ui.theme().background;
            unsafe { frame.clear([background.r, background.g, background.b, 1.0]) };
            overlay
                .renderer
                .draw(&gpu, &frame, &overlay.ui.draw, &mut overlay.ui.fonts)
                .map_err(|error| format!("Couldn't draw: {error}"))?;
            overlay.presenter.present(&mut gpu, frame).map_err(|error| format!("Couldn't present: {error}"))?;
            gpu.report_messages();
            drew = true;
        }
        // Waits, without spinning, for any of the displays to take another
        // frame, or for a message.
        if !drew {
            let mut presenters: Vec<&mut Presenter> = shown.iter_mut().map(|overlay| &mut overlay.presenter).collect();
            Presenter::wait_any_ready(&mut presenters, &gpu, WAIT_MILLISECONDS).map_err(|error| format!("Couldn't wait for the GPU: {error}"))?;
        }
    }
}
