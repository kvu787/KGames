// 2D drawing for KGames' user interfaces: a Canvas that builds a frame's
// rounded boxes, shadows, lines and text as instances that the pipeline draws
// in one draw call, the fonts of Fluent 2's type ramp and its icons, the light
// and dark themes, and rectangles, stacks and lines of text in fixed pixels.
// Library/UserInterface builds cards and widgets on it.

#[path = "Canvas.rs"]
mod canvas;
#[path = "Fonts.rs"]
mod fonts;
#[path = "Layout.rs"]
mod layout;
#[path = "Pipeline.rs"]
mod pipeline;
#[path = "Theme.rs"]
pub mod theme;

pub use canvas::{Cached, Canvas, Color, Instance};
pub use fonts::{ELLIPSIS, Fonts, ICON_SIZES, Icon, TextStyle};
pub use layout::{Align, Axis, Lines, Rect, Stack};
pub use pipeline::Pipeline;
pub use theme::Theme;
