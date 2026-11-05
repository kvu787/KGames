// A text field: a line of text to type in, like Fluent's text box. A press
// gives it keyboard focus, and puts the caret at the place nearest it, and
// dragging selects from there. A double-click selects a word. While it has
// focus, it takes typed characters and the keys that edit:
//
// | Keys                     | What they do                                     |
// | ------------------------ | ------------------------------------------------ |
// | Left, Right, Home, End   | Move the caret, and with Shift, select           |
// | Ctrl+Left, Ctrl+Right    | Move by words                                    |
// | Backspace, Delete        | Delete the selection, or a character, or with    |
// |                          | Control, a word                                  |
// | Ctrl+A                   | Selects everything                               |
// | Ctrl+C, Ctrl+X, Ctrl+V   | Copy, cut and paste, with the interface's        |
// |                          | clipboard                                        |
// | Enter                    | Submits it                                       |
//
// It takes every other key without Control but Tab, Esc and the function
// keys too, so that typing doesn't set off the program's shortcuts. Esc
// takes focus away, and Tab moves it on. While it has focus and text, a
// button at its right clears it.
//
// Characters that the fonts don't have aren't taken. Text wider than the
// field scrolls to keep the caret in it.

use application::Event;
use canvas::{Fonts, Icon, Rect, TextStyle};
use windows_api::*;

use crate::interface::{Edit, Id, Layer, Sense};
use crate::surfaces::focus_ring;
use crate::tokens::*;
use crate::{Button, Frame, Response};

const KEYS: &[i32] = &[VK_BACK, VK_DELETE, VK_LEFT, VK_RIGHT, VK_HOME, VK_END, VK_RETURN];
const CONTROL_KEYS: &[i32] = &[
    'A' as i32, 'C' as i32, 'X' as i32, 'V' as i32, VK_LEFT, VK_RIGHT, VK_BACK, VK_DELETE, VK_HOME, VK_END,
];

// The caret blinks this many times a second while it doesn't move.
const BLINKS_PER_SECOND: f64 = 1.0;
// The button that clears the field.
const CLEAR_WIDTH: f32 = 30.0;

// What a text field shows besides its text.
#[derive(Clone, Copy, Debug)]
pub struct TextField<'a> {
    // Shown, faint, while it's empty.
    pub placeholder: &'a str,
    // The most characters it takes.
    pub limit: usize,
    // An icon before the text, like a magnifying glass for a search.
    pub icon: Option<Icon>,
}

impl<'a> TextField<'a> {
    pub fn new(placeholder: &'a str) -> Self {
        Self {
            placeholder,
            limit: 256,
            icon: None,
        }
    }

    pub fn limit(self, limit: usize) -> Self {
        Self { limit, ..self }
    }

    pub fn icon(self, icon: Icon) -> Self {
        Self { icon: Some(icon), ..self }
    }
}

// The byte where a character starts, or the text's length past its end.
fn byte_at(text: &str, character: usize) -> usize {
    text.char_indices().nth(character).map_or(text.len(), |(index, _)| index)
}

// How far the place before character `index` is from the text's start.
fn x_of(fonts: &Fonts, text: &str, index: usize) -> f32 {
    fonts.measure(&text[..byte_at(text, index)], TextStyle::Body)
}

// The place between characters nearest `x` from the text's start.
fn index_at(fonts: &Fonts, text: &str, x: f32) -> usize {
    let metrics = fonts.style(TextStyle::Body);
    let mut left = 0.0;
    for (index, character) in text.chars().enumerate() {
        let advance = metrics.advance(character);
        if x < left + advance * 0.5 {
            return index;
        }
        left += advance;
    }
    text.chars().count()
}

// The start of the word before `index`, past any spaces, as Ctrl+Left goes.
fn word_before(characters: &[char], index: usize) -> usize {
    let mut index = index;
    while index > 0 && characters[index - 1].is_whitespace() {
        index -= 1;
    }
    while index > 0 && !characters[index - 1].is_whitespace() {
        index -= 1;
    }
    index
}

// The start of the next word after `index`, as Ctrl+Right goes.
fn word_after(characters: &[char], index: usize) -> usize {
    let mut index = index;
    while index < characters.len() && !characters[index].is_whitespace() {
        index += 1;
    }
    while index < characters.len() && characters[index].is_whitespace() {
        index += 1;
    }
    index
}

// The word around `index`, or the spaces, for a double-click.
fn word_around(characters: &[char], index: usize) -> (usize, usize) {
    let index = index.min(characters.len());
    let space = |at: usize| characters.get(at).is_some_and(|character| character.is_whitespace());
    let kind = if index < characters.len() { space(index) } else { index > 0 && space(index - 1) };
    let mut start = index;
    while start > 0 && space(start - 1) == kind {
        start -= 1;
    }
    let mut end = index;
    while end < characters.len() && space(end) == kind {
        end += 1;
    }
    (start, end)
}

impl Edit {
    fn selection(&self) -> (usize, usize) {
        (self.caret.min(self.anchor), self.caret.max(self.anchor))
    }

    fn has_selection(&self) -> bool {
        self.caret != self.anchor
    }

    // Moves the caret, and with `extend`, keeps the other end of the
    // selection where it was.
    fn go(&mut self, index: usize, extend: bool) {
        self.caret = index;
        if !extend {
            self.anchor = index;
        }
    }

    // Deletes what's selected, and returns whether anything was.
    fn delete_selection(&mut self, text: &mut String) -> bool {
        let (start, end) = self.selection();
        if start == end {
            return false;
        }
        text.replace_range(byte_at(text, start)..byte_at(text, end), "");
        self.go(start, false);
        true
    }

    // Puts `insert` where the caret is, over the selection, as much of it as
    // fits in `limit` characters.
    fn insert(&mut self, text: &mut String, insert: &str, limit: usize) {
        self.delete_selection(text);
        let room = limit.saturating_sub(text.chars().count());
        let insert: String = insert.chars().take(room).collect();
        text.insert_str(byte_at(text, self.caret), &insert);
        self.go(self.caret + insert.chars().count(), false);
    }
}

impl Frame<'_> {
    // A text field in `area`, which edits `text`. The button that clears it
    // is called "<name>/Clear".
    pub fn text_field(&mut self, name: &str, area: Rect, text: &mut String, field: TextField) -> Response {
        let id = Id::new(name);
        let sense = Sense {
            click: true,
            focus: true,
            keys: KEYS,
            control_keys: CONTROL_KEYS,
            text: true,
            ..Sense::HOVER
        };
        let mut response = self.interact_on(name, "TextField", "", area, sense, Layer::Base, None);
        let fonts = self.fonts();
        let seconds = self.seconds();
        let enabled = self.enabled();
        let clear_name = format!("{name}/Clear");
        let clear_area = Rect::new(area.right() - CLEAR_WIDTH - 2.0, area.y + 4.0, CLEAR_WIDTH, area.height - 8.0);
        let icon_room = if field.icon.is_some() { 16.0 + SPACE_S } else { 0.0 };
        let text_left = area.x + TEXT_INSET - 1.0 + icon_room;
        let length = text.chars().count();
        let mut edit = self.state.edits.get(&id).copied().unwrap_or_default();
        edit.caret = edit.caret.min(length);
        edit.anchor = edit.anchor.min(length);
        let before = text.clone();
        let start = edit;

        // A press on the clear button empties the field, which keeps focus.
        let cleared = self.events_for(Id::new(&clear_name)).iter().any(|event| matches!(event, Event::Press(_)));
        if cleared {
            text.clear();
            edit = Edit::default();
        }
        for event in self.events_for(id) {
            let characters: Vec<char> = text.chars().collect();
            let length = characters.len();
            match event {
                Event::Press([x, _]) => {
                    let index = index_at(fonts, text, x - text_left + edit.scroll);
                    // Dragging after a press selects, but not after a
                    // double-click, which selected a word.
                    edit.dragging = !self.state.double_click;
                    if self.state.double_click {
                        let (word_start, word_end) = word_around(&characters, index);
                        edit.anchor = word_start;
                        edit.caret = word_end;
                    } else {
                        edit.go(index, false);
                    }
                }
                Event::Character(character) if Fonts::has_glyph(character) && !character.is_control() => {
                    edit.insert(text, &character.to_string(), field.limit);
                }
                Event::Key { key, control: true, shift, .. } => match key as i32 {
                    key if key == 'A' as i32 => {
                        edit.anchor = 0;
                        edit.caret = length;
                    }
                    key if key == 'C' as i32 || key == 'X' as i32 => {
                        let (start, end) = edit.selection();
                        if start < end {
                            self.clipboard.set(&text[byte_at(text, start)..byte_at(text, end)]);
                            if key == 'X' as i32 {
                                edit.delete_selection(text);
                            }
                        }
                    }
                    key if key == 'V' as i32 => {
                        if let Some(pasted) = self.clipboard.get() {
                            let pasted: String = pasted
                                .chars()
                                .map(|character| if character.is_whitespace() { ' ' } else { character })
                                .filter(|character| Fonts::has_glyph(*character))
                                .collect();
                            edit.insert(text, &pasted, field.limit);
                        }
                    }
                    VK_LEFT => edit.go(word_before(&characters, edit.caret), shift),
                    VK_RIGHT => edit.go(word_after(&characters, edit.caret), shift),
                    VK_HOME => edit.go(0, shift),
                    VK_END => edit.go(length, shift),
                    VK_BACK if !edit.delete_selection(text) => {
                        let from = word_before(&characters, edit.caret);
                        edit.anchor = from;
                        edit.delete_selection(text);
                    }
                    VK_DELETE if !edit.delete_selection(text) => {
                        let to = word_after(&characters, edit.caret);
                        edit.anchor = to;
                        edit.delete_selection(text);
                    }
                    _ => {}
                },
                Event::Key { key, shift, repeat, .. } => match key as i32 {
                    VK_BACK if !edit.delete_selection(text) && edit.caret > 0 => {
                        edit.anchor = edit.caret - 1;
                        edit.delete_selection(text);
                    }
                    VK_DELETE if !edit.delete_selection(text) && edit.caret < length => {
                        edit.anchor = edit.caret + 1;
                        edit.delete_selection(text);
                    }
                    VK_LEFT if edit.has_selection() && !shift => edit.go(edit.selection().0, false),
                    VK_RIGHT if edit.has_selection() && !shift => edit.go(edit.selection().1, false),
                    VK_LEFT => edit.go(edit.caret.saturating_sub(1), shift),
                    VK_RIGHT => edit.go((edit.caret + 1).min(length), shift),
                    VK_HOME => edit.go(0, shift),
                    VK_END => edit.go(length, shift),
                    VK_RETURN if !repeat => response.submitted = true,
                    _ => {}
                },
                _ => {}
            }
        }
        if response.held
            && !response.pressed
            && edit.dragging
            && let Some([x, _]) = self.cursor()
        {
            edit.caret = index_at(fonts, text, x - text_left + edit.scroll);
        }

        // Keeps the caret in view, and the field full when there's more text
        // before what it shows.
        let clear_room = if response.focused && enabled && !text.is_empty() { CLEAR_WIDTH } else { 0.0 };
        let visible = (area.right() - TEXT_INSET - clear_room - text_left).max(1.0);
        let caret_x = x_of(fonts, text, edit.caret);
        let total = fonts.measure(text, TextStyle::Body);
        if caret_x < edit.scroll {
            edit.scroll = caret_x;
        } else if caret_x > edit.scroll + visible {
            edit.scroll = caret_x - visible;
        }
        edit.scroll = edit.scroll.min((total - visible).max(0.0)).max(0.0);
        response.changed = *text != before;
        let moved = edit.caret != start.caret || edit.anchor != start.anchor || response.changed;
        if moved || response.pressed {
            edit.moved_at = seconds;
        }
        self.state.edits.insert(id, edit);

        let (hover, _) = self.hover_and_press(id, &response);
        let theme = self.theme();
        let canvas = &mut self.canvas;
        // The face, with a line along its bottom: the accent's, thicker,
        // while it has focus.
        if enabled {
            let (fill, line, thickness) = if response.focused {
                (theme.control, theme.accent, 2.0)
            } else {
                (theme.control.mix(theme.control_hover, hover), theme.control_strong, 1.0)
            };
            canvas.fill(area, RADIUS_CONTROL, line);
            let top = Rect::new(area.x, area.y, area.width, area.height - thickness);
            canvas.fill_stroke(top, RADIUS_CONTROL, fill, 1.0, theme.control_stroke);
        } else {
            canvas.fill_stroke(area, RADIUS_CONTROL, theme.control_disabled, 1.0, theme.control_stroke);
        }
        if let Some(icon) = field.icon {
            let color = if enabled { theme.text_secondary } else { theme.text_disabled };
            canvas.icon([area.x + TEXT_INSET + 8.0, area.center()[1]], icon, 16.0, color);
        }
        let line_height = canvas.line_height(TextStyle::Body);
        let text_y = area.y + ((area.height - line_height) * 0.5).round();
        let inner = Rect::from_edges(text_left, area.y + 2.0, text_left + visible, area.bottom() - 2.0);
        canvas.push_clip(inner);
        let origin = text_left - edit.scroll;
        if text.is_empty() {
            let color = if enabled { theme.text_secondary } else { theme.text_disabled };
            canvas.text([text_left, text_y], field.placeholder, TextStyle::Body, color);
        } else {
            let color = if enabled { theme.text } else { theme.text_disabled };
            canvas.text([origin, text_y], text, TextStyle::Body, color);
            let (start, end) = edit.selection();
            if response.focused && start < end {
                let left = origin + x_of(fonts, text, start);
                let right = origin + x_of(fonts, text, end);
                let selection = Rect::from_edges(left, text_y, right, text_y + line_height);
                canvas.fill(selection, 2.0, theme.accent);
                let selected = &text[byte_at(text, start)..byte_at(text, end)];
                canvas.text([left, text_y], selected, TextStyle::Body, theme.on_accent);
            }
        }
        let blink = ((seconds - edit.moved_at) * BLINKS_PER_SECOND * 2.0).floor() as i64 % 2 == 0;
        if response.focused && blink {
            let x = (origin + caret_x).round();
            canvas.rect(Rect::new(x - 0.5, text_y + 1.0, 1.0, line_height - 2.0), theme.text);
        }
        canvas.pop_clip();
        if clear_room > 0.0 {
            // A part of the field, so that pressing it keeps the field's
            // focus.
            self.button_part(Layer::Base, &clear_name, clear_area, Button::icon_only(Icon::Dismiss).subtle(), Some(id));
        }
        if self.shows_focus(&response) {
            focus_ring(&mut self.canvas, theme, area, RADIUS_CONTROL);
        }
        response
    }
}
