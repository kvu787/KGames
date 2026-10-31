// 2D drawing for KGames' user interfaces: a Canvas that builds a frame's
// rectangles, antialiased lines and circles, and text as one triangle list,
// the pipeline that draws it in one draw call, the light theme's colors, and
// helpers for laying out panels in fixed pixels.

#[path = "Canvas.rs"]
mod canvas;
#[path = "Font.rs"]
mod font;
#[path = "Layout.rs"]
mod layout;
#[path = "Pipeline.rs"]
mod pipeline;
#[path = "Theme.rs"]
pub mod theme;

pub use canvas::{Cached, Canvas, Color, Vertex};
pub use font::Font;
pub use layout::*;
pub use pipeline::Pipeline;
