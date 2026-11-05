// Draws frames of an interface without a window or a GPU, for tests: the
// test makes up each frame's input, like a script would, and the interface
// lays out and draws on a canvas as in the program, so that a press lands
// where it would on the screen. A program's tests drive it with
// `Headless::program`, which handles a frame like the main loop does.

use std::sync::OnceLock;

use application::Event;
use canvas::{Font, Rect};
use windows_api::*;
use windows_core::Result;

use crate::panels::FONT_SIZE;
use crate::{Frame, Interface, Program, Status};

// The client areas of the two window setups that programs support.
pub const MAXIMIZED: [f32; 2] = [2560.0, 1369.0];
pub const FULLSCREEN: [f32; 2] = [2560.0, 1440.0];

// The font, made once for all of a test binary's tests.
pub fn font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| Font::new(FONT_SIZE).expect("the font is installed"))
}

// What a program's frame did, besides what it drew.
#[derive(Clone, Copy, Debug, Default)]
pub struct Outcome {
    // Lines of text that didn't fit where they went.
    pub dropped_lines: usize,
    // Whether it asked to switch to or from borderless fullscreen.
    pub fullscreen: bool,
}

pub struct Headless {
    pub interface: Interface,
    // The next frame's number, and its time, at 60 frames a second.
    pub number: u64,
    pub cursor: Option<[f32; 2]>,
    // Whether the window has focus. Without it, frames get no input.
    pub focused: bool,
    pub size: [f32; 2],
    pub notices: Vec<String>,
}

impl Default for Headless {
    fn default() -> Self {
        Self::new(MAXIMIZED)
    }
}

impl Headless {
    // An interface in a window of this size, which has focus.
    pub fn new(size: [f32; 2]) -> Self {
        Self {
            interface: Interface::default(),
            number: 0,
            cursor: None,
            focused: true,
            size,
            notices: Vec::new(),
        }
    }

    // Draws a frame with these events, and returns what `draw` does. Like
    // in a script, a press, release or turn of the wheel moves the cursor
    // to it.
    pub fn frame<R>(&mut self, events: &[Event], draw: impl FnOnce(&mut Frame) -> R) -> R {
        for event in events {
            if let Event::Press(point) | Event::Release(point) | Event::Wheel(point, _) = event {
                self.cursor = Some(*point);
            }
        }
        // Without focus, the window only gives the release of a press that
        // it got with focus.
        let events = events
            .iter()
            .filter(|event| self.focused || matches!(event, Event::Release(_)))
            .copied()
            .collect();
        let input = application::Frame {
            number: self.number,
            seconds: self.number as f64 / 60.0,
            focused: self.focused,
            events,
            cursor: if self.focused { self.cursor } else { None },
            program_steps: Vec::new(),
            checkpoints: Vec::new(),
        };
        self.number += 1;
        let status = Status {
            notices: &self.notices,
            ..Status::HEADLESS
        };
        let mut frame = self.interface.frame(font(), &input, status, self.size);
        draw(&mut frame)
    }

    // Draws a frame of a program, with these events, as the main loop does.
    pub fn program<P: Program>(&mut self, program: &mut P, events: &[Event]) -> Result<Outcome> {
        self.frame(events, |frame| {
            if P::FULLSCREEN_KEY && frame.shortcut(VK_F11) {
                frame.toggle_fullscreen();
            }
            program.frame(frame)?;
            Ok(Outcome {
                dropped_lines: frame.dropped_lines(),
                fullscreen: frame.fullscreen_requested(),
            })
        })
    }

    // Moves the cursor, for the next frame.
    pub fn move_to(&mut self, point: [f32; 2]) {
        self.cursor = Some(point);
    }

    // Where the last frame's widget of this name is. Panics, with the names
    // there are, if it didn't draw one.
    pub fn target(&self, name: &str) -> Rect {
        self.interface
            .target(name)
            .unwrap_or_else(|| panic!("no {name} in {:?}", self.interface.target_names()))
    }

    pub fn center(&self, name: &str) -> [f32; 2] {
        self.target(name).center()
    }
}

// A key going down, without Control or Shift.
pub fn key(key: i32) -> Event {
    Event::Key {
        key: key as u32,
        control: false,
        shift: false,
        repeat: false,
    }
}

pub fn shifted(key: i32) -> Event {
    Event::Key {
        key: key as u32,
        control: false,
        shift: true,
        repeat: false,
    }
}

pub fn control(key: i32) -> Event {
    Event::Key {
        key: key as u32,
        control: true,
        shift: false,
        repeat: false,
    }
}

// The characters of `text`, typed.
pub fn typed(text: &str) -> Vec<Event> {
    text.chars().map(Event::Character).collect()
}

// A press and its release at a point.
pub fn click(point: [f32; 2]) -> Vec<Event> {
    vec![Event::Press(point), Event::Release(point)]
}
