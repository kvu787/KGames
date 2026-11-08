// Text and the small things that go with it, like XAML's TextBlock in the
// type ramp's styles: labels, captions, paragraphs, label and value fields,
// lines of several styles; keycaps for shortcuts; WinUI's InfoBadge; the
// status icon of InfoBar; swatches; dividers; and a canvas for a program's
// own drawing. None of them is a target: the pointer goes through them.

use crate::core::tree::{Draw, Widget};
use crate::design::controls::{INFO_BADGE_DOT, INFO_BADGE_FONT_SIZE, INFO_BADGE_HEIGHT, INFO_BADGE_TEXT_MARGIN};
use crate::design::icons::Icon;
use crate::design::theme::Theme;
use crate::design::tokens::{BODY, BODY_STRONG, CAPTION, RADIUS_CONTROL, SPACE_XS};
use crate::drawing::geometry::{Color, Point, Rect, Size};
use crate::drawing::painter::{Align, Border, Painter};
use crate::drawing::text::{Fonts, TextStyle};
use crate::widgets::info_bar::Severity;

// A keycap's height, and the gap between the keycaps of a shortcut.
pub const KEYCAP_HEIGHT: f32 = 20.0;
const KEYCAP_PADDING: f32 = 6.0;
const KEYCAP_GAP: f32 = 4.0;

// A text's color: one of the theme's roles for text, or a color of its own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextColor {
    Primary,
    Secondary,
    Tertiary,
    Disabled,
    Accent,
    Custom(Color),
}

impl TextColor {
    pub fn resolve(self, theme: &Theme, enabled: bool) -> Color {
        match self {
            TextColor::Primary if !enabled => theme.text_disabled,
            TextColor::Secondary if !enabled => theme.text_disabled,
            TextColor::Primary => theme.text_primary,
            TextColor::Secondary => theme.text_secondary,
            TextColor::Tertiary => theme.text_tertiary,
            TextColor::Disabled => theme.text_disabled,
            TextColor::Accent => theme.accent_text,
            TextColor::Custom(color) => color,
        }
    }
}

impl From<Color> for TextColor {
    fn from(color: Color) -> Self {
        TextColor::Custom(color)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Flow {
    // One line, centered in the rectangle's height, cut short with an
    // ellipsis if it's too wide.
    Fit,
    // Wrapped at the rectangle's width, from its top.
    Wrap,
}

// A line of text, or a paragraph.
pub struct Text {
    text: String,
    style: TextStyle,
    color: TextColor,
    align: Align,
    flow: Flow,
}

impl Text {
    // Body text in the primary color, on one line.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: BODY,
            color: TextColor::Primary,
            align: Align::Start,
            flow: Flow::Fit,
        }
    }

    pub fn label(text: impl Into<String>) -> Self {
        Self::new(text)
    }

    // Body Strong in the primary color.
    pub fn heading(text: impl Into<String>) -> Self {
        Self::new(text).style(BODY_STRONG)
    }

    // Caption in the secondary color.
    pub fn caption(text: impl Into<String>) -> Self {
        Self::new(text).style(CAPTION).color(TextColor::Secondary)
    }

    // Body text wrapped at the width, in the primary color.
    pub fn paragraph(text: impl Into<String>) -> Self {
        Self::new(text).wrap()
    }

    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    pub fn color(mut self, color: impl Into<TextColor>) -> Self {
        self.color = color.into();
        self
    }

    pub fn secondary(self) -> Self {
        self.color(TextColor::Secondary)
    }

    pub fn tertiary(self) -> Self {
        self.color(TextColor::Tertiary)
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn center(self) -> Self {
        self.align(Align::Center)
    }

    pub fn end(self) -> Self {
        self.align(Align::End)
    }

    pub fn wrap(mut self) -> Self {
        self.flow = Flow::Wrap;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl<M: 'static> Widget<M> for Text {
    fn kind(&self) -> &'static str {
        "TextBlock"
    }

    fn label(&self) -> Option<String> {
        Some(self.text.clone())
    }

    fn draw(&self, ctx: &mut Draw) {
        let color = self.color.resolve(ctx.theme(), ctx.enabled());
        let rect = ctx.rect();
        let mut painter = ctx.painter();
        match self.flow {
            Flow::Fit => {
                painter.text_fit(rect, &self.text, self.style, color, self.align);
            }
            Flow::Wrap => {
                painter.text_wrapped(rect, &self.text, self.style, color);
            }
        }
    }
}

// A label in the secondary color, and its value after it, at `label_width`
// from the left.
pub struct Field {
    label: String,
    label_width: f32,
    value: String,
}

impl Field {
    pub fn new(label: impl Into<String>, label_width: f32, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            label_width,
            value: value.into(),
        }
    }
}

impl<M: 'static> Widget<M> for Field {
    fn kind(&self) -> &'static str {
        "Field"
    }

    fn label(&self) -> Option<String> {
        Some(self.value.clone())
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let enabled = ctx.enabled();
        let (secondary, primary) = (TextColor::Secondary.resolve(ctx.theme(), enabled), TextColor::Primary.resolve(ctx.theme(), enabled));
        let mut painter = ctx.painter();
        painter.text_fit(rect.left_strip(self.label_width - SPACE_XS), &self.label, BODY, secondary, Align::Start);
        let value_rect = Rect::new(rect.x + self.label_width, rect.y, (rect.width - self.label_width).max(0.0), rect.height);
        painter.text_fit(value_rect, &self.value, BODY, primary, Align::Start);
    }
}

// A run of text in its own style and color, for a line of several.
#[derive(Clone, Debug)]
pub struct Run {
    pub text: String,
    pub style: TextStyle,
    pub color: TextColor,
}

impl Run {
    pub fn new(text: impl Into<String>, style: TextStyle, color: impl Into<TextColor>) -> Self {
        Self {
            text: text.into(),
            style,
            color: color.into(),
        }
    }
}

// Runs of text in their own styles and colors, one after another on a
// line, from the rectangle's top left.
pub struct Spans {
    runs: Vec<Run>,
}

impl Spans {
    pub fn new(runs: Vec<Run>) -> Self {
        Self { runs }
    }

    // The width of the runs, for laying them out.
    pub fn width(&self, fonts: &mut Fonts) -> f32 {
        self.runs.iter().map(|run| fonts.measure(&run.text, run.style).width).sum()
    }
}

impl<M: 'static> Widget<M> for Spans {
    fn kind(&self) -> &'static str {
        "TextBlock"
    }

    fn draw(&self, ctx: &mut Draw) {
        let origin = ctx.rect().origin();
        let enabled = ctx.enabled();
        let theme = ctx.theme().clone();
        let spans: Vec<crate::drawing::painter::Span> = self.runs.iter().map(|run| crate::drawing::painter::Span::new(&run.text, run.style, run.color.resolve(&theme, enabled))).collect();
        ctx.painter().spans(origin, &spans);
    }
}

// An icon, centered in its rectangle.
pub struct IconView {
    icon: Icon,
    size: f32,
    color: TextColor,
}

impl IconView {
    pub fn new(icon: Icon, size: f32) -> Self {
        Self {
            icon,
            size,
            color: TextColor::Primary,
        }
    }

    pub fn color(mut self, color: impl Into<TextColor>) -> Self {
        self.color = color.into();
        self
    }
}

impl<M: 'static> Widget<M> for IconView {
    fn kind(&self) -> &'static str {
        "Icon"
    }

    fn draw(&self, ctx: &mut Draw) {
        let color = self.color.resolve(ctx.theme(), ctx.enabled());
        let center = ctx.rect().center();
        ctx.painter().icon(center, self.icon, self.size, color);
    }
}

// InfoBar's status icon: a circle in the status's color with its mark,
// centered in its rectangle.
pub struct StatusIcon {
    severity: Severity,
    size: f32,
}

impl StatusIcon {
    pub fn new(severity: Severity, size: f32) -> Self {
        Self { severity, size }
    }
}

pub(crate) fn status_icon(painter: &mut Painter, center: Point, severity: Severity, size: f32) {
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

impl<M: 'static> Widget<M> for StatusIcon {
    fn kind(&self) -> &'static str {
        "StatusIcon"
    }

    fn draw(&self, ctx: &mut Draw) {
        let center = ctx.rect().center();
        status_icon(&mut ctx.painter(), center, self.severity, self.size);
    }
}

// A 1-pixel divider along the rectangle's middle, across it if it's wider
// than it's tall, or down it.
pub struct Divider;

impl<M: 'static> Widget<M> for Divider {
    fn kind(&self) -> &'static str {
        "Divider"
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let color = ctx.theme().divider;
        let line = if rect.width >= rect.height {
            Rect::new(rect.x, (rect.center().y - 0.5).round(), rect.width, 1.0)
        } else {
            Rect::new((rect.center().x - 0.5).round(), rect.y, 1.0, rect.height)
        };
        ctx.painter().fill(line, color);
    }
}

// A color in a rounded square, only to show, like a paint's in a row.
pub struct Swatch {
    color: Color,
}

impl Swatch {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl<M: 'static> Widget<M> for Swatch {
    fn kind(&self) -> &'static str {
        "Swatch"
    }

    fn label(&self) -> Option<String> {
        Some(self.color.to_hex())
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let mut painter = ctx.painter();
        let theme = painter.theme();
        painter.shape(rect, RADIUS_CONTROL, self.color, Some(Border::new(theme.control_stroke_secondary, 1.0)));
    }
}

// A shortcut's keys, like "Ctrl+S", as keycaps, from the rectangle's top
// left.
pub struct Keycaps {
    keys: String,
}

impl Keycaps {
    pub fn new(keys: impl Into<String>) -> Self {
        Self { keys: keys.into() }
    }

    // How wide the keycaps are.
    // How wide one key's keycap is, like "Ctrl".
    pub fn key_width(fonts: &mut Fonts, key: &str) -> f32 {
        keycap_width(fonts, key)
    }

    pub fn width(fonts: &mut Fonts, keys: &str) -> f32 {
        keys_width(fonts, keys)
    }
}

impl<M: 'static> Widget<M> for Keycaps {
    fn kind(&self) -> &'static str {
        "Keycaps"
    }

    fn label(&self) -> Option<String> {
        Some(self.keys.clone())
    }

    fn draw(&self, ctx: &mut Draw) {
        let origin = ctx.rect().origin();
        draw_keys(&mut ctx.painter(), origin, &self.keys);
    }
}

// A key, like "Ctrl" or "S", on a keycap whose top left is at the point.
// Returns its width.
pub(crate) fn draw_keycap(painter: &mut Painter, origin: Point, key: &str) -> f32 {
    let width = keycap_width(painter.fonts(), key);
    let rect = Rect::new(origin.x, origin.y, width, KEYCAP_HEIGHT);
    let theme = painter.theme();
    painter.shape(rect, RADIUS_CONTROL, theme.control_fill, Some(Border::new(theme.control_stroke_secondary, 1.0)));
    painter.text_in(rect, key, CAPTION, theme.text_secondary, Align::Center);
    width
}

pub(crate) fn keycap_width(fonts: &mut Fonts, key: &str) -> f32 {
    (fonts.measure(key, CAPTION).width.ceil() + KEYCAP_PADDING * 2.0).max(KEYCAP_HEIGHT)
}

// A shortcut's keys as keycaps from the point. Returns the width.
pub(crate) fn draw_keys(painter: &mut Painter, origin: Point, keys: &str) -> f32 {
    let mut x = origin.x;
    for (index, key) in keys_of(keys).iter().enumerate() {
        if index > 0 {
            x += KEYCAP_GAP;
        }
        x += draw_keycap(painter, Point::new(x, origin.y), key);
    }
    x - origin.x
}

pub(crate) fn keys_width(fonts: &mut Fonts, keys: &str) -> f32 {
    let names = keys_of(keys);
    names.iter().map(|key| keycap_width(fonts, key)).sum::<f32>() + KEYCAP_GAP * names.len().saturating_sub(1) as f32
}

// WinUI's InfoBadge: a count or a short word on a pill, or a dot if the
// text is empty, centered in its rectangle.
pub struct Badge {
    text: String,
}

impl Badge {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn dot() -> Self {
        Self::new("")
    }

    // Its size, for laying it out.
    pub fn size(fonts: &mut Fonts, text: &str) -> Size {
        if text.is_empty() {
            return Size::square(INFO_BADGE_DOT);
        }
        let style = TextStyle::new(CAPTION.font, CAPTION.weight, INFO_BADGE_FONT_SIZE, INFO_BADGE_HEIGHT);
        let width = (fonts.measure(text, style).width.ceil() + INFO_BADGE_TEXT_MARGIN.horizontal()).max(INFO_BADGE_HEIGHT);
        Size::new(width, INFO_BADGE_HEIGHT)
    }
}

impl<M: 'static> Widget<M> for Badge {
    fn kind(&self) -> &'static str {
        "InfoBadge"
    }

    fn label(&self) -> Option<String> {
        Some(self.text.clone())
    }

    fn draw(&self, ctx: &mut Draw) {
        let center = ctx.rect().center();
        let mut painter = ctx.painter();
        let theme = painter.theme();
        if self.text.is_empty() {
            painter.circle(center, INFO_BADGE_DOT / 2.0, theme.accent);
            return;
        }
        let style = TextStyle::new(CAPTION.font, CAPTION.weight, INFO_BADGE_FONT_SIZE, INFO_BADGE_HEIGHT);
        let width = (painter.measure(&self.text, style).width.ceil() + INFO_BADGE_TEXT_MARGIN.horizontal()).max(INFO_BADGE_HEIGHT);
        let rect = Rect::from_center(center, Size::new(width, INFO_BADGE_HEIGHT)).round();
        painter.rounded(rect, INFO_BADGE_HEIGHT / 2.0, theme.accent);
        let text_rect = Rect::new(rect.x, rect.y - INFO_BADGE_TEXT_MARGIN.bottom / 2.0, rect.width, rect.height);
        painter.text_in(text_rect, &self.text, style, theme.text_on_accent, Align::Center);
    }
}

// A program's own drawing in its rectangle, like a graph: whatever `draw`
// paints, from the state it captured when the view was built.
pub struct Canvas {
    draw: Box<Painting>,
}

// What a canvas paints, given the painter and its rectangle.
type Painting = dyn Fn(&mut Painter, Rect);

impl Canvas {
    pub fn new(draw: impl Fn(&mut Painter, Rect) + 'static) -> Self {
        Self { draw: Box::new(draw) }
    }
}

impl<M: 'static> Widget<M> for Canvas {
    fn kind(&self) -> &'static str {
        "Canvas"
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        (self.draw)(&mut ctx.painter(), rect);
    }
}

// The keys of a combination like "Ctrl+S" or "RB+Left stick", each on its
// own keycap. "+" alone is the plus key's.
fn keys_of(keys: &str) -> Vec<String> {
    let parts: Vec<String> = keys.split('+').map(|key| key.trim().to_string()).collect();
    if parts.len() > 1 && parts.iter().all(|part| !part.is_empty()) { parts } else { vec![keys.to_string()] }
}
