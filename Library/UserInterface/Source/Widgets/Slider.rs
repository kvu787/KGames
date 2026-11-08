// WinUI's Slider, horizontal: a rail in ControlStrongFill, the part up to the
// value in the accent, and a round thumb whose accent dot is bigger with the
// pointer over it and smaller while it's pressed. A press anywhere on the
// rail moves the thumb there and drags it; the arrows, Page Up, Page Down,
// Home and End move it; and the wheel turns it a step a notch, adding up
// parts of notches. While it's dragged, its value shows above the thumb.
//
// Its value is an f64, which the program keeps in any number type. The
// rail's ends can be wider than the values allowed, like a rail from 0 to 1
// whose values must stay inside it, and the value never goes outside what's
// allowed. Only input moves the value: the slider sends the value the input
// asks for, and the program keeps it, so one from a file that's between
// steps or past the range stays until the user changes it.

use crate::core::interaction::{ControlState, Interaction, Sense};
use crate::core::tree::{Dispatch, Draw, Widget};
use crate::design::controls::{
    SLIDER_HEADER_GAP, SLIDER_HEIGHT, SLIDER_INNER_THUMB_SCALE, SLIDER_INNER_THUMB_SCALE_DISABLED, SLIDER_INNER_THUMB_SCALE_POINTER_OVER,
    SLIDER_INNER_THUMB_SCALE_PRESSED, SLIDER_INNER_THUMB_SIZE, SLIDER_THUMB_OVERHANG, SLIDER_THUMB_RADIUS, SLIDER_THUMB_SIZE,
    SLIDER_TOOLTIP_FONT_SIZE, SLIDER_TOOLTIP_PADDING, SLIDER_TRACK_HEIGHT, SLIDER_TRACK_RADIUS, TOOLTIP_OFFSET,
};
use crate::design::tokens::{BODY, ELEVATION_TOOLTIP, RADIUS_CONTROL};
use crate::drawing::geometry::{Insets, Point, Rect, Size};
use crate::drawing::painter::{Align, Border, Layer};
use crate::drawing::text::TextStyle;
use crate::input::devices::Key;
use crate::widgets::widget::{Elevation, Reaction, StateColors, react};

pub struct Slider<M> {
    value: f64,
    minimum: f64,
    maximum: f64,
    step: Option<f64>,
    large_step: Option<f64>,
    // The values allowed, inside the rail's ends.
    allowed: (f64, f64),
    header: Option<String>,
    // Decimals of the value shown in the header, and of the label.
    show_value: Option<usize>,
    decimals: usize,
    suffix: String,
    on_change: Option<Reaction<f64, M>>,
}

// The rail's ends in a slider's rectangle: where its thumb's center goes
// at its minimum and its maximum.
pub fn slider_rail(rect: Rect) -> (f32, f32) {
    let track = rect.bottom_strip(SLIDER_HEIGHT);
    (track.x + SLIDER_THUMB_SIZE / 2.0, track.right() - SLIDER_THUMB_SIZE / 2.0)
}

// Turns of the wheel that haven't added up to a step yet.
#[derive(Default)]
struct WheelRemainder(f32);

// Rounds away the noise of floating-point steps.
fn tidy(value: f64) -> f64 {
    (value * 1e9).round() / 1e9
}

impl<M: Clone + 'static> Slider<M> {
    pub fn new(value: impl Into<f64>, minimum: f64, maximum: f64) -> Self {
        Self {
            value: value.into(),
            minimum,
            maximum,
            step: None,
            large_step: None,
            allowed: (minimum.min(maximum), minimum.max(maximum)),
            header: None,
            show_value: None,
            decimals: 2,
            suffix: String::new(),
            on_change: None,
        }
    }

    // What moving it sends, with the value.
    pub fn on_change(mut self, reaction: impl Fn(f64) -> M + 'static) -> Self {
        self.on_change = Some(react(reaction));
        self
    }

    // The value moves in steps of this, from the minimum.
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    // How far Page Up and Page Down move it.
    pub fn large_step(mut self, step: f64) -> Self {
        self.large_step = Some(step);
        self
    }

    // The values allowed, inside the rail's ends.
    pub fn allowed(mut self, minimum: f64, maximum: f64) -> Self {
        self.allowed = (minimum.min(maximum), minimum.max(maximum));
        self
    }

    // A header above the rail.
    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    // Shows the value at the header's right, with this many decimals.
    pub fn show_value(mut self, decimals: usize) -> Self {
        self.show_value = Some(decimals);
        self.decimals = decimals;
        self
    }

    // The value's decimals, which show in the header and its tooltip.
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    // How tall a slider is, with a header or without.
    pub fn height(headed: bool) -> f32 {
        if headed { BODY.line_height + SLIDER_HEADER_GAP + SLIDER_HEIGHT } else { SLIDER_HEIGHT }
    }

    fn format(&self, value: f64) -> String {
        format!("{value:.*}{}", self.decimals, self.suffix)
    }

    fn fraction(&self, value: f64) -> f32 {
        let span = self.maximum - self.minimum;
        if span == 0.0 { 0.0 } else { ((value - self.minimum) / span).clamp(0.0, 1.0) as f32 }
    }

    // The value on a step nearest `value`, within what's allowed.
    fn snap(&self, value: f64) -> f64 {
        let snapped = match self.step {
            Some(step) if step > 0.0 => self.minimum + ((value - self.minimum) / step).round() * step,
            _ => value,
        };
        tidy(snapped).clamp(self.allowed.0, self.allowed.1)
    }

    fn track_row(&self, rect: Rect) -> Rect {
        rect.bottom_strip(SLIDER_HEIGHT)
    }
}

impl<M: Clone + 'static> Widget<M> for Slider<M> {
    fn kind(&self) -> &'static str {
        "Slider"
    }

    fn label(&self) -> Option<String> {
        Some(self.format(self.value))
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NAVIGATE.takes_wheel())
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let hit = interaction.whole();
        let rect = ctx.rect();
        let (left, right) = slider_rail(rect);
        let before = self.value;
        let mut value = before;
        let span = self.maximum - self.minimum;
        let step = self.step.unwrap_or(span / 100.0);
        let large_step = self.large_step.unwrap_or(span / 10.0);

        if hit.pressing()
            && let Some(pointer) = interaction.pointer
        {
            let fraction = ((pointer.x - left) / (right - left).max(1.0)).clamp(0.0, 1.0) as f64;
            value = self.snap(self.minimum + span * fraction);
        }
        if interaction.focused {
            let input = ctx.input();
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
        if hit.wheel.y != 0.0 {
            // Parts of notches add up, and a value between steps goes to the
            // next step the way the wheel turns, never back.
            let remainder = &mut ctx.memory::<WheelRemainder>().0;
            *remainder += hit.wheel.y;
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
        if value != before
            && let Some(reaction) = &self.on_change
        {
            ctx.send(reaction(value));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let track_row = self.track_row(rect);
        let (left, right) = slider_rail(rect);
        let value = self.value;
        let text = self.format(value);
        let state = ctx.state();
        let pressed = ctx.look().hit.pressing();
        let visual_state = if pressed { ControlState::Pressed } else { state };
        let scale = match visual_state {
            ControlState::PointerOver => SLIDER_INNER_THUMB_SCALE_POINTER_OVER,
            ControlState::Pressed => SLIDER_INNER_THUMB_SCALE_PRESSED,
            ControlState::Disabled => SLIDER_INNER_THUMB_SCALE_DISABLED,
            ControlState::Normal => SLIDER_INNER_THUMB_SCALE,
        };
        let theme = ctx.theme();
        let rail = StateColors::new(theme.control_strong_fill, theme.control_strong_fill, theme.control_strong_fill, theme.control_strong_fill_disabled);
        let accent = StateColors::new(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled);
        let header_color = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
        let value_color = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_secondary };
        let thumb_x = left + (right - left) * self.fraction(value);
        let center_y = track_row.center().y.round();
        let focused = ctx.focused();
        let mut painter = ctx.painter();
        let theme = painter.theme();
        if let Some(header) = &self.header {
            let header_row = Rect::new(rect.x, rect.y, rect.width, BODY.line_height);
            painter.text_fit(header_row, header, BODY, header_color, Align::Start);
            if self.show_value.is_some() {
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

        // The value above the thumb while it's dragged, like WinUI's thumb
        // tooltip.
        if pressed {
            let style = TextStyle::new(BODY.font, BODY.weight, SLIDER_TOOLTIP_FONT_SIZE, 20.0);
            let size = painter.measure(&text, style);
            let tip = Size::new(size.width.ceil() + SLIDER_TOOLTIP_PADDING.horizontal(), size.height + SLIDER_TOOLTIP_PADDING.vertical());
            let tip_rect = Rect::new((thumb_x - tip.width / 2.0).round(), (disc.y - TOOLTIP_OFFSET / 2.0 - tip.height).round(), tip.width, tip.height);
            let layer = painter.layer();
            painter.set_layer(Layer::Tooltip);
            painter.shadow(tip_rect, RADIUS_CONTROL, ELEVATION_TOOLTIP);
            painter.shape(tip_rect, RADIUS_CONTROL, theme.acrylic, Some(Border::new(theme.surface_stroke_flyout, 1.0)));
            painter.text_in(tip_rect.pad(SLIDER_TOOLTIP_PADDING), &text, style, theme.text_primary, Align::Center);
            painter.set_layer(layer);
        }
        ctx.focus_visual_around(focused, track_row, RADIUS_CONTROL, Insets::new(7.0, 0.0, 7.0, 0.0));
    }
}
