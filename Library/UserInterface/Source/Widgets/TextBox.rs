// WinUI's TextBox, one line: a field that takes typing while it has focus,
// with a caret that blinks, a selection that the pointer and Shift make,
// Ctrl's word moves, copy, cut and paste, a delete button while it has text
// and focus, and a button of its own, like AutoSuggestBox's query button.
//
// At rest its border is TextControlElevationBorderBrush, whose bottom edge is
// the strong stroke; with focus its fill is ControlFillColorInputActive and
// its bottom border is 2 pixels of the accent.
//
// The field owns its text. `input` and `paint` are the dispatch and draw
// halves, which NumberBox and ColorField compose too.

use crate::core::control::{Control, NodeContext, UiEventKind, adopted};
use crate::core::ui::{ControlState, Cursor, Response, Sense, Ui};
use crate::design::controls::{TEXT_BOX_BUTTON_GLYPH, TEXT_BOX_BUTTON_MARGIN, TEXT_BOX_QUERY_BUTTON_WIDTH, TEXT_BOX_BUTTON_WIDTH, TEXT_BOX_CARET_WIDTH, TEXT_BOX_PADDING};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, DURATION_FASTER, RADIUS_CONTROL};
use crate::drawing::geometry::{Color, Insets, Point, Rect};
use crate::drawing::painter::Align;
use crate::input::devices::{Key, Modifiers};
use crate::widgets::widget::{Elevation, StateColors};

// How long the caret shows, and hides, as it blinks: Windows' default.
const CARET_BLINK: f64 = 0.53;

pub struct TextBox {
    text: String,
    placeholder: String,
    button: Option<Icon>,
    max_length: Option<usize>,
    clear_button: bool,
    accept: Option<fn(char) -> bool>,
    // Room at its right end for controls of another widget's, like a number
    // box's spin buttons.
    trailing: f32,
    // Room at its left end, like a color field's swatch.
    leading: f32,
    editing: Editing,
    // The buttons' responses, from input, for paint.
    button_response: Option<Response>,
    clear_response: Option<Response>,
}

// What a field's input half tells its owner.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextBoxOutcome {
    // The text changed this frame.
    pub changed: bool,
    // Enter was pressed in it.
    pub submitted: bool,
    // Its own button was clicked.
    pub button_clicked: bool,
}

impl TextBox {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            placeholder: String::new(),
            button: None,
            max_length: None,
            clear_button: true,
            accept: None,
            trailing: 0.0,
            leading: 0.0,
            editing: Editing::default(),
            button_response: None,
            clear_response: None,
        }
    }

    // What the field says while it's empty, in the secondary color.
    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    // A button of its own at its right, named "<name>/Button", like a
    // search box's, whose clicks arrive as Clicked events from that name.
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

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.editing.caret = boundary(&self.text, self.editing.caret);
        self.editing.anchor = boundary(&self.text, self.editing.anchor);
    }

    // The input half: the caret, the selection, typing, the clipboard, and
    // the field's buttons.
    pub(crate) fn input(&mut self, ui: &mut Ui, name: &str, rect: Rect, response: &Response) -> TextBoxOutcome {
        let mut outcome = TextBoxOutcome::default();
        let focused = response.focused;
        let content = self.content_rect(rect, focused, response.enabled);
        let now = ui.time();
        let text_before = self.text.clone();
        let editing = &mut self.editing;
        // The program may have changed the text since the last frame.
        editing.caret = boundary(&self.text, editing.caret);
        editing.anchor = boundary(&self.text, editing.anchor);

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
            let index = hit(ui, pointer, editing.scroll, &self.text);
            if response.double_clicked {
                let (start, end) = word_at(&self.text, index);
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
            editing.caret = hit(ui, pointer, editing.scroll, &self.text);
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
                            editing.caret = if control { previous_word(&self.text, editing.caret) } else { previous_boundary(&self.text, editing.caret) };
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
                            editing.caret = if control { next_word(&self.text, editing.caret) } else { next_boundary(&self.text, editing.caret) };
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
                                (Key::BACKSPACE, true) => previous_word(&self.text, editing.caret),
                                (Key::BACKSPACE, false) => previous_boundary(&self.text, editing.caret),
                                (_, true) => next_word(&self.text, editing.caret),
                                (_, false) => next_boundary(&self.text, editing.caret),
                            };
                        }
                        replace_selection(&mut self.text, editing, "");
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
                            replace_selection(&mut self.text, editing, "");
                        }
                        moved = true;
                    }
                    Key::V if modifiers == Modifiers::CONTROL => {
                        if let Some(pasted) = ui.clipboard().text() {
                            let line: String = pasted.chars().filter(|character| !character.is_control()).filter(|character| self.accept.is_none_or(|accept| accept(*character))).collect();
                            let line = limit(&self.text, editing, &line, self.max_length);
                            replace_selection(&mut self.text, editing, &line);
                        }
                        moved = true;
                    }
                    Key::ENTER => outcome.submitted = true,
                    _ => {}
                }
            }
            if !control {
                let typed: String = text_typed.chars().filter(|character| !character.is_control()).filter(|character| self.accept.is_none_or(|accept| accept(*character))).collect();
                if !typed.is_empty() {
                    let typed = limit(&self.text, editing, &typed, self.max_length);
                    replace_selection(&mut self.text, editing, &typed);
                    moved = true;
                }
            }
            if moved {
                editing.moved = now;
            }
        } else if !focused {
            editing.anchor = editing.caret;
        }

        // The buttons: the field's own, then delete.
        let mut button_x = rect.right() - self.trailing;
        self.button_response = None;
        self.clear_response = None;
        if self.button.is_some() {
            // AutoSuggestBox's query button: 32 by 28, inset 1 by 3.
            button_x -= TEXT_BOX_QUERY_BUTTON_WIDTH;
            let column = Rect::new(button_x, rect.center().y - 14.0, TEXT_BOX_QUERY_BUTTON_WIDTH, 28.0);
            let button = column.pad(Insets::new(1.0, 3.0, 1.0, 3.0));
            let button_response = ui.interact(&format!("{name}/Button"), button, Sense::CLICK, "Button");
            outcome.button_clicked = button_response.clicked;
            self.button_response = Some(button_response);
        }
        if self.clear_shown(response) {
            button_x -= TEXT_BOX_BUTTON_WIDTH;
            let column = Rect::new(button_x, rect.y, TEXT_BOX_BUTTON_WIDTH, rect.height);
            let clear_response = ui.interact(&format!("{name}/Clear"), column.pad(TEXT_BOX_BUTTON_MARGIN), Sense::CLICK, "Button");
            if clear_response.clicked {
                self.text.clear();
                self.editing.caret = 0;
                self.editing.anchor = 0;
            }
            self.clear_response = Some(clear_response);
        }

        // The text scrolls to keep the caret in sight.
        let editing = &mut self.editing;
        let caret_x = ui.fonts().layout(&self.text, BODY, None).caret_x(editing.caret);
        let text_width = ui.measure(&self.text, BODY).width;
        if caret_x - editing.scroll > content.width - TEXT_BOX_CARET_WIDTH {
            editing.scroll = caret_x - content.width + TEXT_BOX_CARET_WIDTH;
        }
        if caret_x - editing.scroll < 0.0 {
            editing.scroll = caret_x;
        }
        editing.scroll = editing.scroll.clamp(0.0, (text_width - content.width + TEXT_BOX_CARET_WIDTH).max(0.0));

        outcome.changed = self.text != text_before;
        ui.set_label(name, &self.text.clone());
        outcome
    }

    // The draw half: the field, its text, its selection, its caret, and its
    // buttons over it.
    pub(crate) fn paint(&mut self, ui: &mut Ui, rect: Rect, response: &Response) {
        let focused = response.focused;
        let content = self.content_rect(rect, focused, response.enabled);
        let text_y = (rect.center().y - BODY.line_height / 2.0).round();
        let now = ui.time();
        let state = response.state();
        let theme = ui.theme();
        let fill = if focused && response.enabled {
            theme.control_fill_input_active
        } else {
            StateColors::new(theme.control_fill, theme.control_fill_secondary, theme.control_fill_secondary, theme.control_fill_disabled).get(state)
        };
        let fill = ui.animate_color(response.id, 0, fill, DURATION_FASTER);
        let caret_x = ui.fonts().layout(&self.text, BODY, None).caret_x(self.editing.caret);
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
        let origin = Point::new(content.x - self.editing.scroll, text_y);
        if self.text.is_empty() {
            if !self.placeholder.is_empty() {
                let color = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_secondary };
                painter.text_in(Rect::new(content.x, rect.y, content.width, rect.height), &self.placeholder, BODY, color, Align::Start);
            }
        } else {
            let color = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
            painter.text(origin, &self.text, BODY, color);
            if focused && self.editing.has_selection() {
                let (start, end) = self.editing.selection();
                let layout = painter.fonts().layout(&self.text, BODY, None);
                let (left, right) = (layout.caret_x(start), layout.caret_x(end));
                let selected = Rect::new(origin.x + left, text_y, right - left, BODY.line_height);
                painter.fill(selected, theme.accent_selected_text_background);
                painter.push_clip(selected);
                painter.text(origin, &self.text, BODY, theme.text_on_accent_selected);
                painter.pop_clip();
            }
        }
        if focused && response.enabled && (((now - self.editing.moved) / CARET_BLINK) as u64).is_multiple_of(2) {
            let caret = Rect::new((origin.x + caret_x).round(), text_y + 2.0, TEXT_BOX_CARET_WIDTH, BODY.line_height - 4.0);
            painter.fill(caret, theme.text_primary);
        }
        painter.pop_clip();
        if let (Some(icon), Some(button_response)) = (self.button, self.button_response) {
            draw_text_control_button(ui, &button_response, icon);
        }
        if let Some(clear_response) = self.clear_response {
            draw_text_control_button(ui, &clear_response, Icon::Clear);
        }
    }

    fn clear_shown(&self, response: &Response) -> bool {
        self.clear_button && response.focused && !self.text.is_empty() && response.enabled
    }

    fn content_rect(&self, rect: Rect, focused: bool, enabled: bool) -> Rect {
        let mut button_space = self.trailing;
        if self.button.is_some() {
            button_space += TEXT_BOX_QUERY_BUTTON_WIDTH;
        }
        if self.clear_button && focused && !self.text.is_empty() && enabled {
            button_space += TEXT_BOX_BUTTON_WIDTH;
        }
        Rect::new(
            rect.x + TEXT_BOX_PADDING.left + self.leading,
            rect.y,
            (rect.width - TEXT_BOX_PADDING.left - self.leading - if button_space > 0.0 { 2.0 } else { TEXT_BOX_PADDING.right } - button_space).max(0.0),
            rect.height,
        )
    }
}

impl Control for TextBox {
    fn kind(&self) -> &'static str {
        "TextBox"
    }

    fn sense(&self) -> Sense {
        Sense::TEXT
    }

    fn label(&self) -> Option<String> {
        Some(self.text.clone())
    }

    fn adopt(&mut self, previous: Box<dyn Control>) {
        if let Some(previous) = adopted::<TextBox>(previous) {
            self.text = previous.text;
            self.editing = previous.editing;
        }
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let outcome = self.input(ui, &ctx.name, ctx.rect, &ctx.response);
        if outcome.changed {
            ctx.emit(ui, UiEventKind::TextChanged(self.text.clone()));
        }
        if outcome.submitted {
            ctx.emit(ui, UiEventKind::Submitted(self.text.clone()));
        }
        if outcome.button_clicked {
            ui.emit(&format!("{}/Button", ctx.name), UiEventKind::Clicked);
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        self.paint(ui, ctx.rect, &ctx.response);
    }
}

// What a field keeps between frames: its caret and the other end of its
// selection, as byte indices, how far its text is scrolled, and when the
// caret last moved, which restarts its blinking.
#[derive(Clone, Copy, Debug, Default)]
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
// TextControlButton's brushes are. It never takes focus from the field. The
// interact half runs in dispatch; this draws it from its response.
pub(crate) fn draw_text_control_button(ui: &mut Ui, response: &Response, icon: Icon) {
    let state = response.state();
    let theme = ui.theme();
    let background = StateColors::new(Color::TRANSPARENT, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, Color::TRANSPARENT).get(state);
    let foreground = StateColors::new(theme.text_secondary, theme.text_secondary, theme.text_tertiary, theme.text_disabled).get(state);
    let background = ui.animate_color(response.id, 0, background, DURATION_FASTER);
    let mut painter = ui.painter();
    painter.rounded(response.rect, RADIUS_CONTROL, background);
    painter.icon(response.rect.center(), icon, TEXT_BOX_BUTTON_GLYPH, foreground);
}
