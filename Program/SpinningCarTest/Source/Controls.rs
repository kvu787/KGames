// What the test's own key does. Library/SceneTest handles F1 and F11.

use application::Event;

const S: u32 = 'S' as u32;

// S keeps the car in the middle, or lets it move. Holding it down repeats it,
// which does nothing more.
pub fn toggles_stationary(event: &Event) -> bool {
    matches!(event, Event::Key { key: S, repeat: false, .. })
}
