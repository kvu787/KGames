// WinUI's controls that are on or off: CheckBox, RadioButton, and
// ToggleSwitch. Each is a target as wide as its rectangle, so its label takes
// clicks too, and its focus visual goes around the label, as XAML's do.

use crate::core::interaction::{ControlState, Response, Sense, id};
use crate::core::ui::Ui;
use crate::design::controls::{
    CHECK_BOX_GLYPH_SIZE, CHECK_BOX_PADDING, CHECK_BOX_SIZE, FOCUS_VISUAL_MARGIN_WIDE, RADIO_BUTTON_GLYPH, RADIO_BUTTON_GLYPH_DISABLED,
    RADIO_BUTTON_GLYPH_POINTER_OVER, RADIO_BUTTON_GLYPH_PRESSED, RADIO_BUTTON_PADDING, RADIO_BUTTON_SIZE,
    TOGGLE_SWITCH_CONTENT_GAP, TOGGLE_SWITCH_HEIGHT, TOGGLE_SWITCH_KNOB, TOGGLE_SWITCH_KNOB_POINTER_OVER, TOGGLE_SWITCH_KNOB_PRESSED_HEIGHT,
    TOGGLE_SWITCH_KNOB_PRESSED_MARGIN, TOGGLE_SWITCH_KNOB_PRESSED_WIDTH, TOGGLE_SWITCH_KNOB_TRAVEL, TOGGLE_SWITCH_WIDTH,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, CONTROL_HEIGHT, RADIUS_CONTROL};
use crate::drawing::geometry::{Point, Rect, Size};
use crate::drawing::painter::{Align, Border};
use crate::widgets::widget::{Elevation, StateColors, Widget};

// A box that's checked or not, with a label after it.
pub struct CheckBox<'a, 'v> {
    value: &'v mut bool,
    label: &'a str,
}

impl<'a, 'v> CheckBox<'a, 'v> {
    pub fn new(value: &'v mut bool, label: &'a str) -> Self {
        Self { value, label }
    }

    // The width that fits the box and its label.
    pub fn width(ui: &mut Ui, label: &str) -> f32 {
        CHECK_BOX_SIZE + if label.is_empty() { 0.0 } else { CHECK_BOX_PADDING.left + ui.measure(label, BODY).width.ceil() }
    }
}

impl Widget for CheckBox<'_, '_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let mut response = ui.interact(name, rect, Sense::BUTTON, "CheckBox");
        if response.clicked {
            *self.value = !*self.value;
            response.changed = true;
        }
        let checked = *self.value;
        ui.set_label(name, if checked { "Checked" } else { "Unchecked" });
        let state = response.state();
        let theme = ui.theme();
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
        let fill = fill.get(state);
        let box_rect = Rect::new(rect.x, (rect.center().y - CHECK_BOX_SIZE / 2.0).round(), CHECK_BOX_SIZE, CHECK_BOX_SIZE);
        let mut painter = ui.painter();
        // The stroke is inside the box, as a Rectangle's is.
        painter.shape(box_rect, RADIUS_CONTROL, fill, Some(Border::new(stroke.get(state), 1.0)));
        if checked {
            painter.icon(box_rect.center() + Point::new(0.0, 1.0), Icon::CheckMark, CHECK_BOX_GLYPH_SIZE, glyph.get(state));
        }
        if !self.label.is_empty() {
            let left = box_rect.right() + CHECK_BOX_PADDING.left;
            painter.text_fit(Rect::new(left, rect.y, rect.right() - left, rect.height), self.label, BODY, text, Align::Start);
        }
        ui.focus_visual_around(response.focused, rect, RADIUS_CONTROL, FOCUS_VISUAL_MARGIN_WIDE);
        response
    }
}

// One of a group of choices, checked while the group's value is its own.
pub struct RadioButton<'a, 'v, T: PartialEq + Copy> {
    value: &'v mut T,
    own: T,
    label: &'a str,
}

impl<'a, 'v, T: PartialEq + Copy> RadioButton<'a, 'v, T> {
    pub fn new(value: &'v mut T, own: T, label: &'a str) -> Self {
        Self { value, own, label }
    }

    pub fn width(ui: &mut Ui, label: &str) -> f32 {
        RADIO_BUTTON_SIZE + if label.is_empty() { 0.0 } else { RADIO_BUTTON_PADDING.left + ui.measure(label, BODY).width.ceil() }
    }
}

impl<T: PartialEq + Copy> Widget for RadioButton<'_, '_, T> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let mut response = ui.interact(name, rect, Sense::BUTTON, "RadioButton");
        if response.clicked && *self.value != self.own {
            *self.value = self.own;
            response.changed = true;
        }
        let checked = *self.value == self.own;
        ui.set_label(name, if checked { "Checked" } else { "Unchecked" });
        let state = response.state();
        let theme = ui.theme();
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
        let glyph_fill = theme.text_on_accent;
        // The checked dot is bigger with the pointer over it, and smaller
        // while it's pressed.
        let glyph_size = match state {
            ControlState::PointerOver => RADIO_BUTTON_GLYPH_POINTER_OVER,
            ControlState::Pressed => RADIO_BUTTON_GLYPH_PRESSED,
            ControlState::Disabled => RADIO_BUTTON_GLYPH_DISABLED,
            ControlState::Normal => RADIO_BUTTON_GLYPH,
        };
        let fill = fill.get(state);
        let center = Point::new(rect.x + RADIO_BUTTON_SIZE / 2.0, rect.center().y.round());
        let circle = Rect::from_center(center, Size::square(RADIO_BUTTON_SIZE));
        let mut painter = ui.painter();
        let theme = painter.theme();
        painter.shape(circle, RADIO_BUTTON_SIZE / 2.0, fill, Some(Border::new(stroke.get(state), 1.0)));
        if checked {
            let glyph = Rect::from_center(center, Size::square(glyph_size));
            let elevation = if state == ControlState::Disabled { Elevation::Control } else { Elevation::Accent };
            painter.shape(glyph, glyph_size / 2.0, glyph_fill, Some(Border::new(elevation.brush(theme, glyph), 1.0)));
        }
        if !self.label.is_empty() {
            let left = circle.right() + RADIO_BUTTON_PADDING.left;
            painter.text_fit(Rect::new(left, rect.y, rect.right() - left, rect.height), self.label, BODY, text, Align::Start);
        }
        ui.focus_visual_around(response.focused, rect, RADIUS_CONTROL, FOCUS_VISUAL_MARGIN_WIDE);
        response
    }
}

// A column of radio buttons that pick one of a group's items, by index.
// Each item is a row of CONTROL_HEIGHT named "{name}/{item}", so a group
// "Size" with "Small" in it has a target "Size/Small".
pub struct RadioGroup<'a, 'v> {
    selected: &'v mut usize,
    items: &'a [&'a str],
}

impl<'a, 'v> RadioGroup<'a, 'v> {
    pub fn new(selected: &'v mut usize, items: &'a [&'a str]) -> Self {
        Self { selected, items }
    }

    // The group's height: a CONTROL_HEIGHT row an item.
    pub fn height(items: usize) -> f32 {
        items as f32 * CONTROL_HEIGHT
    }
}

impl Widget for RadioGroup<'_, '_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let mut response = Response {
            id: id(name),
            rect,
            enabled: ui.enabled(),
            ..Response::default()
        };
        let mut row = rect;
        row.height = CONTROL_HEIGHT;
        for (index, item) in self.items.iter().enumerate() {
            let one = ui.add(&format!("{name}/{item}"), row, RadioButton::new(self.selected, index, item));
            response.hovered |= one.hovered;
            response.clicked |= one.clicked;
            response.changed |= one.changed;
            row.y += CONTROL_HEIGHT;
        }
        response
    }
}

// A switch that's on or off, with a label after it. Its knob is at the
// track's end for its value, bigger with the pointer over it, and stretched
// toward the other end while it's pressed.
pub struct ToggleSwitch<'a, 'v> {
    value: &'v mut bool,
    label: &'a str,
}

impl<'a, 'v> ToggleSwitch<'a, 'v> {
    pub fn new(value: &'v mut bool, label: &'a str) -> Self {
        Self { value, label }
    }

    pub fn width(ui: &mut Ui, label: &str) -> f32 {
        TOGGLE_SWITCH_WIDTH + if label.is_empty() { 0.0 } else { TOGGLE_SWITCH_CONTENT_GAP + ui.measure(label, BODY).width.ceil() }
    }
}

impl Widget for ToggleSwitch<'_, '_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let mut response = ui.interact(name, rect, Sense::BUTTON, "ToggleSwitch");
        if response.clicked {
            *self.value = !*self.value;
            response.changed = true;
        }
        let on = *self.value;
        ui.set_label(name, if on { "On" } else { "Off" });
        let state = response.state();
        let theme = ui.theme();
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
        let pressed = state == ControlState::Pressed;
        let (knob_width, knob_height) = match state {
            ControlState::PointerOver => (TOGGLE_SWITCH_KNOB_POINTER_OVER, TOGGLE_SWITCH_KNOB_POINTER_OVER),
            ControlState::Pressed => (TOGGLE_SWITCH_KNOB_PRESSED_WIDTH, TOGGLE_SWITCH_KNOB_PRESSED_HEIGHT),
            ControlState::Disabled | ControlState::Normal => (TOGGLE_SWITCH_KNOB, TOGGLE_SWITCH_KNOB),
        };
        let fill = fill.get(state);
        let track = Rect::new(rect.x, (rect.center().y - TOGGLE_SWITCH_HEIGHT / 2.0).round(), TOGGLE_SWITCH_WIDTH, TOGGLE_SWITCH_HEIGHT);
        let mut painter = ui.painter();
        painter.shape(track, TOGGLE_SWITCH_HEIGHT / 2.0, fill, stroke.map(|stroke| Border::new(stroke.get(state), 1.0)));
        // The knob is centered in a 20-pixel square at the track's end; a
        // stretched one keeps 3 pixels from that end.
        let square = if on { track.x + TOGGLE_SWITCH_KNOB_TRAVEL } else { track.x };
        let center_x = match (pressed, on) {
            (false, _) => square + TOGGLE_SWITCH_HEIGHT / 2.0,
            (true, true) => square + TOGGLE_SWITCH_HEIGHT - TOGGLE_SWITCH_KNOB_PRESSED_MARGIN - knob_width / 2.0,
            (true, false) => square + TOGGLE_SWITCH_KNOB_PRESSED_MARGIN + knob_width / 2.0,
        };
        let knob_rect = Rect::from_center(Point::new(center_x, track.center().y), Size::new(knob_width, knob_height));
        painter.rounded(knob_rect, knob_height / 2.0, knob.get(state));
        if !self.label.is_empty() {
            let left = track.right() + TOGGLE_SWITCH_CONTENT_GAP;
            painter.text_fit(Rect::new(left, rect.y, rect.right() - left, rect.height), self.label, BODY, text, Align::Start);
        }
        ui.focus_visual_around(response.focused, rect, RADIUS_CONTROL, FOCUS_VISUAL_MARGIN_WIDE);
        response
    }
}
