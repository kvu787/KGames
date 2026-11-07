// WinUI's controls that are on or off: CheckBox, ToggleSwitch, and a column
// of RadioButtons. Each is a target as wide as its rectangle, so its label
// takes clicks too, and its focus visual goes around the label, as XAML's do.

use crate::core::control::{AutoTarget, Control, NodeContext, UiEventKind, adopted};
use crate::core::ui::{ControlState, Response, Sense, Ui};
use crate::design::controls::{
    CHECK_BOX_GLYPH_SIZE, CHECK_BOX_PADDING, CHECK_BOX_SIZE, FOCUS_VISUAL_MARGIN_WIDE, RADIO_BUTTON_GLYPH, RADIO_BUTTON_GLYPH_DISABLED,
    RADIO_BUTTON_GLYPH_DURATION, RADIO_BUTTON_GLYPH_POINTER_OVER, RADIO_BUTTON_GLYPH_PRESSED, RADIO_BUTTON_PADDING, RADIO_BUTTON_SIZE,
    TOGGLE_SWITCH_CONTENT_GAP, TOGGLE_SWITCH_HEIGHT, TOGGLE_SWITCH_KNOB, TOGGLE_SWITCH_KNOB_POINTER_OVER, TOGGLE_SWITCH_KNOB_PRESSED_HEIGHT,
    TOGGLE_SWITCH_KNOB_PRESSED_MARGIN, TOGGLE_SWITCH_KNOB_PRESSED_WIDTH, TOGGLE_SWITCH_KNOB_TRAVEL, TOGGLE_SWITCH_WIDTH,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, CONTROL_HEIGHT, CURVE_FAST_OUT_SLOW_IN, DURATION_FAST, DURATION_FASTER, DURATION_NORMAL, RADIUS_CONTROL};
use crate::drawing::geometry::{Point, Rect, Size};
use crate::drawing::painter::{Align, Border};
use crate::widgets::widget::{Elevation, StateColors};

// A box that's checked or not, with a label after it.
#[derive(Clone, Debug)]
pub struct CheckBox {
    checked: bool,
    label: String,
}

impl CheckBox {
    pub fn new(checked: bool, label: &str) -> Self {
        Self {
            checked,
            label: label.to_string(),
        }
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    // The width that fits the box and its label.
    pub fn width(ui: &mut Ui, label: &str) -> f32 {
        CHECK_BOX_SIZE + if label.is_empty() { 0.0 } else { CHECK_BOX_PADDING.left + ui.measure(label, BODY).width.ceil() }
    }
}

impl Control for CheckBox {
    fn kind(&self) -> &'static str {
        "CheckBox"
    }

    fn sense(&self) -> Sense {
        Sense::BUTTON
    }

    fn label(&self) -> Option<String> {
        Some(if self.checked { "Checked".to_string() } else { "Unchecked".to_string() })
    }

    fn adopt(&mut self, previous: Box<dyn Control>) {
        if let Some(previous) = adopted::<CheckBox>(previous) {
            self.checked = previous.checked;
        }
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        if ctx.response.clicked {
            self.checked = !self.checked;
            ctx.emit(ui, UiEventKind::Toggled(self.checked));
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        draw_check_box(ui, &ctx.response, ctx.rect, self.checked, &self.label);
    }
}

fn draw_check_box(ui: &mut Ui, response: &Response, rect: Rect, checked: bool, label: &str) {
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
    let fill = ui.animate_color(response.id, 0, fill.get(state), DURATION_FASTER);
    let box_rect = Rect::new(rect.x, (rect.center().y - CHECK_BOX_SIZE / 2.0).round(), CHECK_BOX_SIZE, CHECK_BOX_SIZE);
    let mut painter = ui.painter();
    // The stroke is inside the box, as a Rectangle's is.
    painter.shape(box_rect, RADIUS_CONTROL, fill, Some(Border::new(stroke.get(state), 1.0)));
    if checked {
        painter.icon(box_rect.center() + Point::new(0.0, 1.0), Icon::CheckMark, CHECK_BOX_GLYPH_SIZE, glyph.get(state));
    }
    if !label.is_empty() {
        let left = box_rect.right() + CHECK_BOX_PADDING.left;
        painter.text_fit(Rect::new(left, rect.y, rect.right() - left, rect.height), label, BODY, text, Align::Start);
    }
    ui.focus_visual_around(response.focused, rect, RADIUS_CONTROL, FOCUS_VISUAL_MARGIN_WIDE);
}

// A column of radio buttons, of which one is checked. Each is a target named
// "<name>/<label>" and its own focus stop, as WinUI's are. Choosing one
// emits Selected with its index.
#[derive(Clone, Debug)]
pub struct RadioGroup {
    items: Vec<String>,
    selected: usize,
    row_height: f32,
    // Room before each row, like a group indented under a check box.
    indent: f32,
    responses: Vec<Response>,
}

impl RadioGroup {
    pub fn new(items: &[&str], selected: usize) -> Self {
        Self {
            items: items.iter().map(|item| item.to_string()).collect(),
            selected,
            row_height: CONTROL_HEIGHT,
            indent: 0.0,
            responses: Vec::new(),
        }
    }

    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn set_selected(&mut self, selected: usize) {
        self.selected = selected.min(self.items.len().saturating_sub(1));
    }

    // How tall `count` rows are.
    pub fn height(count: usize) -> f32 {
        count as f32 * CONTROL_HEIGHT
    }

    fn row(&self, rect: Rect, index: usize) -> Rect {
        Rect::new(rect.x + self.indent, rect.y + self.row_height * index as f32, rect.width - self.indent, self.row_height)
    }
}

impl Control for RadioGroup {
    fn kind(&self) -> &'static str {
        "RadioButtons"
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::None
    }

    fn adopt(&mut self, previous: Box<dyn Control>) {
        if let Some(previous) = adopted::<RadioGroup>(previous) {
            self.selected = previous.selected.min(self.items.len().saturating_sub(1));
        }
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        self.responses.clear();
        for index in 0..self.items.len() {
            let row = self.row(ctx.rect, index);
            let item_name = format!("{}/{}", ctx.name, self.items[index]);
            let response = ui.interact(&item_name, row, Sense::BUTTON, "RadioButton");
            ui.set_label(&item_name, if self.selected == index { "Checked" } else { "Unchecked" });
            if response.clicked && self.selected != index {
                self.selected = index;
                ctx.emit(ui, UiEventKind::Selected(index));
            }
            self.responses.push(response);
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        for (index, response) in self.responses.clone().iter().enumerate() {
            let row = self.row(ctx.rect, index);
            draw_radio(ui, response, row, self.selected == index, &self.items[index]);
        }
    }
}

fn draw_radio(ui: &mut Ui, response: &Response, rect: Rect, checked: bool, label: &str) {
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
    // The checked dot grows with the pointer over it and shrinks as it's
    // pressed, along ControlFastOutSlowIn.
    let (glyph_size, duration) = match state {
        ControlState::PointerOver => (RADIO_BUTTON_GLYPH_POINTER_OVER, RADIO_BUTTON_GLYPH_DURATION),
        ControlState::Pressed => (RADIO_BUTTON_GLYPH_PRESSED, RADIO_BUTTON_GLYPH_DURATION),
        ControlState::Disabled => (RADIO_BUTTON_GLYPH_DISABLED, DURATION_FAST),
        ControlState::Normal => (RADIO_BUTTON_GLYPH, 0.0),
    };
    let glyph_size = ui.animate(response.id, 1, glyph_size, duration, CURVE_FAST_OUT_SLOW_IN);
    let fill = ui.animate_color(response.id, 0, fill.get(state), DURATION_FASTER);
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
    if !label.is_empty() {
        let left = circle.right() + RADIO_BUTTON_PADDING.left;
        painter.text_fit(Rect::new(left, rect.y, rect.right() - left, rect.height), label, BODY, text, Align::Start);
    }
    ui.focus_visual_around(response.focused, rect, RADIUS_CONTROL, FOCUS_VISUAL_MARGIN_WIDE);
}

// A switch that's on or off, with a label after it. Its knob slides between
// the track's ends, grows with the pointer over it, and stretches toward the
// other end as it's pressed.
#[derive(Clone, Debug)]
pub struct ToggleSwitch {
    on: bool,
    label: String,
}

impl ToggleSwitch {
    pub fn new(on: bool, label: &str) -> Self {
        Self { on, label: label.to_string() }
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    pub fn set_on(&mut self, on: bool) {
        self.on = on;
    }

    pub fn width(ui: &mut Ui, label: &str) -> f32 {
        TOGGLE_SWITCH_WIDTH + if label.is_empty() { 0.0 } else { TOGGLE_SWITCH_CONTENT_GAP + ui.measure(label, BODY).width.ceil() }
    }
}

impl Control for ToggleSwitch {
    fn kind(&self) -> &'static str {
        "ToggleSwitch"
    }

    fn sense(&self) -> Sense {
        Sense::BUTTON
    }

    fn label(&self) -> Option<String> {
        Some(if self.on { "On".to_string() } else { "Off".to_string() })
    }

    fn adopt(&mut self, previous: Box<dyn Control>) {
        if let Some(previous) = adopted::<ToggleSwitch>(previous) {
            self.on = previous.on;
        }
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        if ctx.response.clicked {
            self.on = !self.on;
            ctx.emit(ui, UiEventKind::Toggled(self.on));
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let (rect, response, on) = (ctx.rect, &ctx.response, self.on);
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
        let (knob_width, knob_height, duration) = match state {
            ControlState::PointerOver => (TOGGLE_SWITCH_KNOB_POINTER_OVER, TOGGLE_SWITCH_KNOB_POINTER_OVER, DURATION_FASTER),
            ControlState::Pressed => (TOGGLE_SWITCH_KNOB_PRESSED_WIDTH, TOGGLE_SWITCH_KNOB_PRESSED_HEIGHT, DURATION_FASTER),
            ControlState::Disabled => (TOGGLE_SWITCH_KNOB, TOGGLE_SWITCH_KNOB, DURATION_NORMAL),
            ControlState::Normal => (TOGGLE_SWITCH_KNOB, TOGGLE_SWITCH_KNOB, DURATION_FASTER),
        };
        let knob_width = ui.animate(response.id, 1, knob_width, duration, CURVE_FAST_OUT_SLOW_IN);
        let knob_height = ui.animate(response.id, 2, knob_height, duration, CURVE_FAST_OUT_SLOW_IN);
        let travel = ui.animate(response.id, 3, if on { 1.0 } else { 0.0 }, DURATION_FAST, CURVE_FAST_OUT_SLOW_IN);
        let fill = ui.animate_color(response.id, 0, fill.get(state), DURATION_FASTER);
        let track = Rect::new(rect.x, (rect.center().y - TOGGLE_SWITCH_HEIGHT / 2.0).round(), TOGGLE_SWITCH_WIDTH, TOGGLE_SWITCH_HEIGHT);
        let mut painter = ui.painter();
        painter.shape(track, TOGGLE_SWITCH_HEIGHT / 2.0, fill, stroke.map(|stroke| Border::new(stroke.get(state), 1.0)));
        // The knob is centered in a 20-pixel square at the track's end; a
        // stretched one keeps 3 pixels from that end.
        let square = track.x + TOGGLE_SWITCH_KNOB_TRAVEL * travel;
        let stretch = (knob_width - knob_height).max(0.0) / (TOGGLE_SWITCH_KNOB_PRESSED_WIDTH - TOGGLE_SWITCH_KNOB_PRESSED_HEIGHT);
        let centered = square + TOGGLE_SWITCH_HEIGHT / 2.0;
        let pressed_center = if on {
            square + TOGGLE_SWITCH_HEIGHT - TOGGLE_SWITCH_KNOB_PRESSED_MARGIN - knob_width / 2.0
        } else {
            square + TOGGLE_SWITCH_KNOB_PRESSED_MARGIN + knob_width / 2.0
        };
        let center_x = centered + (pressed_center - centered) * stretch.clamp(0.0, 1.0);
        let knob_rect = Rect::from_center(Point::new(center_x, track.center().y), Size::new(knob_width, knob_height));
        painter.rounded(knob_rect, knob_height / 2.0, knob.get(state));
        if !self.label.is_empty() {
            let left = track.right() + TOGGLE_SWITCH_CONTENT_GAP;
            painter.text_fit(Rect::new(left, rect.y, rect.right() - left, rect.height), &self.label, BODY, text, Align::Start);
        }
        ui.focus_visual_around(response.focused, rect, RADIUS_CONTROL, FOCUS_VISUAL_MARGIN_WIDE);
    }
}
