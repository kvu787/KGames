// How the font finds its glyphs in the box at the top left of a sheet, which
// characters get which, and the font that Glyphs.aseprite draws.

mod common;

use aseprite::Canvas;
use common::{GLYPH_WIDTH, ROW_HEIGHTS, alpha, column_left, real_font, sheet, sheet_font};
use pixel_font_tester::font::{CHARACTERS, Font, SHEET_WIDTH};

// The alpha of every pixel with ink in a character's glyph, or none if it's
// blank.
fn inks(font: &Font, character: u8) -> Vec<u8> {
    let mut inks: Vec<u8> = font.glyph(character).iter().copied().filter(|&coverage| coverage > 0).collect();
    inks.dedup();
    inks
}

#[test]
fn the_metrics_come_from_the_glyphs_and_the_gaps_between_them() {
    let font = sheet_font();
    assert_eq!(font.glyph_width, GLYPH_WIDTH);
    // The small letters' row is the tallest.
    assert_eq!(font.glyph_height, 7);
    // The narrowest gaps, 2, though the gap before column 13 is 3.
    assert_eq!(font.advance, GLYPH_WIDTH + 2);
    assert_eq!(font.line_height, 7 + 2);
}

#[test]
fn digits_and_letters_get_the_glyphs_in_their_places() {
    let font = sheet_font();
    for (row, characters) in [b"0123456789".as_slice(), b"ABCDEFGHIJKLMNOPQRSTUVWXYZ", b"abcdefghijklmnopqrstuvwxyz"].iter().enumerate() {
        for (column, &character) in characters.iter().enumerate() {
            assert_eq!(inks(&font, character), [alpha(row as u32, column as u32)], "{}", character as char);
            assert!(font.has_own_glyph(character), "{}", character as char);
        }
    }
}

#[test]
fn a_column_a_pixel_further_right_is_read_where_it_is() {
    let font = sheet_font();
    // N is column 13 of the capitals: whole, though a pixel right of the
    // grid of the columns before it.
    assert!(font.glyph(b'N')[..GLYPH_WIDTH as usize].iter().all(|&coverage| coverage == alpha(1, 13)));
    assert_eq!(column_left(13) - column_left(12), 6);
}

#[test]
fn a_narrow_glyph_keeps_its_place_in_its_column() {
    let font = sheet_font();
    for y in 0..ROW_HEIGHTS[2] {
        let row: Vec<u8> = (0..GLYPH_WIDTH).map(|x| font.coverage(b'i', x, y)).collect();
        assert_eq!(row, [0, alpha(2, 8), 0], "row {y}");
    }
}

#[test]
fn a_shorter_row_s_glyphs_don_t_reach_into_the_next_row() {
    let font = sheet_font();
    for y in 0..font.glyph_height {
        let expected = if y < ROW_HEIGHTS[0] { alpha(0, 0) } else { 0 };
        assert!((0..GLYPH_WIDTH).all(|x| font.coverage(b'0', x, y) == expected), "row {y}");
    }
}

#[test]
fn space_is_blank_and_every_other_character_without_a_glyph_has_the_box() {
    let font = sheet_font();
    assert!(font.glyph(b' ').iter().all(|&coverage| coverage == 0));
    assert!(font.has_own_glyph(b' '));
    let the_box = alpha(0, 10);
    for character in (0..CHARACTERS as u8).filter(|character| !character.is_ascii_alphanumeric() && *character != b' ') {
        assert_eq!(inks(&font, character), [the_box], "{character}");
        assert!(!font.has_own_glyph(character), "{character}");
    }
    // Bytes past ASCII, which the tester never has, get it too.
    assert_eq!(inks(&font, 200), [the_box]);
}

#[test]
fn the_atlas_has_every_glyph_side_by_side() {
    let font = sheet_font();
    let (width, height, pixels) = font.atlas();
    assert_eq!((width, height), (font.glyph_width * CHARACTERS as u32, font.glyph_height));
    for character in [b' ', b'0', b'A', b'i', b'z', b'~', 127] {
        for y in 0..height {
            for x in 0..font.glyph_width {
                let at = (y * width + u32::from(character) * font.glyph_width + x) as usize;
                assert_eq!(pixels[at], font.coverage(character, x, y), "{character} at {x}, {y}");
            }
        }
    }
}

#[test]
fn a_sheet_it_can_t_read_says_why() {
    let mut missing = sheet();
    // The capitals' last glyph.
    for y in 8..13 {
        for x in column_left(25)..column_left(25) + GLYPH_WIDTH {
            missing.pixels[(y * missing.width + x) as usize] = [0; 4];
        }
    }
    let error = Font::from_canvas(&missing).unwrap_err();
    assert!(error.contains("Row 2") && error.contains("25 glyphs"), "{error}");

    let mut extra_row = sheet();
    let at = (30 * extra_row.width + 3) as usize;
    extra_row.pixels[at] = [0, 0, 0, 255];
    let error = Font::from_canvas(&extra_row).unwrap_err();
    assert!(error.contains("4 rows"), "{error}");

    let error = Font::from_canvas(&Canvas::new(SHEET_WIDTH - 1, 100)).unwrap_err();
    assert!(error.contains("smaller"), "{error}");
}

// What these check holds however the glyphs in Glyphs.aseprite are drawn,
// as long as the box has its three rows of 26.
#[test]
fn glyphs_aseprite_has_a_font() {
    let font = real_font();
    assert!(font.advance > font.glyph_width && font.line_height > font.glyph_height);
    let the_box = font.glyph(b'!').to_vec();
    assert!(the_box.iter().any(|&coverage| coverage > 0));
    for character in (b'0'..=b'9').chain(b'A'..=b'Z').chain(b'a'..=b'z') {
        let glyph = font.glyph(character);
        assert!(glyph.iter().any(|&coverage| coverage > 0), "{} is blank", character as char);
        assert_ne!(glyph, the_box, "{} is the box", character as char);
    }
}
