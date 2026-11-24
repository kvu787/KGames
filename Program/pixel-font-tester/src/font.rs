// The pixel font: its glyphs, as Glyphs.aseprite, next to this crate's
// Cargo.toml, draws them, and Glyphs.txt, next to it, maps them to
// characters.
//
// Glyphs.aseprite's grid layer marks a cell for each glyph, and its glyphs
// layer draws them, each in its cell; its other layers, like the background,
// are left out. The font finds the cells in the grid, rather than at fixed
// places, so that the sheet can grow:
//
// - A row of cells is a run of the grid's rows of pixels with ink in them,
//   and a cell is a run of columns of pixels with ink in them, in its row,
//   from the first to the last of its rows with ink in those columns. So a
//   cell's outline and the lines inside it, like the baseline, are one cell.
//   A pixel has ink if it isn't wholly transparent.
// - The cells are all the same size, which is a glyph's: the font is
//   monospaced. The grid's lines are over the glyph's pixels, not around
//   them, and the glyph is every pixel of the glyphs layer in its cell, with
//   its alpha as its coverage.
// - A character advances by a glyph's width and the narrowest gap between two
//   cells side by side, and a line by a glyph's height and the narrowest gap
//   between two rows of cells.
//
// Glyphs.txt's lines are the grid's rows of cells, top to bottom, and each
// line's characters are its row's cells, left to right. Space has a cell,
// which is blank. Every character without one, like tab, has the box: the
// outline of a glyph's whole cell.

use std::path::{Path, PathBuf};

use aseprite::Canvas;

use crate::text::is_text;

// The layers of Glyphs.aseprite that the font reads.
pub const GRID_LAYER: &str = "grid";
pub const GLYPHS_LAYER: &str = "glyphs";

// The characters that the font has glyphs for: ASCII's.
pub const CHARACTERS: usize = 128;

// Glyphs.aseprite, which draws the font.
pub fn glyphs_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Glyphs.aseprite")
}

// Glyphs.txt, which maps its cells to characters.
pub fn mapping_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Glyphs.txt")
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
    // Whether each character has a cell of its own, rather than the box.
    own: Vec<bool>,
}

// A run of rows or columns, first and last.
#[derive(Clone, Copy, Debug)]
struct Run {
    first: u32,
    last: u32,
}

impl Run {
    fn length(self) -> u32 {
        self.last + 1 - self.first
    }
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

// A cell of the grid: its columns and rows.
#[derive(Clone, Copy, Debug)]
struct Cell {
    columns: Run,
    rows: Run,
}

// The characters of each row of cells: the mapping's lines, which have to be
// characters that the text can have, each once.
fn rows_of_characters(mapping: &str) -> Result<Vec<Vec<u8>>, String> {
    let mut mapped = [false; CHARACTERS];
    let mut rows = Vec::new();
    for (line, text) in mapping.lines().enumerate() {
        let mut row = Vec::new();
        for character in text.chars() {
            if !is_text(character) {
                return Err(format!(
                    "Line {} of the mapping has {character:?}, which isn't one of ASCII's printable characters or tab.",
                    line + 1
                ));
            }
            let byte = character as u8;
            if std::mem::replace(&mut mapped[usize::from(byte)], true) {
                return Err(format!("The mapping has {character:?} more than once."));
            }
            row.push(byte);
        }
        rows.push(row);
    }
    Ok(rows)
}

impl Font {
    // The font in the .aseprite file at the first path, mapped by the text
    // file at the second.
    pub fn load(glyphs: &Path, mapping: &Path) -> Result<Font, String> {
        let grid = aseprite::load_layer(glyphs, GRID_LAYER)?;
        let ink = aseprite::load_layer(glyphs, GLYPHS_LAYER)?;
        let text = std::fs::read_to_string(mapping).map_err(|error| format!("Couldn't read {}: {error}", mapping.display()))?;
        Font::from_sheet(&grid, &ink, &text)
            .map_err(|error| format!("{} and {} have no font that can be read: {error}", glyphs.display(), mapping.display()))
    }

    // The font in the cells of the grid layer, drawn by the glyphs layer,
    // which the mapping's lines map to characters.
    pub fn from_sheet(grid: &Canvas, glyphs: &Canvas, mapping: &str) -> Result<Font, String> {
        if (grid.width, grid.height) != (glyphs.width, glyphs.height) {
            return Err(format!(
                "The grid layer is {} × {} pixels, and the glyphs layer {} × {}.",
                grid.width, grid.height, glyphs.width, glyphs.height
            ));
        }
        let characters = rows_of_characters(mapping)?;
        let ink = |x: u32, y: u32| grid.pixel(x, y)[3] > 0;

        let bands = runs((0..grid.height).map(|y| (0..grid.width).any(|x| ink(x, y))));
        if bands.is_empty() {
            return Err("The grid has no cells.".to_string());
        }
        if bands.len() != characters.len() {
            return Err(format!("The grid has {} rows of cells, and the mapping {} lines.", bands.len(), characters.len()));
        }
        let mut cells: Vec<Vec<Cell>> = Vec::new();
        for (index, band) in bands.iter().enumerate() {
            let row: Vec<Cell> = runs((0..grid.width).map(|x| (band.first..=band.last).any(|y| ink(x, y))))
                .into_iter()
                .map(|columns| {
                    let rows = runs((band.first..=band.last).map(|y| (columns.first..=columns.last).any(|x| ink(x, y))));
                    let rows = Run {
                        first: band.first + rows[0].first,
                        last: band.first + rows[rows.len() - 1].last,
                    };
                    Cell { columns, rows }
                })
                .collect();
            if row.len() != characters[index].len() {
                return Err(format!(
                    "Row {} of the grid has {} cells, and line {} of the mapping {} characters.",
                    index + 1,
                    row.len(),
                    index + 1,
                    characters[index].len()
                ));
            }
            cells.push(row);
        }

        let first = cells[0][0];
        let (glyph_width, glyph_height) = (first.columns.length(), first.rows.length());
        for (row, cells) in cells.iter().enumerate() {
            for (column, cell) in cells.iter().enumerate() {
                let size = (cell.columns.length(), cell.rows.length());
                if size != (glyph_width, glyph_height) {
                    return Err(format!(
                        "The cells aren't all the same size: the first is {glyph_width} × {glyph_height} pixels, and cell {} of row {} is {} × {}.",
                        column + 1,
                        row + 1,
                        size.0,
                        size.1
                    ));
                }
            }
        }
        let column_gap = cells.iter().flat_map(|row| row.windows(2).map(|pair| pair[1].columns.first - pair[0].columns.last - 1)).min().unwrap_or(0);
        let row_gap = bands.windows(2).map(|pair| pair[1].first - pair[0].last - 1).min().unwrap_or(0);

        // Every pixel that the glyphs layer draws is in a cell.
        let mut in_cell = vec![false; (grid.width * grid.height) as usize];
        for cell in cells.iter().flatten() {
            for y in cell.rows.first..=cell.rows.last {
                for x in cell.columns.first..=cell.columns.last {
                    in_cell[(y * grid.width + x) as usize] = true;
                }
            }
        }
        if let Some(at) = (0..in_cell.len()).find(|&at| glyphs.pixels[at][3] > 0 && !in_cell[at]) {
            let (x, y) = (at as u32 % grid.width, at as u32 / grid.width);
            return Err(format!("The glyphs layer draws at x = {x}, y = {y}, outside every cell of the grid."));
        }

        let glyph = |cell: Cell| {
            let mut pixels = Vec::with_capacity((glyph_width * glyph_height) as usize);
            for y in cell.rows.first..=cell.rows.last {
                for x in cell.columns.first..=cell.columns.last {
                    pixels.push(glyphs.pixel(x, y)[3]);
                }
            }
            pixels
        };
        let the_box: Vec<u8> = (0..glyph_height)
            .flat_map(|y| (0..glyph_width).map(move |x| if x == 0 || y == 0 || x == glyph_width - 1 || y == glyph_height - 1 { 255 } else { 0 }))
            .collect();
        let mut font_glyphs = vec![the_box; CHARACTERS];
        let mut own = vec![false; CHARACTERS];
        for (row, characters) in characters.iter().enumerate() {
            for (column, &character) in characters.iter().enumerate() {
                font_glyphs[usize::from(character)] = glyph(cells[row][column]);
                own[usize::from(character)] = true;
            }
        }
        Ok(Font {
            glyph_width,
            glyph_height,
            advance: glyph_width + column_gap,
            line_height: glyph_height + row_gap,
            glyphs: font_glyphs,
            own,
        })
    }

    // The pixels between characters that the sheet has: the narrowest gap
    // between its cells.
    pub fn spacing(&self) -> u32 {
        self.advance - self.glyph_width
    }

    // A character's glyph: glyph_width × glyph_height coverage, from 0 to
    // 255, a row at a time from the top. Bytes past ASCII have the box.
    pub fn glyph(&self, character: u8) -> &[u8] {
        &self.glyphs[usize::from(character).min(CHARACTERS - 1)]
    }

    // Whether a character has a cell of its own, rather than the box. Space
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
