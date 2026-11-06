// Controls that are on or off, or one of a few: checkboxes, radio buttons
// and toggle switches, each with its label to its right, which a press on
// also flips. As WinUI's CheckBox, RadioButton and ToggleSwitch.

use crate::geometry::{Color, Point, Rect, Size};
use crate::icons::Icon;
use crate::painter::{Align, Corners, Stroke};
use crate::tokens::{BODY, CURVE_DECELERATE, DURATION_FAST, DURATION_FASTER, ICON_SMALL, RADIUS_CONTROL, SPACE_M, SPACE_S};
use crate::ui::{Response, Sense, Ui};
use crate::widgets::common::{Visual, focus_ring};

const BOX: f32 = 20.0;
const SWITCH_WIDTH: f32 = 40.0;
const SWITCH_HEIGHT: f32 = 20.0;

impl Ui {
    pub fn checkbox(&mut self, name: &str, rect: Rect, checked: &mut bool, label: &str) -> Response {
        let mut response = self.interact(name, rect, Sense::BUTTON, "CheckBox");
        self.set_label(name, label);
        if response.clicked {
            *checked = !*checked;
            response.changed = true;
        }
        let visual = Visual::of(self, &response);
        let theme = self.theme();
        let (fill, stroke) = if *checked {
            (visual.pick(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled), None)
        } else {
            (
                visual.pick(
                    theme.control_alt_fill_secondary,
                    theme.control_alt_fill_tertiary,
                    theme.control_alt_fill_quarternary,
                    theme.control_alt_fill_disabled,
                ),
                Some(Stroke::new(
                    visual.pick(
                        theme.control_strong_stroke,
                        theme.control_strong_stroke,
                        theme.control_strong_stroke_disabled,
                        theme.control_strong_stroke_disabled,
                    ),
                    1.0,
                )),
            )
        };
        let glyph = visual.pick(theme.text_on_accent, theme.text_on_accent, theme.text_on_accent_secondary, theme.text_on_accent_disabled);
        let text = if visual == Visual::Disabled { theme.text_disabled } else { theme.text_primary };
        let fill = self.animate_color(response.id, 0, fill, DURATION_FASTER);
        let check = self.animate(response.id, 1, if *checked { 1.0 } else { 0.0 }, DURATION_FASTER, CURVE_DECELERATE);
        let square = Rect::new(rect.x, (rect.center().y - BOX / 2.0).round(), BOX, BOX);
        let mut painter = self.painter();
        painter.shape(square, Corners::all(RADIUS_CONTROL), fill, stroke);
        if check > 0.0 {
            painter.icon(square.center(), Icon::CheckMark, ICON_SMALL, glyph.fade(check));
        }
        draw_label(&mut painter, rect, BOX + SPACE_S, label, text);
        focus_ring(self, &response, square, RADIUS_CONTROL);
        response
    }

    // One of a group of choices: it's checked while `selected` is `value`,
    // and a press makes it so.
    pub fn radio<T: PartialEq + Copy>(&mut self, name: &str, rect: Rect, selected: &mut T, value: T, label: &str) -> Response {
        let mut response = self.interact(name, rect, Sense::BUTTON, "RadioButton");
        self.set_label(name, label);
        if response.clicked && *selected != value {
            *selected = value;
            response.changed = true;
        }
        let checked = *selected == value;
        let visual = Visual::of(self, &response);
        let theme = self.theme();
        let center = Point::new(rect.x + BOX / 2.0, rect.center().y.round());
        let (fill, stroke) = if checked {
            (visual.pick(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled), Color::TRANSPARENT)
        } else {
            (
                visual.pick(
                    theme.control_alt_fill_secondary,
                    theme.control_alt_fill_tertiary,
                    theme.control_alt_fill_quarternary,
                    theme.control_alt_fill_disabled,
                ),
                visual.pick(
                    theme.control_strong_stroke,
                    theme.control_strong_stroke,
                    theme.control_strong_stroke_disabled,
                    theme.control_strong_stroke_disabled,
                ),
            )
        };
        let dot = if checked { visual.pick(12.0, 14.0, 10.0, 12.0) } else { 0.0 };
        let dot_color = if visual == Visual::Disabled { theme.text_on_accent_disabled } else { theme.text_on_accent };
        let text = if visual == Visual::Disabled { theme.text_disabled } else { theme.text_primary };
        let fill = self.animate_color(response.id, 0, fill, DURATION_FASTER);
        let dot = self.animate(response.id, 1, dot, DURATION_FAST, CURVE_DECELERATE);
        let outer = Rect::from_center(center, Size::new(BOX, BOX));
        let mut painter = self.painter();
        painter.shape(outer, Corners::all(BOX / 2.0), fill, Some(Stroke::new(stroke, 1.0)));
        if dot > 0.5 {
            painter.circle(center, dot / 2.0, dot_color);
        }
        draw_label(&mut painter, rect, BOX + SPACE_S, label, text);
        focus_ring(self, &response, outer, BOX / 2.0);
        response
    }

    // A switch that's on or off, with its label to its right.
    pub fn toggle_switch(&mut self, name: &str, rect: Rect, on: &mut bool, label: &str) -> Response {
        let mut response = self.interact(name, rect, Sense::BUTTON, "ToggleSwitch");
        self.set_label(name, label);
        if response.clicked {
            *on = !*on;
            response.changed = true;
        }
        let visual = Visual::of(self, &response);
        let theme = self.theme();
        let track = Rect::new(rect.x, (rect.center().y - SWITCH_HEIGHT / 2.0).round(), SWITCH_WIDTH, SWITCH_HEIGHT);
        let (fill, stroke, knob_color) = if *on {
            (
                visual.pick(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled),
                Color::TRANSPARENT,
                if visual == Visual::Disabled { theme.text_on_accent_disabled } else { theme.text_on_accent },
            )
        } else {
            (
                visual.pick(
                    theme.control_alt_fill_secondary,
                    theme.control_alt_fill_tertiary,
                    theme.control_alt_fill_quarternary,
                    theme.control_alt_fill_disabled,
                ),
                visual.pick(
                    theme.control_strong_stroke,
                    theme.control_strong_stroke,
                    theme.control_strong_stroke,
                    theme.control_strong_stroke_disabled,
                ),
                if visual == Visual::Disabled { theme.text_disabled } else { theme.text_secondary },
            )
        };
        let text = if visual == Visual::Disabled { theme.text_disabled } else { theme.text_primary };
        // The knob grows under the pointer, and stretches while pressed.
        let (knob_width, knob_height) = visual.pick((12.0, 12.0), (14.0, 14.0), (17.0, 14.0), (12.0, 12.0));
        let fill = self.animate_color(response.id, 0, fill, DURATION_FASTER);
        let position = self.animate(response.id, 1, if *on { 1.0 } else { 0.0 }, DURATION_FAST, CURVE_DECELERATE);
        let knob_width = self.animate(response.id, 2, knob_width, DURATION_FASTER, CURVE_DECELERATE);
        let knob_height = self.animate(response.id, 3, knob_height, DURATION_FASTER, CURVE_DECELERATE);
        let travel_left = track.x + SWITCH_HEIGHT / 2.0;
        let travel_right = track.right() - SWITCH_HEIGHT / 2.0;
        let knob_center = Point::new(travel_left + (travel_right - travel_left) * position, track.center().y);
        let knob = Rect::from_center(knob_center, Size::new(knob_width, knob_height));
        let mut painter = self.painter();
        painter.shape(track, Corners::all(SWITCH_HEIGHT / 2.0), fill, Some(Stroke::new(stroke, 1.0)));
        painter.rounded(knob, knob_height / 2.0, knob_color);
        draw_label(&mut painter, rect, SWITCH_WIDTH + SPACE_M, label, text);
        focus_ring(self, &response, track, SWITCH_HEIGHT / 2.0);
        response
    }
}

fn draw_label(painter: &mut crate::painter::Painter, rect: Rect, indent: f32, label: &str, color: Color) {
    if label.is_empty() {
        return;
    }
    let area = Rect::new(rect.x + indent, rect.y, (rect.width - indent).max(0.0), rect.height);
    painter.text_fit(area, label, BODY, color, Align::Start);
}

// How wide a checkbox, radio button or switch is with its label.
impl Ui {
    pub fn checkbox_width(&mut self, label: &str) -> f32 {
        BOX + SPACE_S + self.measure(label, BODY).width.ceil()
    }

    pub fn switch_width(&mut self, label: &str) -> f32 {
        SWITCH_WIDTH + SPACE_M + self.measure(label, BODY).width.ceil()
    }
}
