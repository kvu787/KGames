// WinUI's buttons: Button, in its standard, accent and subtle styles, with
// text, an icon, or both, and a chevron as a DropDownButton has, with a menu
// that opens below it; ToggleButton, which is the accent's colors while it's
// on; and HyperlinkButton. Their colors change at once between visual
// states, without WinUI's BrushTransition.

use crate::core::interaction::{ControlState, Cursor, Interaction, Part, Sense};
use crate::core::interface::Ui;
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{BUTTON_ICON_GAP, BUTTON_ICON_ONLY_WIDTH, BUTTON_PADDING, DROP_DOWN_CHEVRON_GAP, DROP_DOWN_CHEVRON_SIZE, HYPERLINK_PADDING};
use crate::design::icons::Icon;
use crate::design::theme::Theme;
use crate::design::tokens::{BODY, ICON, RADIUS_CONTROL};
use crate::drawing::geometry::{Color, Point, Rect};
use crate::drawing::painter::{Align, Border};
use crate::widgets::menu::{self, MenuItem};
use crate::widgets::widget::{Elevation, Reaction, StateColors, react};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonStyle {
    #[default]
    Standard,
    Accent,
    // Transparent until the pointer is over it, like a toolbar's.
    Subtle,
}

pub struct Button<M> {
    text: String,
    icon: Option<Icon>,
    style: ButtonStyle,
    // A chevron after the text, as a DropDownButton has.
    chevron: bool,
    // A ToggleButton's state: on, so the accent's colors.
    checked: bool,
    on_press: Option<M>,
    // A menu that a press opens below the button, named "<name>/Menu".
    menu: Option<Vec<MenuItem<M>>>,
}

impl<M: Clone + 'static> Button<M> {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            style: ButtonStyle::Standard,
            chevron: false,
            checked: false,
            on_press: None,
            menu: None,
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

    pub fn chevron(mut self) -> Self {
        self.chevron = true;
        self
    }

    // What a click sends.
    pub fn on_press(mut self, message: M) -> Self {
        self.on_press = Some(message);
        self
    }

    // A menu that opens below it, with a chevron after its text, as a
    // DropDownButton.
    pub fn menu(mut self, items: Vec<MenuItem<M>>) -> Self {
        self.menu = Some(items);
        self.chevron = true;
        self
    }

    // How wide the button is for its text, icon and chevron: ButtonPadding
    // around them.
    pub fn width(&self, ui: &mut Ui) -> f32 {
        let mut width = BUTTON_PADDING.horizontal();
        if self.icon.is_some() {
            width += if self.text.is_empty() { BUTTON_ICON_ONLY_WIDTH - BUTTON_PADDING.horizontal() } else { ICON + BUTTON_ICON_GAP };
        }
        if !self.text.is_empty() {
            width += ui.measure(&self.text, BODY).width.ceil();
        }
        if self.chevron {
            width += DROP_DOWN_CHEVRON_GAP + DROP_DOWN_CHEVRON_SIZE;
        }
        width
    }

    fn menu_name(name: &str) -> String {
        format!("{name}/Menu")
    }
}

// The brushes of a button in each state, per style.
struct ButtonColors {
    background: StateColors,
    foreground: StateColors,
    // An elevation border, and a flat border for the pressed and disabled
    // states.
    border: Option<(Elevation, StateColors)>,
}

fn colors(theme: &Theme, style: ButtonStyle, checked: bool) -> ButtonColors {
    if checked {
        return ButtonColors {
            background: StateColors::new(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled),
            foreground: StateColors::new(theme.text_on_accent, theme.text_on_accent, theme.text_on_accent_secondary, theme.text_on_accent_disabled),
            border: Some((Elevation::Accent, StateColors::new(theme.control_stroke_on_accent, theme.control_stroke_on_accent, theme.control_fill_transparent, theme.control_fill_transparent))),
        };
    }
    match style {
        ButtonStyle::Standard => ButtonColors {
            background: StateColors::new(theme.control_fill, theme.control_fill_secondary, theme.control_fill_tertiary, theme.control_fill_disabled),
            foreground: StateColors::new(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled),
            border: Some((Elevation::Control, StateColors::new(theme.control_stroke, theme.control_stroke, theme.control_stroke, theme.control_stroke))),
        },
        ButtonStyle::Accent => ButtonColors {
            background: StateColors::new(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled),
            foreground: StateColors::new(theme.text_on_accent, theme.text_on_accent, theme.text_on_accent_secondary, theme.text_on_accent_disabled),
            border: Some((Elevation::Accent, StateColors::new(theme.control_stroke_on_accent, theme.control_stroke_on_accent, theme.control_fill_transparent, theme.control_fill_transparent))),
        },
        ButtonStyle::Subtle => ButtonColors {
            background: StateColors::new(theme.subtle_fill_transparent, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, theme.subtle_fill_disabled),
            foreground: StateColors::new(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled),
            border: None,
        },
    }
}

// Draws a button's face in the rectangle: its background, border, icon,
// text and chevron.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_button(ctx: &mut Draw, rect: Rect, text: &str, icon: Option<Icon>, style: ButtonStyle, checked: bool, chevron: bool, state: ControlState) {
    let colors = colors(ctx.theme(), style, checked);
    let background = colors.background.get(state);
    let foreground = colors.foreground.get(state);
    let mut painter = ctx.painter();
    let theme = painter.theme();
    // At rest and with the pointer over, the border is the elevation
    // gradient; pressed and disabled, it's flat.
    let border = colors.border.map(|(elevation, flat)| match state {
        ControlState::Normal | ControlState::PointerOver => elevation.border(theme, rect),
        ControlState::Pressed | ControlState::Disabled => Border::new(flat.get(state), 1.0),
    });
    painter.shape(rect, RADIUS_CONTROL, background, border);
    let content = rect.pad(BUTTON_PADDING);
    let icon_width = icon.map_or(0.0, |_| if text.is_empty() { ICON } else { ICON + BUTTON_ICON_GAP });
    let text_width = if text.is_empty() { 0.0 } else { painter.measure(text, BODY).width.min(content.width - icon_width) };
    let chevron_width = if chevron { DROP_DOWN_CHEVRON_GAP + DROP_DOWN_CHEVRON_SIZE } else { 0.0 };
    let total = icon_width + text_width + chevron_width;
    let mut x = (content.x + (content.width - total) / 2.0).max(content.x).round();
    if let Some(icon) = icon {
        painter.icon(Point::new(x + ICON / 2.0, rect.center().y), icon, ICON, foreground);
        x += icon_width;
    }
    if !text.is_empty() {
        let text_rect = Rect::new(x, rect.y, text_width, rect.height);
        painter.text_fit(text_rect, text, BODY, foreground, Align::Start);
        x += text_width;
    }
    if chevron {
        painter.icon(Point::new(x + DROP_DOWN_CHEVRON_GAP + DROP_DOWN_CHEVRON_SIZE / 2.0, rect.center().y), Icon::ChevronDown, DROP_DOWN_CHEVRON_SIZE, foreground);
    }
}

impl<M: Clone + 'static> Widget<M> for Button<M> {
    fn kind(&self) -> &'static str {
        if self.menu.is_some() { "DropDownButton" } else { "Button" }
    }

    fn label(&self) -> Option<String> {
        (!self.text.is_empty()).then(|| self.text.clone())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::CLICK)
    }

    fn parts(&self, ctx: &mut Parts) {
        if let Some(items) = &self.menu {
            let name = Self::menu_name(ctx.name());
            let owner = ctx.id();
            menu::parts_menu(ctx, &name, owner, items);
        }
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let hit = interaction.whole();
        match &self.menu {
            None => {
                if hit.pressed && let Some(message) = &self.on_press {
                    ctx.send(message.clone());
                }
            }
            Some(items) => {
                let name = Self::menu_name(ctx.name());
                if hit.pressed {
                    if ctx.popup_open() {
                        ctx.close_popup();
                    } else {
                        let rect = ctx.rect();
                        menu::open_menu(ctx, ctx.id(), &name, Point::new(rect.x, rect.bottom() + 4.0));
                    }
                }
                if ctx.popup_open() {
                    menu::interact_menu(interaction, ctx, &name, ctx.id(), items);
                }
            }
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let state = ctx.state();
        let rect = ctx.rect();
        draw_button(ctx, rect, &self.text, self.icon, self.style, self.checked, self.chevron, state);
        if let Some(items) = &self.menu
            && ctx.popup_open()
        {
            let name = Self::menu_name(ctx.name());
            let owner = ctx.id();
            menu::draw_menu(ctx, &name, owner, items, 0);
        }
    }
}

// A button that's on or off, in the accent's colors while it's on.
pub struct ToggleButton<M> {
    button: Button<M>,
    on: bool,
    on_toggle: Option<Reaction<bool, M>>,
}

impl<M: Clone + 'static> ToggleButton<M> {
    pub fn new(on: bool, button: Button<M>) -> Self {
        Self { button, on, on_toggle: None }
    }

    // What switching it sends, with whether it's on then.
    pub fn on_toggle(mut self, reaction: impl Fn(bool) -> M + 'static) -> Self {
        self.on_toggle = Some(react(reaction));
        self
    }
}

impl<M: Clone + 'static> Widget<M> for ToggleButton<M> {
    fn kind(&self) -> &'static str {
        "ToggleButton"
    }

    fn label(&self) -> Option<String> {
        Some(if self.on { "On".to_string() } else { "Off".to_string() })
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::CLICK)
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        if interaction.whole().pressed
            && let Some(reaction) = &self.on_toggle
        {
            ctx.send(reaction(!self.on));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let state = ctx.state();
        let rect = ctx.rect();
        draw_button(ctx, rect, &self.button.text, self.button.icon, self.button.style, self.on, self.button.chevron, state);
    }
}

// A link that's a button: accent-colored text, which takes a subtle fill
// with the pointer over it.
pub struct HyperlinkButton<M> {
    text: String,
    on_press: Option<M>,
}

impl<M: Clone + 'static> HyperlinkButton<M> {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), on_press: None }
    }

    pub fn on_press(mut self, message: M) -> Self {
        self.on_press = Some(message);
        self
    }
}

impl<M: Clone + 'static> Widget<M> for HyperlinkButton<M> {
    fn kind(&self) -> &'static str {
        "HyperlinkButton"
    }

    fn label(&self) -> Option<String> {
        Some(self.text.clone())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::CLICK)
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let hit = interaction.whole();
        if hit.hovered && interaction.enabled {
            ctx.set_cursor(Cursor::Hand);
        }
        if hit.pressed && let Some(message) = &self.on_press {
            ctx.send(message.clone());
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let state = ctx.state();
        let rect = ctx.rect();
        let theme = ctx.theme();
        let background = StateColors::new(theme.subtle_fill_transparent, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, theme.subtle_fill_disabled).get(state);
        let foreground = StateColors::new(theme.accent_text, theme.accent_text_secondary, theme.accent_text_tertiary, theme.accent_text_disabled).get(state);
        let mut painter = ctx.painter();
        painter.shape(rect, RADIUS_CONTROL, background, None);
        painter.text_fit(rect.pad(HYPERLINK_PADDING), &self.text, BODY, foreground, Align::Center);
    }
}

// A button inside a text field, like its delete button: transparent until
// the pointer is over it, with its glyph in the secondary color, as
// TextControlButton's brushes are.
pub(crate) fn draw_text_control_button(ctx: &mut Draw, part: &Part, rect: Rect, icon: Icon, glyph: f32) {
    let state = ctx.look().part_state(part);
    let theme = ctx.theme();
    let background = StateColors::new(Color::TRANSPARENT, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, Color::TRANSPARENT).get(state);
    let foreground = StateColors::new(theme.text_secondary, theme.text_secondary, theme.text_tertiary, theme.text_disabled).get(state);
    let mut painter = ctx.painter();
    painter.rounded(rect, RADIUS_CONTROL, background);
    painter.icon(rect.center(), icon, glyph, foreground);
}
