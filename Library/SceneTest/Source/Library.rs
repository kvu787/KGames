// What MinimalSceneTest and SpinningCarTest share: a 3D scene that fills the
// window at either supported height, drawn with the lowest latency, and a
// text box over its bottom left corner with the test's name, its frame rate,
// the GPU, the display, and whether the window has focus. F1 hides the text
// box, and F11 switches between maximized and borderless fullscreen. Each
// test gives `run` its scene, as a Scene, which adds to the text box and
// handles the test's own keys.

use std::path::Path;

use application::{Arguments, Event, FrameCounter, ProgramSettings, Settings, Window};
use canvas::theme::{DIM, SUCCESS, TEXT, TITLE, WARNING};
use canvas::{Canvas, Color, Font, MARGIN, PADDING, draw_window_problems, text_box, theme};
use graphics::Renderer;
use json::Value;
use windows_core::Result;

// The em height of the font, in pixels. Programs only support 100% scaling
// (see Library/Application), so this never changes.
pub const FONT_SIZE: i32 = 15;

// A line of the text box, as pieces of text, each in its color.
pub type Line = Vec<(String, Color)>;

// What a test has of its own: its scene, what the text box says about it,
// and its keys.
pub trait Scene {
    // What the scene shows, after the test's name, like "A sphere moving at
    // 8 units per second".
    fn description(&self) -> &str;

    // Lines of the text box after the display's, like what a key set.
    fn lines(&self) -> Vec<Line> {
        Vec::new()
    }

    // The test's own keys, each with what it does, after F1's and F11's.
    fn shortcuts(&self) -> &[(&'static str, &'static str)] {
        &[]
    }

    // An input event that F1 and F11 don't take, like the test's own keys.
    fn input(&mut self, _event: &Event) {}

    // The window was minimized, and the loop slept until it was restored.
    fn pause(&mut self) {}

    // Moves the scene to a frame's time: seconds since the window opened, or
    // a script's fixed clock. Every frame comes here, even when the window
    // is in an unsupported setup and the scene isn't drawn.
    fn advance(&mut self, seconds: f64);

    // Records the scene's draws into the frame.
    unsafe fn draw(&self, renderer: &Renderer) -> Result<()>;

    // The scene's part of the state that scripts save and check.
    fn state(&self) -> Vec<(&'static str, Value)>;
}

// What the keys that every test has do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    // F1: shows or hides the text box.
    ToggleText,
    // F11: switches between maximized and borderless fullscreen.
    ToggleFullscreen,
}

const F1: u32 = 0x70;
const F11: u32 = 0x7A;

// Holding a key down repeats it, which does nothing more.
pub fn action(event: &Event) -> Option<Action> {
    match event {
        Event::Key { key: F1, repeat: false, .. } => Some(Action::ToggleText),
        Event::Key { key: F11, repeat: false, .. } => Some(Action::ToggleFullscreen),
        _ => None,
    }
}

// Runs the test called `name`, with the settings file at `settings_path`:
// opens its window, makes its scene with `make_scene`, and draws frames until
// the window closes, or a script ends.
pub fn run<S: ProgramSettings, T: Scene>(
    name: &str,
    settings_path: &Path,
    make_scene: impl FnOnce(&mut Renderer, &S) -> Result<T>,
) -> Result<()> {
    unsafe {
        application::prepare_process()?;
        // Problems with the settings and the command line, which the text
        // box lists.
        let mut notices = Vec::new();
        let arguments = Arguments::parse(&mut notices);
        let (settings, scene_settings) = Settings::for_program::<S>(settings_path, &arguments, &mut notices);
        let mut window = Window::open(name, &settings, &arguments, &mut notices)?;

        let font = Font::new(FONT_SIZE)?;
        let mut renderer = Renderer::new(window.handle, settings.allow_tearing, true)?;
        let overlay = canvas::Pipeline::new(&mut renderer, &font)?;
        let mut scene = make_scene(&mut renderer, &scene_settings)?;
        // The title bar says how frames are presented, as it affects latency.
        let description = renderer.description();
        window.set_title(&format!("{name} - {description}"))?;
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
                // spinning.
                window.wait_for_message()?;
                frames.pause();
                scene.pause();
                continue;
            }

            // Spin until the GPU and DXGI can take a frame, like GameInputDemo,
            // which explains why. The loop keeps handling window messages
            // through the wait.
            if !renderer.is_ready()? {
                std::hint::spin_loop();
                continue;
            }
            // The frame's setup, before the time and input are read.
            renderer.begin_frame()?;

            // Read the time and input only now, right before drawing, so the
            // frame shows the newest.
            let frame = window.begin_frame();
            frames.frame(frame.seconds);
            for event in &frame.events {
                match action(event) {
                    Some(Action::ToggleText) => show_text = !show_text,
                    Some(Action::ToggleFullscreen) => window.toggle_fullscreen()?,
                    None => scene.input(event),
                }
            }
            scene.advance(frame.seconds);

            let mut canvas = Canvas::new(&font, &mut vertices);
            let (width, height) = (renderer.width as f32, renderer.height as f32);
            renderer.clear(theme::BACKGROUND);
            if window.problems.is_empty() {
                scene.draw(&renderer)?;
                if show_text {
                    let lines = text(name, &scene, &frames, frame.focused, &description, &window.display, &notices);
                    // At the bottom left, which the tests' objects never
                    // cross: they go off the top left and bottom right
                    // corners.
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
                    let mut state = scene.state();
                    state.push(("TextShown", show_text.into()));
                    Value::Object(state.into_iter().map(|(name, value)| (name.to_string(), value)).collect())
                },
            )?;
            renderer.present()?;
        }
        // What the frames used is dropped before the renderer.
        renderer.wait_for_gpu()?;
        window.finish()
    }
}

// The text box's lines.
fn text(
    name: &str,
    scene: &impl Scene,
    frames: &FrameCounter,
    focused: bool,
    adapter: &str,
    display: &str,
    notices: &[String],
) -> Vec<Line> {
    let piece = |text: &str, color| (text.to_string(), color);
    let rate = match frames.per_second {
        Some(rate) => format!("{rate:.0}  ({:.2} ms per frame)", 1000.0 / rate),
        None => "measuring".to_string(),
    };
    let focus = if focused {
        piece("Window has focus", SUCCESS)
    } else {
        piece("Window doesn't have focus: input is ignored", WARNING)
    };
    let mut lines = vec![
        vec![piece(name, TITLE), piece(&format!("  {}", scene.description()), DIM)],
        vec![piece("FPS: ", DIM), piece(&rate, TEXT)],
        vec![piece("GPU: ", DIM), piece(adapter, TEXT)],
        vec![piece("Display: ", DIM), piece(display, TEXT)],
    ];
    lines.extend(scene.lines());
    lines.push(vec![focus]);
    let mut shortcuts = Line::new();
    for (key, does) in [("F1", "hide this"), ("F11", "borderless fullscreen")]
        .iter()
        .chain(scene.shortcuts())
    {
        if !shortcuts.is_empty() {
            shortcuts.push(piece("   ", DIM));
        }
        shortcuts.push(piece(key, TEXT));
        shortcuts.push(piece(&format!(" {does}"), DIM));
    }
    lines.push(shortcuts);
    lines.extend(notices.iter().map(|notice| vec![piece(notice, WARNING)]));
    lines
}
