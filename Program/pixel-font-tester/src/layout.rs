// Where things are in the window, in its pixels, which are the display's:
// the text box, a little bigger than half the window each way, in its middle,
// and the text in it, from its top left, left to right and top to bottom.
//
// The GUI is pixel perfect, like Aseprite's: everything is made of whole
// pixels of its own, each an integer number of the display's across and
// down, with nothing between pixels, no antialiasing, and no filtering.
// There are two kinds of those pixels:
//
// - The GUI's own, GUI_SCALE × GUI_SCALE of the display's, like Aseprite's
//   screen scaling: the box, its border and padding, and what's wrong with
//   an unsupported setup, are laid out in them, so they're on the GUI's grid.
// - The text's, `scale` × `scale` of the display's, which Ctrl and a number
//   set: the font's pixels, the spacing between characters, which Shift and
//   a number sets, and the caret, one of them wide.

use crate::font::Font;
use crate::text::Position;

// The client area of the one supported setup: borderless fullscreen on a
// 2560 × 1440 display.
pub const WINDOW_WIDTH: i32 = 2560;
pub const WINDOW_HEIGHT: i32 = 1440;

// How many of the display's pixels each of the GUI's pixels is, across and
// down. The window is 1280 × 720 of them.
pub const GUI_SCALE: i32 = 2;

// The box, with its border: 960 × 540 of the GUI's pixels, in the middle of
// the window.
pub const BOX: Rect = Rect::new(160, 90, 1120, 630).scaled(GUI_SCALE);
pub const BORDER: i32 = GUI_SCALE;
// Between the border and the text.
pub const PADDING: i32 = 8 * GUI_SCALE;
// Where what's wrong with an unsupported setup starts, from the window's
// top left.
pub const PROBLEMS_MARGIN: i32 = 16 * GUI_SCALE;

// Pixels from left and top, in, to right and bottom, out.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        Rect { left, top, right, bottom }
    }

    pub const fn width(self) -> i32 {
        self.right - self.left
    }

    pub const fn height(self) -> i32 {
        self.bottom - self.top
    }

    pub fn is_empty(self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }

    pub fn intersection(self, other: Rect) -> Rect {
        Rect::new(self.left.max(other.left), self.top.max(other.top), self.right.min(other.right), self.bottom.min(other.bottom))
    }

    pub const fn inset(self, by: i32) -> Rect {
        Rect::new(self.left + by, self.top + by, self.right - by, self.bottom - by)
    }

    // The rectangle with each of its pixels `scale` × `scale` pixels.
    pub const fn scaled(self, scale: i32) -> Rect {
        Rect::new(self.left * scale, self.top * scale, self.right * scale, self.bottom * scale)
    }

    pub fn contains(self, x: i32, y: i32) -> bool {
        (self.left..self.right).contains(&x) && (self.top..self.bottom).contains(&y)
    }
}

// The box inside its border.
pub const fn field() -> Rect {
    BOX.inset(BORDER)
}

// Where the text goes, inside the padding. Glyphs are cut at its edges.
pub const fn text_area() -> Rect {
    field().inset(PADDING)
}

// How the text is set, which Ctrl and Shift with a number change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    // How many of the display's pixels each of the font's pixels is, across
    // and down.
    pub scale: u32,
    // How many of the font's pixels are between characters.
    pub spacing: u32,
}

impl Style {
    // How far a character moves the next one right, in the display's pixels.
    pub fn advance(self, font: &Font) -> u32 {
        (font.glyph_width + self.spacing) * self.scale
    }

    // How far a line moves the next one down, in the display's pixels.
    pub fn line_height(self, font: &Font) -> u32 {
        font.line_height * self.scale
    }
}

// How many columns the text area shows whole: at least one.
pub fn visible_columns(font: &Font, style: Style) -> usize {
    fit(text_area().width(), font.glyph_width * style.scale, style.advance(font))
}

// How many lines the text area shows whole: at least one.
pub fn visible_lines(font: &Font, style: Style) -> usize {
    fit(text_area().height(), font.glyph_height * style.scale, style.line_height(font))
}

// How many things `size` long, `step` apart, fit in `room`.
fn fit(room: i32, size: u32, step: u32) -> usize {
    if room < size as i32 { 1 } else { ((room - size as i32) / step as i32 + 1) as usize }
}

// The top left pixel of the glyph at the position, with the text scrolled to
// show `scroll` at the text area's top left.
pub fn glyph_origin(font: &Font, style: Style, scroll: Position, position: Position) -> (i32, i32) {
    let area = text_area();
    let column = position.column as i32 - scroll.column as i32;
    let line = position.line as i32 - scroll.line as i32;
    (area.left + column * style.advance(font) as i32, area.top + line * style.line_height(font) as i32)
}

// The caret before the position: one of the font's pixels wide, as tall as
// a glyph, in the gap before the glyph there, in its middle, or right of it
// when that's two pixels apart. Without a gap, it's over the last column of
// the glyph before.
pub fn caret(font: &Font, style: Style, scroll: Position, position: Position) -> Rect {
    let (x, y) = glyph_origin(font, style, scroll, position);
    let scale = style.scale;
    let left = x - (style.spacing.div_ceil(2).max(1) * scale) as i32;
    Rect::new(left, y, left + scale as i32, y + (font.glyph_height * scale) as i32)
}
