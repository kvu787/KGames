// Where the box, the text and the caret are in the window, in its pixels:
// the box in the middle, the text left to right and top to bottom from its
// top left, cut at its edges, and every quad on whole pixels at every size.

mod common;

use common::real_font;
use pixel_font_tester::font::Font;
use pixel_font_tester::layout::{self, BOX, GUI_SCALE, Rect, Style, WINDOW_HEIGHT, WINDOW_WIDTH};
use pixel_font_tester::scene::{Quad, SOLID, Scene, pack, scene};
use pixel_font_tester::tester::{Input, Key, Modifiers, Tester, Theme};
use pixel_font_tester::text::{Position, Text};

fn tester(text: &str, scale: u32, font: &Font) -> Tester {
    Tester::new(Text::typed(text), scale, Theme::Light, true, font)
}

fn glyph_quads(scene: &Scene) -> Vec<Quad> {
    scene.quads.iter().copied().filter(|quad| quad.glyph != SOLID).collect()
}

#[test]
fn the_box_is_in_the_middle_of_the_window() {
    let left_margin = BOX.left;
    let right_margin = WINDOW_WIDTH - BOX.right;
    let top_margin = BOX.top;
    let bottom_margin = WINDOW_HEIGHT - BOX.bottom;
    assert_eq!((left_margin, top_margin), (right_margin, bottom_margin));
    assert!(BOX.width() > WINDOW_WIDTH / 2 && BOX.height() > WINDOW_HEIGHT / 2);
    let area = layout::text_area();
    assert!(BOX.inset(layout::BORDER) == layout::field() && area.left > BOX.left && area.top > BOX.top);
}

// The GUI is pixel perfect, like Aseprite's: what isn't the text is made of
// the GUI's own pixels, GUI_SCALE × GUI_SCALE of the display's, on their
// grid, and so is the text's place in the box.
#[test]
fn the_box_and_the_setup_s_problems_are_on_the_gui_s_pixel_grid() {
    let on_grid = |rect: Rect| [rect.left, rect.top, rect.right, rect.bottom].iter().all(|value| value % GUI_SCALE == 0);
    const {
        assert!(GUI_SCALE > 1);
        assert!(WINDOW_WIDTH % GUI_SCALE == 0 && WINDOW_HEIGHT % GUI_SCALE == 0);
        assert!(layout::BORDER >= GUI_SCALE && layout::BORDER % GUI_SCALE == 0);
    }
    for rect in [BOX, layout::field(), layout::text_area()] {
        assert!(on_grid(rect), "{rect:?}");
    }

    let font = real_font();
    let tester = tester("", 3, &font);
    let problems = ["The display is 1920 x 1080 pixels.".to_string(), "Second".to_string()];
    for quad in &scene(&tester, &font, &problems).quads {
        assert_eq!(quad.scale, GUI_SCALE as u32);
        assert!(quad.origin.iter().all(|value| value % GUI_SCALE == 0), "{quad:?}");
    }
}

#[test]
fn text_flows_left_to_right_and_top_to_bottom_from_the_box_s_top_left() {
    let font = real_font();
    for scale in [1, 2, 3, 10] {
        let tester = tester("ab\ncd", scale, &font);
        let quads = glyph_quads(&scene(&tester, &font, &[]));
        let characters: Vec<u32> = quads.iter().map(|quad| quad.glyph).collect();
        assert_eq!(characters, b"abcd".map(u32::from), "scale {scale}");
        let area = layout::text_area();
        let advance = (font.advance * scale) as i32;
        let line = (font.line_height * scale) as i32;
        let origins: Vec<[i32; 2]> = quads.iter().map(|quad| quad.origin).collect();
        assert_eq!(
            origins,
            [[area.left, area.top], [area.left + advance, area.top], [area.left, area.top + line], [area.left + advance, area.top + line]],
            "scale {scale}"
        );
        for quad in &quads {
            assert_eq!(quad.scale, scale);
            let size = [(font.glyph_width * scale) as i32, (font.glyph_height * scale) as i32];
            assert_eq!(quad.area, Rect::new(quad.origin[0], quad.origin[1], quad.origin[0] + size[0], quad.origin[1] + size[1]));
        }
    }
}

#[test]
fn spaces_advance_without_a_quad_and_characters_without_glyphs_draw_the_box() {
    let font = real_font();
    let tester = tester("a !\t", 1, &font);
    let quads = glyph_quads(&scene(&tester, &font, &[]));
    let characters: Vec<u32> = quads.iter().map(|quad| quad.glyph).collect();
    // The renderer's atlas has the box for ! and tab.
    assert_eq!(characters, b"a!\t".map(u32::from));
    assert_eq!(quads[1].origin[0] - quads[0].origin[0], 2 * font.advance as i32);
}

#[test]
fn the_box_draws_under_the_text_and_the_caret_over_it_in_the_theme_s_colors() {
    let font = real_font();
    for theme in [Theme::Light, Theme::Dark] {
        let mut tester = tester("a", 2, &font);
        tester.theme = theme;
        let colors = theme.colors();
        let scene = scene(&tester, &font, &[]);
        assert_eq!(scene.background, colors.background);
        let first = scene.quads[0];
        assert_eq!((first.area, first.glyph, first.color), (BOX, SOLID, pack(colors.border)));
        let second = scene.quads[1];
        assert_eq!((second.area, second.glyph, second.color), (layout::field(), SOLID, pack(colors.field)));
        assert_eq!(scene.quads[2].color, pack(colors.text));
        let caret = *scene.quads.last().unwrap();
        assert_eq!((caret.glyph, caret.color), (SOLID, pack(colors.text)));
    }
}

#[test]
fn the_caret_is_one_of_the_font_s_pixels_wide_in_the_gap_before_its_character() {
    let font = real_font();
    for scale in [1, 2, 3, 10] {
        let mut tester = tester("abc", scale, &font);
        tester.handle(Input::Key(Key::Left, Modifiers::NONE), &font);
        let scene = scene(&tester, &font, &[]);
        let quads = glyph_quads(&scene);
        let caret = scene.quads.last().unwrap().area;
        let c = quads[2];
        let b_right = quads[1].area.right;
        assert_eq!(caret.width(), scale as i32);
        assert_eq!(caret.right, c.origin[0], "scale {scale}");
        assert!(caret.left >= b_right, "scale {scale}: the caret covers b");
        assert_eq!((caret.top, caret.height()), (c.origin[1], (font.glyph_height * scale) as i32));
    }
}

#[test]
fn spacing_puts_that_many_of_the_font_s_pixels_between_characters_at_every_size() {
    let font = real_font();
    for scale in [1, 2, 7] {
        for spacing in 0..=9 {
            let mut tester = tester("abc\nd", scale, &font);
            tester.set_spacing(spacing, &font);
            let quads = glyph_quads(&scene(&tester, &font, &[]));
            // Each glyph's right edge, then the spacing, then the next one.
            assert_eq!(quads[1].area.left - quads[0].area.right, (spacing * scale) as i32, "scale {scale}, spacing {spacing}");
            assert_eq!(quads[2].area.left - quads[1].area.right, (spacing * scale) as i32, "scale {scale}, spacing {spacing}");
            // Lines are as far apart as ever.
            assert_eq!(quads[3].origin[1] - quads[0].origin[1], (font.line_height * scale) as i32);
            assert_eq!(quads[3].origin[0], quads[0].origin[0]);
        }
    }
}

#[test]
fn the_caret_is_in_the_gap_and_without_one_over_the_last_column_before_it() {
    let font = real_font();
    for scale in [1, 3] {
        for spacing in 0..=9 {
            let mut tester = tester("ab", scale, &font);
            tester.set_spacing(spacing, &font);
            tester.handle(Input::Key(Key::Left, Modifiers::NONE), &font);
            let scene = scene(&tester, &font, &[]);
            let quads = glyph_quads(&scene);
            let caret = scene.quads.last().unwrap().area;
            let (a, b) = (quads[0].area, quads[1].area);
            assert_eq!(caret.width(), scale as i32);
            if spacing == 0 {
                assert_eq!((caret.left, caret.right), (a.right - scale as i32, b.left), "scale {scale}");
            } else {
                assert!(caret.left >= a.right && caret.right <= b.left, "scale {scale}, spacing {spacing}: {caret:?} isn't in the gap");
                // In its middle, or right of it.
                let (left_room, right_room) = (caret.left - a.right, b.left - caret.right);
                assert!(right_room <= left_room && left_room - right_room <= scale as i32, "scale {scale}, spacing {spacing}");
            }
        }
    }
}

#[test]
fn glyphs_are_cut_at_the_text_area_s_edges_and_lines_past_them_aren_t_drawn() {
    let font = real_font();
    let style = Style { scale: 3, spacing: font.spacing() };
    let columns = layout::visible_columns(&font, style);
    let lines = layout::visible_lines(&font, style);
    let row = "W".repeat(columns + 3);
    let text = vec![row; lines + 3].join("\n");
    let mut tester = tester(&text, 3, &font);
    tester.handle(Input::Key(Key::Home, Modifiers::CONTROL), &font);
    let area = layout::text_area();
    let quads = glyph_quads(&scene(&tester, &font, &[]));
    for quad in &quads {
        assert!(!quad.area.is_empty());
        assert_eq!(quad.area, quad.area.intersection(area), "{quad:?} is outside the text area");
    }
    // Whole columns and lines, then at most one cut column and line.
    let drawn_columns = quads.iter().filter(|quad| quad.origin[1] == area.top).count();
    let drawn_lines = quads.iter().filter(|quad| quad.origin[0] == area.left).count();
    assert!(drawn_columns == columns || drawn_columns == columns + 1, "{drawn_columns} columns");
    assert!(drawn_lines == lines || drawn_lines == lines + 1, "{drawn_lines} lines");
    let cut = quads.iter().filter(|quad| quad.area.width() < (font.glyph_width * 3) as i32 || quad.area.height() < (font.glyph_height * 3) as i32);
    assert!(cut.clone().all(|quad| quad.area.right == area.right || quad.area.bottom == area.bottom));
}

#[test]
fn scrolled_text_starts_with_the_first_column_and_line_that_show() {
    let font = real_font();
    let columns = layout::visible_columns(&font, Style { scale: 10, spacing: font.spacing() });
    let line = format!("A{}Z", "x".repeat(columns));
    let tester = tester(&format!("{line}\n{line}"), 10, &font);
    // As far as shows the caret, after Z, and the character after it.
    assert_eq!(tester.scroll, Position { line: 0, column: line.len() + 1 - columns });
    let area = layout::text_area();
    let quads = glyph_quads(&scene(&tester, &font, &[]));
    // A doesn't show, even cut: the first glyph is at the text area's left,
    // and Z is whole.
    assert!(quads.iter().all(|quad| quad.glyph != u32::from(b'A')));
    assert_eq!(quads[0].origin, [area.left, area.top]);
    let z = quads.iter().find(|quad| quad.glyph == u32::from(b'Z') && quad.origin[1] > area.top).unwrap();
    assert_eq!(z.area.width(), (font.glyph_width * 10) as i32);
}

#[test]
fn every_quad_is_on_whole_pixels_inside_the_window_at_every_size() {
    let font = real_font();
    let text: String = (0x20u8..0x7F).map(char::from).collect::<String>() + "\t\n" + &"line\n".repeat(80);
    for scale in 1..=10 {
        let tester = tester(&text, scale, &font);
        let scene = scene(&tester, &font, &[]);
        let window = Rect::new(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
        for quad in &scene.quads {
            // Rect's corners are integers: what matters is that they're in
            // the window, and that a glyph's origin is a whole number of its
            // scaled pixels from the area that it covers.
            assert_eq!(quad.area, quad.area.intersection(window));
            if quad.glyph != SOLID {
                assert_eq!(quad.scale, scale);
                assert!(quad.area.left >= quad.origin[0] && quad.area.top >= quad.origin[1]);
                let glyph = Rect::new(
                    quad.origin[0],
                    quad.origin[1],
                    quad.origin[0] + (font.glyph_width * scale) as i32,
                    quad.origin[1] + (font.glyph_height * scale) as i32,
                );
                assert_eq!(quad.area, quad.area.intersection(glyph));
            }
        }
    }
}

#[test]
fn an_unsupported_setup_shows_what_s_wrong_instead_of_the_box() {
    let font = real_font();
    let tester = tester("hidden", 1, &font);
    let problems = ["The display is 1920 x 1080 pixels".to_string()];
    let scene = scene(&tester, &font, &problems);
    assert!(scene.quads.iter().all(|quad| quad.glyph != SOLID), "the box is drawn");
    let characters: String = scene.quads.iter().map(|quad| char::from(quad.glyph as u8)).collect();
    assert_eq!(characters, "Thedisplayis1920x1080pixels");
}
