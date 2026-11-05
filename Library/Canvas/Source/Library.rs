// 2D drawing for KGames' user interfaces: a Canvas that builds a frame's
// rectangles, antialiased lines and circles, and text as one triangle list,
// the pipeline that draws it in one draw call, the light theme's colors,
// and rectangles and lines of text in fixed pixels. Library/UserInterface
// builds panels and widgets on it.

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
