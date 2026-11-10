// GameInputDemo's parts, as a library, so that Test/GameInputDemo can test
// what the demo makes of readings, and what input does to its cards, without
// a window, and without GameInput. main.rs is the program.

pub mod cards;
pub mod demo;
pub mod device_tree;
pub mod devices;
pub mod game_input_source;
pub mod input;
pub mod scripted_source;
pub mod settings;

pub const NAME: &str = "GameInput Demo";
