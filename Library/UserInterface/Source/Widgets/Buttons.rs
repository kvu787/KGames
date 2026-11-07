// WinUI's buttons: Button, in its standard, accent and subtle styles, with
// text, an icon, or both, and a chevron as a DropDownButton has; ToggleButton,
// which is the accent's colors while it's on; and HyperlinkButton.
//
// Their backgrounds cross-fade between visual states over
// ControlFasterAnimationDuration, as WinUI's BrushTransition does; their
// text and borders change at once.

use crate::core::control::{Control, NodeContext, UiEventKind};
use crate::core::ui::{ControlState, Cursor, Response, Sense, Ui};
use crate::design::controls::{BUTTON_ICON_GAP, BUTTON_ICON_ONLY_WIDTH, BUTTON_PADDING, DROP_DOWN_CHEVRON_GAP, DROP_DOWN_CHEVRON_SIZE, HYPERLINK_PADDING};
use crate::design::icons::Icon;
use crate::design::theme::Theme;
use crate::design::tokens::{BODY, DURATION_FASTER, ICON, ICON_SMALL, RADIUS_CONTROL};
use crate::drawing::geometry::{Color, Point, Rect};
use crate::drawing::painter::{Align, Border};
use crate::widgets::menu::MenuItem;
use crate::widgets::widget::{Elevation, StateColors};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonStyle {
    #[default]
    Standard,
    // AccentButtonStyle, for the one command that matters most.
    Accent,
    // Invisible until the pointer is over it, like a toolbar's buttons.
    Subtle,
}

#[derive(Clone, Debug, Default)]
pub struct Button {
    text: String,
    icon: Option<Icon>,
    style: ButtonStyle,
    // Drawn in the accent's colors, like a ToggleButton that's on.
    checked: bool,
    // A chevron after the text, like a DropDownButton's.
    chevron: bool,
    // A menu that opens below it when it's clicked, like a DropDownButton's,
    // under its own name, whose Chose events carry the item chosen.
    menu: Option<(String, Vec<MenuItem>)>,
}

impl Button {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            ..Self::default()
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

    pub fn set_style(&mut self, style: ButtonStyle) {
        self.style = style;
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    pub fn chevron(mut self) -> Self {
        self.chevron = true;
        self
    }

    // A menu named `name` that the button opens, like a DropDownButton's,
    // with a chevron to say so.
    pub fn flyout(mut self, name: &str, items: Vec<MenuItem>) -> Self {
        self.menu = Some((name.to_string(), items));
        self.chevron = true;
        self
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn set_icon(&mut self, icon: Icon) {
        self.icon = Some(icon);
    }

    pub fn text(&self) -> &str {
        &self.text
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
        width += ui.measure(&self.text, BODY).width.ceil();
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

impl Control for Button {
    fn kind(&self) -> &'static str {
        "Button"
    }

    fn sense(&self) -> Sense {
        Sense::BUTTON
    }

    fn label(&self) -> Option<String> {
        (!self.text.is_empty()).then(|| self.text.clone())
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        if ctx.response.clicked {
            ctx.emit(ui, UiEventKind::Clicked);
        }
        if ctx.response.double_clicked {
            ctx.emit(ui, UiEventKind::DoubleClicked);
        }
        if ctx.response.secondary_clicked {
            ctx.emit(ui, UiEventKind::SecondaryClicked);
        }
        if let Some((name, items)) = &self.menu {
            let name = name.clone();
            if let Some(chosen) = ui.flyout_menu(&name, &ctx.response, &items.clone()) {
                ui.emit(&name, UiEventKind::Chose(chosen));
            }
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let button = ButtonFace {
            text: &self.text,
            icon: self.icon,
            style: self.style,
            checked: self.checked,
            chevron: self.chevron,
        };
        draw_button(ui, &button, &ctx.response, ctx.rect);
    }
}

// A button's look, for drawing one anywhere, like a dialog's buttons.
pub(crate) struct ButtonFace<'a> {
    pub text: &'a str,
    pub icon: Option<Icon>,
    pub style: ButtonStyle,
    pub checked: bool,
    pub chevron: bool,
}

// An immediate button inside another control, like a dialog's: registers it,
// draws it, and says what the input did.
pub(crate) fn inline_button(ui: &mut Ui, name: &str, rect: Rect, text: &str, style: ButtonStyle) -> Response {
    let response = ui.interact(name, rect, Sense::BUTTON, "Button");
    if !text.is_empty() {
        ui.set_label(name, text);
    }
    let face = ButtonFace {
        text,
        icon: None,
        style,
        checked: false,
        chevron: false,
    };
    draw_button(ui, &face, &response, rect);
    response
}

pub(crate) fn draw_button(ui: &mut Ui, button: &ButtonFace, response: &Response, rect: Rect) {
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
#[derive(Clone, Debug)]
pub struct ToggleButton {
    on: bool,
    button: Button,
}

impl ToggleButton {
    pub fn new(on: bool, button: Button) -> Self {
        Self { on, button }
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    pub fn set_on(&mut self, on: bool) {
        self.on = on;
    }
}

impl Control for ToggleButton {
    fn kind(&self) -> &'static str {
        "ToggleButton"
    }

    fn sense(&self) -> Sense {
        Sense::BUTTON
    }

    fn label(&self) -> Option<String> {
        Some(if self.on { "On".to_string() } else { "Off".to_string() })
    }

    fn adopt(&mut self, previous: Box<dyn Control>) {
        if let Some(previous) = crate::core::control::adopted::<ToggleButton>(previous) {
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
        let face = ButtonFace {
            text: &self.button.text,
            icon: self.button.icon,
            style: self.button.style,
            checked: self.on,
            chevron: self.button.chevron,
        };
        draw_button(ui, &face, &ctx.response, ctx.rect);
    }
}

// A link that's a button: accent-colored text, which takes a subtle fill
// with the pointer over it.
#[derive(Clone, Debug)]
pub struct HyperlinkButton {
    text: String,
}

impl HyperlinkButton {
    pub fn new(text: &str) -> Self {
        Self { text: text.to_string() }
    }

    pub fn width(&self, ui: &mut Ui) -> f32 {
        HYPERLINK_PADDING.horizontal() + ui.measure(&self.text, BODY).width.ceil()
    }
}

impl Control for HyperlinkButton {
    fn kind(&self) -> &'static str {
        "HyperlinkButton"
    }

    fn sense(&self) -> Sense {
        Sense::BUTTON
    }

    fn label(&self) -> Option<String> {
        Some(self.text.clone())
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        if ctx.response.clicked {
            ctx.emit(ui, UiEventKind::Clicked);
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let (rect, response) = (ctx.rect, &ctx.response);
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
        painter.text_fit(rect.pad(HYPERLINK_PADDING), &self.text, BODY, foreground, Align::Center);
        ui.focus_visual_if(response.focused, rect, RADIUS_CONTROL);
    }
}

impl Ui {
    // The width of a button that fits its content.
    pub fn button_width(&mut self, button: &Button) -> f32 {
        button.width(self)
    }
}
