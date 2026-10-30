// No console window in any build.
#![windows_subsystem = "windows"]

#[path = "CarMesh.rs"]
mod car_mesh;
#[path = "Scene.rs"]
mod scene;
#[path = "Settings.rs"]
mod settings;

use std::path::Path;

use application::{Arguments, Event, FrameCounter, Settings, Window};
use canvas::theme::{DIM, SUCCESS, TEXT, TITLE, WARNING};
use canvas::{Canvas, Font, MARGIN, PADDING, draw_window_problems, text_box, theme};
use graphics::Renderer;
use scene::Scene;
use settings::SceneSettings;
use windows_api::*;
use windows_core::Result;

const NAME: &str = "Spinning Car Test";
// The em height of the font, in pixels. Programs only support 100% scaling
// (see Library/Application), so this never changes.
const FONT_SIZE: i32 = 15;
// F1 shows or hides the text over the scene, and S, as in the C++ scene,
// keeps the car in the middle.
const KEY_TOGGLE_TEXT: u32 = 0x70;
const KEY_STATIONARY: u32 = 'S' as u32;

fn main() -> Result<()> {
    unsafe {
        application::prepare_process()?;
        // Problems with the settings and the command line, which the text
        // over the scene lists.
        let mut notices = Vec::new();
        let settings_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), r"\Settings.json"));
        let background_path =
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), r"\Assets\SceneBackground.png"));
        let (mut settings, scene_settings) =
            Settings::load::<SceneSettings>(settings_path, &mut notices);
        let arguments = Arguments::parse(&mut settings, &mut notices);
        let mut window = Window::open(NAME, &settings, &arguments, &mut notices)?;

        let font = Font::new(FONT_SIZE)?;
        let mut renderer = Renderer::new(window.handle, settings.allow_tearing, true)?;
        let overlay = canvas::Pipeline::new(&mut renderer, &font)?;
        let mut scene = Scene::new(&mut renderer, &scene_settings, background_path)?;
        // The title bar says how frames are presented, as it affects latency.
        let description = renderer.description();
        window.set_title(&format!("{NAME} - {description}"))?;
        window.show(arguments.activate);
        let mut frames = FrameCounter::default();
        let mut vertices = Vec::new();
        let mut show_text = true;

        // Handle all pending messages, then render one frame once the
        // renderer is ready for it.
        while window.handle_messages() {
            if let Some([width, height]) = window.take_resize() {
                renderer.resize(width, height)?;
            }

            if window.minimized() {
                // Minimized: sleep until something happens instead of
                // spinning. The car keeps time meanwhile, like in the C++ scene.
                window.wait_for_message()?;
                frames.pause();
                continue;
            }

            // Spin until the GPU and DXGI can take a frame, like GameInputDemo,
            // which explains why. The loop keeps handling window messages
            // through the wait.
            if !renderer.is_ready()? {
                std::hint::spin_loop();
                continue;
            }

            // Read input only now, right before drawing, so the frame shows
            // the newest. The scene reads the time as it draws.
            frames.frame();
            for event in window.take_events() {
                if let Event::Key { key, .. } = event {
                    match key {
                        KEY_TOGGLE_TEXT => show_text = !show_text,
                        KEY_STATIONARY => scene.stationary = !scene.stationary,
                        key if key == VK_F11 as u32 => window.toggle_fullscreen()?,
                        _ => {}
                    }
                }
            }

            let mut canvas = Canvas::new(&font, &mut vertices);
            let (width, height) = (renderer.width as f32, renderer.height as f32);
            renderer.clear(theme::BACKGROUND);
            if window.problems.is_empty() {
                scene.draw(&renderer)?;
                if show_text {
                    let rate = match frames.per_second {
                        Some(rate) => format!("{rate:.0}  ({:.2} ms per frame)", 1000.0 / rate),
                        None => "measuring".to_string(),
                    };
                    let focus = if window.focused() {
                        ("Window has focus".to_string(), SUCCESS)
                    } else {
                        ("Window doesn't have focus: input is ignored".to_string(), WARNING)
                    };
                    let car = if scene.stationary { "stationary" } else { "moving" };
                    let piece = |text: &str, color| (text.to_string(), color);
                    let mut lines = vec![
                        vec![
                            piece(NAME, TITLE),
                            piece("  3dTestScene_CppDx12's car, at 8 units and 90 degrees per second", DIM),
                        ],
                        vec![piece("FPS: ", DIM), piece(&rate, TEXT)],
                        vec![piece("GPU: ", DIM), piece(&description, TEXT)],
                        vec![piece("Display: ", DIM), piece(&window.display, TEXT)],
                        vec![piece("Car: ", DIM), piece(car, TEXT)],
                        vec![focus],
                        vec![
                            piece("F1", TEXT),
                            piece(" hide this   ", DIM),
                            piece("F11", TEXT),
                            piece(" borderless fullscreen   ", DIM),
                            piece("S", TEXT),
                            piece(" stationary car", DIM),
                        ],
                    ];
                    lines.extend(notices.iter().map(|notice| vec![piece(notice, WARNING)]));
                    // At the bottom left, which the car never crosses: it
                    // goes off the top left and bottom right corners.
                    let box_height = canvas.line_height() * lines.len() as f32 + PADDING * 2.0;
                    text_box(&mut canvas, MARGIN, height - MARGIN - box_height, &lines);
                }
            } else {
                draw_window_problems(&mut canvas, &window.supported, &window.problems, width);
            }
            overlay.draw(&renderer, canvas.vertices);
            renderer.present()?;
        }
        // What the frames used is dropped before the renderer.
        renderer.wait_for_gpu()
    }
}
