// WinUI's TextBox, one line: a field that takes typing while it has focus,
// with a caret that blinks, a selection that the pointer and Shift make,
// Ctrl's word moves, copy, cut and paste, a delete button while it has text
// and focus, and a button of its own, like AutoSuggestBox's query button.
//
// At rest its border is TextControlElevationBorderBrush, whose bottom edge is
// the strong stroke; with focus its fill is ControlFillColorInputActive and
// its bottom border is 2 pixels of the accent.
//
// The text is the program's: the field sends the text as the typing leaves
// it, and the program keeps it. The caret, the selection and the scroll are
// the field's own, kept between frames. The editing, the buttons and the
// drawing are shared with the widgets built of a field, NumberBox and
// ColorField.

use crate::core::interaction::{ControlState, Cursor, Interaction, Part, Sense};
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{TEXT_BOX_BUTTON_GLYPH, TEXT_BOX_BUTTON_MARGIN, TEXT_BOX_BUTTON_WIDTH, TEXT_BOX_CARET_WIDTH, TEXT_BOX_PADDING, TEXT_BOX_QUERY_BUTTON_WIDTH};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, RADIUS_CONTROL};
use crate::drawing::geometry::{Insets, Point, Rect};
use crate::drawing::painter::Align;
use crate::input::devices::{Key, Modifiers};
use crate::widgets::buttons::draw_text_control_button;
use crate::widgets::widget::{Elevation, Reaction, StateColors, react};

// How long the caret shows, and hides, as it blinks: Windows' default.
const CARET_BLINK: f64 = 0.53;

// How a field is set up: what the text box and the widgets built of it
// share.
#[derive(Clone)]
pub(crate) struct FieldSettings {
    pub placeholder: String,
    pub button: Option<Icon>,
    pub max_length: Option<usize>,
    pub clear_button: bool,
    pub accept: Option<fn(char) -> bool>,
    // Room at its right end for controls of another widget's, like a number
    // box's spin buttons, and at its left, like a color field's swatch.
    pub trailing: f32,
    pub leading: f32,
}

impl Default for FieldSettings {
    fn default() -> Self {
        Self {
            placeholder: String::new(),
            button: None,
            max_length: None,
            clear_button: true,
            accept: None,
            trailing: 0.0,
            leading: 0.0,
        }
    }
}

pub struct TextBox<M> {
    text: String,
    settings: FieldSettings,
    on_change: Option<Reaction<String, M>>,
    on_submit: Option<M>,
    on_button: Option<M>,
}

impl<M: Clone + 'static> TextBox<M> {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            settings: FieldSettings::default(),
            on_change: None,
            on_submit: None,
            on_button: None,
        }
    }

    // What typing sends, with the text as it is then.
    pub fn on_change(mut self, reaction: impl Fn(String) -> M + 'static) -> Self {
        self.on_change = Some(react(reaction));
        self
    }

    // What Enter in the field sends.
    pub fn on_submit(mut self, message: M) -> Self {
        self.on_submit = Some(message);
        self
    }

    // What a click on its own button sends.
    pub fn on_button(mut self, message: M) -> Self {
        self.on_button = Some(message);
        self
    }

    // What the field says while it's empty, in the secondary color.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.settings.placeholder = placeholder.into();
        self
    }

    // A button of its own at its right, named "<name>/Button", like a
    // search box's.
    pub fn button(mut self, icon: Icon) -> Self {
        self.settings.button = Some(icon);
        self
    }

    // At most this many characters.
    pub fn max_length(mut self, length: usize) -> Self {
        self.settings.max_length = Some(length);
        self
    }

    pub fn without_clear_button(mut self) -> Self {
        self.settings.clear_button = false;
        self
    }

    // Only characters that `accept` says yes to are typed or pasted.
    pub fn accept(mut self, accept: fn(char) -> bool) -> Self {
        self.settings.accept = Some(accept);
        self
    }
}

// What a field keeps between frames: its caret and the other end of its
// selection, as byte indices, how far its text is scrolled, and when the
// caret last moved, which restarts its blinking.
#[derive(Default)]
pub(crate) struct Editing {
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

// Where a field's content, clear button and own button are.
struct FieldLayout {
    content: Rect,
    clear: Option<Rect>,
    button: Option<Rect>,
}

fn field_layout(rect: Rect, settings: &FieldSettings, clear_shown: bool) -> FieldLayout {
    let mut button_space = settings.trailing;
    let mut button_x = rect.right() - settings.trailing;
    let mut button = None;
    if let Some(_icon) = settings.button {
        button_space += TEXT_BOX_QUERY_BUTTON_WIDTH;
        // AutoSuggestBox's query button: 32 by 28, inset 1 by 3.
        button_x -= TEXT_BOX_QUERY_BUTTON_WIDTH;
        let column = Rect::new(button_x, rect.center().y - 14.0, TEXT_BOX_QUERY_BUTTON_WIDTH, 28.0);
        button = Some(column.pad(Insets::new(1.0, 3.0, 1.0, 3.0)));
    }
    let mut clear = None;
    if clear_shown {
        button_space += TEXT_BOX_BUTTON_WIDTH;
        button_x -= TEXT_BOX_BUTTON_WIDTH;
        let column = Rect::new(button_x, rect.y, TEXT_BOX_BUTTON_WIDTH, rect.height);
        clear = Some(column.pad(TEXT_BOX_BUTTON_MARGIN));
    }
    let content = Rect::new(
        rect.x + TEXT_BOX_PADDING.left + settings.leading,
        rect.y,
        (rect.width - TEXT_BOX_PADDING.left - settings.leading - if button_space > 0.0 { 2.0 } else { TEXT_BOX_PADDING.right } - button_space).max(0.0),
        rect.height,
    );
    FieldLayout { content, clear, button }
}

// What the frame's input did to a field.
#[derive(Default)]
pub(crate) struct Edited {
    // The text as the typing leaves it, if it changed.
    pub text: Option<String>,
    pub submitted: bool,
    pub button_clicked: bool,
}

// Registers a field's buttons as parts: its clear button while it has
// text and focus, and its own button.
pub(crate) fn field_parts(ctx: &mut Parts, text: &str, settings: &FieldSettings) {
    let clear_shown = settings.clear_button && ctx.is_focused() && !text.is_empty() && ctx.enabled();
    let layout = field_layout(ctx.rect(), settings, clear_shown);
    if let Some(button) = layout.button {
        ctx.part(Part::Named("Button".to_string()), "Button", button, Sense::CLICK, "Button", None);
    }
    if let Some(clear) = layout.clear {
        ctx.part(Part::Named("Clear".to_string()), "Clear", clear, Sense::CLICK, "Button", None);
    }
}

// Does what the input did to a field with the text, and says how the text
// came out.
pub(crate) fn field_interact<M>(text: &str, settings: &FieldSettings, interaction: &Interaction, ctx: &mut Dispatch<M>) -> Edited {
    let mut edited = Edited::default();
    let hit = interaction.whole();
    let focused = interaction.focused;
    let clear_shown = settings.clear_button && ctx.is_focused() && !text.is_empty() && ctx.enabled();
    let layout = field_layout(ctx.rect(), settings, clear_shown);
    let content = layout.content;
    let now = ctx.time();
    let mut text = text.to_string();
    let text_before = text.clone();
    let mut editing = std::mem::take(ctx.memory::<Editing>());
    // The program may have changed the text since the last frame.
    editing.caret = boundary(&text, editing.caret);
    editing.anchor = boundary(&text, editing.anchor);

    // The pointer puts the caret where it presses, selects as it drags,
    // and selects a word with a double-click.
    if hit.hovered && interaction.enabled {
        ctx.set_cursor(Cursor::Text);
    }
    if hit.pressed && let Some(pointer) = interaction.pointer {
        let index = {
            let layout = ctx.fonts().layout(&text, BODY, None);
            layout.hit(Point::new(pointer.x - content.x + editing.scroll, BODY.line_height / 2.0))
        };
        if hit.double_clicked {
            let (start, end) = word_at(&text, index);
            editing.anchor = start;
            editing.caret = end;
        } else {
            editing.caret = index;
            if !ctx.modifiers().shift {
                editing.anchor = index;
            }
            editing.selecting = true;
        }
        editing.moved = now;
    } else if hit.held && editing.selecting && let Some(pointer) = interaction.pointer {
        let layout = ctx.fonts().layout(&text, BODY, None);
        editing.caret = layout.hit(Point::new(pointer.x - content.x + editing.scroll, BODY.line_height / 2.0));
    }
    if !hit.held {
        editing.selecting = false;
    }

    if focused && interaction.enabled {
        let modifiers = ctx.modifiers();
        let keys: Vec<Key> = ctx.input().keys_pressed().iter().map(|(key, _)| *key).collect();
        let text_typed = ctx.input().text.clone();
        let control = modifiers.control;
        let shift = modifiers.shift;
        let mut moved = false;
        for key in keys {
            match key {
                Key::LEFT => {
                    if editing.has_selection() && !shift {
                        editing.caret = editing.selection().0;
                    } else {
                        editing.caret = if control { previous_word(&text, editing.caret) } else { previous_boundary(&text, editing.caret) };
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
                        editing.caret = if control { next_word(&text, editing.caret) } else { next_boundary(&text, editing.caret) };
                    }
                    if !shift {
                        editing.anchor = editing.caret;
                    }
                    moved = true;
                }
                Key::HOME | Key::END => {
                    editing.caret = if key == Key::HOME { 0 } else { text.len() };
                    if !shift {
                        editing.anchor = editing.caret;
                    }
                    moved = true;
                }
                Key::BACKSPACE | Key::DELETE => {
                    if !editing.has_selection() {
                        editing.anchor = match (key, control) {
                            (Key::BACKSPACE, true) => previous_word(&text, editing.caret),
                            (Key::BACKSPACE, false) => previous_boundary(&text, editing.caret),
                            (_, true) => next_word(&text, editing.caret),
                            (_, false) => next_boundary(&text, editing.caret),
                        };
                    }
                    replace_selection(&mut text, &mut editing, "");
                    moved = true;
                }
                Key::A if modifiers == Modifiers::CONTROL => {
                    editing.anchor = 0;
                    editing.caret = text.len();
                    moved = true;
                }
                Key::C | Key::X if modifiers == Modifiers::CONTROL && editing.has_selection() => {
                    let (start, end) = editing.selection();
                    let copied = text[start..end].to_string();
                    // A cut takes the text away only once it's on the
                    // clipboard, so that a clipboard that another program
                    // holds open doesn't lose it.
                    if ctx.clipboard().set_text(&copied) && key == Key::X {
                        replace_selection(&mut text, &mut editing, "");
                    }
                    moved = true;
                }
                Key::V if modifiers == Modifiers::CONTROL => {
                    if let Some(pasted) = ctx.clipboard().text() {
                        let line: String = pasted.chars().filter(|character| !character.is_control()).filter(|character| settings.accept.is_none_or(|accept| accept(*character))).collect();
                        let line = limit(&text, &editing, &line, settings.max_length);
                        replace_selection(&mut text, &mut editing, &line);
                    }
                    moved = true;
                }
                Key::ENTER => edited.submitted = true,
                _ => {}
            }
        }
        if !control {
            let typed: String = text_typed.chars().filter(|character| !character.is_control()).filter(|character| settings.accept.is_none_or(|accept| accept(*character))).collect();
            if !typed.is_empty() {
                let typed = limit(&text, &editing, &typed, settings.max_length);
                replace_selection(&mut text, &mut editing, &typed);
                moved = true;
            }
        }
        if moved {
            editing.moved = now;
        }
    } else if !focused {
        editing.anchor = editing.caret;
    }

    // The buttons: delete, and the field's own.
    if interaction.pressed(&Part::Named("Button".to_string())) {
        edited.button_clicked = true;
    }
    if interaction.pressed(&Part::Named("Clear".to_string())) {
        text.clear();
        editing.caret = 0;
        editing.anchor = 0;
    }
    *ctx.memory::<Editing>() = editing;
    if text != text_before {
        edited.text = Some(text);
    }
    edited
}

// Draws a field with the text, as it looks.
pub(crate) fn field_draw(ctx: &mut Draw, text: &str, settings: &FieldSettings) {
    let rect = ctx.rect();
    let state = ctx.state();
    let focused = ctx.focused();
    let enabled = ctx.enabled();
    let now = ctx.time();
    let clear_shown = settings.clear_button && focused && !text.is_empty() && enabled;
    let layout = field_layout(rect, settings, clear_shown);
    let content = layout.content;
    let text_y = (rect.center().y - BODY.line_height / 2.0).round();

    // The text scrolls to keep the caret in sight.
    let mut editing = std::mem::take(ctx.memory::<Editing>());
    editing.caret = boundary(text, editing.caret);
    editing.anchor = boundary(text, editing.anchor);
    let caret_x = ctx.fonts().layout(text, BODY, None).caret_x(editing.caret);
    let text_width = ctx.measure(text, BODY).width;
    if caret_x - editing.scroll > content.width - TEXT_BOX_CARET_WIDTH {
        editing.scroll = caret_x - content.width + TEXT_BOX_CARET_WIDTH;
    }
    if caret_x - editing.scroll < 0.0 {
        editing.scroll = caret_x;
    }
    editing.scroll = editing.scroll.clamp(0.0, (text_width - content.width + TEXT_BOX_CARET_WIDTH).max(0.0));

    let theme = ctx.theme();
    let fill = if focused && enabled {
        theme.control_fill_input_active
    } else {
        StateColors::new(theme.control_fill, theme.control_fill_secondary, theme.control_fill_secondary, theme.control_fill_disabled).get(state)
    };
    let (placeholder_color, text_color, selection_fill, selected_text, caret_color) = (
        if state == ControlState::Disabled { theme.text_disabled } else { theme.text_secondary },
        if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary },
        theme.accent_selected_text_background,
        theme.text_on_accent_selected,
        theme.text_primary,
    );
    let border = if state == ControlState::Disabled {
        crate::drawing::painter::Border::new(theme.control_stroke, 1.0)
    } else if focused {
        Elevation::TextControlFocused.border(theme, rect)
    } else {
        Elevation::TextControl.border(theme, rect)
    };
    {
        let mut painter = ctx.painter();
        painter.shape(rect, RADIUS_CONTROL, fill, Some(border));
        painter.push_clip(content.expand(1.0).intersect(rect));
        let origin = Point::new(content.x - editing.scroll, text_y);
        if text.is_empty() {
            if !settings.placeholder.is_empty() {
                painter.text_in(Rect::new(content.x, rect.y, content.width, rect.height), &settings.placeholder, BODY, placeholder_color, Align::Start);
            }
        } else {
            painter.text(origin, text, BODY, text_color);
            if focused && editing.has_selection() {
                let (start, end) = editing.selection();
                let (left, right) = {
                    let layout = painter.fonts().layout(text, BODY, None);
                    (layout.caret_x(start), layout.caret_x(end))
                };
                let selected = Rect::new(origin.x + left, text_y, right - left, BODY.line_height);
                painter.fill(selected, selection_fill);
                painter.push_clip(selected);
                painter.text(origin, text, BODY, selected_text);
                painter.pop_clip();
            }
        }
        if focused && enabled && (((now - editing.moved) / CARET_BLINK) as u64).is_multiple_of(2) {
            let caret = Rect::new((origin.x + caret_x).round(), text_y + 2.0, TEXT_BOX_CARET_WIDTH, BODY.line_height - 4.0);
            painter.fill(caret, caret_color);
        }
        painter.pop_clip();
    }
    *ctx.memory::<Editing>() = editing;
    if let Some(button) = layout.button
        && let Some(icon) = settings.button
    {
        draw_text_control_button(ctx, &Part::Named("Button".to_string()), button, icon, TEXT_BOX_BUTTON_GLYPH);
    }
    if let Some(clear) = layout.clear {
        draw_text_control_button(ctx, &Part::Named("Clear".to_string()), clear, Icon::Clear, TEXT_BOX_BUTTON_GLYPH);
    }
}

impl<M: Clone + 'static> Widget<M> for TextBox<M> {
    fn kind(&self) -> &'static str {
        "TextBox"
    }

    fn label(&self) -> Option<String> {
        Some(self.text.clone())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::TEXT)
    }

    fn parts(&self, ctx: &mut Parts) {
        field_parts(ctx, &self.text, &self.settings);
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let edited = field_interact(&self.text, &self.settings, interaction, ctx);
        if let Some(text) = edited.text
            && let Some(reaction) = &self.on_change
        {
            ctx.send(reaction(text));
        }
        if edited.submitted && let Some(message) = &self.on_submit {
            ctx.send(message.clone());
        }
        if edited.button_clicked && let Some(message) = &self.on_button {
            ctx.send(message.clone());
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        field_draw(ctx, &self.text, &self.settings);
    }
}
