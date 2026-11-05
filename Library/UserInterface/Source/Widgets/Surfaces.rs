// Surfaces and things that inform: cards, info bars, tooltips, keycaps,
// color swatches, badges, dividers and text.

use crate::geometry::{Color, Point, Rect, Size};
use crate::icons::Icon;
use crate::painter::{Align, Corners, Layer, Stroke};
use crate::text::TextStyle;
use crate::tokens::{
    BODY, BODY_STRONG, CAPTION, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, CONTROL_HEIGHT, DURATION_FASTER,
    ELEVATION_16, ICON, ICON_SMALL, MONO, RADIUS_CONTROL, RADIUS_OVERLAY, SPACE_L, SPACE_M, SPACE_S,
    SPACE_XS,
};
use crate::ui::{Response, Sense, Ui};
use crate::widgets::common::Visual;

// A card's parts: its header, right of its title, for controls that go
// there, and the area under the header for its content.
#[derive(Clone, Copy, Debug)]
pub struct Card {
    pub rect: Rect,
    pub header: Rect,
    pub content: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Informational,
    Success,
    Warning,
    Error,
}

impl Severity {
    pub fn name(self) -> &'static str {
        match self {
            Severity::Informational => "Informational",
            Severity::Success => "Success",
            Severity::Warning => "Warning",
            Severity::Error => "Error",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InfoBar<'a> {
    pub severity: Severity,
    pub title: &'a str,
    pub message: &'a str,
    pub closable: bool,
}

impl<'a> InfoBar<'a> {
    pub fn new(severity: Severity, title: &'a str, message: &'a str) -> Self {
        Self {
            severity,
            title,
            message,
            closable: false,
        }
    }

    pub fn closable(mut self) -> Self {
        self.closable = true;
        self
    }
}

// A line of a tooltip with more than a sentence.
#[derive(Clone, Copy, Debug)]
pub enum TooltipLine<'a> {
    Heading(&'a str),
    Text(&'a str),
    // A label and its value, in columns.
    Pair(&'a str, &'a str),
    // Text in Cascadia Mono, whose columns line up.
    Mono(&'a str),
    Separator,
}

const INFO_BAR_ICON_X: f32 = 16.0;
const INFO_BAR_TEXT_X: f32 = 48.0;
const INFO_BAR_PADDING: f32 = 14.0;
const INFO_BAR_CLOSE: f32 = 32.0;
const TOOLTIP_MAX_WIDTH: f32 = 320.0;
const TOOLTIP_PADDING_X: f32 = 9.0;
const TOOLTIP_PADDING_Y: f32 = 6.0;

impl Ui {
    // A card: a surface that groups related content, with an optional title.
    pub fn card(&mut self, rect: Rect, title: &str) -> Card {
        let theme = self.theme();
        let (fill, stroke, text) = (theme.card_background, theme.card_stroke, theme.text_primary);
        self.cover(rect);
        let mut painter = self.painter();
        painter.shape(rect, Corners::all(RADIUS_OVERLAY), fill, Some(Stroke::new(stroke, 1.0)));
        card_parts(&mut painter, rect, title, text)
    }

    // A card that floats over a 3D view, opaque and raised.
    pub fn floating_card(&mut self, rect: Rect, title: &str) -> Card {
        let theme = self.theme();
        let (fill, stroke, text) = (theme.flyout, theme.surface_stroke_flyout, theme.text_primary);
        self.cover(rect);
        let mut painter = self.painter();
        painter.shadow(rect, RADIUS_OVERLAY, ELEVATION_16);
        painter.shape(rect, Corners::all(RADIUS_OVERLAY), fill, Some(Stroke::new(stroke, 1.0)));
        card_parts(&mut painter, rect, title, text)
    }

    // How tall an info bar is at a width: one line if its title and message
    // fit, or else its message wraps under its title.
    pub fn info_bar_height(&mut self, width: f32, info: &InfoBar) -> f32 {
        let (_, height) = info_bar_layout(self, width, info);
        height
    }

    // A bar that tells of something, in its severity's colors. Its response
    // is clicked when its close button is pressed.
    pub fn info_bar(&mut self, name: &str, rect: Rect, info: InfoBar) -> Response {
        let theme = self.theme().clone();
        let (background, color, mark) = match info.severity {
            Severity::Informational => (theme.attention_background, theme.attention, Icon::StatusInfo),
            Severity::Success => (theme.success_background, theme.success, Icon::StatusSuccess),
            Severity::Warning => (theme.caution_background, theme.caution, Icon::StatusWarning),
            Severity::Error => (theme.critical_background, theme.critical, Icon::StatusError),
        };
        self.cover(rect);
        let (one_line, _) = info_bar_layout(self, rect.width, &info);
        let mut painter = self.painter();
        painter.shape(rect, Corners::all(RADIUS_CONTROL), background, Some(Stroke::new(theme.card_stroke, 1.0)));
        let line_top = rect.y + INFO_BAR_PADDING;
        let icon_center = Point::new(rect.x + INFO_BAR_ICON_X + ICON / 2.0, line_top + 10.0);
        painter.icon(icon_center, Icon::StatusCircle, ICON, color);
        painter.icon(icon_center, mark, ICON, theme.text_on_accent);
        let text_right = rect.right() - if info.closable { INFO_BAR_CLOSE + SPACE_M } else { SPACE_L };
        let text_x = rect.x + INFO_BAR_TEXT_X;
        let title_size = painter.text(Point::new(text_x, line_top), info.title, BODY_STRONG, theme.text_primary);
        if !info.message.is_empty() {
            if one_line {
                let x = text_x + title_size.width + if info.title.is_empty() { 0.0 } else { SPACE_M };
                painter.text(Point::new(x, line_top), info.message, BODY, theme.text_primary);
            } else {
                let top = if info.title.is_empty() { line_top } else { line_top + BODY.line_height };
                painter.text_wrapped(Rect::new(text_x, top, text_right - text_x, 0.0), info.message, BODY, theme.text_primary);
            }
        }
        let mut response = Response {
            rect,
            ..Default::default()
        };
        self.register(name, rect, Sense::NONE, "InfoBar");
        self.set_label(name, info.title);
        if info.closable {
            let close = Rect::new(rect.right() - INFO_BAR_CLOSE - SPACE_S, rect.y + SPACE_S, INFO_BAR_CLOSE, INFO_BAR_CLOSE);
            let button = self.interact(&format!("{name}/Close"), close, Sense::BUTTON, "Button");
            let visual = Visual::of(self, &button);
            let fill = visual.pick(Color::TRANSPARENT, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, Color::TRANSPARENT);
            let fill = self.animate_color(button.id, 0, fill, DURATION_FASTER);
            let glyph = visual.pick(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled);
            let mut painter = self.painter();
            painter.rounded(close, RADIUS_CONTROL, fill);
            painter.icon(close.center(), Icon::Dismiss, ICON_SMALL, glyph);
            crate::widgets::common::control_focus_ring(self, &button);
            response.clicked = button.clicked;
        }
        response
    }

    // Shows a sentence about the widget with the name while the pointer
    // rests on it.
    pub fn tooltip(&mut self, name: &str, text: &str) {
        self.tooltip_lines(name, &[TooltipLine::Text(text)]);
    }

    // Shows lines about the widget with the name while the pointer rests on
    // it, in a box that fits them.
    pub fn tooltip_lines(&mut self, name: &str, lines: &[TooltipLine]) {
        let Some(anchor) = self.tooltip_due(crate::ui::id(name)) else {
            return;
        };
        let theme = self.theme().clone();
        let window = self.bounds();
        let mut painter = self.painter();
        // Measure the lines.
        let label_width = lines
            .iter()
            .filter_map(|line| match line {
                TooltipLine::Pair(label, _) => Some(painter.measure(label, CAPTION).width),
                _ => None,
            })
            .fold(0.0f32, f32::max);
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        for line in lines {
            let (w, h) = match line {
                TooltipLine::Heading(text) => {
                    let size = painter.measure(text, BODY_STRONG);
                    (size.width, size.height)
                }
                TooltipLine::Text(text) => {
                    let size = painter.measure_wrapped(text, BODY, TOOLTIP_MAX_WIDTH);
                    (size.width.min(TOOLTIP_MAX_WIDTH), size.height)
                }
                TooltipLine::Pair(_, value) => {
                    let size = painter.measure(value, CAPTION);
                    (label_width + SPACE_M + size.width, CAPTION.line_height)
                }
                TooltipLine::Mono(text) => {
                    let size = painter.measure(text, MONO);
                    (size.width, size.height)
                }
                TooltipLine::Separator => (0.0, SPACE_S + 1.0),
            };
            width = width.max(w);
            height += h;
        }
        let size = Size::new((width + TOOLTIP_PADDING_X * 2.0).ceil(), (height + TOOLTIP_PADDING_Y * 2.0 + 2.0).ceil());
        let mut rect = Rect::new(anchor.x - SPACE_S, anchor.y + 24.0, size.width, size.height);
        if rect.bottom() > window.bottom() - SPACE_XS {
            rect.y = anchor.y - SPACE_M - size.height;
        }
        rect.x = rect.x.clamp(SPACE_XS, (window.right() - size.width - SPACE_XS).max(SPACE_XS));
        rect.y = rect.y.clamp(SPACE_XS, (window.bottom() - size.height - SPACE_XS).max(SPACE_XS));
        let rect = rect.round();

        painter.set_layer(Layer::Tooltip);
        painter.push_clip_absolute(window);
        painter.shadow(rect, RADIUS_CONTROL, ELEVATION_16);
        painter.shape(rect, Corners::all(RADIUS_CONTROL), theme.flyout, Some(Stroke::new(theme.surface_stroke_flyout, 1.0)));
        let mut y = rect.y + TOOLTIP_PADDING_Y + 1.0;
        let x = rect.x + TOOLTIP_PADDING_X;
        for line in lines {
            match line {
                TooltipLine::Heading(text) => {
                    y += painter.text(Point::new(x, y), text, BODY_STRONG, theme.text_primary).height;
                }
                TooltipLine::Text(text) => {
                    y += painter.text_wrapped(Rect::new(x, y, TOOLTIP_MAX_WIDTH, 0.0), text, BODY, theme.text_primary).height;
                }
                TooltipLine::Pair(label, value) => {
                    painter.text(Point::new(x, y), label, CAPTION, theme.text_secondary);
                    painter.text(Point::new(x + label_width + SPACE_M, y), value, CAPTION, theme.text_primary);
                    y += CAPTION.line_height;
                }
                TooltipLine::Mono(text) => {
                    y += painter.text(Point::new(x, y), text, MONO, theme.text_primary).height;
                }
                TooltipLine::Separator => {
                    painter.fill(Rect::new(rect.x + 1.0, y + SPACE_S / 2.0, rect.width - 2.0, 1.0), theme.divider);
                    y += SPACE_S + 1.0;
                }
            }
        }
        painter.pop_clip();
        painter.set_layer(Layer::Base);
        self.shown_tooltip = Some((name.to_string(), rect));
    }

    // A key's name in a box, like the keys of a shortcut. Returns its width.
    pub fn keycap(&mut self, origin: Point, key: &str) -> f32 {
        let theme = self.theme();
        let (fill, stroke, bottom, text) = (theme.control_fill, theme.control_stroke, theme.control_stroke_secondary, theme.text_secondary);
        let mut painter = self.painter();
        let width = (painter.measure(key, CAPTION).width + SPACE_S * 2.0).max(22.0).ceil();
        let rect = Rect::new(origin.x, origin.y, width, 22.0);
        painter.shape(rect, Corners::all(RADIUS_CONTROL), fill, Some(Stroke::elevated(stroke, bottom)));
        painter.text_in(rect, key, CAPTION, text, Align::Center);
        width
    }

    // A square of color that can be chosen, like a palette's.
    pub fn swatch(&mut self, name: &str, rect: Rect, color: Color, selected: bool) -> Response {
        let response = self.interact(name, rect, Sense::BUTTON, "Swatch");
        self.set_label(name, &color.to_hex());
        let visual = Visual::of(self, &response);
        let theme = self.theme();
        let stroke = visual.pick(theme.control_stroke, theme.control_strong_stroke, theme.control_strong_stroke, theme.control_stroke);
        let (accent, inner) = (theme.accent, theme.focus_inner);
        let mut painter = self.painter();
        painter.shape(rect, Corners::all(RADIUS_CONTROL), color, Some(Stroke::new(stroke, 1.0)));
        if selected {
            painter.shape(rect.expand(1.0), Corners::all(RADIUS_CONTROL + 1.0), Color::TRANSPARENT, Some(Stroke::new(inner, 1.0)));
            painter.shape(rect.expand(3.0), Corners::all(RADIUS_CONTROL + 3.0), Color::TRANSPARENT, Some(Stroke::new(accent, 2.0)));
        } else {
            crate::widgets::common::focus_ring(self, &response, rect, RADIUS_CONTROL);
        }
        response
    }

    // A small pill with a count or a word, in the accent color, as WinUI's
    // InfoBadge. Returns its width.
    pub fn badge(&mut self, center_left: Point, text: &str) -> f32 {
        let theme = self.theme();
        let (fill, color) = (theme.accent, theme.text_on_accent);
        let mut painter = self.painter();
        let width = (painter.measure(text, CAPTION).width + SPACE_S * 2.0).max(16.0).ceil();
        let rect = Rect::new(center_left.x, (center_left.y - 8.0).round(), width, 16.0);
        painter.rounded(rect, 8.0, fill);
        painter.text_in(rect, text, CAPTION, color, Align::Center);
        width
    }

    // A line that separates, 1 pixel thick.
    pub fn divider(&mut self, rect: Rect) {
        let color = self.theme().divider;
        let mut painter = self.painter();
        if rect.width >= rect.height {
            painter.fill(Rect::new(rect.x, rect.center().y.floor(), rect.width, 1.0), color);
        } else {
            painter.fill(Rect::new(rect.center().x.floor(), rect.y, 1.0, rect.height), color);
        }
    }

    // One line of text in the style and color, vertically centered in the
    // rectangle, cut short with an ellipsis if it doesn't fit.
    pub fn text(&mut self, rect: Rect, text: &str, style: TextStyle, color: Color) -> Rect {
        self.painter().text_fit(rect, text, style, color, Align::Start)
    }

    pub fn text_aligned(&mut self, rect: Rect, text: &str, style: TextStyle, color: Color, align: Align) -> Rect {
        self.painter().text_fit(rect, text, style, color, align)
    }

    // Body text in the primary text color.
    pub fn label(&mut self, rect: Rect, text: &str) -> Rect {
        let color = self.theme().text_primary;
        self.text(rect, text, BODY, color)
    }

    // A caption in the secondary text color.
    pub fn caption(&mut self, rect: Rect, text: &str) -> Rect {
        let color = self.theme().text_secondary;
        self.text(rect, text, CAPTION, color)
    }

    // Text wrapped to the rectangle's width. Returns its height.
    pub fn paragraph(&mut self, rect: Rect, text: &str, style: TextStyle, color: Color) -> f32 {
        self.painter().text_wrapped(rect, text, style, color).height
    }

    // A label and its value on one line: the label in the secondary color,
    // the value in the primary one, `label_width` to its right.
    pub fn field(&mut self, rect: Rect, label: &str, label_width: f32, value: &str) {
        let theme = self.theme();
        let (secondary, primary) = (theme.text_secondary, theme.text_primary);
        let mut painter = self.painter();
        painter.text_fit(rect.left_strip(label_width - SPACE_S), label, BODY, secondary, Align::Start);
        painter.text_fit(rect.inset_by(label_width, 0.0, 0.0, 0.0), value, BODY, primary, Align::Start);
    }

    // An icon in a color, centered in the rectangle.
    pub fn icon(&mut self, rect: Rect, icon: Icon, size: f32, color: Color) {
        self.painter().icon(rect.center(), icon, size, color);
    }
}

fn card_parts(painter: &mut crate::painter::Painter, rect: Rect, title: &str, color: Color) -> Card {
    let inner = rect.inset(CARD_PADDING);
    if title.is_empty() {
        return Card {
            rect,
            header: Rect::new(inner.x, inner.y, inner.width, 0.0),
            content: inner,
        };
    }
    let header_row = inner.top_strip(CARD_HEADER_HEIGHT);
    let title_rect = painter.text_fit(header_row, title, BODY_STRONG, color, Align::Start);
    let header = Rect::new(
        title_rect.right() + SPACE_M,
        header_row.y,
        (header_row.right() - title_rect.right() - SPACE_M).max(0.0),
        CONTROL_HEIGHT,
    );
    let content = inner.inset_by(0.0, CARD_HEADER_HEIGHT + CARD_HEADER_GAP, 0.0, 0.0);
    Card { rect, header, content }
}

// Whether an info bar's title and message fit on one line, and its height.
fn info_bar_layout(ui: &mut Ui, width: f32, info: &InfoBar) -> (bool, f32) {
    let text_width = width - INFO_BAR_TEXT_X - if info.closable { INFO_BAR_CLOSE + SPACE_M } else { SPACE_L };
    let title = if info.title.is_empty() { 0.0 } else { ui.measure(info.title, BODY_STRONG).width + SPACE_M };
    let message = ui.measure(info.message, BODY).width;
    if title + message <= text_width {
        return (true, BODY.line_height + INFO_BAR_PADDING * 2.0);
    }
    let wrapped = ui.fonts().layout(info.message, BODY, Some(text_width.max(1.0))).height;
    let title_height = if info.title.is_empty() { 0.0 } else { BODY.line_height };
    (false, title_height + wrapped + INFO_BAR_PADDING * 2.0)
}
