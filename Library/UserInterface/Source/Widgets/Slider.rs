// WinUI's Slider, horizontal: a rail in ControlStrongFill, the part up to the
// value in the accent, and a round thumb whose accent dot grows with the
// pointer over it and shrinks as it's pressed. A press anywhere on the rail
// moves the thumb there and drags it; the arrows, Page Up, Page Down, Home
// and End move it; and the wheel turns it a step a notch, adding up parts
// of notches. While it's dragged, its value shows above the thumb.
//
// Its value can be any number type. The rail's ends can be wider than the
// values allowed, like a rail from 0 to 1 whose values must stay inside it,
// and the value never goes outside what's allowed.

use crate::core::interaction::{ControlState, Response, Sense};
use crate::core::ui::Ui;
use crate::design::controls::{
    SLIDER_HEADER_GAP, SLIDER_HEIGHT, SLIDER_INNER_THUMB_DURATION, SLIDER_INNER_THUMB_DURATION_POINTER, SLIDER_INNER_THUMB_SCALE,
    SLIDER_INNER_THUMB_SCALE_DISABLED, SLIDER_INNER_THUMB_SCALE_POINTER_OVER, SLIDER_INNER_THUMB_SCALE_PRESSED, SLIDER_INNER_THUMB_SIZE,
    SLIDER_THUMB_OVERHANG, SLIDER_THUMB_RADIUS, SLIDER_THUMB_SIZE, SLIDER_TOOLTIP_FONT_SIZE, SLIDER_TOOLTIP_PADDING, SLIDER_TRACK_HEIGHT,
    SLIDER_TRACK_RADIUS, TOOLTIP_OFFSET,
};
use crate::design::tokens::{BODY, CURVE_FAST_OUT_SLOW_IN, ELEVATION_TOOLTIP, RADIUS_CONTROL};
use crate::drawing::text::TextStyle;
use crate::drawing::geometry::{Point, Rect, Size};
use crate::drawing::painter::{Align, Border, Layer};
use crate::input::devices::Key;
use crate::widgets::widget::{Elevation, StateColors, Widget};

// A number that a slider or a number box can change.
pub trait Number: Copy + PartialEq {
    fn to_f64(self) -> f64;
    fn from_f64(value: f64) -> Self;
}

macro_rules! numbers {
    ($($kind:ty),*) => {
        $(impl Number for $kind {
            fn to_f64(self) -> f64 {
                self as f64
            }

            fn from_f64(value: f64) -> Self {
                value as $kind
            }
        })*
    };
}

numbers!(f32, f64);

macro_rules! whole_numbers {
    ($($kind:ty),*) => {
        $(impl Number for $kind {
            fn to_f64(self) -> f64 {
                self as f64
            }

            fn from_f64(value: f64) -> Self {
                value.round() as $kind
            }
        })*
    };
}

whole_numbers!(i32, i64, u8, u32, usize);

// A step's value without what binary fractions add to it, like 0.107
// instead of 0.10700000000000001, to nine decimals.
fn tidy(value: f64) -> f64 {
    (value * 1e9).round() / 1e9
}

pub struct Slider<'a, 'v, T: Number> {
    value: &'v mut T,
    // The rail's ends.
    minimum: f64,
    maximum: f64,
    // The values allowed, inside the rail.
    allowed: (f64, f64),
    step: Option<f64>,
    large_step: Option<f64>,
    header: Option<&'a str>,
    // The value's decimals, which show in the header and its tooltip.
    decimals: usize,
    suffix: &'a str,
    show_value: bool,
}

impl<'a, 'v, T: Number> Slider<'a, 'v, T> {
    pub fn new(value: &'v mut T, minimum: f64, maximum: f64) -> Self {
        Self {
            value,
            minimum,
            maximum,
            allowed: (minimum.min(maximum), minimum.max(maximum)),
            step: None,
            large_step: None,
            header: None,
            decimals: 0,
            suffix: "",
            show_value: false,
        }
    }

    // Values snap to steps from the minimum, and the keys and wheel move by
    // one.
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    // How far Page Up and Page Down move it: a tenth of the rail if not set.
    pub fn large_step(mut self, step: f64) -> Self {
        self.large_step = Some(step);
        self
    }

    // Only these values are allowed, though the rail is wider.
    pub fn allowed(mut self, minimum: f64, maximum: f64) -> Self {
        self.allowed = (minimum.min(maximum), minimum.max(maximum));
        self
    }

    // A header above the rail, with the value at its right when show_value.
    pub fn header(mut self, header: &'a str) -> Self {
        self.header = Some(header);
        self
    }

    // Shows the value with this many decimals and the suffix, like "°".
    pub fn show_value(mut self, decimals: usize) -> Self {
        self.show_value = true;
        self.decimals = decimals;
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = suffix;
        self
    }

    // How tall a slider is, with or without its header.
    pub fn height(with_header: bool) -> f32 {
        if with_header { BODY.line_height + SLIDER_HEADER_GAP + SLIDER_HEIGHT } else { SLIDER_HEIGHT }
    }

    fn format(&self, value: f64) -> String {
        format!("{value:.*}{}", self.decimals, self.suffix)
    }

    fn snap(&self, value: f64) -> f64 {
        let value = match self.step {
            Some(step) if step > 0.0 => tidy(self.minimum + ((value - self.minimum) / step).round() * step),
            _ => value,
        };
        value.clamp(self.allowed.0, self.allowed.1)
    }

    fn fraction(&self, value: f64) -> f32 {
        let span = self.maximum - self.minimum;
        if span == 0.0 { 0.0 } else { ((value - self.minimum) / span).clamp(0.0, 1.0) as f32 }
    }
}

// The ends of a slider's rail in its rectangle, where the thumb's middle
// goes for its minimum and maximum, for tests that press along it.
pub fn slider_rail(rect: Rect) -> (f32, f32) {
    let track = rect.bottom_strip(SLIDER_HEIGHT);
    (track.x + SLIDER_THUMB_SIZE / 2.0, track.right() - SLIDER_THUMB_SIZE / 2.0)
}

// Turns of the wheel that haven't added up to a step yet.
#[derive(Default)]
struct WheelRemainder(f32);

impl<T: Number> Widget for Slider<'_, '_, T> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let track_row = rect.bottom_strip(SLIDER_HEIGHT);
        let mut response = ui.interact(name, track_row, Sense::NAVIGATE.takes_wheel(), "Slider");
        let (left, right) = slider_rail(rect);
        let before = self.value.to_f64();
        let mut value = before;
        let span = self.maximum - self.minimum;
        let step = self.step.unwrap_or(span / 100.0);
        let large_step = self.large_step.unwrap_or(span / 10.0);

        if response.held && response.held_button == Some(crate::input::devices::MouseButton::Left)
            && let Some(pointer) = response.pointer
        {
            let fraction = ((pointer.x - left) / (right - left).max(1.0)).clamp(0.0, 1.0) as f64;
            value = self.snap(self.minimum + span * fraction);
            value = value.clamp(self.allowed.0, self.allowed.1);
        }
        if response.focused {
            let input = ui.input();
            let mut moved = 0.0;
            if input.key_pressed(Key::RIGHT) || input.key_pressed(Key::UP) {
                moved += step;
            }
            if input.key_pressed(Key::LEFT) || input.key_pressed(Key::DOWN) {
                moved -= step;
            }
            if input.key_pressed(Key::PAGE_UP) {
                moved += large_step;
            }
            if input.key_pressed(Key::PAGE_DOWN) {
                moved -= large_step;
            }
            if moved != 0.0 {
                value = self.snap(value + moved * span.signum());
            }
            if input.key_pressed(Key::HOME) {
                value = self.allowed.0;
            }
            if input.key_pressed(Key::END) {
                value = self.allowed.1;
            }
        }
        if response.wheel.y != 0.0 {
            // Parts of notches add up, and a value between steps goes to the
            // next step the way the wheel turns, never back.
            let remainder = &mut ui.memory::<WheelRemainder>(response.id).0;
            *remainder += response.wheel.y;
            let notches = remainder.trunc();
            *remainder -= notches;
            if notches != 0.0 {
                let direction = (notches as f64) * span.signum();
                let current = match self.step {
                    Some(step) if step > 0.0 => {
                        // A value on a step, give or take rounding, is on it;
                        // between steps, it goes from the one behind it.
                        let index = (value - self.minimum) / step;
                        let nearest = index.round();
                        let snapped = if (index - nearest).abs() < 1e-6 {
                            nearest
                        } else if direction > 0.0 {
                            index.floor()
                        } else {
                            index.ceil()
                        };
                        self.minimum + snapped * step
                    }
                    _ => value,
                };
                let moved = tidy(current + direction * step).clamp(self.allowed.0, self.allowed.1);
                // Never back against the wheel, like from a value just past
                // what's allowed.
                if (moved - value) * direction > 0.0 {
                    value = moved;
                }
            }
        }
        // Only input moves the value: one that's outside what's allowed,
        // like from a file, stays until the user changes it.
        if value != before {
            *self.value = T::from_f64(value);
            response.changed = true;
        }
        let value = self.value.to_f64();
        let text = self.format(value);
        ui.set_label(name, &text);

        let state = response.state();
        let pressed = response.held && response.held_button == Some(crate::input::devices::MouseButton::Left);
        let visual_state = if pressed { ControlState::Pressed } else { state };
        let (scale, duration) = match visual_state {
            ControlState::PointerOver => (SLIDER_INNER_THUMB_SCALE_POINTER_OVER, SLIDER_INNER_THUMB_DURATION_POINTER),
            ControlState::Pressed => (SLIDER_INNER_THUMB_SCALE_PRESSED, SLIDER_INNER_THUMB_DURATION_POINTER),
            ControlState::Disabled => (SLIDER_INNER_THUMB_SCALE_DISABLED, SLIDER_INNER_THUMB_DURATION),
            ControlState::Normal => (SLIDER_INNER_THUMB_SCALE, SLIDER_INNER_THUMB_DURATION),
        };
        let scale = ui.animate(response.id, 0, scale, duration, CURVE_FAST_OUT_SLOW_IN);
        let theme = ui.theme();
        let rail = StateColors::new(theme.control_strong_fill, theme.control_strong_fill, theme.control_strong_fill, theme.control_strong_fill_disabled);
        let accent = StateColors::new(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled);
        let header_color = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
        let value_color = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_secondary };
        let thumb_x = left + (right - left) * self.fraction(value);
        let center_y = track_row.center().y.round();
        let mut painter = ui.painter();
        let theme = painter.theme();
        if let Some(header) = self.header {
            let header_row = Rect::new(rect.x, rect.y, rect.width, BODY.line_height);
            painter.text_fit(header_row, header, BODY, header_color, Align::Start);
            if self.show_value {
                painter.text_in(header_row, &text, BODY, value_color, Align::End);
            }
        }
        let track = Rect::new(track_row.x, center_y - SLIDER_TRACK_HEIGHT / 2.0, track_row.width, SLIDER_TRACK_HEIGHT);
        painter.rounded(track, SLIDER_TRACK_RADIUS, rail.get(state));
        // The accent fills from the minimum's end, or from zero when the
        // rail goes through it.
        let origin = if self.minimum < 0.0 && self.maximum > 0.0 { left + (right - left) * self.fraction(0.0) } else { track.x };
        let (from, to) = if thumb_x >= origin { (origin, thumb_x) } else { (thumb_x, origin) };
        painter.rounded(Rect::new(from, track.y, to - from, track.height), SLIDER_TRACK_RADIUS, accent.get(state));
        // The thumb is 18 pixels in layout, and its disc overhangs that by 2
        // on each side, with ControlElevationBorderBrush around it.
        let thumb = Rect::from_center(Point::new(thumb_x, center_y), Size::square(SLIDER_THUMB_SIZE));
        let disc = thumb.expand(SLIDER_THUMB_OVERHANG);
        painter.shape(disc, SLIDER_THUMB_RADIUS, theme.control_solid_fill, Some(Elevation::Control.border(theme, disc)));
        let inner = SLIDER_INNER_THUMB_SIZE * scale;
        painter.circle(thumb.center(), inner / 2.0, accent.get(visual_state));
        ui.focus_visual_around(response.focused, track_row, RADIUS_CONTROL, crate::drawing::geometry::Insets::new(7.0, 0.0, 7.0, 0.0));

        // The value above the thumb while it's dragged, like WinUI's thumb
        // tooltip.
        if pressed {
            let style = TextStyle::new(BODY.font, BODY.weight, SLIDER_TOOLTIP_FONT_SIZE, 20.0);
            let size = ui.measure(&text, style);
            let tip = Size::new(size.width.ceil() + SLIDER_TOOLTIP_PADDING.horizontal(), size.height + SLIDER_TOOLTIP_PADDING.vertical());
            let tip_rect = Rect::new((thumb_x - tip.width / 2.0).round(), (disc.y - TOOLTIP_OFFSET / 2.0 - tip.height).round(), tip.width, tip.height);
            let layer = ui.layer();
            ui.draw.set_layer(Layer::Tooltip);
            let mut painter = ui.painter();
            let theme = painter.theme();
            painter.shadow(tip_rect, RADIUS_CONTROL, ELEVATION_TOOLTIP);
            painter.shape(tip_rect, RADIUS_CONTROL, theme.acrylic, Some(Border::new(theme.surface_stroke_flyout, 1.0)));
            painter.text_in(tip_rect.pad(SLIDER_TOOLTIP_PADDING), &text, style, theme.text_primary, Align::Center);
            ui.draw.set_layer(layer);
        }
        response
    }
}
