// What MinimalSceneTest and SpinningCarTest share: a 3D scene that fills the
// window at either supported height, drawn with the lowest latency, and a
// text box over its bottom left corner with the test's name, its frame rate,
// the GPU, the display, and whether the window has focus. F1 hides the text
// box, and F11 switches between maximized and borderless fullscreen. Each
// test gives `run` its scene, as a Scene, which adds to the text box and
// handles the test's own keys. Library/UserInterface runs it, with the
// scene under the text box.

use std::path::Path;

use application::{Event, ProgramSettings};
use graphics::Renderer;
use json::Value;
use user_interface::theme::{DIM, SUCCESS, TEXT, TITLE, WARNING};
use user_interface::{Color, Frame, MARGIN, PADDING, Program, Setup};
use windows_core::Result;

// A line of the text box, as pieces of text, each in its color.
pub type Line = Vec<(String, Color)>;

// What a test has of its own: its scene, what the text box says about it,
// and its keys.
pub trait Scene: Sized {
    // The test's own settings, in its Settings.json.
    type Settings: ProgramSettings;

    // The test's name, for its window's title and the text box.
    const NAME: &'static str;

    // Makes the scene, with the renderer, which has a depth buffer.
    unsafe fn new(renderer: &mut Renderer, settings: &Self::Settings) -> Result<Self>;

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

// A scene test, as a program that Library/UserInterface runs.
pub struct SceneTest<T> {
    scene: T,
    show_text: bool,
}

impl<T: Scene> Program for SceneTest<T> {
    type Settings = T::Settings;
    const NAME: &'static str = T::NAME;
    const DEPTH: bool = true;
    // F11 is one of the keys that `action` reads.
    const FULLSCREEN_KEY: bool = false;

    fn new(setup: Setup<'_, T::Settings>) -> Result<Self> {
        Ok(SceneTest {
            scene: unsafe { T::new(setup.renderer, &setup.program_settings)? },
            show_text: true,
        })
    }

    // The scene has no widgets, so every event is the program's.
    fn frame(&mut self, frame: &mut Frame) -> Result<()> {
        let events: Vec<Event> = frame.program_events().copied().collect();
        for event in &events {
            match action(event) {
                Some(Action::ToggleText) => self.show_text = !self.show_text,
                Some(Action::ToggleFullscreen) => frame.toggle_fullscreen(),
                None => self.scene.input(event),
            }
        }
        self.scene.advance(frame.seconds());
        if self.show_text {
            let lines = text(&self.scene, frame);
            // At the bottom left, which the tests' objects never cross: they
            // go off the top left and bottom right corners.
            let box_height = frame.line_height() * lines.len() as f32 + PADDING * 2.0;
            let y = frame.height - MARGIN - box_height;
            frame.text_box(MARGIN, y, &lines);
        }
        Ok(())
    }

    unsafe fn draw_under(&mut self, renderer: &Renderer) -> Result<()> {
        unsafe { self.scene.draw(renderer) }
    }

    fn state(&self) -> Value {
        let mut state = self.scene.state();
        state.push(("TextShown", self.show_text.into()));
        Value::Object(state.into_iter().map(|(name, value)| (name.to_string(), value)).collect())
    }

    fn minimized(&mut self) {
        self.scene.pause();
    }
}

// Runs the test whose scene is T, from its folder, which has its
// Settings.json: opens its window, makes its scene, and draws frames until
// the window closes, or a script ends.
pub fn run<T: Scene>(folder: &Path) -> Result<()> {
    user_interface::run::<SceneTest<T>>(folder)
}

// The text box's lines.
fn text<T: Scene>(scene: &T, frame: &Frame) -> Vec<Line> {
    let piece = |text: &str, color| (text.to_string(), color);
    let status = frame.status;
    let rate = match status.frames_per_second {
        Some(rate) => format!("{rate:.0}  ({:.2} ms per frame)", 1000.0 / rate),
        None => "measuring".to_string(),
    };
    let focus = if frame.focused() {
        piece("Window has focus", SUCCESS)
    } else {
        piece("Window doesn't have focus: input is ignored", WARNING)
    };
    let mut lines = vec![
        vec![piece(T::NAME, TITLE), piece(&format!("  {}", scene.description()), DIM)],
        vec![piece("FPS: ", DIM), piece(&rate, TEXT)],
        vec![piece("GPU: ", DIM), piece(status.adapter, TEXT)],
        vec![piece("Display: ", DIM), piece(status.display, TEXT)],
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
    lines.extend(status.notices.iter().map(|notice| vec![piece(notice, WARNING)]));
    lines
}
