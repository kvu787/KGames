// PixelFontTester: a box to type ASCII into, which shows it in the pixel font
// that Glyphs.aseprite draws and Glyphs.txt maps, on the display's pixel
// grid, at 1 to 10 times its size. See README.md.
//
// It's built on Win32 and Direct3D 12 directly, with Library/graphics, not on
// Library/user-interface, and its text is the font's alone, without
// DirectWrite. Its parts are here, which Test/pixel-font-tester tests without
// a window; main.rs runs them.

pub mod arguments;
pub mod font;
pub mod layout;
pub mod renderer;
pub mod scene;
pub mod tester;
pub mod text;
pub mod window;
