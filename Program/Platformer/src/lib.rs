// The platformer's parts, as a library, so that Test/Platformer can test
// them without a window: the levels, the physics, the events and the game
// they drive, the progress that persists, and the program that
// Library/UserInterface runs. main.rs is the program.

pub mod cards;
pub mod controls;
pub mod events;
pub mod game;
pub mod level;
pub mod physics;
pub mod program;
pub mod progress;

pub const NAME: &str = "Platformer";
// The program's folder, whose Save.json holds the progress.
pub const FOLDER: &str = env!("CARGO_MANIFEST_DIR");
