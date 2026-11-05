// UiShowcase's parts, as a library, so that Test/UiShowcase can test what
// input does to its widgets, and where its panels go, without a window.
// Main.rs is the program.

#[path = "Panels.rs"]
pub mod panels;
#[path = "Showcase.rs"]
pub mod showcase;

pub const NAME: &str = "UI Showcase";
