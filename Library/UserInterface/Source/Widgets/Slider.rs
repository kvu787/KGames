// Sliders, progress bars and progress rings, as WinUI's Slider, ProgressBar
// and ProgressRing.

use crate::geometry::{Point, Rect, Size};
use crate::input::Key;
use crate::painter::{Align, Corners, Layer, Stroke};
use crate::tokens::{CAPTION, CURVE_DECELERATE, DURATION_FASTER, ELEVATION_16, RADIUS_CONTROL};
use crate::ui::{Response, Sense, Ui};
use crate::widgets::common::{Visual, focus_ring};

const THUMB: f32 = 20.0;
const TRACK: f32 = 4.0;

// A slider's range and steps.
#[derive(Clone, Copy, Debug)]
pub struct Slider {
    pub min: f32,
    pub max: f32,
    // Each value is a whole number of steps from min. 0 for any value.
    pub step: f32,
    // What Page Up and Page Down move it by, and Shift with an arrow.
    pub large_step: f32,
    // Shows the value above the thumb while it's dragged, with this many
    // decimals.
    pub decimals: Option<usize>,
}

impl Slider {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            step: 0.0,
            large_step: (max - min) / 10.0,
            decimals: None,
        }
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self.large_step = step * 10.0;
        self
    }

    pub fn large_step(mut self, step: f32) -> Self {
        self.large_step = step;
        self
    }

    pub fn show_value(mut self, decimals: usize) -> Self {
        self.decimals = Some(decimals);
        self
    }

    // The value on a step, inside the range: a range that isn't a whole
    // number of steps stops at its last step.
    pub fn snap(&self, value: f32) -> f32 {
        if self.step <= 0.0 {
            return value.clamp(self.min, self.max);
        }
        let steps = ((self.max - self.min) / self.step + 1e-4).floor();
        let index = ((value - self.min) / self.step).round().clamp(0.0, steps);
        self.min + index * self.step
    }

    fn fraction(&self, value: f32) -> f32 {
        if self.max <= self.min { 0.0 } else { ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0) }
    }
}

// Where a slider's thumb travels: THUMB / 2 in from each end of its area.
pub fn slider_rail(rect: Rect) -> (f32, f32) {
    (rect.x + THUMB / 2.0, rect.right() - THUMB / 2.0)
}

impl Ui {
    pub fn slider(&mut self, name: &str, rect: Rect, value: &mut f32, slider: Slider) -> Response {
        let mut response = self.interact(name, rect, Sense::NAVIGATE, "Slider");
        let before = *value;
        let (left, right) = slider_rail(rect);
        if response.held
            && let Some(pointer) = self.input().pointer
        {
            let t = ((pointer.x - left) / (right - left)).clamp(0.0, 1.0);
            *value = slider.snap(slider.min + t * (slider.max - slider.min));
        }
        let step = if slider.step > 0.0 { slider.step } else { (slider.max - slider.min) / 100.0 };
        if response.focused && self.enabled() {
            let input = self.input();
            let large = input.modifiers().shift;
            let amount = if large { slider.large_step } else { step };
            let mut target = *value;
            if input.key_pressed(Key::RIGHT) || input.key_pressed(Key::UP) {
                target += amount;
            }
            if input.key_pressed(Key::LEFT) || input.key_pressed(Key::DOWN) {
                target -= amount;
            }
            if input.key_pressed(Key::PAGE_UP) {
                target += slider.large_step;
            }
            if input.key_pressed(Key::PAGE_DOWN) {
                target -= slider.large_step;
            }
            if input.key_pressed(Key::HOME) {
                target = slider.min;
            }
            if input.key_pressed(Key::END) {
                target = slider.max;
            }
            if target != *value {
                *value = slider.snap(target);
                self.set_focus_visible(true);
            }
        }
        // The wheel moves it a step a notch, up for more.
        if response.hovered && !response.held {
            let notches = self.input().wheel.y.round();
            if notches != 0.0 {
                *value = slider.snap(*value + notches * step);
            }
        }
        response.changed = *value != before;
        draw_slider(self, &response, rect, *value, &slider);
        response
    }

    // How far along a task is, from 0 to 1, or None for a task whose end
    // isn't known, which shows as a segment that moves.
    pub fn progress_bar(&mut self, rect: Rect, value: Option<f32>) {
        let theme = self.theme();
        let track_color = if self.enabled() { theme.control_strong_stroke } else { theme.control_strong_stroke_disabled };
        let fill = if self.enabled() { theme.accent } else { theme.accent_disabled };
        let time = self.time();
        let mut painter = self.painter();
        let middle = rect.center().y.round();
        let track = Rect::new(rect.x, middle - 0.5, rect.width, 1.0);
        let bar = |from: f32, to: f32| Rect::new(rect.x + rect.width * from, middle - 1.5, rect.width * (to - from), 3.0);
        match value {
            Some(value) => {
                painter.rounded(track, 0.5, track_color);
                let value = value.clamp(0.0, 1.0);
                if value > 0.0 {
                    painter.rounded(bar(0.0, value), 1.5, fill);
                }
            }
            None => {
                // A segment that sweeps across, every two seconds.
                let phase = (time % 2.0 / 2.0) as f32;
                let start = (phase * 1.6 - 0.4).clamp(0.0, 1.0);
                let end = (phase * 1.6).clamp(0.0, 1.0);
                if end > start {
                    painter.push_clip(rect);
                    painter.rounded(bar(start, end), 1.5, fill);
                    painter.pop_clip();
                }
            }
        }
    }

    // A ring that fills clockwise, or spins for a task whose end isn't known.
    pub fn progress_ring(&mut self, center: Point, diameter: f32, value: Option<f32>) {
        let theme = self.theme();
        let fill = if self.enabled() { theme.accent } else { theme.accent_disabled };
        let track = theme.control_strong_stroke_disabled;
        let time = self.time() as f32;
        let width = (diameter / 8.0).max(2.0);
        let radius = (diameter - width) / 2.0;
        let mut painter = self.painter();
        match value {
            Some(value) => {
                painter.ring(center, radius + width / 2.0, width, track);
                painter.arc(center, radius, width, 0.0, value.clamp(0.0, 1.0) * std::f32::consts::TAU, fill);
            }
            None => {
                let turn = time * std::f32::consts::TAU * 0.8;
                let sweep = (1.0 + (time * 2.0).sin()) * 1.2 + 0.4;
                painter.arc(center, radius, width, turn, sweep, fill);
            }
        }
    }
}

fn draw_slider(ui: &mut Ui, response: &Response, rect: Rect, value: f32, slider: &Slider) {
    let visual = Visual::of(ui, response);
    let theme = ui.theme();
    let (left, right) = slider_rail(rect);
    let middle = rect.center().y.round();
    let rail_color = if visual == Visual::Disabled { theme.control_strong_fill_disabled } else { theme.control_strong_fill };
    let value_color = visual.pick(theme.accent, theme.accent, theme.accent_tertiary, theme.accent_disabled);
    let thumb_fill = theme.control_solid_fill;
    let thumb_stroke = Stroke::elevated(theme.control_stroke, theme.control_stroke_secondary);
    let tooltip_fill = theme.flyout;
    let tooltip_stroke = theme.surface_stroke_flyout;
    let text = theme.text_primary;
    let inner = visual.pick(12.0, 14.0, 10.0, 12.0);
    let inner = ui.animate(response.id, 0, inner, DURATION_FASTER, CURVE_DECELERATE);
    let x = left + (right - left) * slider.fraction(value);
    let held = response.held;
    let mut painter = ui.painter();
    let rail = Rect::new(rect.x + THUMB / 2.0 - TRACK / 2.0, middle - TRACK / 2.0, rect.width - THUMB + TRACK, TRACK);
    painter.rounded(rail, TRACK / 2.0, rail_color);
    painter.rounded(Rect::new(rail.x, rail.y, x - rail.x, TRACK), TRACK / 2.0, value_color);
    let thumb = Rect::from_center(Point::new(x, middle), Size::new(THUMB, THUMB));
    painter.shape(thumb, Corners::all(THUMB / 2.0), thumb_fill, Some(thumb_stroke));
    painter.circle(Point::new(x, middle), inner / 2.0, value_color);
    if held && let Some(decimals) = slider.decimals {
        let label = format!("{value:.decimals$}");
        let size = painter.measure(&label, CAPTION);
        let bubble = Rect::from_center(Point::new(x, thumb.top() - 22.0), Size::new(size.width + 18.0, 28.0)).round();
        painter.set_layer(Layer::Tooltip);
        painter.shadow(bubble, RADIUS_CONTROL, ELEVATION_16);
        painter.shape(bubble, Corners::all(RADIUS_CONTROL), tooltip_fill, Some(Stroke::new(tooltip_stroke, 1.0)));
        painter.text_in(bubble, &label, CAPTION, text, Align::Center);
        painter.set_layer(Layer::Base);
    }
    focus_ring(ui, response, thumb, THUMB / 2.0);
}
