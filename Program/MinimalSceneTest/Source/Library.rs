// MinimalSceneTest's parts, as a library, so that Test/MinimalSceneTest can
// test the sphere's motion, the keys and the settings without a window.
// Main.rs is the program.

#[path = "Controls.rs"]
pub mod controls;
#[path = "Scene.rs"]
pub mod scene;
#[path = "Settings.rs"]
pub mod settings;

pub const NAME: &str = "Minimal Scene Test";
// The em height of the font, in pixels. Programs only support 100% scaling
// (see Library/Application), so this never changes.
pub const FONT_SIZE: i32 = 15;
