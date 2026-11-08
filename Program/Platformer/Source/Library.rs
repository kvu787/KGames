// The platformer's parts, as a library, so that Test/Platformer can test
// them without a window: the levels, the physics, the events and the game
// they drive, the progress that persists, and the program that
// Library/UserInterface runs. Main.rs is the program.

#[path = "Cards.rs"]
pub mod cards;
#[path = "Controls.rs"]
pub mod controls;
#[path = "Events.rs"]
pub mod events;
#[path = "Game.rs"]
pub mod game;
#[path = "Level.rs"]
pub mod level;
#[path = "Physics.rs"]
pub mod physics;
#[path = "Program.rs"]
pub mod program;
#[path = "Progress.rs"]
pub mod progress;

pub const NAME: &str = "Platformer";
// The program's folder, whose Save.json holds the progress.
pub const FOLDER: &str = env!("CARGO_MANIFEST_DIR");
