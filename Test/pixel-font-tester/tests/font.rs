// How the font finds its glyphs in the grid's cells, which characters the
// mapping gives which, and the font that Glyphs.aseprite draws and Glyphs.txt
// maps.

mod common;

use aseprite::Canvas;
use common::{CELL_HEIGHT, CELL_WIDTH, MAPPING, ROW_TOPS, alpha, cell_left, glyphs, grid, mapped, real_font, sheet_font, the_box};
use pixel_font_tester::font::{CHARACTERS, Font};

fn clear(canvas: &mut Canvas, x: u32, y: u32) {
    let index = (y * canvas.width + x) as usize;
    canvas.pixels[index] = [0; 4];
}

#[test]
fn the_metrics_come_from_the_cells_and_the_gaps_between_them() {
    let font = sheet_font();
    assert_eq!((font.glyph_width, font.glyph_height), (CELL_WIDTH, CELL_HEIGHT));
    // The narrowest gaps, 2, though the gap before the capitals' E is 3,
    // and so is the one before the last row.
    assert_eq!(cell_left(1, 4) - cell_left(1, 3) - CELL_WIDTH, 3);
    assert_eq!(ROW_TOPS[2] - ROW_TOPS[1] - CELL_HEIGHT, 3);
    assert_eq!(font.advance, CELL_WIDTH + 2);
    assert_eq!(font.line_height, CELL_HEIGHT + 2);
    assert_eq!(font.spacing(), 2);
}

#[test]
fn each_line_of_the_mapping_is_a_row_of_cells_and_each_character_a_cell() {
    let font = sheet_font();
    for (row, column, character) in mapped() {
        assert!(font.has_own_glyph(character), "{}", character as char);
        let glyph = font.glyph(character);
        match character {
            b' ' => assert!(glyph.iter().all(|&coverage| coverage == 0), "space isn't blank"),
            // One pixel wide, in its cell's middle.
            b'i' => {
                for y in 0..CELL_HEIGHT {
                    let row_of_pixels: Vec<u8> = (0..CELL_WIDTH).map(|x| font.coverage(b'i', x, y)).collect();
                    assert_eq!(row_of_pixels, [0, alpha(row, column), 0], "row {y}");
                }
            }
            // Every pixel of its cell, those under the grid's lines too.
            _ => assert!(glyph.iter().all(|&coverage| coverage == alpha(row, column)), "{}", character as char),
        }
    }
}

#[test]
fn every_character_without_a_cell_has_the_box() {
    let font = sheet_font();
    let the_box = the_box(&font);
    let mapped: Vec<u8> = MAPPING.bytes().filter(|&byte| byte != b'\n').collect();
    for character in (0..CHARACTERS as u8).filter(|character| !mapped.contains(character)) {
        assert_eq!(font.glyph(character), the_box, "{character}");
        assert!(!font.has_own_glyph(character), "{character}");
    }
    // Bytes past ASCII, which the tester never has, get it too.
    assert_eq!(font.glyph(200), the_box);
    assert!(!font.has_own_glyph(200));
}

#[test]
fn the_atlas_has_every_glyph_side_by_side() {
    let font = sheet_font();
    let (width, height, pixels) = font.atlas();
    assert_eq!((width, height), (font.glyph_width * CHARACTERS as u32, font.glyph_height));
    for character in [b' ', b'0', b'E', b'i', b'!', b'z', b'\t', 127] {
        for y in 0..height {
            for x in 0..font.glyph_width {
                let at = (y * width + u32::from(character) * font.glyph_width + x) as usize;
                assert_eq!(pixels[at], font.coverage(character, x, y), "{character} at {x}, {y}");
            }
        }
    }
}

#[test]
fn a_mapping_that_doesn_t_fit_the_grid_says_why() {
    let error = |mapping: &str| Font::from_sheet(&grid(), &glyphs(), mapping).unwrap_err();
    let message = error("0123\nABCDEFGH\n");
    assert!(message.contains("3 rows") && message.contains("2 lines"), "{message}");
    let message = error("012\nABCDEFGH\n i.!\n");
    assert!(message.contains("Row 1") && message.contains("4 cells") && message.contains("3 characters"), "{message}");
    let message = error("0120\nABCDEFGH\n i.!\n");
    assert!(message.contains("'0' more than once"), "{message}");
    let message = error("012é\nABCDEFGH\n i.!\n");
    assert!(message.contains("Line 1") && message.contains("'é'"), "{message}");
    let message = error("");
    assert!(message.contains("3 rows") && message.contains("0 lines"), "{message}");
}

#[test]
fn a_sheet_it_can_t_read_says_why() {
    // The capitals' A, a pixel narrower: its right edge is gone.
    let mut narrow = grid();
    for y in ROW_TOPS[1]..ROW_TOPS[1] + CELL_HEIGHT {
        clear(&mut narrow, cell_left(1, 0) + CELL_WIDTH - 1, y);
    }
    let message = Font::from_sheet(&narrow, &glyphs(), MAPPING).unwrap_err();
    assert!(message.contains("same size") && message.contains("cell 1 of row 2 is 2 × 5"), "{message}");

    // The glyphs layer draws between two cells.
    let mut outside = glyphs();
    let index = (ROW_TOPS[0] * outside.width + cell_left(0, 0) + CELL_WIDTH) as usize;
    outside.pixels[index] = [0, 0, 0, 255];
    let message = Font::from_sheet(&grid(), &outside, MAPPING).unwrap_err();
    assert!(message.contains(&format!("x = {}, y = {}", cell_left(0, 0) + CELL_WIDTH, ROW_TOPS[0])), "{message}");

    let message = Font::from_sheet(&Canvas::new(45, 25), &Canvas::new(45, 25), MAPPING).unwrap_err();
    assert!(message.contains("no cells"), "{message}");
    let message = Font::from_sheet(&grid(), &Canvas::new(44, 25), MAPPING).unwrap_err();
    assert!(message.contains("44 × 25"), "{message}");
}

// What these check holds however the glyphs in Glyphs.aseprite are drawn,
// as long as Glyphs.txt maps every one of ASCII's printable characters.
#[test]
fn glyphs_aseprite_has_a_glyph_for_every_printable_character() {
    let font = real_font();
    assert!(font.advance > font.glyph_width && font.line_height > font.glyph_height);
    let the_box = the_box(&font);
    assert!(font.glyph(b' ').iter().all(|&coverage| coverage == 0), "space isn't blank");
    for character in b'!'..=b'~' {
        let glyph = font.glyph(character);
        assert!(font.has_own_glyph(character), "{} has no cell", character as char);
        assert!(glyph.iter().any(|&coverage| coverage > 0), "{} is blank", character as char);
        assert_ne!(glyph, the_box, "{} is the box", character as char);
    }
    assert_eq!(font.glyph(b'\t'), the_box);
}
