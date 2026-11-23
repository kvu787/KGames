// The pixel font: its glyphs, as Glyphs.aseprite at the root of the
// repository draws them in the 210 × 40 box at the top left of its canvas.
//
// The box has three rows of glyphs: the digits, then boxes, then the capital
// letters, then the small ones, 26 to a row. The font finds them in it,
// rather than at fixed places, so that a glyph can be moved a pixel without
// breaking it:
//
// - A row of glyphs is a run of the box's rows of pixels with ink in them,
//   and a glyph is a run of columns of pixels with ink in them, in its row.
// - Glyphs are in columns, and each column's left edge is its leftmost
//   glyph's. A narrow glyph, like i or l, keeps its place in its column, so
//   the font is monospaced.
// - A glyph is as wide as the widest glyph from its column's left edge, and
//   as tall as the tallest row: a row's top is the top of its tallest glyphs,
//   which for the small letters are those with ascenders, and the small
//   letters' row is taller by their descenders.
// - A character advances by a glyph's width and the narrowest gap between
//   two columns, and a line by a glyph's height and the narrowest gap between
//   two rows.
//
// Space is blank, and every other character without a glyph of its own,
// like punctuation and tab, has the box, the first one after the digits.

use std::path::{Path, PathBuf};

use aseprite::Canvas;

// The box at the top left of the canvas that has the glyphs.
pub const SHEET_WIDTH: u32 = 210;
pub const SHEET_HEIGHT: u32 = 40;

// The characters of each row's glyphs, left to right. The digits' row has
// boxes after them, which have no characters.
const ROWS: [&[u8]; 3] = [b"0123456789", b"ABCDEFGHIJKLMNOPQRSTUVWXYZ", b"abcdefghijklmnopqrstuvwxyz"];
const COLUMNS: usize = 26;
// The row and column of the box glyph.
const BOX: (usize, usize) = (0, 10);

// The characters that the font has glyphs for: ASCII's.
pub const CHARACTERS: usize = 128;

// Glyphs.aseprite at the root of the repository.
pub fn glyphs_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("Glyphs.aseprite")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Font {
    // A glyph's size, in the font's pixels.
    pub glyph_width: u32,
    pub glyph_height: u32,
    // How far a character moves the next one right, and a line the next one
    // down, in the font's pixels.
    pub advance: u32,
    pub line_height: u32,
    // Each ASCII character's glyph: its coverage, from 0 to 255, a row at a
    // time from the top.
    glyphs: Vec<Vec<u8>>,
    // Whether each character has a glyph of its own, rather than the box.
    own: Vec<bool>,
}

// A run of rows or columns, first and last.
#[derive(Clone, Copy, Debug)]
struct Run {
    first: u32,
    last: u32,
}

// The runs of trues.
fn runs(values: impl Iterator<Item = bool>) -> Vec<Run> {
    let mut runs = Vec::new();
    let mut start = None;
    let mut index = 0;
    for value in values {
        match (value, start) {
            (true, None) => start = Some(index),
            (false, Some(first)) => {
                runs.push(Run { first, last: index - 1 });
                start = None;
            }
            _ => {}
        }
        index += 1;
    }
    if let Some(first) = start {
        runs.push(Run { first, last: index - 1 });
    }
    runs
}

impl Font {
    // The font in the .aseprite file at the path.
    pub fn load(path: &Path) -> Result<Font, String> {
        let canvas = aseprite::load(path)?;
        Font::from_canvas(&canvas).map_err(|error| format!("{} has no font that can be read: {error}", path.display()))
    }

    // The font in the box at the top left of the canvas.
    pub fn from_canvas(canvas: &Canvas) -> Result<Font, String> {
        if canvas.width < SHEET_WIDTH || canvas.height < SHEET_HEIGHT {
            return Err(format!(
                "The canvas is {} × {} pixels, smaller than the {SHEET_WIDTH} × {SHEET_HEIGHT} box of glyphs at its top left.",
                canvas.width, canvas.height
            ));
        }
        let coverage = |x: u32, y: u32| if x < SHEET_WIDTH && y < SHEET_HEIGHT { canvas.pixel(x, y)[3] } else { 0 };
        let ink = |x: u32, y: u32| coverage(x, y) > 0;

        let rows = runs((0..SHEET_HEIGHT).map(|y| (0..SHEET_WIDTH).any(|x| ink(x, y))));
        if rows.len() != ROWS.len() {
            return Err(format!("The box of glyphs has {} rows of glyphs, not {}.", rows.len(), ROWS.len()));
        }
        let mut glyph_runs = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            let columns = runs((0..SHEET_WIDTH).map(|x| (row.first..=row.last).any(|y| ink(x, y))));
            if columns.len() != COLUMNS {
                return Err(format!("Row {} of the box of glyphs has {} glyphs, not {COLUMNS}.", index + 1, columns.len()));
            }
            glyph_runs.push(columns);
        }

        let lefts: Vec<u32> = (0..COLUMNS).map(|column| glyph_runs.iter().map(|row| row[column].first).min().unwrap()).collect();
        let glyph_width = glyph_runs.iter().flat_map(|row| row.iter().zip(&lefts).map(|(run, left)| run.last + 1 - left)).max().unwrap();
        let glyph_height = rows.iter().map(|row| row.last + 1 - row.first).max().unwrap();
        let mut column_gap = u32::MAX;
        for pair in lefts.windows(2) {
            let right = pair[0] + glyph_width;
            if pair[1] < right {
                return Err(format!("The glyphs at x = {} and x = {} are closer than a glyph's width, {glyph_width}.", pair[0], pair[1]));
            }
            column_gap = column_gap.min(pair[1] - right);
        }
        let row_gap = rows.windows(2).map(|pair| pair[1].first - pair[0].last - 1).min().unwrap();

        // A row's glyph, from its row's pixels only, as a glyph is taller
        // than the shorter rows, and would reach into the next.
        let glyph = |row: usize, column: usize| {
            let (left, band) = (lefts[column], rows[row]);
            let mut pixels = vec![0u8; (glyph_width * glyph_height) as usize];
            for y in 0..=(band.last - band.first) {
                for x in 0..glyph_width {
                    pixels[(y * glyph_width + x) as usize] = coverage(left + x, band.first + y);
                }
            }
            pixels
        };
        let mut glyphs = vec![glyph(BOX.0, BOX.1); CHARACTERS];
        let mut own = vec![false; CHARACTERS];
        glyphs[usize::from(b' ')] = vec![0; (glyph_width * glyph_height) as usize];
        own[usize::from(b' ')] = true;
        for (row, characters) in ROWS.iter().enumerate() {
            for (column, &character) in characters.iter().enumerate() {
                glyphs[usize::from(character)] = glyph(row, column);
                own[usize::from(character)] = true;
            }
        }
        Ok(Font {
            glyph_width,
            glyph_height,
            advance: glyph_width + column_gap,
            line_height: glyph_height + row_gap,
            glyphs,
            own,
        })
    }

    // A character's glyph: glyph_width × glyph_height coverage, from 0 to
    // 255, a row at a time from the top. Bytes past ASCII have the box.
    pub fn glyph(&self, character: u8) -> &[u8] {
        &self.glyphs[usize::from(character).min(CHARACTERS - 1)]
    }

    // Whether a character has a glyph of its own, rather than the box. Space
    // does: it's blank.
    pub fn has_own_glyph(&self, character: u8) -> bool {
        self.own.get(usize::from(character)).copied().unwrap_or(false)
    }

    // The coverage of a pixel of a character's glyph.
    pub fn coverage(&self, character: u8, x: u32, y: u32) -> u8 {
        self.glyph(character)[(y * self.glyph_width + x) as usize]
    }

    // Every glyph side by side in one row, character c's from x = c ×
    // glyph_width, as the renderer's texture has them: its width, height
    // and coverage.
    pub fn atlas(&self) -> (u32, u32, Vec<u8>) {
        let width = self.glyph_width * CHARACTERS as u32;
        let mut pixels = vec![0u8; (width * self.glyph_height) as usize];
        for (character, glyph) in self.glyphs.iter().enumerate() {
            for y in 0..self.glyph_height {
                let row = &glyph[(y * self.glyph_width) as usize..((y + 1) * self.glyph_width) as usize];
                let at = (y * width) as usize + character * self.glyph_width as usize;
                pixels[at..at + row.len()].copy_from_slice(row);
            }
        }
        (width, self.glyph_height, pixels)
    }
}
