// SimplePaintDemo's parts, as a library, so that Test/SimplePaintDemo can
// test what input does to the editor, and where the cards go, without a
// window. Main.rs runs the program, which Demo.rs is.

#[path = "Cards.rs"]
pub mod cards;
#[path = "Demo.rs"]
pub mod demo;
#[path = "Editor.rs"]
pub mod editor;
#[path = "Preview.rs"]
pub mod preview;
