// What the test's keys do.

use application::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    // F1: shows or hides the text over the scene.
    ToggleText,
    // F11: switches between borderless fullscreen and maximized.
    ToggleFullscreen,
}

const F1: u32 = 0x70;
const F11: u32 = 0x7A;

pub fn action(event: &Event) -> Option<Action> {
    match event {
        Event::Key { key: F1, .. } => Some(Action::ToggleText),
        Event::Key { key: F11, .. } => Some(Action::ToggleFullscreen),
        _ => None,
    }
}
