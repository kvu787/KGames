// What a frame draws: the window's background, and quads over it, in order.
// A quad is a rectangle of whole pixels, filled with a color, or with a
// glyph's pixels in it, each `scale` × `scale` of the screen's.

use crate::font::Font;
use crate::layout::{self, Rect};
use crate::tester::Tester;
use crate::text::Position;

// A quad's glyph when it's a rectangle of its color.
pub const SOLID: u32 = u32::MAX;

// Laid out like the shader's Quad.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quad {
    // The pixels that it covers. A glyph's quad covers the part of its glyph
    // that shows.
    pub area: Rect,
    // Where its glyph's top left pixel is on the screen, which may be
    // outside the area, when the glyph is cut.
    pub origin: [i32; 2],
    pub scale: u32,
    // The character whose glyph it draws, or SOLID.
    pub glyph: u32,
    // Red, green, blue and alpha, a byte each, red lowest.
    pub color: u32,
}

pub struct Scene {
    pub background: [u8; 3],
    pub quads: Vec<Quad>,
}

pub fn pack(color: [u8; 3]) -> u32 {
    u32::from(color[0]) | u32::from(color[1]) << 8 | u32::from(color[2]) << 16 | 0xFF << 24
}

fn solid(area: Rect, color: [u8; 3]) -> Quad {
    Quad {
        area,
        origin: [area.left, area.top],
        scale: 1,
        glyph: SOLID,
        color: pack(color),
    }
}

// The quad of a character's glyph with its top left at the origin, cut to
// what shows in `clip`, or none if nothing does or it's blank.
fn glyph(font: &Font, character: u8, origin: (i32, i32), scale: u32, clip: Rect, color: [u8; 3]) -> Option<Quad> {
    let (x, y) = origin;
    let whole = Rect::new(x, y, x + (font.glyph_width * scale) as i32, y + (font.glyph_height * scale) as i32);
    let area = whole.intersection(clip);
    if area.is_empty() || font.glyph(character).iter().all(|&coverage| coverage == 0) {
        return None;
    }
    Some(Quad {
        area,
        origin: [x, y],
        scale,
        glyph: u32::from(character),
        color: pack(color),
    })
}

// The tester's frame: the box, with the text that shows in it and the
// caret, or, when the window's setup isn't supported, what's wrong with it.
pub fn scene(tester: &Tester, font: &Font, problems: &[String]) -> Scene {
    let colors = tester.theme.colors();
    let mut quads = Vec::new();
    if !problems.is_empty() {
        // In the font itself, in the GUI's pixels, at the top left.
        let window = Rect::new(0, 0, layout::WINDOW_WIDTH, layout::WINDOW_HEIGHT);
        let scale = layout::GUI_SCALE as u32;
        for (line, problem) in problems.iter().enumerate() {
            let y = layout::PROBLEMS_MARGIN + (line as u32 * font.line_height * scale) as i32;
            for (column, character) in problem.chars().enumerate() {
                let x = layout::PROBLEMS_MARGIN + (column as u32 * font.advance * scale) as i32;
                let character = if character.is_ascii() { character as u8 } else { b'?' };
                quads.extend(glyph(font, character, (x, y), scale, window, colors.text));
            }
        }
        return Scene {
            background: colors.background,
            quads,
        };
    }

    quads.push(solid(layout::BOX, colors.border));
    quads.push(solid(layout::field(), colors.field));
    let area = layout::text_area();
    let (scale, scroll) = (tester.scale, tester.scroll);
    for (line, characters) in tester.text.lines().iter().enumerate().skip(scroll.line) {
        let (_, y) = layout::glyph_origin(font, scale, scroll, Position { line, column: 0 });
        if y >= area.bottom {
            break;
        }
        for (column, &character) in characters.iter().enumerate().skip(scroll.column) {
            let origin = layout::glyph_origin(font, scale, scroll, Position { line, column });
            if origin.0 >= area.right {
                break;
            }
            quads.extend(glyph(font, character, origin, scale, area, colors.text));
        }
    }
    let caret = layout::caret(font, scale, scroll, tester.text.caret()).intersection(layout::field());
    if !caret.is_empty() {
        quads.push(solid(caret, colors.text));
    }
    Scene {
        background: colors.background,
        quads,
    }
}
