// What MinimalSceneTest and SpinningCarTest share: a 3D scene that fills the
// window at either supported height, drawn with the lowest latency, and a
// card over its bottom left corner with the test's name, its frame rate,
// the GPU, the display, and whether the window has focus. F1 hides the card,
// and F11 switches between maximized and borderless fullscreen. Each test
// gives `run` its scene, as a Scene, which adds rows to the card and handles
// the test's own keys. Library/UserInterface runs it, with the scene under
// the card.

use std::path::Path;

use application::{Event, ProgramSettings};
use graphics::Renderer;
use json::Value;
use user_interface::{
    Align, CARD_PADDING, Frame, Icon, MARGIN, Program, Rect, SPACE_M, SPACE_S, Setup, TextStyle, badge_width,
    keycap_width,
};
use windows_core::Result;

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

    // Rows of the card after the display's, each a label and a value, like
    // what a key set.
    fn rows(&self) -> Vec<(&'static str, String)> {
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
    // F1: shows or hides the card.
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
            card(&self.scene, frame);
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

// The label column's width, and a row's height.
const LABEL_WIDTH: f32 = 72.0;
const ROW: f32 = 20.0;
const KEYCAP_ROW: f32 = 28.0;

// The card over the scene, at the bottom left, which the tests' objects
// never cross: they go off the top left and bottom right corners. It's as
// big as what it says.
fn card<T: Scene>(scene: &T, frame: &mut Frame) {
    let theme = frame.theme();
    let fonts = frame.fonts();
    let status = frame.status;
    let rate = match status.frames_per_second {
        Some(rate) => format!("{rate:.0}  ({:.2} ms per frame)", 1000.0 / rate),
        None => "measuring".to_string(),
    };
    let mut rows = vec![("FPS", rate), ("GPU", status.adapter.to_string()), ("Display", status.display.to_string())];
    rows.extend(scene.rows());
    let shortcuts: Vec<(&str, &str)> = [("F1", "hide this"), ("F11", "borderless fullscreen")]
        .into_iter()
        .chain(scene.shortcuts().iter().copied())
        .collect();
    let (focus, focus_color) = if frame.focused() {
        ("Window has focus", theme.success)
    } else {
        ("No focus: input is ignored", theme.caution)
    };

    // As wide as its widest line.
    let title_width = fonts.measure(T::NAME, TextStyle::BodyStrong) + SPACE_M + fonts.measure(scene.description(), TextStyle::Body);
    let rows_width = rows
        .iter()
        .map(|(_, value)| LABEL_WIDTH + fonts.measure(value, TextStyle::Body))
        .fold(0.0, f32::max);
    let shortcuts_width: f32 = shortcuts
        .iter()
        .map(|(key, does)| keycap_width(fonts, key) + 6.0 + fonts.measure(does, TextStyle::Caption) + SPACE_M)
        .sum();
    let notices_width = status
        .notices
        .iter()
        .map(|notice| 24.0 + fonts.measure(notice, TextStyle::Body))
        .fold(0.0, f32::max);
    let content_width = title_width
        .max(rows_width)
        .max(shortcuts_width)
        .max(badge_width(fonts, focus))
        .max(notices_width)
        .min(frame.width - MARGIN * 2.0 - CARD_PADDING * 2.0)
        .ceil();
    let content_height = ROW + SPACE_S + ROW * rows.len() as f32 + SPACE_S + KEYCAP_ROW + KEYCAP_ROW
        + ROW * status.notices.len() as f32;
    let area = Rect::new(
        MARGIN,
        frame.height - MARGIN - content_height - CARD_PADDING * 2.0,
        content_width + CARD_PADDING * 2.0,
        content_height + CARD_PADDING * 2.0,
    );
    frame.overlay_card(area);

    let inner = area.inset(CARD_PADDING);
    let canvas = &mut frame.canvas;
    canvas.push_clip(inner);
    let mut y = inner.y;
    let end = canvas.text([inner.x, y], T::NAME, TextStyle::BodyStrong, theme.text);
    canvas.text([end + SPACE_M, y], scene.description(), TextStyle::Body, theme.text_secondary);
    y += ROW + SPACE_S;
    for (label, value) in &rows {
        canvas.text([inner.x, y], label, TextStyle::Body, theme.text_secondary);
        canvas.text([inner.x + LABEL_WIDTH, y], value, TextStyle::Body, theme.text);
        y += ROW;
    }
    y += SPACE_S;
    canvas.pop_clip();
    frame.badge([inner.x, y + KEYCAP_ROW * 0.5], focus, focus_color);
    y += KEYCAP_ROW;
    let mut x = inner.x;
    let middle = y + KEYCAP_ROW * 0.5;
    for (key, does) in &shortcuts {
        x = frame.keycap([x, middle], key).right() + 6.0;
        let text = Rect::new(x, y, inner.right() - x, KEYCAP_ROW);
        x = frame.label(text, does, TextStyle::Caption, theme.text_secondary, Align::Start) + SPACE_M;
    }
    y += KEYCAP_ROW;
    for notice in status.notices {
        frame.canvas.icon([inner.x + 8.0, y + ROW * 0.5], Icon::Warning, 16.0, theme.caution);
        let text = Rect::new(inner.x + 24.0, y, inner.width - 24.0, ROW);
        frame.label(text, notice, TextStyle::Body, theme.text, Align::Start);
        y += ROW;
    }
}
