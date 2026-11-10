// Veehiicuul's parts, as a library, so that Test/Veehiicuul can test them
// without a window: the scenes, with their terrain and tracks, the models
// and vehicles, how a car drives and what it touches, the editors that make
// them, and the game that plays them. main.rs is the program.

pub mod camera;
pub mod collision;
pub mod controls;
pub mod curve;
pub mod document;
pub mod drawing;
pub mod driving;
pub mod editing;
pub mod files;
pub mod game;
pub mod ground;
pub mod models;
pub mod program;
pub mod scene;
pub mod scene_editor;
pub mod settings;
pub mod terrain;
pub mod terrain_editor;
pub mod track;
pub mod track_editor;
pub mod vehicle_editor;
pub mod vehicles;

pub const NAME: &str = "Veehiicuul";
// The game's folder, which it reads its files from: its scenes, vehicles,
// models and settings.
pub const FOLDER: &str = env!("CARGO_MANIFEST_DIR");
