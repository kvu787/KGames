// Buttons, in Fluent's styles: standard, accent for the action that a card is
// for, subtle for buttons in a row of tools or inside another widget, and
// danger for an action that throws something away. A button acts on the
// press, which comes before the release, and on Space and Enter while it has
// keyboard focus.

use canvas::{Color, Fonts, Icon, Rect, TextStyle};

use crate::interface::{Id, Layer, Sense};
use crate::surfaces::{focus_ring, raised};
use crate::tokens::*;
use crate::{Frame, Response};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    #[default]
    Standard,
    Accent,
    Subtle,
    Danger,
}

// What a button shows: a label, an icon before it, or both.
#[derive(Clone, Copy, Debug)]
pub struct Button<'a> {
    pub label: &'a str,
    pub icon: Option<Icon>,
    pub style: ButtonStyle,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            style: ButtonStyle::Standard,
        }
    }

    pub fn icon_only(icon: Icon) -> Self {
        Self {
            label: "",
            icon: Some(icon),
            style: ButtonStyle::Standard,
        }
    }

    pub fn icon(self, icon: Icon) -> Self {
        Self { icon: Some(icon), ..self }
    }

    pub fn style(self, style: ButtonStyle) -> Self {
        Self { style, ..self }
    }

    pub fn accent(self) -> Self {
        self.style(ButtonStyle::Accent)
    }

    pub fn subtle(self) -> Self {
        self.style(ButtonStyle::Subtle)
    }

    pub fn danger(self) -> Self {
        self.style(ButtonStyle::Danger)
    }

    // How wide it is for what it shows: its label and icon, with padding,
    // or a square for an icon alone.
    pub fn width(&self, fonts: &Fonts) -> f32 {
        let label = fonts.measure(self.label, TextStyle::Body).ceil();
        match (self.icon.is_some(), self.label.is_empty()) {
            (true, true) => CONTROL_HEIGHT,
            (true, false) => label + 16.0 + SPACE_S + TEXT_INSET * 2.0,
            (false, _) => label + TEXT_INSET * 2.0,
        }
    }
}

impl Frame<'_> {
    pub fn button(&mut self, name: &str, area: Rect, button: Button) -> Response {
        self.button_on(Layer::Base, name, area, button)
    }

    // How wide a button is for what it shows.
    pub fn button_width(&self, button: &Button) -> f32 {
        button.width(self.fonts())
    }

    pub(crate) fn button_on(&mut self, layer: Layer, name: &str, area: Rect, button: Button) -> Response {
        self.button_part(layer, name, area, button, None)
    }

    // A button, or with an owner, a button that's a part of another widget,
    // like a text field's button that clears it, which doesn't take focus
    // from it.
    pub(crate) fn button_part(&mut self, layer: Layer, name: &str, area: Rect, button: Button, owner: Option<Id>) -> Response {
        let id = Id::new(name);
        let sense = if owner.is_some() { Sense::CLICK } else { Sense::BUTTON };
        let response = self.interact_on(name, "Button", button.label, area, sense, layer, owner);
        let (hover, press) = self.hover_and_press(id, &response);
        let enabled = self.enabled();
        let theme = self.theme();
        let pressed_text = |rest: Color, pressed: Color| rest.mix(pressed, press);
        let (text, icon) = match (button.style, enabled) {
            (ButtonStyle::Standard, true) => {
                let fill = theme.control.mix(theme.control_hover, hover).mix(theme.control_pressed, press);
                let bottom = theme.control_stroke_bottom.mix(theme.control_stroke, press);
                raised(self.layer(layer), area, RADIUS_CONTROL, fill, theme.control_stroke, bottom);
                let text = pressed_text(theme.text, theme.text_secondary);
                (text, text)
            }
            (ButtonStyle::Standard, false) => {
                let canvas = self.layer(layer);
                canvas.fill_stroke(area, RADIUS_CONTROL, theme.control_disabled, 1.0, theme.control_stroke);
                (theme.text_disabled, theme.text_disabled)
            }
            (ButtonStyle::Accent | ButtonStyle::Danger, _) => {
                let (rest, hovered, pressed) = if button.style == ButtonStyle::Accent {
                    (theme.accent, theme.accent_hover, theme.accent_pressed)
                } else {
                    let critical = theme.critical;
                    let lift = if theme.dark { Color::rgb(0x000000) } else { Color::rgb(0xFFFFFF) };
                    (critical, critical.mix(lift, 0.1), critical.mix(lift, 0.2))
                };
                let canvas = self.layer(layer);
                if enabled {
                    let fill = rest.mix(hovered, hover).mix(pressed, press);
                    let bottom = fill.mix(Color::rgb(0x000000), 0.3);
                    raised(canvas, area, RADIUS_CONTROL, fill, fill, bottom);
                    let text = pressed_text(theme.on_accent, theme.on_accent_secondary);
                    (text, text)
                } else {
                    canvas.fill(area, RADIUS_CONTROL, theme.accent_disabled);
                    (theme.on_accent, theme.on_accent)
                }
            }
            (ButtonStyle::Subtle, _) => {
                let fill = Color::TRANSPARENT.mix(theme.subtle_hover, hover).mix(theme.subtle_pressed, press);
                self.layer(layer).fill(area, RADIUS_CONTROL, fill);
                let text = if enabled { pressed_text(theme.text, theme.text_secondary) } else { theme.text_disabled };
                (text, if enabled { pressed_text(theme.text_secondary, theme.text_tertiary) } else { text })
            }
        };
        let canvas = self.layer(layer);
        let [middle_x, middle_y] = area.center();
        match (button.icon, button.label.is_empty()) {
            (Some(glyph), true) => canvas.icon([middle_x, middle_y], glyph, 16.0, icon),
            (Some(glyph), false) => {
                let width = 16.0 + SPACE_S + canvas.measure(button.label, TextStyle::Body);
                let x = (middle_x - width * 0.5).round();
                canvas.icon([x + 8.0, middle_y], glyph, 16.0, icon);
                let text_area = Rect::from_edges(x + 16.0 + SPACE_S, area.y, area.right() - TEXT_INSET, area.bottom());
                canvas.text_in(text_area, button.label, TextStyle::Body, text, canvas::Align::Start);
            }
            (None, _) => {
                let text_area = area.inset_by(SPACE_S, 0.0);
                canvas.text_in(text_area, button.label, TextStyle::Body, text, canvas::Align::Center);
            }
        }
        if self.shows_focus(&response) {
            let theme = self.theme();
            focus_ring(self.layer(layer), theme, area, RADIUS_CONTROL);
        }
        response
    }
}
