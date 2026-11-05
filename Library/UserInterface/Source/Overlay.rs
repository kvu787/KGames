// Windows without a frame that float over the others, like IdentifyDisplays'
// boxes, one on each display. Each has its own interface, drawn with one
// GPU, and they draw only until their contents show: they don't change, so
// the loop then waits for messages. A click on any of them, or Esc while one
// has focus, closes them all.

use graphics::{AdapterChoice, Gpu, Presenter};
use windows_api::RECT;

use crate::clipboard::Clipboard;
use crate::geometry::Size;
use crate::renderer::Renderer;
use crate::text::Fonts;
use crate::theme::ThemeKind;
use crate::ui::Ui;
use crate::window::Window;

// An overlay's title and where it is on the desktop, in physical pixels.
#[derive(Clone, Debug)]
pub struct Overlay {
    pub title: String,
    pub rect: RECT,
}

struct Shown {
    window: Window,
    presenter: Presenter,
    renderer: Renderer,
    ui: Ui,
    frames: u32,
}

// Frames that an overlay draws before the loop waits: the second makes sure
// that the first has reached the screen.
const FRAMES: u32 = 2;

// Shows the overlays, with focus, and draws each with `draw`, given its
// index, until one closes.
pub fn run_overlays(overlays: &[Overlay], theme: ThemeKind, mut draw: impl FnMut(&mut Ui, usize)) -> Result<(), String> {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).map_err(|error| format!("Couldn't start Direct3D 12: {error}"))?;
    let mut shown = Vec::new();
    for overlay in overlays {
        let window = Window::create_overlay(&overlay.title, overlay.rect).map_err(|error| format!("Couldn't open a window: {error}"))?;
        let (width, height) = window.client_size();
        let presenter = Presenter::new(&gpu, window.hwnd(), width, height, false)
            .map_err(|error| format!("Couldn't make a swap chain: {error}"))?;
        let renderer = Renderer::new(&gpu).map_err(|error| format!("Couldn't make a renderer: {error}"))?;
        let fonts = Fonts::new().map_err(|error| format!("Couldn't start DirectWrite: {error}"))?;
        shown.push(Shown {
            window,
            presenter,
            renderer,
            ui: Ui::new(fonts, theme, Clipboard::Own(String::new())),
            frames: 0,
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
        if shown.iter().any(|overlay| overlay.window.close_requested()) {
            return Ok(());
        }
        let mut waiting = false;
        for (index, overlay) in shown.iter_mut().enumerate() {
            if overlay.frames >= FRAMES {
                continue;
            }
            if !overlay.presenter.poll_ready(&gpu) {
                waiting = true;
                continue;
            }
            let (width, height) = overlay.presenter.size();
            let events = overlay.window.take_events();
            overlay.ui.begin_frame(&events, start.elapsed().as_secs_f64(), Size::new(width as f32, height as f32), false);
            draw(&mut overlay.ui, index);
            overlay.ui.end_frame();
            let frame = overlay.presenter.begin().map_err(|error| format!("Couldn't begin a frame: {error}"))?;
            let background = overlay.ui.theme().background;
            unsafe { frame.clear([background.r, background.g, background.b, 1.0]) };
            overlay
                .renderer
                .draw(&gpu, &frame, &overlay.ui.draw, &mut overlay.ui.fonts)
                .map_err(|error| format!("Couldn't draw: {error}"))?;
            overlay.presenter.present(&mut gpu, frame).map_err(|error| format!("Couldn't present: {error}"))?;
            gpu.report_messages();
            overlay.frames += 1;
            waiting |= overlay.frames < FRAMES;
        }
        if !waiting && let Some(first) = shown.first() {
            first.window.wait();
        }
    }
}
