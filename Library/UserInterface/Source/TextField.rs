// A text field: a line of text to type in. A press gives it keyboard focus,
// and puts the caret at the character nearest it. While it has focus, it
// takes typed characters, Backspace, Delete, Left, Right, Home, End, and
// Enter, which submits it, and every other key without Control but Tab, Esc
// and the function keys, so that typing doesn't set off the program's
// shortcuts. Esc takes focus away, and Tab moves it on.
//
// The font is monospaced, so the caret's place is a number of characters.
// Characters that the font doesn't have aren't taken. Text wider than the
// field scrolls to keep the caret in it.

use application::Event;
use canvas::Rect;
use canvas::theme::{ACCENT, BORDER, DIM, FAINT, INSET, PANEL, TEXT, TITLE};
use canvas::Font;
use windows_api::*;

use crate::interface::{Id, Layer, Sense};
use crate::{Frame, Response};

const KEYS: &[i32] = &[VK_BACK, VK_DELETE, VK_LEFT, VK_RIGHT, VK_HOME, VK_END, VK_RETURN];

// The caret blinks this many times a second while it doesn't move.
const BLINKS_PER_SECOND: f64 = 1.0;

// The byte where a character starts, or the text's length past its end.
fn byte_at(text: &str, character: usize) -> usize {
    text.char_indices().nth(character).map_or(text.len(), |(index, _)| index)
}

impl Frame<'_> {
    // A text field in `area`, which edits `text`, and shows `placeholder`,
    // faint, while it's empty. `limit` is the most characters it takes.
    pub fn text_field(&mut self, name: &str, area: Rect, text: &mut String, placeholder: &str, limit: usize) -> Response {
        let id = Id::new(name);
        let sense = Sense {
            click: true,
            focus: true,
            keys: KEYS,
            text: true,
            ..Sense::HOVER
        };
        let mut response = self.interact_on(name, "TextField", "", area, sense, Layer::Base, None);
        let char_width = self.char_width();
        let inset = (self.line_height() * 0.4).round();
        let columns = (((area.width - inset * 2.0) / char_width).floor() as usize).max(1);
        let length = text.chars().count();
        let seconds = self.seconds();
        let mut caret = self.state.carets.get(&id).copied().unwrap_or_default();
        caret.position = caret.position.min(length);
        let before = text.clone();
        let start = caret;

        for event in self.events_for(id) {
            let length = text.chars().count();
            match event {
                Event::Press([x, _]) => {
                    let column = ((x - area.x - inset) / char_width).round().max(0.0) as usize;
                    caret.position = (caret.scroll + column).min(length);
                }
                Event::Character(character) if Font::has_glyph(character) && length < limit => {
                    text.insert(byte_at(text, caret.position), character);
                    caret.position += 1;
                }
                Event::Key { key, repeat, .. } => match key as i32 {
                    VK_BACK if caret.position > 0 => {
                        caret.position -= 1;
                        text.remove(byte_at(text, caret.position));
                    }
                    VK_DELETE if caret.position < length => {
                        text.remove(byte_at(text, caret.position));
                    }
                    VK_LEFT => caret.position = caret.position.saturating_sub(1),
                    VK_RIGHT => caret.position = (caret.position + 1).min(length),
                    VK_HOME => caret.position = 0,
                    VK_END => caret.position = length,
                    VK_RETURN if !repeat => response.submitted = true,
                    _ => {}
                },
                _ => {}
            }
        }
        // Keeps the caret in view, and the field full when there's more
        // text before what it shows.
        let length = text.chars().count();
        if caret.position < caret.scroll {
            caret.scroll = caret.position;
        } else if caret.position > caret.scroll + columns {
            caret.scroll = caret.position - columns;
        }
        caret.scroll = caret.scroll.min(length.saturating_sub(columns));
        response.changed = *text != before;
        if caret.position != start.position || response.changed || (response.focused && response.pressed) {
            caret.moved_at = seconds;
        }
        self.state.carets.insert(id, caret);

        let enabled = self.enabled();
        let canvas = &mut self.canvas;
        canvas.rect(area.x, area.y, area.width, area.height, if enabled { PANEL } else { INSET });
        if response.focused {
            canvas.rect_outline(area.x, area.y, area.width, area.height, 2.0, ACCENT);
        } else {
            let outline = if response.hovered && enabled { DIM } else { BORDER };
            canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, outline);
        }
        let text_x = area.x + inset;
        let text_y = area.y + ((area.height - canvas.line_height()) * 0.5).round();
        if text.is_empty() {
            canvas.text_clipped(text_x, text_y, placeholder, area.width - inset * 2.0, FAINT);
        } else {
            let shown: String = text.chars().skip(caret.scroll).take(columns).collect();
            canvas.text(text_x, text_y, &shown, if enabled { TEXT } else { FAINT });
        }
        let blink = ((seconds - caret.moved_at) * BLINKS_PER_SECOND * 2.0).floor() as i64 % 2 == 0;
        if response.focused && blink {
            let x = text_x + (caret.position - caret.scroll) as f32 * char_width;
            canvas.rect(x.round() - 1.0, text_y, 2.0, canvas.line_height(), TITLE);
        }
        response
    }
}
