// SpinningCarTest's parts, as a library, so that Test/SpinningCarTest can
// test them without a window: where the car is, the keys, the card, the
// settings, and frames drawn offscreen. Main.rs runs the program.

#[path = "Controls.rs"]
pub mod controls;
#[path = "Depth.rs"]
pub mod depth;
#[path = "InfoCard.rs"]
pub mod info_card;
#[path = "Scene.rs"]
pub mod scene;
#[path = "Settings.rs"]
pub mod settings;
#[path = "SpinningCar.rs"]
pub mod spinning_car;

use user_interface::Options;

// How Library/UserInterface runs the test.
pub fn options() -> Options {
    Options::new("SpinningCarTest", env!("CARGO_MANIFEST_DIR"))
}
