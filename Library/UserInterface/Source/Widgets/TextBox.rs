// WinUI's TextBox, one line: a field that takes typing while it has focus,
// with a caret that blinks, a selection that the pointer and Shift make,
// Ctrl's word moves, copy, cut and paste, a delete button while it has text
// and focus, and a button of its own, like AutoSuggestBox's query button.
//
// At rest its border is TextControlElevationBorderBrush, whose bottom edge is
// the strong stroke; with focus its fill is ControlFillColorInputActive and
// its bottom border is 2 pixels of the accent.

use crate::core::interaction::{ControlState, Cursor, Response, Sense};
use crate::core::ui::Ui;
use crate::design::controls::{TEXT_BOX_BUTTON_GLYPH, TEXT_BOX_BUTTON_MARGIN, TEXT_BOX_QUERY_BUTTON_WIDTH, TEXT_BOX_BUTTON_WIDTH, TEXT_BOX_CARET_WIDTH, TEXT_BOX_PADDING};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, RADIUS_CONTROL};
use crate::drawing::geometry::{Color, Insets, Point, Rect};
use crate::drawing::painter::Align;
use crate::input::devices::{Key, Modifiers};
use crate::widgets::widget::{Elevation, StateColors, Widget};

// How long the caret shows, and hides, as it blinks: Windows' default.
const CARET_BLINK: f64 = 0.53;

pub struct TextBox<'a, 'v> {
    text: &'v mut String,
    placeholder: &'a str,
    button: Option<Icon>,
    max_length: Option<usize>,
    clear_button: bool,
    accept: Option<fn(char) -> bool>,
    // Room at its right end for controls of another widget's, like a number
    // box's spin buttons.
    trailing: f32,
    // Room at its left end, like a color field's swatch.
    leading: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextBoxResponse {
    pub response: Response,
    // The text changed this frame.
    pub changed: bool,
    // Enter was pressed in it.
    pub submitted: bool,
    // Its own button was clicked.
    pub button_clicked: bool,
}

impl<'a, 'v> TextBox<'a, 'v> {
    pub fn new(text: &'v mut String) -> Self {
        Self {
            text,
            placeholder: "",
            button: None,
            max_length: None,
            clear_button: true,
            accept: None,
            trailing: 0.0,
            leading: 0.0,
        }
    }

    // What the field says while it's empty, in the secondary color.
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    // A button of its own at its right, named "<name>/Button", like a
    // search box's.
    pub fn button(mut self, icon: Icon) -> Self {
        self.button = Some(icon);
        self
    }

    // At most this many characters.
    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }

    pub fn without_clear_button(mut self) -> Self {
        self.clear_button = false;
        self
    }

    pub(crate) fn leading(mut self, width: f32) -> Self {
        self.leading = width;
        self
    }

    pub(crate) fn trailing(mut self, width: f32) -> Self {
        self.trailing = width;
        self
    }

    // Only characters that `accept` says yes to are typed or pasted.
    pub fn accept(mut self, accept: fn(char) -> bool) -> Self {
        self.accept = Some(accept);
        self
    }
}

// What a field keeps between frames: its caret and the other end of its
// selection, as byte indices, how far its text is scrolled, and when the
// caret last moved, which restarts its blinking.
#[derive(Default)]
struct Editing {
    caret: usize,
    anchor: usize,
    scroll: f32,
    moved: f64,
    selecting: bool,
}

impl Editing {
    fn selection(&self) -> (usize, usize) {
        (self.caret.min(self.anchor), self.caret.max(self.anchor))
    }

    fn has_selection(&self) -> bool {
        self.caret != self.anchor
    }
}

fn previous_boundary(text: &str, index: usize) -> usize {
    text[..index].char_indices().next_back().map_or(0, |(at, _)| at)
}

fn next_boundary(text: &str, index: usize) -> usize {
    text[index..].chars().next().map_or(text.len(), |character| index + character.len_utf8())
}

fn is_word(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

// The start of the word before the index, past any spaces, as Ctrl+Left goes.
fn previous_word(text: &str, index: usize) -> usize {
    let mut at = index;
    while at > 0 && !is_word(text[..at].chars().next_back().unwrap()) {
        at = previous_boundary(text, at);
    }
    while at > 0 && is_word(text[..at].chars().next_back().unwrap()) {
        at = previous_boundary(text, at);
    }
    at
}

// The start of the next word, past the rest of this one and spaces, as
// Ctrl+Right goes.
fn next_word(text: &str, index: usize) -> usize {
    let mut at = index;
    while at < text.len() && is_word(text[at..].chars().next().unwrap()) {
        at = next_boundary(text, at);
    }
    while at < text.len() && !is_word(text[at..].chars().next().unwrap()) {
        at = next_boundary(text, at);
    }
    at
}

// The byte index, or the start of the character it's in, if it's inside
// one, or the text's end, if it's past it.
fn boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

// The word around the index, for a double-click.
fn word_at(text: &str, index: usize) -> (usize, usize) {
    let mut start = index.min(text.len());
    while start > 0 && is_word(text[..start].chars().next_back().unwrap()) {
        start = previous_boundary(text, start);
    }
    let mut end = index.min(text.len());
    while end < text.len() && is_word(text[end..].chars().next().unwrap()) {
        end = next_boundary(text, end);
    }
    (start, end)
}

impl Widget for TextBox<'_, '_> {
    type Response = TextBoxResponse;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> TextBoxResponse {
        let mut result = TextBoxResponse::default();
        let mut button_space = self.trailing;
        if self.button.is_some() {
            button_space += TEXT_BOX_QUERY_BUTTON_WIDTH;
        }
        let response = ui.interact(name, rect, Sense::TEXT, "TextBox");
        let id = response.id;
        let focused = response.focused;
        let clear_shown = self.clear_button && focused && !self.text.is_empty() && response.enabled;
        if clear_shown {
            button_space += TEXT_BOX_BUTTON_WIDTH;
        }
        let content = Rect::new(
            rect.x + TEXT_BOX_PADDING.left + self.leading,
            rect.y,
            (rect.width - TEXT_BOX_PADDING.left - self.leading - if button_space > 0.0 { 2.0 } else { TEXT_BOX_PADDING.right } - button_space).max(0.0),
            rect.height,
        );
        let text_y = (rect.center().y - BODY.line_height / 2.0).round();
        let now = ui.time();
        let text_before = self.text.clone();
        let mut editing = std::mem::take(ui.memory::<Editing>(id));
        // The program may have changed the text since the last frame.
        editing.caret = boundary(self.text, editing.caret);
        editing.anchor = boundary(self.text, editing.anchor);

        // The pointer puts the caret where it presses, selects as it drags,
        // and selects a word with a double-click.
        if response.hovered && response.enabled {
            ui.set_cursor(Cursor::Text);
        }
        let hit = |ui: &mut Ui, point: Point, scroll: f32, text: &str| -> usize {
            let layout = ui.fonts().layout(text, BODY, None);
            layout.hit(Point::new(point.x - content.x + scroll, BODY.line_height / 2.0))
        };
        if response.pressed && let Some(pointer) = response.pointer {
            let index = hit(ui, pointer, editing.scroll, self.text);
            if response.double_clicked {
                let (start, end) = word_at(self.text, index);
                editing.anchor = start;
                editing.caret = end;
            } else {
                editing.caret = index;
                if !ui.input().modifiers().shift {
                    editing.anchor = index;
                }
                editing.selecting = true;
            }
            editing.moved = now;
        } else if response.held && editing.selecting && let Some(pointer) = response.pointer {
            editing.caret = hit(ui, pointer, editing.scroll, self.text);
        }
        if !response.held {
            editing.selecting = false;
        }

        if focused && response.enabled {
            // The frame's keys and text, apart from the input, which the
            // editing below borrows the interface away from.
            let modifiers = ui.input().modifiers();
            let keys: Vec<Key> = ui.input().keys_pressed().iter().map(|(key, _)| *key).collect();
            let text_typed = ui.input().text.clone();
            let control = modifiers.control;
            let shift = modifiers.shift;
            let mut moved = false;
            for key in keys {
                match key {
                    Key::LEFT => {
                        if editing.has_selection() && !shift {
                            editing.caret = editing.selection().0;
                        } else {
                            editing.caret = if control { previous_word(self.text, editing.caret) } else { previous_boundary(self.text, editing.caret) };
                        }
                        if !shift {
                            editing.anchor = editing.caret;
                        }
                        moved = true;
                    }
                    Key::RIGHT => {
                        if editing.has_selection() && !shift {
                            editing.caret = editing.selection().1;
                        } else {
                            editing.caret = if control { next_word(self.text, editing.caret) } else { next_boundary(self.text, editing.caret) };
                        }
                        if !shift {
                            editing.anchor = editing.caret;
                        }
                        moved = true;
                    }
                    Key::HOME | Key::END => {
                        editing.caret = if key == Key::HOME { 0 } else { self.text.len() };
                        if !shift {
                            editing.anchor = editing.caret;
                        }
                        moved = true;
                    }
                    Key::BACKSPACE | Key::DELETE => {
                        if !editing.has_selection() {
                            editing.anchor = match (key, control) {
                                (Key::BACKSPACE, true) => previous_word(self.text, editing.caret),
                                (Key::BACKSPACE, false) => previous_boundary(self.text, editing.caret),
                                (_, true) => next_word(self.text, editing.caret),
                                (_, false) => next_boundary(self.text, editing.caret),
                            };
                        }
                        replace_selection(self.text, &mut editing, "");
                        moved = true;
                    }
                    Key::A if modifiers == Modifiers::CONTROL => {
                        editing.anchor = 0;
                        editing.caret = self.text.len();
                        moved = true;
                    }
                    Key::C | Key::X if modifiers == Modifiers::CONTROL && editing.has_selection() => {
                        let (start, end) = editing.selection();
                        let copied = self.text[start..end].to_string();
                        ui.clipboard().set_text(&copied);
                        if key == Key::X {
                            replace_selection(self.text, &mut editing, "");
                        }
                        moved = true;
                    }
                    Key::V if modifiers == Modifiers::CONTROL => {
                        if let Some(pasted) = ui.clipboard().text() {
                            let line: String = pasted.chars().filter(|character| !character.is_control()).filter(|character| self.accept.is_none_or(|accept| accept(*character))).collect();
                            let line = limit(self.text, &editing, &line, self.max_length);
                            replace_selection(self.text, &mut editing, &line);
                        }
                        moved = true;
                    }
                    Key::ENTER => result.submitted = true,
                    _ => {}
                }
            }
            if !control {
                let typed: String = text_typed.chars().filter(|character| !character.is_control()).filter(|character| self.accept.is_none_or(|accept| accept(*character))).collect();
                if !typed.is_empty() {
                    let typed = limit(self.text, &editing, &typed, self.max_length);
                    replace_selection(self.text, &mut editing, &typed);
                    moved = true;
                }
            }
            if moved {
                editing.moved = now;
            }
        } else if !focused {
            editing.anchor = editing.caret;
        }

        // The buttons: delete, then the field's own. They're drawn first,
        // and the field is moved behind them once it's drawn.
        let buttons_first = ui.draw.layer_len();
        let mut button_x = rect.right() - self.trailing;
        if let Some(icon) = self.button {
            // AutoSuggestBox's query button: 32 by 28, inset 1 by 3.
            button_x -= TEXT_BOX_QUERY_BUTTON_WIDTH;
            let column = Rect::new(button_x, rect.center().y - 14.0, TEXT_BOX_QUERY_BUTTON_WIDTH, 28.0);
            let button = column.pad(Insets::new(1.0, 3.0, 1.0, 3.0));
            result.button_clicked = text_control_button(ui, &format!("{name}/Button"), button, icon).clicked;
        }
        if clear_shown {
            button_x -= TEXT_BOX_BUTTON_WIDTH;
            let column = Rect::new(button_x, rect.y, TEXT_BOX_BUTTON_WIDTH, rect.height);
            if text_control_button(ui, &format!("{name}/Clear"), column.pad(TEXT_BOX_BUTTON_MARGIN), Icon::Clear).clicked {
                self.text.clear();
                editing.caret = 0;
                editing.anchor = 0;
            }
        }

        // The text scrolls to keep the caret in sight.
        let caret_x = ui.fonts().layout(self.text, BODY, None).caret_x(editing.caret);
        let text_width = ui.measure(self.text, BODY).width;
        if caret_x - editing.scroll > content.width - TEXT_BOX_CARET_WIDTH {
            editing.scroll = caret_x - content.width + TEXT_BOX_CARET_WIDTH;
        }
        if caret_x - editing.scroll < 0.0 {
            editing.scroll = caret_x;
        }
        editing.scroll = editing.scroll.clamp(0.0, (text_width - content.width + TEXT_BOX_CARET_WIDTH).max(0.0));

        // Drawing, behind the buttons.
        let state = response.state();
        let theme = ui.theme();
        let fill = if focused && response.enabled {
            theme.control_fill_input_active
        } else {
            StateColors::new(theme.control_fill, theme.control_fill_secondary, theme.control_fill_secondary, theme.control_fill_disabled).get(state)
        };
        let field_first = ui.draw.layer_len();
        let mut painter = ui.painter();
        let theme = painter.theme();
        let border = if state == ControlState::Disabled {
            crate::drawing::painter::Border::new(theme.control_stroke, 1.0)
        } else if focused {
            Elevation::TextControlFocused.border(theme, rect)
        } else {
            Elevation::TextControl.border(theme, rect)
        };
        painter.shape(rect, RADIUS_CONTROL, fill, Some(border));
        painter.push_clip(content.expand(1.0).intersect(rect));
        let origin = Point::new(content.x - editing.scroll, text_y);
        if self.text.is_empty() {
            if !self.placeholder.is_empty() {
                let color = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_secondary };
                painter.text_in(Rect::new(content.x, rect.y, content.width, rect.height), self.placeholder, BODY, color, Align::Start);
            }
        } else {
            let color = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
            painter.text(origin, self.text, BODY, color);
            if focused && editing.has_selection() {
                let (start, end) = editing.selection();
                let layout = painter.fonts().layout(self.text, BODY, None);
                let (left, right) = (layout.caret_x(start), layout.caret_x(end));
                let selected = Rect::new(origin.x + left, text_y, right - left, BODY.line_height);
                painter.fill(selected, theme.accent_selected_text_background);
                painter.push_clip(selected);
                painter.text(origin, self.text, BODY, theme.text_on_accent_selected);
                painter.pop_clip();
            }
        }
        if focused && response.enabled && (((now - editing.moved) / CARET_BLINK) as u64).is_multiple_of(2) {
            let caret = Rect::new((origin.x + caret_x).round(), text_y + 2.0, TEXT_BOX_CARET_WIDTH, BODY.line_height - 4.0);
            painter.fill(caret, theme.text_primary);
        }
        painter.pop_clip();
        ui.draw.move_to_back(buttons_first, field_first);

        result.changed = *self.text != text_before;
        result.response = Response { changed: result.changed, ..response };
        *ui.memory::<Editing>(id) = editing;
        ui.set_label(name, self.text);
        result
    }
}

// The text, less what would make the field longer than its limit.
fn limit(text: &str, editing: &Editing, insert: &str, max_length: Option<usize>) -> String {
    let Some(max) = max_length else {
        return insert.to_string();
    };
    let (start, end) = editing.selection();
    let kept = text.chars().count() - text[start..end].chars().count();
    insert.chars().take(max.saturating_sub(kept)).collect()
}

fn replace_selection(text: &mut String, editing: &mut Editing, with: &str) {
    let (start, end) = editing.selection();
    text.replace_range(start..end, with);
    editing.caret = start + with.len();
    editing.anchor = editing.caret;
}

// A button inside a text field, like its delete button: transparent until
// the pointer is over it, with its glyph in the secondary color, as
// TextControlButton's brushes are. It never takes focus from the field.
pub(crate) fn text_control_button(ui: &mut Ui, name: &str, rect: Rect, icon: Icon) -> Response {
    let response = ui.interact(name, rect, Sense::CLICK, "Button");
    let state = response.state();
    let theme = ui.theme();
    let background = StateColors::new(Color::TRANSPARENT, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, Color::TRANSPARENT).get(state);
    let foreground = StateColors::new(theme.text_secondary, theme.text_secondary, theme.text_tertiary, theme.text_disabled).get(state);
    let mut painter = ui.painter();
    painter.rounded(rect, RADIUS_CONTROL, background);
    painter.icon(rect.center(), icon, TEXT_BOX_BUTTON_GLYPH, foreground);
    response
}
