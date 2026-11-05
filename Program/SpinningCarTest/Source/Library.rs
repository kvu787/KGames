// SpinningCarTest's parts, as a library, so that Test/SpinningCarTest can
// test the car's placement, its key, the settings and the car mesh without
// a window. Main.rs is the program, which Library/SceneTest runs.

#[path = "Controls.rs"]
pub mod controls;
#[path = "Scene.rs"]
pub mod scene;
#[path = "Settings.rs"]
pub mod settings;

pub const NAME: &str = "Spinning Car Test";
