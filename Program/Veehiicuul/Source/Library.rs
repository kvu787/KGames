// Veehiicuul's parts, as a library, so that Test/Veehiicuul can test them
// without a window: how a car drives, what it touches, the tracks and the
// vehicles from their files, the ground that a track's outlines make, the
// controls, and the game that they make up.

#[path = "Camera.rs"]
pub mod camera;
#[path = "Collision.rs"]
pub mod collision;
#[path = "Controls.rs"]
pub mod controls;
#[path = "Driving.rs"]
pub mod driving;
#[path = "Game.rs"]
pub mod game;
#[path = "Ground.rs"]
pub mod ground;
#[path = "Settings.rs"]
pub mod settings;
#[path = "Track.rs"]
pub mod track;
#[path = "Vehicles.rs"]
pub mod vehicles;

pub const NAME: &str = "Veehiicuul";
