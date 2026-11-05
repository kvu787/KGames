// A slider: a track that sets a number from where it's pressed and dragged
// along it, which the wheel and the arrow keys move a step at a time.
//
// Its value is always a whole number of steps, between its minimum and
// maximum, so a program can make a slider only make values that it accepts.
// A value from elsewhere, like a file, can be between steps, or outside the
// range, and stays as it is until the slider moves.

use application::Event;
use canvas::Rect;
use canvas::theme::{ACCENT, BORDER, FAINT, PANEL};
use windows_api::*;

use crate::interface::{Id, Layer, Sense};
use crate::{Frame, Response};

// The handle's radius. It grows by 2 pixels while it's over the cursor or
// dragged. A slider's area reaches twice this past each end of its track, so
// that the handle is in it at either end.
pub const HANDLE_RADIUS: f32 = 8.0;

const KEYS: &[i32] = &[VK_LEFT, VK_RIGHT, VK_HOME, VK_END];

// The values a slider makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderRange {
    // The values at the track's left and right ends.
    pub start: f64,
    pub end: f64,
    // The first and last steps that the slider makes, which can be inside
    // the track's ends, like for a value that must be above 0.
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

    // Where a value is along the track, from 0 to 1.
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

// The left and right ends of the track of a slider in `area`.
pub fn track(area: Rect) -> [f32; 2] {
    [area.x + HANDLE_RADIUS * 2.0, area.right() - HANDLE_RADIUS * 2.0]
}

impl Frame<'_> {
    // A slider across `area`, whose track runs from twice the handle's
    // radius past its left edge to as far before its right. A press sets the
    // value from where it is along the track, and dragging moves it. Each
    // notch of the wheel moves it a step, and so do Left and Right while it
    // has focus, or 10 with Shift, and Home and End take it to its minimum
    // and maximum.
    pub fn slider(&mut self, name: &str, area: Rect, value: &mut f64, range: &SliderRange) -> Response {
        let id = Id::new(name);
        let sense = Sense {
            click: true,
            wheel: true,
            focus: true,
            keys: KEYS,
            text: false,
        };
        let mut response = self.interact_on(name, "Slider", "", area, sense, Layer::Base, None);
        let before = *value;
        let [left, right] = track(area);
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
                        VK_LEFT => range.nudge(*value, -steps),
                        VK_RIGHT => range.nudge(*value, steps),
                        VK_HOME => range.minimum,
                        VK_END => range.maximum,
                        _ => *value,
                    };
                }
                _ => {}
            }
        }
        // The handle follows the cursor while the slider has the press.
        if response.held
            && let Some([x, _]) = self.cursor()
        {
            *value = from_x(x);
        }
        response.changed = *value != before;

        let enabled = self.enabled();
        let color = if enabled { ACCENT } else { FAINT };
        let canvas = &mut self.canvas;
        let middle = area.y + area.height * 0.5;
        let handle_x = left + (right - left) * range.fraction(*value) as f32;
        canvas.rect(left, middle - 2.0, right - left, 4.0, BORDER);
        canvas.rect(left, middle - 2.0, handle_x - left, 4.0, color);
        let radius = if (response.held || response.hovered) && enabled {
            HANDLE_RADIUS + 2.0
        } else {
            HANDLE_RADIUS
        };
        canvas.circle([handle_x, middle], radius, color);
        canvas.circle([handle_x, middle], radius - 3.0, PANEL);
        if self.shows_focus(&response) {
            self.canvas.ring([handle_x, middle], radius + 4.0, 2.0, ACCENT);
        }
        response
    }
}
