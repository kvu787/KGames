// WinUI's ToolTip: what a control is for, after the pointer rests on it for
// TOOLTIP_DELAY. It's in-app acrylic with a flyout's stroke, raised like a
// tooltip, above the pointer, or below it if there's no room above, and
// always in the window. A press or the wheel puts it away until the pointer
// moves to another control. Elements get one with Element::tooltip; this
// draws it.

use crate::core::interface::Interface;
use crate::core::tree::{Tooltip, TooltipLine};
use crate::design::controls::{TOOLTIP_OFFSET, TOOLTIP_PADDING};
use crate::design::tokens::{CAPTION, CAPTION_STRONG, ELEVATION_TOOLTIP, MONO, RADIUS_CONTROL, SPACE_S, SPACE_XS};
use crate::drawing::geometry::{Point, Rect, Size};
use crate::drawing::painter::{Align, Border, Layer};

// Draws the tooltip of the target with the name, anchored where the pointer
// rested on it.
pub(crate) fn draw_tooltip(ui: &mut Interface, name: &str, tooltip: &Tooltip, anchor: Point) {
    let lines = &tooltip.lines;
    if lines.is_empty() {
        return;
    }
    let bounds = ui.bounds();
    let inner_width = tooltip.max_width.min(bounds.width - SPACE_S * 2.0) - TOOLTIP_PADDING.horizontal();
    // The label column of pairs is as wide as its widest label.
    let label_width = lines
        .iter()
        .filter_map(|line| if let TooltipLine::Pair(label, _) = line { Some(ui.fonts.measure(label, CAPTION).width) } else { None })
        .fold(0.0, f32::max);
    let mut heights = Vec::with_capacity(lines.len());
    let mut width: f32 = 0.0;
    for line in lines {
        let (size, line_width) = match line {
            TooltipLine::Heading(text) => (ui.fonts.measure_wrapped(text, CAPTION_STRONG, inner_width), ui.fonts.measure(text, CAPTION_STRONG).width),
            TooltipLine::Text(text) => (ui.fonts.measure_wrapped(text, CAPTION, inner_width), ui.fonts.measure(text, CAPTION).width),
            TooltipLine::Mono(text) => (ui.fonts.measure_wrapped(text, MONO, inner_width), ui.fonts.measure(text, MONO).width),
            TooltipLine::Pair(_, value) => {
                let value_width = inner_width - label_width - SPACE_S;
                (ui.fonts.measure_wrapped(value, CAPTION, value_width), label_width + SPACE_S + ui.fonts.measure(value, CAPTION).width)
            }
            TooltipLine::Separator => (Size::new(0.0, SPACE_S + 1.0), 0.0),
        };
        heights.push(size.height);
        width = width.max(line_width.min(inner_width));
    }
    let content = Size::new(width.ceil(), heights.iter().sum::<f32>() + SPACE_XS * (lines.len() - 1) as f32);
    let size = Size::new(content.width + TOOLTIP_PADDING.horizontal(), content.height + TOOLTIP_PADDING.vertical());
    let mut x = anchor.x - size.width / 2.0;
    let mut y = anchor.y - TOOLTIP_OFFSET - size.height;
    if y < bounds.y + SPACE_S {
        y = anchor.y + TOOLTIP_OFFSET;
    }
    x = x.clamp(bounds.x + SPACE_S, (bounds.right() - size.width - SPACE_S).max(bounds.x + SPACE_S));
    y = y.clamp(bounds.y + SPACE_S, (bounds.bottom() - size.height - SPACE_S).max(bounds.y + SPACE_S));
    let rect = Rect::new(x.round(), y.round(), size.width, size.height);
    ui.shown_tooltip = Some((name.to_string(), rect));

    let mut painter = ui.painter();
    let theme = painter.theme();
    painter.set_layer(Layer::Tooltip);
    painter.push_clip(bounds);
    painter.shadow(rect, RADIUS_CONTROL, ELEVATION_TOOLTIP);
    painter.shape(rect, RADIUS_CONTROL, theme.acrylic, Some(Border::new(theme.surface_stroke_flyout, 1.0)));
    let mut area = rect.pad(TOOLTIP_PADDING);
    for (line, height) in lines.iter().zip(heights) {
        let row = area.cut_top(height, SPACE_XS);
        match line {
            TooltipLine::Heading(text) => {
                painter.text_wrapped(row, text, CAPTION_STRONG, theme.text_primary);
            }
            TooltipLine::Text(text) => {
                painter.text_wrapped(row, text, CAPTION, theme.text_primary);
            }
            TooltipLine::Mono(text) => {
                painter.text_wrapped(row, text, MONO, theme.text_primary);
            }
            TooltipLine::Pair(label, value) => {
                painter.text_in(Rect::new(row.x, row.y, label_width, CAPTION.line_height), label, CAPTION, theme.text_secondary, Align::Start);
                let value_rect = Rect::new(row.x + label_width + SPACE_S, row.y, row.width - label_width - SPACE_S, row.height);
                painter.text_wrapped(value_rect, value, CAPTION, theme.text_primary);
            }
            TooltipLine::Separator => {
                let y = row.y + (SPACE_S / 2.0).round();
                painter.fill(Rect::new(rect.x, y, rect.width, 1.0), theme.divider);
            }
        }
    }
    painter.pop_clip();
    painter.set_layer(Layer::Base);
}
