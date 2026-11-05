// Veehiicuul's parts, as a library, so that Test/Veehiicuul can test them
// without a window: the scenes, with their terrain and tracks, the models
// and vehicles, how a car drives and what it touches, and the game that
// plays them. Main.rs is the program.

#[path = "Camera.rs"]
pub mod camera;
#[path = "Collision.rs"]
pub mod collision;
#[path = "Controls.rs"]
pub mod controls;
#[path = "Curve.rs"]
pub mod curve;
#[path = "Document.rs"]
pub mod document;
#[path = "Drawing.rs"]
pub mod drawing;
#[path = "Driving.rs"]
pub mod driving;
#[path = "Files.rs"]
pub mod files;
#[path = "Game.rs"]
pub mod game;
#[path = "Ground.rs"]
pub mod ground;
#[path = "Models.rs"]
pub mod models;
#[path = "Program.rs"]
pub mod program;
#[path = "Scene.rs"]
pub mod scene;
#[path = "Settings.rs"]
pub mod settings;
#[path = "Terrain.rs"]
pub mod terrain;
#[path = "Track.rs"]
pub mod track;
#[path = "Vehicles.rs"]
pub mod vehicles;

pub const NAME: &str = "Veehiicuul";
