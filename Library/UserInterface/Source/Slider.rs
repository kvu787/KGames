// A slider: a rail that sets a number from where it's pressed and dragged
// along it, which the wheel and the arrow keys move a step at a time, and
// progress bars.
//
// A slider's value is always a whole number of steps, between its minimum
// and maximum, so a program can make a slider only make values that it
// accepts. A value from elsewhere, like a file, can be between steps, or
// outside the range, and stays as it is until the slider moves.

use application::Event;
use canvas::{Color, Rect};
use windows_api::*;

use crate::interface::{Id, Layer, Sense};
use crate::surfaces::{focus_ring, raised};
use crate::tokens::*;
use crate::{Frame, Response};

const KEYS: &[i32] = &[VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN, VK_HOME, VK_END, VK_PRIOR, VK_NEXT];

// The values a slider makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderRange {
    // The values at the rail's left and right ends.
    pub start: f64,
    pub end: f64,
    // The first and last steps that the slider makes, which can be inside
    // the rail's ends, like for a value that must be above 0.
    pub minimum: f64,
    pub maximum: f64,
    // Steps per unit, so a value on a step is a whole number of them, like
    // 1000 for steps of 0.001.
    pub steps: f64,
}

impl SliderRange {
    // A range from `start` to `end`, both of which the slider makes.
    pub const fn new(start: f64, end: f64, steps: f64) -> Self {
        Self {
            start,
            end,
            minimum: start,
            maximum: end,
            steps,
        }
    }

    // Where a value is along the rail, from 0 to 1.
    pub fn fraction(&self, value: f64) -> f64 {
        ((value - self.start) / (self.end - self.start)).clamp(0.0, 1.0)
    }

    // The step nearest `value`, within the range.
    pub fn nearest(&self, value: f64) -> f64 {
        // Dividing whole numbers, like 107 / 1000, gives the double nearest
        // the decimal, so values save as written, like 0.107.
        let stepped = (value * self.steps).round() / self.steps;
        stepped.clamp(self.minimum, self.maximum)
    }

    // The value `steps` steps from `value`. A value from elsewhere can be
    // past the range, like 0.9990234375 past 0.999, where the nearest step
    // back in it is the other way. Then it stays put.
    pub fn nudge(&self, value: f64, steps: f64) -> f64 {
        let next = self.nearest(value + steps / self.steps);
        if (next - value) * steps > 0.0 { next } else { value }
    }
}

// The left and right ends of the rail of a slider in `area`, which stop a
// thumb's radius short of its edges, so that the thumb is in it at either
// end.
pub fn slider_rail(area: Rect) -> [f32; 2] {
    [area.x + THUMB_RADIUS, area.right() - THUMB_RADIUS]
}

impl Frame<'_> {
    // A slider across `area`. A press sets the value from where it is along
    // the rail, and dragging moves it. Each notch of the wheel moves it a
    // step, and so do the arrow keys while it has focus, or 10 with Shift,
    // and so do Page Up and Page Down. Home and End take it to its minimum
    // and maximum.
    pub fn slider(&mut self, name: &str, area: Rect, value: &mut f64, range: &SliderRange) -> Response {
        let id = Id::new(name);
        let sense = Sense {
            click: true,
            wheel: true,
            focus: true,
            keys: KEYS,
            ..Sense::HOVER
        };
        let mut response = self.interact_on(name, "Slider", "", area, sense, Layer::Base, None);
        let before = *value;
        let [left, right] = slider_rail(area);
        let from_x = |x: f32| {
            let fraction = ((x - left) / (right - left)).clamp(0.0, 1.0) as f64;
            range.nearest(range.start + fraction * (range.end - range.start))
        };
        for event in self.events_for(id) {
            match event {
                Event::Press([x, _]) => *value = from_x(x),
                Event::Wheel(_, notches) => {
                    let (last, part) = self.state.wheel;
                    let turned = notches + if last == Some(id) { part } else { 0.0 };
                    let steps = turned.trunc();
                    self.state.wheel = (Some(id), turned - steps);
                    if steps != 0.0 {
                        *value = range.nudge(*value, steps as f64);
                    }
                }
                Event::Key { key, shift, .. } => {
                    let steps = if shift { 10.0 } else { 1.0 };
                    *value = match key as i32 {
                        VK_LEFT | VK_DOWN => range.nudge(*value, -steps),
                        VK_RIGHT | VK_UP => range.nudge(*value, steps),
                        VK_PRIOR => range.nudge(*value, 10.0),
                        VK_NEXT => range.nudge(*value, -10.0),
                        VK_HOME => range.minimum,
                        VK_END => range.maximum,
                        _ => *value,
                    };
                }
                _ => {}
            }
        }
        // The thumb follows the cursor while the slider has the press.
        if response.held
            && let Some([x, _]) = self.cursor()
        {
            *value = from_x(x);
        }
        response.changed = *value != before;

        let (hover, press) = self.hover_and_press(id, &response);
        let enabled = self.enabled();
        let theme = self.theme();
        let canvas = &mut self.canvas;
        let middle = area.center()[1].round();
        let thumb_x = left + (right - left) * range.fraction(*value) as f32;
        let (filled, rail) = if enabled {
            (theme.accent.mix(theme.accent_hover, hover), theme.control_strong)
        } else {
            (theme.accent_disabled, theme.text_disabled)
        };
        canvas.fill(Rect::from_edges(left, middle - 2.0, right, middle + 2.0), 2.0, rail);
        canvas.fill(Rect::from_edges(left, middle - 2.0, thumb_x, middle + 2.0), 2.0, filled);
        // The thumb: a raised circle, with a dot of the accent in it that
        // grows while the cursor is over it and shrinks while it's dragged.
        let thumb = Rect::new(thumb_x - THUMB_RADIUS, middle - THUMB_RADIUS, THUMB_RADIUS * 2.0, THUMB_RADIUS * 2.0);
        raised(canvas, thumb, THUMB_RADIUS, theme.control, theme.control_stroke, theme.control_stroke_bottom);
        let dot = 12.0 + 2.0 * hover - 4.0 * press;
        canvas.circle([thumb_x, middle], dot * 0.5, filled);
        if self.shows_focus(&response) {
            focus_ring(&mut self.canvas, theme, thumb, THUMB_RADIUS);
        }
        response
    }

    // A bar filled from the left to `fraction`, from 0 to 1, in `color`, or
    // the accent.
    pub fn progress(&mut self, area: Rect, fraction: f32, color: Option<Color>) {
        let theme = self.theme();
        let color = color.unwrap_or(theme.accent);
        let canvas = &mut self.canvas;
        let radius = area.height * 0.5;
        canvas.fill(area, radius, theme.control_strong.alpha(0.3));
        let filled = (area.width * fraction.clamp(0.0, 1.0)).round();
        if filled > 0.0 {
            canvas.fill(Rect::new(area.x, area.y, filled.max(area.height), area.height), radius, color);
        }
    }
}
