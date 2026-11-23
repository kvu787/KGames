// What the tests share: a sheet of glyphs made up to test how the font finds
// them, and the font that Glyphs.aseprite at the root draws.

#![allow(dead_code)]

use aseprite::Canvas;
use pixel_font_tester::font::{Font, SHEET_HEIGHT, SHEET_WIDTH, glyphs_path};

// The made-up sheet's glyphs: 3 pixels wide, 2 apart, from x = 1, but those
// from column 13 on a pixel further right, like Glyphs.aseprite's from column
// 7 on. Rows of 5, 5 and 7 pixels, 2 apart, from y = 1.
pub const GLYPH_WIDTH: u32 = 3;
pub const ROW_TOPS: [u32; 3] = [1, 8, 15];
pub const ROW_HEIGHTS: [u32; 3] = [5, 5, 7];

pub fn column_left(column: u32) -> u32 {
    1 + column * 5 + u32::from(column >= 13)
}

// Each glyph's pixels have an alpha of their own, so that a test can tell
// which glyph a character got: 1 + row × 26 + column.
pub fn alpha(row: u32, column: u32) -> u8 {
    (1 + row * 26 + column) as u8
}

// The made-up sheet, at the top left of a canvas a little bigger than it. The
// i, in the small letters' row, is one pixel wide, in its column's middle.
pub fn sheet() -> Canvas {
    let mut canvas = Canvas::new(SHEET_WIDTH + 20, SHEET_HEIGHT + 20);
    for row in 0..3 {
        for column in 0..26 {
            let narrow = row == 2 && column == 8;
            for y in ROW_TOPS[row as usize]..ROW_TOPS[row as usize] + ROW_HEIGHTS[row as usize] {
                for x in 0..GLYPH_WIDTH {
                    if !narrow || x == 1 {
                        let index = (y * canvas.width + column_left(column) + x) as usize;
                        canvas.pixels[index] = [0, 0, 0, alpha(row, column)];
                    }
                }
            }
        }
    }
    // Outside the box, which the font never reads.
    let outside = (SHEET_HEIGHT + 5) * canvas.width + 5;
    canvas.pixels[outside as usize] = [0, 0, 0, 255];
    canvas
}

pub fn sheet_font() -> Font {
    Font::from_canvas(&sheet()).unwrap()
}

// The font that the tester shows.
pub fn real_font() -> Font {
    Font::load(&glyphs_path()).unwrap()
}
