// A one-line text field, as WinUI's TextBox: a caret and a selection that
// the pointer and the keys move, copy, cut and paste, a button that clears
// it, and placeholder text while it's empty. Its bottom edge turns into a
// thick accent line while it has focus.

use crate::geometry::{Point, Rect};
use crate::icons::Icon;
use crate::input::{Button, Key};
use crate::painter::{Align, Corners, Stroke};
use crate::tokens::{BODY, DURATION_FASTER, ICON, ICON_SMALL, RADIUS_CONTROL};
use crate::ui::{Cursor, Response, Sense, Ui};
use crate::widgets::common::{Visual, control_focus_ring};

const PADDING_LEFT: f32 = 10.0;
const PADDING_RIGHT: f32 = 6.0;
const BUTTON_WIDTH: f32 = 30.0;
// Windows' default caret blink time, on and off.
const BLINK: f64 = 0.53;

#[derive(Clone, Copy, Debug, Default)]
pub struct TextBox<'a> {
    pub placeholder: &'a str,
    // A button at its end, like a search field's magnifying glass, which
    // submits it.
    pub button: Option<Icon>,
    pub max_length: Option<usize>,
}

impl<'a> TextBox<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn button(mut self, icon: Icon) -> Self {
        self.button = Some(icon);
        self
    }

    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextBoxResponse {
    pub response: Response,
    pub changed: bool,
    // Enter was pressed in it, or its button was.
    pub submitted: bool,
}

#[derive(Default)]
struct EditState {
    // Byte indices of the caret, and of the other end of the selection.
    caret: usize,
    anchor: usize,
    scroll: f32,
    selecting: bool,
    was_focused: bool,
    // When the caret last moved, which restarts its blinking.
    moved_at: f64,
}

impl EditState {
    fn selection(&self) -> (usize, usize) {
        (self.caret.min(self.anchor), self.caret.max(self.anchor))
    }
}

impl Ui {
    pub fn text_box(&mut self, name: &str, rect: Rect, text: &mut String, options: TextBox) -> TextBoxResponse {
        let response = self.interact(name, rect, Sense::TEXT, "TextBox");
        self.set_label(name, text);
        let enabled = self.enabled();
        if response.hovered {
            self.set_cursor(Cursor::Text);
        }
        let mut changed = false;
        let mut submitted = false;
        let before = text.clone();

        // Where the text goes, which the buttons inside the field take room
        // from: as it was before this frame's typing, for where presses land.
        let buttons = |text: &str, focused: bool| {
            let mut right = rect.right() - PADDING_RIGHT;
            let submit = options.button.map(|_| {
                right -= BUTTON_WIDTH;
                Rect::new(right + PADDING_RIGHT - 2.0, rect.y + 4.0, BUTTON_WIDTH - 2.0, rect.height - 8.0)
            });
            let clear = (focused && !text.is_empty() && enabled).then(|| {
                right -= BUTTON_WIDTH;
                Rect::new(right + PADDING_RIGHT - 2.0, rect.y + 4.0, BUTTON_WIDTH - 2.0, rect.height - 8.0)
            });
            let area = Rect::new(rect.x + PADDING_LEFT, rect.y, (right - rect.x - PADDING_LEFT).max(0.0), rect.height);
            (area, clear, submit)
        };
        let (area, _, _) = buttons(text, response.focused);

        let time = self.time();
        let id = response.id;
        let pointer = self.input().pointer;
        let clicks = self.input().clicks(Button::Left);
        let shift = self.input().modifiers().shift;
        let mut state = std::mem::take(self.memory::<EditState>(id));
        state.caret = state.caret.min(text.len());
        state.anchor = state.anchor.min(text.len());
        let focused = self.is_focused(name) && enabled;
        if focused && !state.was_focused && !response.pressed {
            // Focus from the keyboard selects everything, ready to replace.
            state.anchor = 0;
            state.caret = text.len();
            state.moved_at = time;
        }

        let text_x = area.x - state.scroll;
        let hit = |ui: &mut Ui, text: &str, point: Point| -> usize {
            let layout = ui.fonts().layout(text, BODY, None);
            layout.hit(Point::new(point.x - text_x, 0.0))
        };
        if response.pressed && enabled
            && let Some(point) = pointer
        {
            let index = hit(self, text, point);
            if clicks >= 2 {
                let (start, end) = word_at(text, index);
                state.anchor = start;
                state.caret = end;
                state.selecting = false;
            } else {
                state.caret = index;
                if !shift {
                    state.anchor = index;
                }
                state.selecting = true;
            }
            state.moved_at = time;
        }
        if state.selecting {
            if response.held {
                if let Some(point) = pointer {
                    state.caret = hit(self, text, point);
                }
            } else {
                state.selecting = false;
            }
        }

        if focused {
            let input = self.input().clone();
            let modifiers = input.modifiers();
            let word = modifiers.control;
            for (key, _) in input.keys_pressed() {
                let (start, end) = state.selection();
                let has_selection = start != end;
                match *key {
                    Key::LEFT => {
                        state.caret = if has_selection && !modifiers.shift {
                            start
                        } else if word {
                            previous_word(text, state.caret)
                        } else {
                            previous_boundary(text, state.caret)
                        };
                        if !modifiers.shift {
                            state.anchor = state.caret;
                        }
                    }
                    Key::RIGHT => {
                        state.caret = if has_selection && !modifiers.shift {
                            end
                        } else if word {
                            next_word(text, state.caret)
                        } else {
                            next_boundary(text, state.caret)
                        };
                        if !modifiers.shift {
                            state.anchor = state.caret;
                        }
                    }
                    Key::HOME | Key::END => {
                        state.caret = if *key == Key::HOME { 0 } else { text.len() };
                        if !modifiers.shift {
                            state.anchor = state.caret;
                        }
                    }
                    Key::BACKSPACE | Key::DELETE => {
                        if has_selection {
                            text.replace_range(start..end, "");
                            state.caret = start;
                        } else if *key == Key::BACKSPACE && state.caret > 0 {
                            let from = if word { previous_word(text, state.caret) } else { previous_boundary(text, state.caret) };
                            text.replace_range(from..state.caret, "");
                            state.caret = from;
                        } else if *key == Key::DELETE && state.caret < text.len() {
                            let to = if word { next_word(text, state.caret) } else { next_boundary(text, state.caret) };
                            text.replace_range(state.caret..to, "");
                        }
                        state.anchor = state.caret;
                    }
                    Key::A if modifiers.control => {
                        state.anchor = 0;
                        state.caret = text.len();
                    }
                    Key::C | Key::X if modifiers.control && has_selection => {
                        let selected = text[start..end].to_string();
                        self.clipboard().set(&selected);
                        if *key == Key::X {
                            text.replace_range(start..end, "");
                            state.caret = start;
                            state.anchor = start;
                        }
                    }
                    Key::V if modifiers.control => {
                        if let Some(pasted) = self.clipboard().get() {
                            let pasted: String = pasted.chars().filter(|c| !c.is_control()).collect();
                            insert(text, &mut state, &pasted, options.max_length);
                        }
                    }
                    Key::ENTER => submitted = true,
                    _ => continue,
                }
                state.moved_at = time;
            }
            let typed: String = input.text.chars().filter(|c| !c.is_control()).collect();
            if !typed.is_empty() && !modifiers.control {
                insert(text, &mut state, &typed, options.max_length);
                state.moved_at = time;
            }
        } else {
            state.anchor = state.caret;
        }
        // The buttons inside it are widgets of their own, which show once
        // there's text to clear.
        let (area, clear_button, submit_button) = buttons(text, focused);
        let clear = clear_button.map(|button| self.interact(&format!("{name}/Clear"), button, Sense::CLICK, "Button"));
        let submit = submit_button.map(|button| self.interact(&format!("{name}/Submit"), button, Sense::CLICK, "Button"));
        if let Some(clear) = &clear
            && clear.clicked
        {
            text.clear();
            state.caret = 0;
            state.anchor = 0;
            self.focus(name);
        }
        if let Some(submit) = &submit
            && submit.clicked
        {
            submitted = true;
        }
        state.was_focused = focused;
        changed |= *text != before;

        // Keeps the caret in view.
        let (caret_x, text_width) = {
            let layout = self.fonts().layout(text, BODY, None);
            (layout.caret_x(state.caret), layout.width)
        };
        if caret_x - state.scroll > area.width - 1.0 {
            state.scroll = caret_x - area.width + 1.0;
        }
        if caret_x - state.scroll < 0.0 {
            state.scroll = caret_x;
        }
        state.scroll = state.scroll.clamp(0.0, (text_width - area.width + 1.0).max(0.0));

        // Drawing.
        let visual = Visual::of(self, &response);
        let theme = self.theme().clone();
        let fill = if focused {
            theme.control_fill_input_active
        } else {
            visual.pick(theme.control_fill, theme.control_fill_secondary, theme.control_fill_secondary, theme.control_fill_disabled)
        };
        let fill = self.animate_color(id, 0, fill, DURATION_FASTER);
        let text_x = area.x - state.scroll;
        let (start, end) = state.selection();
        let (start_x, end_x) = {
            let layout = self.fonts().layout(text, BODY, None);
            (layout.caret_x(start), layout.caret_x(end))
        };
        let caret_on = focused && (((time - state.moved_at) / BLINK) as u64).is_multiple_of(2);
        let mut painter = self.painter();
        let stroke = if visual == Visual::Disabled {
            Stroke::new(theme.control_stroke, 1.0)
        } else {
            Stroke::elevated(theme.control_stroke, theme.control_strong_stroke)
        };
        painter.shape(rect, Corners::all(RADIUS_CONTROL), fill, Some(stroke));
        if focused {
            // The accent line, inside the rounded bottom edge.
            painter.push_clip(Rect::new(rect.x, rect.bottom() - 2.0, rect.width, 2.0));
            painter.shape(rect, Corners::all(RADIUS_CONTROL), theme.accent, None);
            painter.pop_clip();
        }
        painter.push_clip(area);
        let line = Rect::new(text_x, area.y, area.width + state.scroll, area.height);
        if text.is_empty() {
            let placeholder = if visual == Visual::Disabled { theme.text_disabled } else { theme.text_secondary };
            painter.text_in(area, options.placeholder, BODY, placeholder, Align::Start);
        } else {
            let color = if visual == Visual::Disabled { theme.text_disabled } else { theme.text_primary };
            let placed = painter.text_in(line, text, BODY, color, Align::Start);
            if focused && start != end {
                let selection = Rect::new(text_x + start_x, placed.y, end_x - start_x, placed.height);
                painter.fill(selection, theme.text_selection);
                painter.push_clip(selection);
                painter.text_in(line, text, BODY, crate::geometry::Color::WHITE, Align::Start);
                painter.pop_clip();
            }
        }
        if caret_on {
            let x = (text_x + caret_x).round();
            let middle = rect.center().y;
            painter.fill(Rect::new(x, middle - 9.0, 1.0, 18.0), theme.text_primary);
        }
        painter.pop_clip();
        if let (Some(button), Some(clear)) = (clear_button, &clear) {
            draw_inner_button(self, clear, button, Icon::Clear, ICON_SMALL);
        }
        if let (Some(button), Some(submit), Some(icon)) = (submit_button, &submit, options.button) {
            draw_inner_button(self, submit, button, icon, ICON);
        }
        control_focus_ring(self, &response);
        *self.memory::<EditState>(id) = state;

        TextBoxResponse {
            response: Response { changed, ..response },
            changed,
            submitted,
        }
    }
}

fn draw_inner_button(ui: &mut Ui, response: &Response, rect: Rect, icon: Icon, size: f32) {
    let visual = Visual::of(ui, response);
    let theme = ui.theme();
    let fill = visual.pick(
        crate::geometry::Color::TRANSPARENT,
        theme.subtle_fill_secondary,
        theme.subtle_fill_tertiary,
        crate::geometry::Color::TRANSPARENT,
    );
    let color = visual.pick(theme.text_secondary, theme.text_secondary, theme.text_tertiary, theme.text_disabled);
    let mut painter = ui.painter();
    painter.rounded(rect, RADIUS_CONTROL, fill);
    painter.icon(rect.center(), icon, size, color);
}

fn insert(text: &mut String, state: &mut EditState, inserted: &str, max_length: Option<usize>) {
    let (start, end) = state.selection();
    let mut inserted = inserted.to_string();
    if let Some(max) = max_length {
        let room = max.saturating_sub(text.chars().count() - text[start..end].chars().count());
        inserted = inserted.chars().take(room).collect();
    }
    text.replace_range(start..end, &inserted);
    state.caret = start + inserted.len();
    state.anchor = state.caret;
}

fn previous_boundary(text: &str, index: usize) -> usize {
    text[..index].char_indices().next_back().map(|(at, _)| at).unwrap_or(0)
}

fn next_boundary(text: &str, index: usize) -> usize {
    text[index..].chars().next().map(|c| index + c.len_utf8()).unwrap_or(text.len())
}

fn is_word(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

// The start of the word before the index, past any spaces.
fn previous_word(text: &str, index: usize) -> usize {
    let mut at = index;
    let before: Vec<(usize, char)> = text[..index].char_indices().collect();
    let mut iter = before.iter().rev().peekable();
    while let Some(&&(position, character)) = iter.peek() {
        if is_word(character) {
            break;
        }
        at = position;
        iter.next();
    }
    while let Some(&&(position, character)) = iter.peek() {
        if !is_word(character) {
            break;
        }
        at = position;
        iter.next();
    }
    at
}

// The end of the word after the index, past any spaces.
fn next_word(text: &str, index: usize) -> usize {
    let mut at = index;
    let mut chars = text[index..].char_indices().peekable();
    while let Some(&(offset, character)) = chars.peek() {
        if is_word(character) {
            break;
        }
        at = index + offset + character.len_utf8();
        chars.next();
    }
    while let Some(&(offset, character)) = chars.peek() {
        if !is_word(character) {
            break;
        }
        at = index + offset + character.len_utf8();
        chars.next();
    }
    at
}

// The word around the index, for a double-click.
fn word_at(text: &str, index: usize) -> (usize, usize) {
    let mut start = index;
    for (position, character) in text[..index].char_indices().rev() {
        if !is_word(character) {
            break;
        }
        start = position;
    }
    let mut end = index;
    for (offset, character) in text[index..].char_indices() {
        if !is_word(character) {
            break;
        }
        end = index + offset + character.len_utf8();
    }
    (start, end)
}
