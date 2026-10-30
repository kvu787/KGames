// SimplePaintDemo's parts, as a library, so that Test/SimplePaintDemo can
// test what input does to the editor, and where the panels go, without a
// window. Main.rs is the program.

#[path = "Editor.rs"]
pub mod editor;
#[path = "Panels.rs"]
pub mod panels;
#[path = "Preview.rs"]
pub mod preview;

pub const NAME: &str = "SimplePaint Demo";
// The em height of the font, in pixels. Programs only support 100% scaling
// (see Library/Application), so this never changes.
pub const FONT_SIZE: i32 = 15;
