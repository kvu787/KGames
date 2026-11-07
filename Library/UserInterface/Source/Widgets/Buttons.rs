// WinUI's buttons: Button, in its standard, accent and subtle styles, with
// text, an icon, or both, and a chevron as a DropDownButton has; ToggleButton,
// which is the accent's colors while it's on; and HyperlinkButton.
//
// Their backgrounds cross-fade between visual states over
// ControlFasterAnimationDuration, as WinUI's BrushTransition does; their
// text and borders change at once.

use crate::core::interaction::{ControlState, Cursor, Response, Sense};
use crate::core::ui::Ui;
use crate::design::controls::{BUTTON_ICON_GAP, BUTTON_ICON_ONLY_WIDTH, BUTTON_PADDING, DROP_DOWN_CHEVRON_GAP, DROP_DOWN_CHEVRON_SIZE, HYPERLINK_PADDING};
use crate::design::icons::Icon;
use crate::design::theme::Theme;
use crate::design::tokens::{BODY, DURATION_FASTER, ICON, ICON_SMALL, RADIUS_CONTROL};
use crate::drawing::geometry::{Color, Point, Rect};
use crate::drawing::painter::{Align, Border};
use crate::widgets::widget::{Elevation, StateColors, Widget};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonStyle {
    #[default]
    Standard,
    // AccentButtonStyle, for the one command that matters most.
    Accent,
    // Invisible until the pointer is over it, like a toolbar's buttons.
    Subtle,
}

#[derive(Clone, Copy, Debug)]
pub struct Button<'a> {
    text: &'a str,
    icon: Option<Icon>,
    style: ButtonStyle,
    // Drawn in the accent's colors, like a ToggleButton that's on.
    checked: bool,
    // A chevron after the text, like a DropDownButton's.
    chevron: bool,
}

impl<'a> Button<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            icon: None,
            style: ButtonStyle::Standard,
            checked: false,
            chevron: false,
        }
    }

    // A square button with only an icon, which wants a tooltip to say what
    // it does.
    pub fn icon_only(icon: Icon) -> Self {
        Self::new("").icon(icon)
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn accent(mut self) -> Self {
        self.style = ButtonStyle::Accent;
        self
    }

    pub fn subtle(mut self) -> Self {
        self.style = ButtonStyle::Subtle;
        self
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn chevron(mut self) -> Self {
        self.chevron = true;
        self
    }

    // The width that fits its content, between its padding.
    pub fn width(&self, ui: &mut Ui) -> f32 {
        if self.text.is_empty() && !self.chevron {
            return BUTTON_ICON_ONLY_WIDTH;
        }
        let mut width = BUTTON_PADDING.horizontal() + 2.0;
        if self.icon.is_some() {
            width += ICON;
            if !self.text.is_empty() {
                width += BUTTON_ICON_GAP;
            }
        }
        width += ui.measure(self.text, BODY).width.ceil();
        if self.chevron {
            width += DROP_DOWN_CHEVRON_GAP + ICON_SMALL;
        }
        width
    }
}

// The brushes of each visual state: background, foreground and border.
struct ButtonColors {
    background: StateColors,
    foreground: StateColors,
    border: Option<(Elevation, StateColors)>,
}

fn colors(theme: &Theme, style: ButtonStyle, checked: bool) -> ButtonColors {
    if checked || style == ButtonStyle::Accent {
        // AccentButton, and a ToggleButton that's checked.
        return ButtonColors {
            background: StateColors::new(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled),
            foreground: StateColors::new(theme.text_on_accent, theme.text_on_accent, theme.text_on_accent_secondary, theme.text_on_accent_disabled),
            border: Some((Elevation::Accent, StateColors::new(Color::TRANSPARENT, Color::TRANSPARENT, theme.control_fill_transparent, theme.control_fill_transparent))),
        };
    }
    match style {
        ButtonStyle::Subtle => ButtonColors {
            background: StateColors::new(theme.subtle_fill_transparent, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, theme.subtle_fill_disabled),
            foreground: StateColors::new(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled),
            border: None,
        },
        _ => ButtonColors {
            background: StateColors::new(theme.control_fill, theme.control_fill_secondary, theme.control_fill_tertiary, theme.control_fill_disabled),
            foreground: StateColors::new(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled),
            border: Some((Elevation::Control, StateColors::new(Color::TRANSPARENT, Color::TRANSPARENT, theme.control_stroke, theme.control_stroke))),
        },
    }
}

impl Widget for Button<'_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let response = ui.interact(name, rect, Sense::BUTTON, "Button");
        if !self.text.is_empty() {
            ui.set_label(name, self.text);
        }
        draw_button(ui, &self, &response, rect);
        response
    }
}

fn draw_button(ui: &mut Ui, button: &Button, response: &Response, rect: Rect) {
    let state = response.state();
    let colors = colors(ui.theme(), button.style, button.checked);
    let background = ui.animate_color(response.id, 0, colors.background.get(state), DURATION_FASTER);
    let foreground = colors.foreground.get(state);
    let mut painter = ui.painter();
    let theme = painter.theme();
    // At rest and with the pointer over, the border is the elevation
    // gradient; pressed and disabled, it's flat.
    let border = colors.border.map(|(elevation, flat)| match state {
        ControlState::Normal | ControlState::PointerOver => elevation.border(theme, rect),
        _ => Border::new(flat.get(state), 1.0),
    });
    painter.shape(rect, RADIUS_CONTROL, background, border);
    let content = rect.pad(BUTTON_PADDING);
    let text_width = if button.text.is_empty() { 0.0 } else { painter.measure(button.text, BODY).width };
    let icon_width = if button.icon.is_some() { ICON + if button.text.is_empty() { 0.0 } else { BUTTON_ICON_GAP } } else { 0.0 };
    let chevron_width = if button.chevron { DROP_DOWN_CHEVRON_GAP + ICON_SMALL } else { 0.0 };
    let total = (icon_width + text_width + chevron_width).min(content.width.max(0.0));
    let mut x = (rect.center().x - total / 2.0).max(content.x).round();
    if button.chevron && !button.text.is_empty() {
        // A drop-down button's text starts at its padding, and its chevron
        // ends at the other side, as WinUI's do.
        x = content.x;
    }
    if let Some(icon) = button.icon {
        painter.icon(Point::new(x + ICON / 2.0, rect.center().y), icon, ICON, foreground);
        x += icon_width;
    }
    if !button.text.is_empty() {
        let room = if button.chevron { content.right() - x - chevron_width } else { content.right() - x };
        painter.text_fit(Rect::new(x, rect.y, room.max(0.0), rect.height), button.text, BODY, foreground, Align::Start);
    }
    if button.chevron {
        // DropDownButtonForegroundSecondary, unless it's on the accent.
        let chevron = if button.checked || button.style == ButtonStyle::Accent {
            foreground
        } else {
            match state {
                ControlState::Normal => theme.text_secondary,
                ControlState::Disabled => theme.text_disabled,
                _ => theme.text_tertiary,
            }
        };
        let center = Point::new(content.right() - ICON_SMALL / 2.0, rect.center().y);
        painter.icon(center, Icon::ChevronDownSmall, DROP_DOWN_CHEVRON_SIZE, chevron);
    }
    ui.focus_visual_if(response.focused, rect, RADIUS_CONTROL);
}

// A button that stays on or off, in the accent's colors while it's on.
pub struct ToggleButton<'a, 'v> {
    value: &'v mut bool,
    button: Button<'a>,
}

impl<'a, 'v> ToggleButton<'a, 'v> {
    pub fn new(value: &'v mut bool, button: Button<'a>) -> Self {
        Self { value, button }
    }
}

impl Widget for ToggleButton<'_, '_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let mut response = ui.interact(name, rect, Sense::BUTTON, "ToggleButton");
        if response.clicked {
            *self.value = !*self.value;
            response.changed = true;
        }
        ui.set_label(name, if *self.value { "On" } else { "Off" });
        draw_button(ui, &self.button.checked(*self.value), &response, rect);
        response
    }
}

// A link that's a button: accent-colored text, which takes a subtle fill
// with the pointer over it.
pub struct HyperlinkButton<'a> {
    text: &'a str,
}

impl<'a> HyperlinkButton<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn width(&self, ui: &mut Ui) -> f32 {
        HYPERLINK_PADDING.horizontal() + ui.measure(self.text, BODY).width.ceil()
    }
}

impl Widget for HyperlinkButton<'_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let response = ui.interact(name, rect, Sense::BUTTON, "HyperlinkButton");
        ui.set_label(name, self.text);
        let state = response.state();
        if state != ControlState::Disabled && response.hovered {
            ui.set_cursor(Cursor::Hand);
        }
        let theme = ui.theme();
        let background = StateColors::new(theme.subtle_fill_transparent, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, theme.subtle_fill_disabled).get(state);
        let foreground = StateColors::new(theme.accent_text, theme.accent_text_secondary, theme.accent_text_tertiary, theme.accent_text_disabled).get(state);
        let background = ui.animate_color(response.id, 0, background, DURATION_FASTER);
        let mut painter = ui.painter();
        painter.shape(rect, RADIUS_CONTROL, background, None);
        painter.text_fit(rect.pad(HYPERLINK_PADDING), self.text, BODY, foreground, Align::Center);
        ui.focus_visual_if(response.focused, rect, RADIUS_CONTROL);
        response
    }
}

impl Ui {
    // The width of a button that fits its content.
    pub fn button_width(&mut self, button: &Button) -> f32 {
        button.width(self)
    }
}
