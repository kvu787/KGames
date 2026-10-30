// GameInputDemo's parts, as a library, so that Test/GameInputDemo can test
// what the demo makes of readings, and where clicks go, without a window,
// and without GameInput. Main.rs is the program.

#[path = "Devices.rs"]
pub mod devices;
#[path = "GameInputSource.rs"]
pub mod game_input_source;
#[path = "Input.rs"]
pub mod input;
#[path = "Panels.rs"]
pub mod panels;
#[path = "ScriptedSource.rs"]
pub mod scripted_source;
#[path = "Settings.rs"]
pub mod settings;

pub const NAME: &str = "GameInput Demo";
// The em height of the font, in pixels. Programs only support 100% scaling
// (see Library/Application), so this never changes.
pub const FONT_SIZE: i32 = 15;
