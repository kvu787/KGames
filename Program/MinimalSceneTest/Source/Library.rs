// MinimalSceneTest's parts, as a library, so that Test/MinimalSceneTest can
// test them without a window: the sphere's motion, the keys, the card, the
// settings, and frames drawn offscreen. Main.rs runs the program.

#[path = "Depth.rs"]
pub mod depth;
#[path = "InfoCard.rs"]
pub mod info_card;
#[path = "MovingSphere.rs"]
pub mod moving_sphere;
#[path = "Scene.rs"]
pub mod scene;
#[path = "Settings.rs"]
pub mod settings;

use user_interface::{Options, WindowMode};

// How Library/UserInterface runs the test: unlike most programs, it opens
// borderless fullscreen unless Settings.json says otherwise.
pub fn options() -> Options {
    Options {
        window_mode: WindowMode::BorderlessFullscreen,
        ..Options::new("MinimalSceneTest", env!("CARGO_MANIFEST_DIR"))
    }
}
