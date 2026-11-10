// MinimalSceneTest's parts, as a library, so that Test/MinimalSceneTest can
// test them without a window: the sphere's motion, the keys, the card, the
// settings, and frames drawn offscreen. main.rs runs the program.

pub mod moving_sphere;
pub mod scene;
pub mod settings;

use user_interface::Options;

// How Library/UserInterface runs the test.
pub fn options() -> Options {
    Options::new("MinimalSceneTest", env!("CARGO_MANIFEST_DIR"))
}
