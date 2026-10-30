// No console window in any build.
#![windows_subsystem = "windows"]

#[path = "Input.rs"]
mod input;
#[path = "Panels.rs"]
mod panels;
#[path = "Settings.rs"]
mod settings;

use std::path::Path;

use application::{Arguments, Event, FrameCounter, Settings, Window};
use canvas::{Canvas, Font, theme};
use graphics::Renderer;
use input::Input;
use settings::DemoSettings;
use windows_api::*;
use windows_core::Result;

const NAME: &str = "GameInput Demo";
// The em height of the font, in pixels. Programs only support 100% scaling
// (see Library/Application), so this never changes.
const FONT_SIZE: i32 = 15;

fn main() -> Result<()> {
    unsafe {
        application::prepare_process()?;
        // Problems with the settings and the command line, which the header lists.
        let mut notices = Vec::new();
        let settings_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), r"\Settings.json"));
        let (mut settings, demo) = Settings::load::<DemoSettings>(settings_path, &mut notices);
        let arguments = Arguments::parse(&mut settings, &mut notices);
        let mut window = Window::open(NAME, &settings, &arguments, &mut notices)?;

        let font = Font::new(FONT_SIZE)?;
        let mut renderer = Renderer::new(window.handle, settings.allow_tearing, false)?;
        let overlay = canvas::Pipeline::new(&mut renderer, &font)?;
        // The title bar says how frames are presented, as it affects latency.
        window.set_title(&format!("{NAME} - {}", renderer.description()))?;
        window.show(arguments.activate);
        // Without GameInput the window still opens, and says why there's no input.
        let mut input = Input::new(demo.rumble_dead_zone)
            .map_err(|error| format!("{} ({})", error.message(), error.code()));
        let mut frames = FrameCounter::default();
        let mut vertices = Vec::new();
        // Where clicks do something in the last frame drawn, and whether the
        // gamepad select box's list is open.
        let mut targets = panels::Targets::default();
        let mut picker_open = false;

        // Handle all pending messages, then render one frame once the
        // renderer is ready for it.
        while window.handle_messages() {
            if let Some([width, height]) = window.take_resize() {
                renderer.resize(width, height)?;
            }

            if window.minimized() {
                // Minimized: sleep until something happens instead of spinning.
                if let Ok(input) = &mut input {
                    input.stop_rumble();
                }
                window.wait_for_message()?;
                frames.pause();
                continue;
            }

            // Spin until the GPU and DXGI can take a frame. Blocking on an
            // event would instead wait for Windows to wake the thread, which
            // can take a while if its core went into a power-saving state.
            // The wait is short, because frames don't wait for vertical sync,
            // and the loop keeps handling window messages through it.
            if !renderer.is_ready()? {
                std::hint::spin_loop();
                continue;
            }

            // Read input only now, right before drawing, so the frame shows
            // the newest readings.
            frames.frame();
            let focused = window.focused();
            // The select box reads the mouse from the window's messages, not
            // from GameInput, because they say where the cursor is in the window.
            let events = window.take_events();
            if let Ok(input) = &mut input {
                input.update(focused);
                if std::mem::take(&mut input.fullscreen_toggled) {
                    window.toggle_fullscreen()?;
                }
                // A click on the select box opens or closes its list, and a
                // click on the list picks what it's on. Any other click closes it.
                for event in &events {
                    if let Event::Press(point) = *event {
                        match targets.at(point) {
                            Some(panels::Click::Picker) => picker_open = !picker_open,
                            Some(panels::Click::Choose(listen)) => {
                                input.pick_listen(listen.clone());
                                picker_open = false;
                            }
                            None => picker_open = false,
                        }
                    }
                }
            }
            // So do Esc, and the window losing focus.
            let escape = events.iter().any(
                |event| matches!(event, Event::Key { key, .. } if *key == VK_ESCAPE as u32),
            );
            if escape || !focused {
                picker_open = false;
            }
            let mut canvas = Canvas::new(&font, &mut vertices);
            let status = panels::Status {
                focused,
                frames: &frames,
                display: &window.display,
                notices: &notices,
                supported: &window.supported,
                window_problems: &window.problems,
                cursor: window.cursor(),
                picker_open,
            };
            targets = panels::draw(
                &mut canvas,
                &input,
                &status,
                renderer.width as f32,
                renderer.height as f32,
            );
            renderer.clear(theme::BACKGROUND);
            overlay.draw(&renderer, canvas.vertices);
            renderer.present()?;
        }
        // What the frames used is dropped before the renderer.
        renderer.wait_for_gpu()
    }
}
