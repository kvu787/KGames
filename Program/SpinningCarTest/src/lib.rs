// SpinningCarTest's parts, as a library, so that Test/SpinningCarTest can
// test them without a window: where the car is, the keys, the card, the
// settings, and frames drawn offscreen. main.rs runs the program.

pub mod controls;
pub mod scene;
pub mod settings;
pub mod spinning_car;

use user_interface::Options;

// How Library/UserInterface runs the test.
pub fn options() -> Options {
    Options::new("SpinningCarTest", env!("CARGO_MANIFEST_DIR"))
}
