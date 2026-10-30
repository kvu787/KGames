// SpinningCarTest's parts, as a library, so that Test/SpinningCarTest can
// test the car's placement, the keys, the settings and the car mesh without
// a window. Main.rs is the program.

#[path = "CarMesh.rs"]
pub mod car_mesh;
#[path = "Controls.rs"]
pub mod controls;
#[path = "Scene.rs"]
pub mod scene;
#[path = "Settings.rs"]
pub mod settings;

pub const NAME: &str = "Spinning Car Test";
// The em height of the font, in pixels. Programs only support 100% scaling
// (see Library/Application), so this never changes.
pub const FONT_SIZE: i32 = 15;
