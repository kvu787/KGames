// What the test's keys do.

use application::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    // F1: shows or hides the text over the scene.
    ToggleText,
    // F11: switches between maximized and borderless fullscreen.
    ToggleFullscreen,
    // S, as in the C++ scene: keeps the car in the middle, or lets it move.
    ToggleStationary,
}

const F1: u32 = 0x70;
const F11: u32 = 0x7A;
const S: u32 = 'S' as u32;

pub fn action(event: &Event) -> Option<Action> {
    match event {
        Event::Key { key: F1, .. } => Some(Action::ToggleText),
        Event::Key { key: F11, .. } => Some(Action::ToggleFullscreen),
        Event::Key { key: S, .. } => Some(Action::ToggleStationary),
        _ => None,
    }
}
