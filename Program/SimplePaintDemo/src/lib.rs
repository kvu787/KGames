// SimplePaintDemo's parts, as a library, so that Test/SimplePaintDemo can
// test what input does to the editor, and where the cards go, without a
// window. main.rs runs the program, which demo.rs is.

pub mod cards;
pub mod demo;
pub mod editor;
pub mod preview;
