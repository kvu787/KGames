// SimplePaintDemo's parts, as a library, so that Test/SimplePaintDemo can
// test what input does to the editor, and where the panels go, without a
// window. Main.rs runs the program, which Demo.rs is.

#[path = "Demo.rs"]
pub mod demo;
#[path = "Editor.rs"]
pub mod editor;
#[path = "Panels.rs"]
pub mod panels;
#[path = "Preview.rs"]
pub mod preview;

pub const NAME: &str = "SimplePaint Demo";
