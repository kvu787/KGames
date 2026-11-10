// WinUI's controls that are on or off: CheckBox, RadioButton, and
// ToggleSwitch. Each is a target as wide as its rectangle, so its label takes
// clicks too, and its focus visual goes around the label, as XAML's do.

use crate::core::interaction::{ControlState, Interaction, Part, Sense};
use crate::core::interface::Ui;
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{
    CHECK_BOX_GLYPH_SIZE, CHECK_BOX_PADDING, CHECK_BOX_SIZE, FOCUS_VISUAL_MARGIN_WIDE, RADIO_BUTTON_GLYPH, RADIO_BUTTON_GLYPH_DISABLED,
    RADIO_BUTTON_GLYPH_POINTER_OVER, RADIO_BUTTON_GLYPH_PRESSED, RADIO_BUTTON_PADDING, RADIO_BUTTON_SIZE, TOGGLE_SWITCH_CONTENT_GAP,
    TOGGLE_SWITCH_HEIGHT, TOGGLE_SWITCH_KNOB, TOGGLE_SWITCH_KNOB_POINTER_OVER, TOGGLE_SWITCH_KNOB_PRESSED_HEIGHT, TOGGLE_SWITCH_KNOB_PRESSED_MARGIN,
    TOGGLE_SWITCH_KNOB_PRESSED_WIDTH, TOGGLE_SWITCH_KNOB_TRAVEL, TOGGLE_SWITCH_WIDTH,
};
use crate::design::icons::Icon;
use crate::design::theme::Theme;
use crate::design::tokens::{BODY, CONTROL_HEIGHT, RADIUS_CONTROL};
use crate::drawing::geometry::{Point, Rect, Size};
use crate::drawing::painter::{Align, Border, Painter};
use crate::widgets::widget::{Elevation, Reaction, StateColors, react};

// A box that's checked or not, with a label after it.
pub struct CheckBox<M> {
    checked: bool,
    label: String,
    on_change: Option<Reaction<bool, M>>,
}

impl<M: Clone + 'static> CheckBox<M> {
    pub fn new(checked: bool, label: impl Into<String>) -> Self {
        Self {
            checked,
            label: label.into(),
            on_change: None,
        }
    }

    // What a click sends, with whether it's checked then.
    pub fn on_change(mut self, reaction: impl Fn(bool) -> M + 'static) -> Self {
        self.on_change = Some(react(reaction));
        self
    }

    pub fn width(ui: &mut Ui, label: &str) -> f32 {
        CHECK_BOX_SIZE + if label.is_empty() { 0.0 } else { CHECK_BOX_PADDING.left + ui.measure(label, BODY).width.ceil() }
    }
}

impl<M: Clone + 'static> Widget<M> for CheckBox<M> {
    fn kind(&self) -> &'static str {
        "CheckBox"
    }

    fn label(&self) -> Option<String> {
        Some(if self.checked { "Checked".to_string() } else { "Unchecked".to_string() })
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::BUTTON)
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        if interaction.whole().activated
            && let Some(reaction) = &self.on_change
        {
            ctx.send(reaction(!self.checked));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let state = ctx.state();
        let checked = self.checked;
        let theme = ctx.theme();
        // CheckBoxCheckBackgroundFill, CheckBoxCheckBackgroundStroke and
        // CheckBoxCheckGlyphForeground.
        let (fill, stroke) = if checked {
            (
                StateColors::new(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled),
                StateColors::new(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.control_strong_stroke_disabled),
            )
        } else {
            (
                StateColors::new(theme.control_alt_fill_secondary, theme.control_alt_fill_tertiary, theme.control_alt_fill_quarternary, theme.control_alt_fill_disabled),
                StateColors::new(theme.control_strong_stroke, theme.control_strong_stroke, theme.control_strong_stroke_disabled, theme.control_strong_stroke_disabled),
            )
        };
        let glyph = StateColors::new(theme.text_on_accent, theme.text_on_accent, theme.text_on_accent_secondary, theme.text_on_accent_disabled);
        let text = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
        let box_rect = Rect::new(rect.x, (rect.center().y - CHECK_BOX_SIZE / 2.0).round(), CHECK_BOX_SIZE, CHECK_BOX_SIZE);
        let mut painter = ctx.painter();
        // The stroke is inside the box, as a Rectangle's is.
        painter.shape(box_rect, RADIUS_CONTROL, fill.get(state), Some(Border::new(stroke.get(state), 1.0)));
        if checked {
            painter.icon(box_rect.center() + Point::new(0.0, 1.0), Icon::CheckMark, CHECK_BOX_GLYPH_SIZE, glyph.get(state));
        }
        if !self.label.is_empty() {
            let left = box_rect.right() + CHECK_BOX_PADDING.left;
            painter.text_fit(Rect::new(left, rect.y, rect.right() - left, rect.height), &self.label, BODY, text, Align::Start);
        }
        let focused = ctx.focused();
        ctx.focus_visual_around(focused, rect, RADIUS_CONTROL, FOCUS_VISUAL_MARGIN_WIDE);
    }
}

// One of a group of choices, checked while the group's value is its own.
pub struct RadioButton<M> {
    checked: bool,
    label: String,
    on_check: Option<M>,
}

impl<M: Clone + 'static> RadioButton<M> {
    pub fn new(checked: bool, label: impl Into<String>) -> Self {
        Self {
            checked,
            label: label.into(),
            on_check: None,
        }
    }

    // What a click sends, which should check it.
    pub fn on_check(mut self, message: M) -> Self {
        self.on_check = Some(message);
        self
    }

    pub fn width(ui: &mut Ui, label: &str) -> f32 {
        RADIO_BUTTON_SIZE + if label.is_empty() { 0.0 } else { RADIO_BUTTON_PADDING.left + ui.measure(label, BODY).width.ceil() }
    }
}

// Draws a radio button in the rectangle, in the state, checked or not.
fn draw_radio(painter: &mut Painter, theme: &Theme, rect: Rect, state: ControlState, checked: bool, label: &str) {
    // RadioButtonOuterEllipse(Checked)Fill and ...Stroke.
    let (fill, stroke) = if checked {
        let accent = StateColors::new(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled);
        (accent, accent)
    } else {
        (
            StateColors::new(theme.control_alt_fill_secondary, theme.control_alt_fill_tertiary, theme.control_alt_fill_quarternary, theme.control_alt_fill_disabled),
            StateColors::new(theme.control_strong_stroke, theme.control_strong_stroke, theme.control_strong_stroke_disabled, theme.control_strong_stroke_disabled),
        )
    };
    let text = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
    // The checked dot is bigger with the pointer over it, and smaller while
    // it's pressed.
    let glyph_size = match state {
        ControlState::PointerOver => RADIO_BUTTON_GLYPH_POINTER_OVER,
        ControlState::Pressed => RADIO_BUTTON_GLYPH_PRESSED,
        ControlState::Disabled => RADIO_BUTTON_GLYPH_DISABLED,
        ControlState::Normal => RADIO_BUTTON_GLYPH,
    };
    let center = Point::new(rect.x + RADIO_BUTTON_SIZE / 2.0, rect.center().y.round());
    let circle = Rect::from_center(center, Size::square(RADIO_BUTTON_SIZE));
    painter.shape(circle, RADIO_BUTTON_SIZE / 2.0, fill.get(state), Some(Border::new(stroke.get(state), 1.0)));
    if checked {
        let glyph = Rect::from_center(center, Size::square(glyph_size));
        let elevation = if state == ControlState::Disabled { Elevation::Control } else { Elevation::Accent };
        painter.shape(glyph, glyph_size / 2.0, theme.text_on_accent, Some(Border::new(elevation.brush(theme, glyph), 1.0)));
    }
    if !label.is_empty() {
        let left = circle.right() + RADIO_BUTTON_PADDING.left;
        painter.text_fit(Rect::new(left, rect.y, rect.right() - left, rect.height), label, BODY, text, Align::Start);
    }
}

impl<M: Clone + 'static> Widget<M> for RadioButton<M> {
    fn kind(&self) -> &'static str {
        "RadioButton"
    }

    fn label(&self) -> Option<String> {
        Some(if self.checked { "Checked".to_string() } else { "Unchecked".to_string() })
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::BUTTON)
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        if interaction.whole().activated
            && !self.checked
            && let Some(message) = &self.on_check
        {
            ctx.send(message.clone());
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let state = ctx.state();
        let theme = ctx.theme().clone();
        draw_radio(&mut ctx.painter(), &theme, rect, state, self.checked, &self.label);
        let focused = ctx.focused();
        ctx.focus_visual_around(focused, rect, RADIUS_CONTROL, FOCUS_VISUAL_MARGIN_WIDE);
    }
}

// A column of radio buttons that pick one of a group's items, by index.
// Each item is a row of CONTROL_HEIGHT named "{name}/{item}", so a group
// "Size" with "Small" in it has a target "Size/Small".
pub struct RadioGroup<M> {
    selected: usize,
    items: Vec<String>,
    on_select: Option<Reaction<usize, M>>,
}

impl<M: Clone + 'static> RadioGroup<M> {
    pub fn new(selected: usize, items: &[impl AsRef<str>]) -> Self {
        Self {
            selected,
            items: items.iter().map(|item| item.as_ref().to_string()).collect(),
            on_select: None,
        }
    }

    // What a click on an item sends, with the item's index.
    pub fn on_select(mut self, reaction: impl Fn(usize) -> M + 'static) -> Self {
        self.on_select = Some(react(reaction));
        self
    }

    // The group's height: a CONTROL_HEIGHT row an item.
    pub fn height(items: usize) -> f32 {
        items as f32 * CONTROL_HEIGHT
    }

    fn row(rect: Rect, index: usize) -> Rect {
        Rect::new(rect.x, rect.y + CONTROL_HEIGHT * index as f32, rect.width, CONTROL_HEIGHT)
    }
}

impl<M: Clone + 'static> Widget<M> for RadioGroup<M> {
    fn kind(&self) -> &'static str {
        "RadioButtons"
    }

    fn label(&self) -> Option<String> {
        self.items.get(self.selected).cloned()
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NONE)
    }

    fn parts(&self, ctx: &mut Parts) {
        let rect = ctx.rect();
        for (index, item) in self.items.iter().enumerate() {
            let checked = index == self.selected;
            let label = Some(if checked { "Checked".to_string() } else { "Unchecked".to_string() });
            ctx.part(Part::Index(index), item, Self::row(rect, index), Sense::BUTTON, "RadioButton", label);
        }
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        if let Some(Part::Index(index)) = interaction.activated_part()
            && *index != self.selected
            && let Some(reaction) = &self.on_select
        {
            ctx.send(reaction(*index));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let look = ctx.look().clone();
        let theme = ctx.theme().clone();
        for (index, item) in self.items.iter().enumerate() {
            let part = Part::Index(index);
            let row = Self::row(rect, index);
            draw_radio(&mut ctx.painter(), &theme, row, look.part_state(&part), index == self.selected, item);
            let focused = look.focused && look.focused_part.as_ref() == Some(&part);
            ctx.focus_visual_around(focused, row, RADIUS_CONTROL, FOCUS_VISUAL_MARGIN_WIDE);
        }
    }
}

// A switch that's on or off, with a label after it. Its knob is at the
// track's end for its value, bigger with the pointer over it, and stretched
// toward the other end while it's pressed.
pub struct ToggleSwitch<M> {
    on: bool,
    label: String,
    on_change: Option<Reaction<bool, M>>,
}

impl<M: Clone + 'static> ToggleSwitch<M> {
    pub fn new(on: bool, label: impl Into<String>) -> Self {
        Self {
            on,
            label: label.into(),
            on_change: None,
        }
    }

    // What a click sends, with whether it's on then.
    pub fn on_change(mut self, reaction: impl Fn(bool) -> M + 'static) -> Self {
        self.on_change = Some(react(reaction));
        self
    }

    pub fn width(ui: &mut Ui, label: &str) -> f32 {
        TOGGLE_SWITCH_WIDTH + if label.is_empty() { 0.0 } else { TOGGLE_SWITCH_CONTENT_GAP + ui.measure(label, BODY).width.ceil() }
    }
}

impl<M: Clone + 'static> Widget<M> for ToggleSwitch<M> {
    fn kind(&self) -> &'static str {
        "ToggleSwitch"
    }

    fn label(&self) -> Option<String> {
        Some(if self.on { "On".to_string() } else { "Off".to_string() })
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::BUTTON)
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        if interaction.whole().activated
            && let Some(reaction) = &self.on_change
        {
            ctx.send(reaction(!self.on));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let state = ctx.state();
        let on = self.on;
        let theme = ctx.theme();
        // ToggleSwitchFillOn/Off, ToggleSwitchStrokeOff, and the knob's fill.
        // The track that's on has no stroke, and neither has the knob.
        let (fill, stroke, knob) = if on {
            (
                StateColors::new(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled),
                None,
                StateColors::new(theme.text_on_accent, theme.text_on_accent, theme.text_on_accent, theme.text_on_accent_disabled),
            )
        } else {
            (
                StateColors::new(theme.control_alt_fill_secondary, theme.control_alt_fill_tertiary, theme.control_alt_fill_quarternary, theme.control_alt_fill_disabled),
                Some(StateColors::new(theme.control_strong_stroke, theme.control_strong_stroke, theme.control_strong_stroke, theme.control_strong_stroke_disabled)),
                StateColors::new(theme.text_secondary, theme.text_secondary, theme.text_secondary, theme.text_disabled),
            )
        };
        let text = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
        let (knob_width, knob_height) = match state {
            ControlState::PointerOver => (TOGGLE_SWITCH_KNOB_POINTER_OVER, TOGGLE_SWITCH_KNOB_POINTER_OVER),
            ControlState::Pressed => (TOGGLE_SWITCH_KNOB_PRESSED_WIDTH, TOGGLE_SWITCH_KNOB_PRESSED_HEIGHT),
            ControlState::Disabled | ControlState::Normal => (TOGGLE_SWITCH_KNOB, TOGGLE_SWITCH_KNOB),
        };
        let track = Rect::new(rect.x, (rect.center().y - TOGGLE_SWITCH_HEIGHT / 2.0).round(), TOGGLE_SWITCH_WIDTH, TOGGLE_SWITCH_HEIGHT);
        let mut painter = ctx.painter();
        painter.shape(track, TOGGLE_SWITCH_HEIGHT / 2.0, fill.get(state), stroke.map(|stroke| Border::new(stroke.get(state), 1.0)));
        // The knob is centered in a 20-pixel square at the track's end; a
        // stretched one keeps 3 pixels from that end.
        let square = if on { track.x + TOGGLE_SWITCH_KNOB_TRAVEL } else { track.x };
        let center_x = match (state, on) {
            (ControlState::Pressed, true) => square + TOGGLE_SWITCH_HEIGHT - TOGGLE_SWITCH_KNOB_PRESSED_MARGIN - knob_width / 2.0,
            (ControlState::Pressed, false) => square + TOGGLE_SWITCH_KNOB_PRESSED_MARGIN + knob_width / 2.0,
            _ => square + TOGGLE_SWITCH_HEIGHT / 2.0,
        };
        let knob_rect = Rect::from_center(Point::new(center_x, track.center().y), Size::new(knob_width, knob_height));
        painter.rounded(knob_rect, knob_height / 2.0, knob.get(state));
        if !self.label.is_empty() {
            let left = track.right() + TOGGLE_SWITCH_CONTENT_GAP;
            painter.text_fit(Rect::new(left, rect.y, rect.right() - left, rect.height), &self.label, BODY, text, Align::Start);
        }
        let focused = ctx.focused();
        ctx.focus_visual_around(focused, rect, RADIUS_CONTROL, FOCUS_VISUAL_MARGIN_WIDE);
    }
}
