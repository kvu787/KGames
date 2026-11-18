// What the test's own key does. The card handles F2, and Library/user-interface
// F9 and F10.

use user_interface::{Input, Key};

// Whether S went down in this step, which keeps the car in the middle, or
// lets it move.
pub fn toggles_stationary(input: &Input) -> bool {
    input.key_pressed(Key::S)
}
