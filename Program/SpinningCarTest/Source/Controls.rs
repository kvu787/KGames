// What the test's keys do.

use application::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    // F1: shows or hides the text over the scene.
    ToggleText,
    // F11: switches between maximized and borderless fullscreen.
    ToggleFullscreen,
    // S: keeps the car in the middle, or lets it move.
    ToggleStationary,
}

const F1: u32 = 0x70;
const F11: u32 = 0x7A;
const S: u32 = 'S' as u32;

// Holding a key down repeats it, which does nothing more.
pub fn action(event: &Event) -> Option<Action> {
    match event {
        Event::Key { key: F1, repeat: false, .. } => Some(Action::ToggleText),
        Event::Key { key: F11, repeat: false, .. } => Some(Action::ToggleFullscreen),
        Event::Key { key: S, repeat: false, .. } => Some(Action::ToggleStationary),
        _ => None,
    }
}
