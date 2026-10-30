// Direct3D 12 for KGames: the renderer that every program's frames go
// through, helpers for the resources that programs make for their draws,
// and decoding images for textures.

#[path = "Image.rs"]
mod image;
#[path = "Renderer.rs"]
mod renderer;
#[path = "Resources.rs"]
mod resources;

pub use image::*;
pub use renderer::*;
pub use resources::*;
