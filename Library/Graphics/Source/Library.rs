// Direct3D 12 for KGames: the renderer that every program's frames go
// through, and helpers for the resources that programs make for their draws.

#[path = "Renderer.rs"]
mod renderer;
#[path = "Resources.rs"]
mod resources;

pub use renderer::*;
pub use resources::*;
