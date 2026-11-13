// WinUI's TextBox, one line: a field that takes typing while it has focus,
// at a caret that blinks, which a press places and Backspace deletes
// behind, with a delete button while it has text and focus, and a button of
// its own, like AutoSuggestBox's query button.
//
// At rest its border is TextControlElevationBorderBrush, whose bottom edge is
// the strong stroke; with focus its fill is ControlFillColorInputActive and
// its bottom border is 2 pixels of the accent.
//
// The text is the program's: the field sends the text as the typing leaves
// it, and the program keeps it. The caret and the scroll are the field's
// own, kept between frames. The editing, the buttons and the drawing are
// shared with the widgets built of a field, NumberBox and ColorField, for
// which Enter takes what's typed.

use crate::core::interaction::{ControlState, Cursor, Interaction, Part, Sense};
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{TEXT_BOX_BUTTON_GLYPH, TEXT_BOX_BUTTON_MARGIN, TEXT_BOX_BUTTON_WIDTH, TEXT_BOX_CARET_WIDTH, TEXT_BOX_PADDING, TEXT_BOX_QUERY_BUTTON_WIDTH};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, RADIUS_CONTROL};
use crate::drawing::geometry::{Insets, Point, Rect};
use crate::drawing::painter::Align;
use crate::input::devices::Key;
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
    on_button: Option<M>,
}

impl<M: Clone + 'static> TextBox<M> {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            settings: FieldSettings::default(),
            on_change: None,
            on_button: None,
        }
    }

    // What typing sends, with the text as it is then.
    pub fn on_change(mut self, reaction: impl Fn(String) -> M + 'static) -> Self {
        self.on_change = Some(react(reaction));
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

// What a field keeps between frames: its caret, as a byte index, how far its
// text is scrolled, and when the caret last moved, which restarts its
// blinking.
#[derive(Default)]
pub(crate) struct Editing {
    caret: usize,
    scroll: f32,
    moved: f64,
}

// The byte index of the character before the index.
fn previous_boundary(text: &str, index: usize) -> usize {
    text[..index].char_indices().next_back().map_or(0, |(at, _)| at)
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

// What's typed, less what would make the field longer than its limit.
fn limit(text: &str, typed: &str, max_length: Option<usize>) -> String {
    let Some(max) = max_length else {
        return typed.to_string();
    };
    typed.chars().take(max.saturating_sub(text.chars().count())).collect()
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
    let clear_shown = settings.clear_button && ctx.is_focused() && !text.is_empty() && ctx.enabled();
    let content = field_layout(ctx.rect(), settings, clear_shown).content;
    let now = ctx.time();
    let mut text = text.to_string();
    let text_before = text.clone();
    let mut editing = std::mem::take(ctx.memory::<Editing>());
    // The program may have changed the text since the last frame.
    editing.caret = boundary(&text, editing.caret);

    // A press puts the caret where it is.
    if hit.hovered && interaction.enabled {
        ctx.set_cursor(Cursor::Text);
    }
    if hit.pressed && let Some(pointer) = interaction.pointer {
        let layout = ctx.fonts().layout(&text, BODY, None);
        editing.caret = layout.hit(Point::new(pointer.x - content.x + editing.scroll, BODY.line_height / 2.0));
        editing.moved = now;
    }

    // What's typed goes in at the caret, Backspace deletes the character
    // before it, and Enter submits. Keys with Ctrl are the program's.
    if interaction.focused && interaction.enabled && !ctx.modifiers().control {
        let input = ctx.input();
        for (key, _) in input.keys_pressed() {
            match *key {
                Key::BACKSPACE => {
                    let start = previous_boundary(&text, editing.caret);
                    text.replace_range(start..editing.caret, "");
                    editing.caret = start;
                    editing.moved = now;
                }
                Key::ENTER => edited.submitted = true,
                _ => {}
            }
        }
        let typed: String = input.text.chars().filter(|character| !character.is_control()).filter(|character| settings.accept.is_none_or(|accept| accept(*character))).collect();
        let typed = limit(&text, &typed, settings.max_length);
        if !typed.is_empty() {
            text.insert_str(editing.caret, &typed);
            editing.caret += typed.len();
            editing.moved = now;
        }
    }

    // The buttons: delete, and the field's own.
    if interaction.pressed(&Part::Named("Button".to_string())) {
        edited.button_clicked = true;
    }
    if interaction.pressed(&Part::Named("Clear".to_string())) {
        text.clear();
        editing.caret = 0;
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
    let (placeholder_color, text_color, caret_color) = (
        if state == ControlState::Disabled { theme.text_disabled } else { theme.text_secondary },
        if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary },
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
        if edited.button_clicked && let Some(message) = &self.on_button {
            ctx.send(message.clone());
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        field_draw(ctx, &self.text, &self.settings);
    }
}
