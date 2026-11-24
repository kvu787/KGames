// What the tests share: a sheet of glyphs made up to test how the font finds
// them, and the font that the tester's Glyphs.aseprite draws and Glyphs.txt
// maps.

#![allow(dead_code)]

use aseprite::Canvas;
use pixel_font_tester::font::{Font, glyphs_path, mapping_path};

// The made-up sheet's mapping: rows of cells of different lengths, like
// Glyphs.txt's, with space's blank cell in the last.
pub const MAPPING: &str = "0123\nABCDEFGH\n i.!\n";

// Its cells: 3 × 5 pixels, 2 apart, from x = 1, but those from the
// capitals' E on a pixel further right. Rows 2 apart, from y = 1, but the
// last a pixel further down.
pub const CELL_WIDTH: u32 = 3;
pub const CELL_HEIGHT: u32 = 5;
pub const ROW_TOPS: [u32; 3] = [1, 8, 16];

pub fn cell_left(row: u32, column: u32) -> u32 {
    1 + column * 5 + u32::from(row == 1 && column >= 4)
}

// The rows of the mapping, and their characters.
pub fn mapped() -> Vec<(u32, u32, u8)> {
    let mut mapped = Vec::new();
    for (row, line) in MAPPING.lines().enumerate() {
        for (column, character) in line.bytes().enumerate() {
            mapped.push((row as u32, column as u32, character));
        }
    }
    mapped
}

// Each glyph's pixels have an alpha of their own, so that a test can tell
// which glyph a character got: 1 + row × 10 + column.
pub fn alpha(row: u32, column: u32) -> u8 {
    (1 + row * 10 + column) as u8
}

const RED: [u8; 4] = [172, 50, 50, 255];

fn set(canvas: &mut Canvas, x: u32, y: u32, pixel: [u8; 4]) {
    let index = (y * canvas.width + x) as usize;
    canvas.pixels[index] = pixel;
}

// The made-up sheet's grid layer: each cell's outline, over its glyph's
// outermost pixels, like Glyphs.aseprite's, with a line across it, like its
// baseline.
pub fn grid() -> Canvas {
    let mut canvas = Canvas::new(45, 25);
    for (row, column, _) in mapped() {
        let (left, top) = (cell_left(row, column), ROW_TOPS[row as usize]);
        for x in 0..CELL_WIDTH {
            for y in 0..CELL_HEIGHT {
                if x == 0 || x == CELL_WIDTH - 1 || y == 0 || y == 3 || y == CELL_HEIGHT - 1 {
                    set(&mut canvas, left + x, top + y, RED);
                }
            }
        }
    }
    canvas
}

// The made-up sheet's glyphs layer: every glyph fills its cell, but space,
// which is blank, and i, which is one pixel wide, in its cell's middle.
pub fn glyphs() -> Canvas {
    let mut canvas = Canvas::new(45, 25);
    for (row, column, character) in mapped() {
        let (left, top) = (cell_left(row, column), ROW_TOPS[row as usize]);
        for x in 0..CELL_WIDTH {
            for y in 0..CELL_HEIGHT {
                let ink = match character {
                    b' ' => false,
                    b'i' => x == 1,
                    _ => true,
                };
                if ink {
                    set(&mut canvas, left + x, top + y, [0, 0, 0, alpha(row, column)]);
                }
            }
        }
    }
    canvas
}

pub fn sheet_font() -> Font {
    Font::from_sheet(&grid(), &glyphs(), MAPPING).unwrap()
}

// The font that the tester shows.
pub fn real_font() -> Font {
    Font::load(&glyphs_path(), &mapping_path()).unwrap()
}
