// Text and the small things that go with it, like XAML's TextBlock in the
// type ramp's styles: labels, captions, paragraphs, label and value fields,
// lines of several styles; keycaps for shortcuts; WinUI's InfoBadge; the
// status icon of InfoBar; color swatches; and dividers.
//
// The painter helpers on Ui draw them anywhere; the controls under them are
// their retained forms, whose colors are the theme's roles, so they follow
// a theme switch.

use crate::core::control::{AutoTarget, Control, NodeContext, UiEventKind};
use crate::core::ui::{Response, Sense, Ui};
use crate::design::controls::{INFO_BADGE_DOT, INFO_BADGE_FONT_SIZE, INFO_BADGE_HEIGHT, INFO_BADGE_TEXT_MARGIN};
use crate::design::icons::Icon;
use crate::design::theme::Theme;
use crate::design::tokens::{BODY, BODY_STRONG, CAPTION, RADIUS_CONTROL, RADIUS_SMALL, SPACE_XS};
use crate::drawing::geometry::{Color, Point, Rect, Size};
use crate::drawing::painter::{Align, Border, Span};
use crate::drawing::text::TextStyle;
use crate::widgets::info_bar::Severity;

// A keycap's height, and the gap between the keycaps of a shortcut.
pub const KEYCAP_HEIGHT: f32 = 20.0;
const KEYCAP_PADDING: f32 = 6.0;
const KEYCAP_GAP: f32 = 4.0;

impl Ui {
    // A line of text in the style and color, centered in the rectangle's
    // height and cut short with an ellipsis if it's too wide. Returns where
    // it went.
    pub fn text(&mut self, rect: Rect, text: &str, style: TextStyle, color: Color) -> Rect {
        self.painter().text_fit(rect, text, style, color, Align::Start).0
    }

    pub fn text_aligned(&mut self, rect: Rect, text: &str, style: TextStyle, color: Color, align: Align) -> Rect {
        self.painter().text_fit(rect, text, style, color, align).0
    }

    // Body text in the primary color.
    pub fn label(&mut self, rect: Rect, text: &str) -> Rect {
        let color = self.theme().text_primary;
        self.text(rect, text, BODY, color)
    }

    // Body Strong text in the primary color.
    pub fn heading(&mut self, rect: Rect, text: &str) -> Rect {
        let color = self.theme().text_primary;
        self.text(rect, text, BODY_STRONG, color)
    }

    // Caption text in the secondary color.
    pub fn caption(&mut self, rect: Rect, text: &str) -> Rect {
        let color = self.theme().text_secondary;
        self.text(rect, text, CAPTION, color)
    }

    // Text wrapped at the rectangle's width, from its top. Returns its
    // height.
    pub fn paragraph(&mut self, rect: Rect, text: &str, style: TextStyle, color: Color) -> f32 {
        self.painter().text_wrapped(rect, text, style, color).height
    }

    // How tall the text is, wrapped at the width.
    pub fn paragraph_height(&mut self, width: f32, text: &str, style: TextStyle) -> f32 {
        self.fonts().measure_wrapped(text, style, width).height
    }

    // A label in the secondary color, and its value after it, at
    // `label_width` from the left.
    pub fn field(&mut self, rect: Rect, label: &str, label_width: f32, value: &str) {
        let (secondary, primary) = (self.theme().text_secondary, self.theme().text_primary);
        let mut painter = self.painter();
        painter.text_fit(rect.left_strip(label_width - SPACE_XS), label, BODY, secondary, Align::Start);
        let value_rect = Rect::new(rect.x + label_width, rect.y, (rect.width - label_width).max(0.0), rect.height);
        painter.text_fit(value_rect, value, BODY, primary, Align::Start);
    }

    // Runs of text in their own styles and colors, one after another on a
    // line, from the point. Returns the width.
    pub fn spans(&mut self, origin: Point, spans: &[Span]) -> f32 {
        self.painter().spans(origin, spans)
    }

    // An icon, centered in the rectangle.
    pub fn icon(&mut self, rect: Rect, icon: Icon, size: f32, color: Color) {
        self.painter().icon(rect.center(), icon, size, color);
    }

    // InfoBar's status icon: a circle in the status's color with its mark,
    // centered on the point.
    pub fn status_icon(&mut self, center: Point, severity: Severity, size: f32) {
        let mut painter = self.painter();
        let theme = painter.theme();
        let mark = match severity {
            Severity::Informational => Icon::StatusInfo,
            Severity::Success => Icon::StatusSuccess,
            Severity::Warning => Icon::StatusWarning,
            Severity::Error => Icon::StatusError,
        };
        painter.icon(center, Icon::StatusCircle, size, theme.status(severity));
        painter.icon(center, mark, size, theme.text_inverse);
    }

    // A 1-pixel divider along the rectangle's middle, across it if it's
    // wider than it's tall, or down it.
    pub fn divider(&mut self, rect: Rect) {
        let color = self.theme().divider;
        let line = if rect.width >= rect.height {
            Rect::new(rect.x, (rect.center().y - 0.5).round(), rect.width, 1.0)
        } else {
            Rect::new((rect.center().x - 0.5).round(), rect.y, 1.0, rect.height)
        };
        self.painter().fill(line, color);
    }

    // A color in a rounded square, only to show, like a paint's in a row.
    pub fn swatch(&mut self, rect: Rect, color: Color) {
        let mut painter = self.painter();
        let theme = painter.theme();
        painter.shape(rect, RADIUS_CONTROL, color, Some(Border::new(theme.control_stroke_secondary, 1.0)));
    }

    // A key, like "Ctrl" or "S", on a keycap whose top left is at the point.
    // Returns its width.
    pub fn keycap(&mut self, origin: Point, key: &str) -> f32 {
        let width = self.keycap_width(key);
        let rect = Rect::new(origin.x, origin.y, width, KEYCAP_HEIGHT);
        let mut painter = self.painter();
        let theme = painter.theme();
        painter.shape(rect, RADIUS_CONTROL, theme.control_fill, Some(Border::new(theme.control_stroke_secondary, 1.0)));
        painter.text_in(rect, key, CAPTION, theme.text_secondary, Align::Center);
        width
    }

    pub fn keycap_width(&mut self, key: &str) -> f32 {
        (self.measure(key, CAPTION).width.ceil() + KEYCAP_PADDING * 2.0).max(KEYCAP_HEIGHT)
    }

    // A shortcut's keys, like "Ctrl+S", as keycaps. Returns the width.
    pub fn keys(&mut self, origin: Point, keys: &str) -> f32 {
        let mut x = origin.x;
        for (index, key) in keys_of(keys).iter().enumerate() {
            if index > 0 {
                x += KEYCAP_GAP;
            }
            x += self.keycap(Point::new(x, origin.y), key);
        }
        x - origin.x
    }

    pub fn keys_width(&mut self, keys: &str) -> f32 {
        let names = keys_of(keys);
        names.iter().map(|key| self.keycap_width(key)).sum::<f32>() + KEYCAP_GAP * names.len().saturating_sub(1) as f32
    }

    // WinUI's InfoBadge: a count or a short word on a pill, or a dot if the
    // text is empty, centered on the point. Returns its size.
    pub fn badge(&mut self, center: Point, text: &str) -> Size {
        let mut painter = self.painter();
        let theme = painter.theme();
        if text.is_empty() {
            painter.circle(center, INFO_BADGE_DOT / 2.0, theme.accent);
            return Size::square(INFO_BADGE_DOT);
        }
        let style = TextStyle::new(CAPTION.font, CAPTION.weight, INFO_BADGE_FONT_SIZE, INFO_BADGE_HEIGHT);
        let width = (painter.measure(text, style).width.ceil() + INFO_BADGE_TEXT_MARGIN.horizontal()).max(INFO_BADGE_HEIGHT);
        let rect = Rect::from_center(center, Size::new(width, INFO_BADGE_HEIGHT)).round();
        painter.rounded(rect, INFO_BADGE_HEIGHT / 2.0, theme.accent);
        let text_rect = Rect::new(rect.x, rect.y - INFO_BADGE_TEXT_MARGIN.bottom / 2.0, rect.width, rect.height);
        painter.text_in(text_rect, text, style, theme.text_on_accent, Align::Center);
        rect.size()
    }
}

// The keys of a shortcut's name, with "Ctrl+S" as ["Ctrl", "S"], and a plain
// key's name as itself.
// The keys of a combination like "Ctrl+S" or "RB+Left stick", each on its
// own keycap. "+" alone is the plus key's.
fn keys_of(keys: &str) -> Vec<String> {
    let parts: Vec<String> = keys.split('+').map(|key| key.trim().to_string()).collect();
    if parts.len() > 1 && parts.iter().all(|part| !part.is_empty()) { parts } else { vec![keys.to_string()] }
}

// A text's color: one of the theme's roles, so that it follows a theme
// switch, or a color of the program's own.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TextColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Disabled,
    Accent,
    Custom(Color),
}

impl TextColor {
    pub fn resolve(self, theme: &Theme) -> Color {
        match self {
            TextColor::Primary => theme.text_primary,
            TextColor::Secondary => theme.text_secondary,
            TextColor::Tertiary => theme.text_tertiary,
            TextColor::Disabled => theme.text_disabled,
            TextColor::Accent => theme.accent_text,
            TextColor::Custom(color) => color,
        }
    }
}

// A line or paragraph of text: XAML's TextBlock. One line is cut short with
// an ellipsis; a wrapped one wraps at the node's width.
#[derive(Clone, Debug)]
pub struct Text {
    text: String,
    style: TextStyle,
    color: TextColor,
    align: Align,
    wrap: bool,
}

impl Text {
    pub fn new(text: &str, style: TextStyle) -> Self {
        Self {
            text: text.to_string(),
            style,
            color: TextColor::Primary,
            align: Align::Start,
            wrap: false,
        }
    }

    // Body text in the primary color.
    pub fn label(text: &str) -> Self {
        Self::new(text, BODY)
    }

    // Body Strong text in the primary color.
    pub fn heading(text: &str) -> Self {
        Self::new(text, BODY_STRONG)
    }

    // Caption text in the secondary color.
    pub fn caption(text: &str) -> Self {
        Self::new(text, CAPTION).secondary()
    }

    // Body text in the secondary color, wrapped.
    pub fn paragraph(text: &str) -> Self {
        Self::new(text, BODY).wrapped()
    }

    pub fn secondary(mut self) -> Self {
        self.color = TextColor::Secondary;
        self
    }

    pub fn color(mut self, color: TextColor) -> Self {
        self.color = color;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn wrapped(mut self) -> Self {
        self.wrap = true;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn set_color(&mut self, color: TextColor) {
        self.color = color;
    }
}

impl Control for Text {
    fn kind(&self) -> &'static str {
        "TextBlock"
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::None
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let color = if ui.enabled() { self.color } else { TextColor::Disabled };
        let color = color.resolve(ui.theme());
        if self.wrap {
            ui.paragraph(ctx.rect, &self.text, self.style, color);
        } else {
            ui.text_aligned(ctx.rect, &self.text, self.style, color, self.align);
        }
    }
}

// A label in the secondary color, and its value after it, at `label_width`
// from the left.
#[derive(Clone, Debug)]
pub struct Field {
    label: String,
    value: String,
    label_width: f32,
}

impl Field {
    pub fn new(label: &str, label_width: f32, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            label_width,
        }
    }

    pub fn set_value(&mut self, value: &str) {
        self.value = value.to_string();
    }
}

impl Control for Field {
    fn kind(&self) -> &'static str {
        "Field"
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::None
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let (label, value, width) = (self.label.clone(), self.value.clone(), self.label_width);
        ui.field(ctx.rect, &label, width, &value);
    }
}

// An icon, centered in its rectangle, in a theme role's color.
#[derive(Clone, Debug)]
pub struct IconBlock {
    icon: Icon,
    size: f32,
    color: TextColor,
}

impl IconBlock {
    pub fn new(icon: Icon, size: f32) -> Self {
        Self {
            icon,
            size,
            color: TextColor::Primary,
        }
    }

    pub fn color(mut self, color: TextColor) -> Self {
        self.color = color;
        self
    }

    pub fn set_icon(&mut self, icon: Icon) {
        self.icon = icon;
    }
}

impl Control for IconBlock {
    fn kind(&self) -> &'static str {
        "IconBlock"
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::None
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let color = self.color.resolve(ui.theme());
        ui.icon(ctx.rect, self.icon, self.size, color);
    }
}

// A 1-pixel divider along the rectangle's middle.
#[derive(Clone, Copy, Debug, Default)]
pub struct Divider;

impl Control for Divider {
    fn kind(&self) -> &'static str {
        "Divider"
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::None
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        ui.divider(ctx.rect);
    }
}

// A color in a rounded square, which shows that it's chosen with an accent
// ring, like the swatches of a palette. It emits Clicked.
#[derive(Clone, Copy, Debug)]
pub struct Swatch {
    color: Color,
    selected: bool,
}

impl Swatch {
    pub fn new(color: Color, selected: bool) -> Self {
        Self { color, selected }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

impl Control for Swatch {
    fn kind(&self) -> &'static str {
        "Swatch"
    }

    fn sense(&self) -> Sense {
        Sense::BUTTON
    }

    fn label(&self) -> Option<String> {
        Some(self.color.to_hex())
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        if ctx.response.clicked {
            ctx.emit(ui, UiEventKind::Clicked);
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let (rect, response): (Rect, &Response) = (ctx.rect, &ctx.response);
        let hovered = response.hovered;
        let mut painter = ui.painter();
        let theme = painter.theme();
        let inner = rect.inset(if self.selected { 3.0 } else { 0.0 });
        painter.shape(inner, RADIUS_SMALL.max(RADIUS_CONTROL - 1.0), self.color, Some(Border::new(theme.control_stroke_secondary, 1.0)));
        if self.selected {
            painter.shape(rect, RADIUS_CONTROL + 1.0, Color::TRANSPARENT, Some(Border::new(theme.accent, 2.0)));
        } else if hovered {
            painter.shape(rect, RADIUS_CONTROL, Color::TRANSPARENT, Some(Border::new(theme.control_strong_stroke, 1.0)));
        }
        ui.focus_visual_if(response.focused, rect, RADIUS_CONTROL);
    }
}
