// What the test's own key does. The card handles F1, and Library/UserInterface
// F11.

use user_interface::{Input, Key};

// Whether S went down this frame, which keeps the car in the middle, or lets
// it move. Holding it down repeats it, which does nothing more.
pub fn toggles_stationary(input: &Input) -> bool {
    input.key_pressed_once(Key::S)
}
