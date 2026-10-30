// No console window in any build.
#![windows_subsystem = "windows"]

use std::path::Path;

use application::{Arguments, FrameCounter, Settings, Window};
use canvas::theme::{DIM, SUCCESS, TEXT, TITLE, WARNING};
use canvas::{Canvas, Font, MARGIN, PADDING, draw_window_problems, text_box, theme};
use graphics::Renderer;
use minimal_scene_test::controls::{self, Action};
use minimal_scene_test::scene::{Motion, Scene};
use minimal_scene_test::settings::SceneSettings;
use minimal_scene_test::{FONT_SIZE, NAME};
use windows_core::Result;

fn main() -> Result<()> {
    unsafe {
        application::prepare_process()?;
        // Problems with the settings and the command line, which the text
        // over the scene lists.
        let mut notices = Vec::new();
        let arguments = Arguments::parse(&mut notices);
        let settings_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), r"\Settings.json"));
        let (settings, scene_settings) =
            Settings::for_program::<SceneSettings>(settings_path, &arguments, &mut notices);
        let mut window = Window::open(NAME, &settings, &arguments, &mut notices)?;

        let font = Font::new(FONT_SIZE)?;
        let mut renderer = Renderer::new(window.handle, settings.allow_tearing, true)?;
        let overlay = canvas::Pipeline::new(&mut renderer, &font)?;
        let scene = Scene::new(&renderer, &scene_settings)?;
        let mut motion = Motion::default();
        // The title bar says how frames are presented, as it affects latency.
        let description = renderer.description();
        window.set_title(&format!("{NAME} - {description}"))?;
        window.show(arguments.activate);
        let mut frames = FrameCounter::default();
        let mut vertices = Vec::new();
        let mut show_text = true;
        // The last frame's time, for how far the sphere moves.
        let mut last_seconds: Option<f64> = None;

        // Handle all pending messages, then render one frame once the
        // renderer is ready for it.
        while window.handle_messages() {
            if let Some([width, height]) = window.take_resize() {
                renderer.resize(width, height)?;
            }

            if window.minimized() {
                // Minimized: sleep until something happens instead of
                // spinning. The sphere doesn't jump ahead afterward.
                window.wait_for_message()?;
                frames.pause();
                last_seconds = None;
                continue;
            }

            // Spin until the GPU and DXGI can take a frame, like GameInputDemo,
            // which explains why. The loop keeps handling window messages
            // through the wait.
            if !renderer.is_ready()? {
                std::hint::spin_loop();
                continue;
            }

            // Read the time and input only now, right before drawing, so the
            // frame shows the newest.
            let frame = window.begin_frame();
            frames.frame(frame.seconds);
            if let Some(last) = last_seconds {
                motion.advance((frame.seconds - last) as f32);
            }
            last_seconds = Some(frame.seconds);
            for event in &frame.events {
                match controls::action(event) {
                    Some(Action::ToggleText) => show_text = !show_text,
                    Some(Action::ToggleFullscreen) => window.toggle_fullscreen()?,
                    None => {}
                }
            }

            let mut canvas = Canvas::new(&font, &mut vertices);
            let (width, height) = (renderer.width as f32, renderer.height as f32);
            renderer.clear(theme::BACKGROUND);
            if window.problems.is_empty() {
                scene.draw(&renderer, motion.position)?;
                if show_text {
                    let rate = match frames.per_second {
                        Some(rate) => format!("{rate:.0}  ({:.2} ms per frame)", 1000.0 / rate),
                        None => "measuring".to_string(),
                    };
                    let focus = if frame.focused {
                        ("Window has focus".to_string(), SUCCESS)
                    } else {
                        ("Window doesn't have focus: input is ignored".to_string(), WARNING)
                    };
                    let piece = |text: &str, color| (text.to_string(), color);
                    let mut lines = vec![
                        vec![
                            piece(NAME, TITLE),
                            piece("  A sphere moving at 8 units per second", DIM),
                        ],
                        vec![piece("FPS: ", DIM), piece(&rate, TEXT)],
                        vec![piece("GPU: ", DIM), piece(&description, TEXT)],
                        vec![piece("Display: ", DIM), piece(&window.display, TEXT)],
                        vec![focus],
                        vec![
                            piece("F1", TEXT),
                            piece(" hide this   ", DIM),
                            piece("F11", TEXT),
                            piece(" borderless fullscreen", DIM),
                        ],
                    ];
                    lines.extend(notices.iter().map(|notice| vec![piece(notice, WARNING)]));
                    // At the bottom left, which the sphere never crosses:
                    // it goes off the top left and bottom right corners.
                    let box_height = canvas.line_height() * lines.len() as f32 + PADDING * 2.0;
                    text_box(&mut canvas, MARGIN, height - MARGIN - box_height, &lines);
                }
            } else {
                draw_window_problems(&mut canvas, &window.supported, &window.problems, width);
            }
            overlay.draw(&renderer, canvas.vertices);
            window.handle_checkpoints(
                &frame,
                |path| renderer.capture(path),
                || {
                    json::object([
                        ("SpherePosition", motion.position.into()),
                        ("SphereDirection", motion.direction.into()),
                        ("TextShown", show_text.into()),
                    ])
                },
            )?;
            renderer.present()?;
        }
        // What the frames used is dropped before the renderer.
        renderer.wait_for_gpu()?;
        window.finish()
    }
}
