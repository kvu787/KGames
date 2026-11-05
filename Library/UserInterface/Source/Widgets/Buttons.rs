// Buttons: standard, accent and subtle, with text, an icon, or both; toggle
// buttons; and hyperlink buttons. As WinUI's Button, AccentButtonStyle,
// SubtleButtonStyle, ToggleButton and HyperlinkButton.

use crate::geometry::{Color, Point, Rect};
use crate::icons::Icon;
use crate::painter::{Align, Corners, Stroke};
use crate::tokens::{BODY, DURATION_FASTER, ICON, RADIUS_CONTROL, SPACE_S};
use crate::ui::{Response, Sense, Ui};
use crate::widgets::common::{Visual, control_focus_ring};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonStyle {
    // On a control fill, for most commands.
    #[default]
    Standard,
    // In the accent color, for the one command that matters most.
    Accent,
    // Without a fill until the pointer is on it, for toolbars and the like.
    Subtle,
}

#[derive(Clone, Copy, Debug)]
pub struct Button<'a> {
    pub label: &'a str,
    pub icon: Option<Icon>,
    pub style: ButtonStyle,
}

// Padding on each side of a button's content.
pub const BUTTON_PADDING: f32 = 11.0;

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            style: ButtonStyle::Standard,
        }
    }

    // An icon without text, in a square button.
    pub fn icon_only(icon: Icon) -> Self {
        Self {
            label: "",
            icon: Some(icon),
            style: ButtonStyle::Standard,
        }
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
}

impl Ui {
    // How wide a button must be for its content.
    pub fn button_width(&mut self, button: &Button) -> f32 {
        let text = if button.label.is_empty() { 0.0 } else { self.measure(button.label, BODY).width.ceil() };
        let icon = if button.icon.is_some() { ICON } else { 0.0 };
        let gap = if text > 0.0 && icon > 0.0 { SPACE_S } else { 0.0 };
        if text == 0.0 {
            return crate::tokens::CONTROL_HEIGHT;
        }
        BUTTON_PADDING * 2.0 + text + icon + gap
    }

    pub fn button(&mut self, name: &str, rect: Rect, button: Button) -> Response {
        let response = self.interact(name, rect, Sense::BUTTON, "Button");
        if !button.label.is_empty() {
            self.set_label(name, button.label);
        }
        let visual = Visual::of(self, &response);
        draw_button(self, &response, rect, &button, button.style, visual);
        response
    }

    // A button that stays on or off, in the accent color while it's on.
    pub fn toggle_button(&mut self, name: &str, rect: Rect, on: &mut bool, button: Button) -> Response {
        let mut response = self.interact(name, rect, Sense::BUTTON, "ToggleButton");
        if !button.label.is_empty() {
            self.set_label(name, button.label);
        }
        if response.clicked {
            *on = !*on;
            response.changed = true;
        }
        let visual = Visual::of(self, &response);
        let style = if *on { ButtonStyle::Accent } else { button.style };
        draw_button(self, &response, rect, &button, style, visual);
        response
    }

    // Text in the accent color that acts like a button, like a link.
    pub fn hyperlink(&mut self, name: &str, rect: Rect, text: &str) -> Response {
        let response = self.interact(name, rect, Sense::BUTTON, "Hyperlink");
        self.set_label(name, text);
        let visual = Visual::of(self, &response);
        let theme = self.theme();
        let fill = visual.pick(Color::TRANSPARENT, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, Color::TRANSPARENT);
        let color = visual.pick(
            theme.accent_text,
            theme.accent_text.fade(0.9),
            theme.accent_text.fade(0.8),
            theme.text_disabled,
        );
        let fill = self.animate_color(response.id, 0, fill, DURATION_FASTER);
        let mut painter = self.painter();
        painter.rounded(rect, RADIUS_CONTROL, fill);
        painter.text_in(rect, text, BODY, color, Align::Center);
        control_focus_ring(self, &response);
        response
    }
}

fn draw_button(ui: &mut Ui, response: &Response, rect: Rect, button: &Button, style: ButtonStyle, visual: Visual) {
    let theme = ui.theme();
    let (fill, stroke, text) = match style {
        ButtonStyle::Standard => (
            visual.pick(theme.control_fill, theme.control_fill_secondary, theme.control_fill_tertiary, theme.control_fill_disabled),
            match visual {
                Visual::Rest | Visual::Hover => Some(Stroke::elevated(theme.control_stroke, theme.control_stroke_secondary)),
                _ => Some(Stroke::new(theme.control_stroke, 1.0)),
            },
            visual.pick(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled),
        ),
        ButtonStyle::Accent => (
            visual.pick(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled),
            match visual {
                Visual::Rest | Visual::Hover => Some(Stroke::elevated(theme.control_stroke_on_accent, theme.control_stroke_on_accent_secondary)),
                _ => None,
            },
            visual.pick(theme.text_on_accent, theme.text_on_accent, theme.text_on_accent_secondary, theme.text_on_accent_disabled),
        ),
        ButtonStyle::Subtle => (
            visual.pick(Color::TRANSPARENT, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, Color::TRANSPARENT),
            None,
            visual.pick(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled),
        ),
    };
    let fill = ui.animate_color(response.id, 0, fill, DURATION_FASTER);
    let mut painter = ui.painter();
    painter.shape(rect, Corners::all(RADIUS_CONTROL), fill, stroke);
    draw_content(&mut painter, rect, button.label, button.icon, text);
    control_focus_ring(ui, response);
}

// A button's icon and label, centered together.
pub(crate) fn draw_content(painter: &mut crate::painter::Painter, rect: Rect, label: &str, icon: Option<Icon>, color: Color) {
    let text_width = if label.is_empty() { 0.0 } else { painter.measure(label, BODY).width };
    let icon_width = if icon.is_some() { ICON } else { 0.0 };
    let gap = if text_width > 0.0 && icon_width > 0.0 { SPACE_S } else { 0.0 };
    let total = text_width + icon_width + gap;
    let left = (rect.x + (rect.width - total) / 2.0).round();
    if let Some(icon) = icon {
        painter.icon(Point::new(left + ICON / 2.0, rect.center().y), icon, ICON, color);
    }
    if !label.is_empty() {
        let x = left + icon_width + gap;
        let label_rect = Rect::new(x, rect.y, rect.right() - x, rect.height);
        painter.text_in(label_rect, label, BODY, color, Align::Start);
    }
}
