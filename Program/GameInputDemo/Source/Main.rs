// No console window in any build.
#![windows_subsystem = "windows"]

use std::path::Path;

use application::{Arguments, Event, FrameCounter, Settings, Window};
use canvas::{Canvas, Font, theme};
use game_input_demo::devices::InputSource;
use game_input_demo::game_input_source::GameInputSource;
use game_input_demo::input::Input;
use game_input_demo::panels;
use game_input_demo::scripted_source::ScriptedSource;
use game_input_demo::settings::DemoSettings;
use game_input_demo::{FONT_SIZE, NAME};
use graphics::Renderer;
use windows_api::*;
use windows_core::{Error, Result};

fn main() -> Result<()> {
    unsafe {
        application::prepare_process()?;
        // Problems with the settings and the command line, which the header lists.
        let mut notices = Vec::new();
        let arguments = Arguments::parse(&mut notices);
        let settings_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), r"\Settings.json"));
        let (settings, demo) =
            Settings::for_program::<DemoSettings>(settings_path, &arguments, &mut notices);
        let mut window = Window::open(NAME, &settings, &arguments, &mut notices)?;

        let font = Font::new(FONT_SIZE)?;
        let mut renderer = Renderer::new(window.handle, settings.allow_tearing, false)?;
        let overlay = canvas::Pipeline::new(&mut renderer, &font)?;
        // The title bar says how frames are presented, as it affects latency.
        window.set_title(&format!("{NAME} - {}", renderer.description()))?;
        window.show(arguments.activate);
        // A scripted run reads the script's devices and readings instead of
        // GameInput's. Without GameInput the window still opens, and says why
        // there's no input.
        let source: std::result::Result<Box<dyn InputSource>, String> = if window.scripted() {
            Ok(Box::new(ScriptedSource::default()))
        } else {
            GameInputSource::new()
                .map(|source| Box::new(source) as Box<dyn InputSource>)
                .map_err(|error| format!("{} ({})", error.message(), error.code()))
        };
        let mut input = source.map(|source| Input::new(source, demo.rumble_dead_zone));
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
                // A minimized window doesn't have focus, but minimizing it
                // takes its focus in the same messages, so no frame sees it
                // without focus. Without this, its input would still count as
                // focused when it's restored, and what was typed into other
                // windows meanwhile would be handled then. This drops that,
                // stops the motors, and makes restoring it count as getting
                // focus.
                if let Ok(input) = &mut input {
                    input.update(false);
                }
                // Then sleep until something happens instead of spinning.
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
            let frame = window.begin_frame();
            frames.frame(frame.seconds);
            let focused = frame.focused;
            // The select box reads the mouse from the window's messages, not
            // from GameInput, because they say where the cursor is in the window.
            let events = &frame.events;
            if let Ok(input) = &mut input {
                if let Some(scripted) = input.scripted() {
                    scripted.now = (frame.seconds * 1_000_000.0) as u64;
                    for step in &frame.program_steps {
                        scripted.apply(step).map_err(|problem| {
                            Error::new(E_INVALIDARG, format!("Frame {}: {problem}", frame.number))
                        })?;
                    }
                }
                input.update(focused);
                if std::mem::take(&mut input.fullscreen_toggled) {
                    window.toggle_fullscreen()?;
                }
                // A click on the select box opens or closes its list, and a
                // click on the list picks what it's on. Any other click closes it.
                for event in events {
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
                cursor: frame.cursor,
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
            window.handle_checkpoints(
                &frame,
                |path| renderer.capture(path),
                || {
                    let mut state = match &input {
                        Ok(input) => input.state(),
                        Err(_) => json::Value::Object(Vec::new()),
                    };
                    if let json::Value::Object(members) = &mut state {
                        members.push(("PickerOpen".to_string(), picker_open.into()));
                        members.push(("Targets".to_string(), targets.state()));
                    }
                    state
                },
            )?;
            renderer.present()?;
        }
        // What the frames used is dropped before the renderer.
        renderer.wait_for_gpu()?;
        window.finish()
    }
}
