#include "UserInterface/Widgets/ToolTip.h"

#include "Base/Sum.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Painter.h"

#include <algorithm>
#include <cmath>
#include <string>
#include <utility>
#include <vector>

namespace user_interface {

void draw_tooltip(Interface& ui, std::string_view name, const Tooltip& tooltip, Point anchor) {
    const std::vector<TooltipLine>& lines = tooltip.lines;
    if (lines.empty()) {
        return;
    }
    Rect bounds = ui.bounds();
    float inner_width = std::min(tooltip.max_width, bounds.width - tokens::SpaceS * 2.0f) - controls::TooltipPadding.horizontal();
    // The label column of pairs is as wide as its widest label.
    float label_width = 0.0f;
    for (const TooltipLine& line : lines) {
        if (const TooltipLine::Pair* pair = base::as<TooltipLine::Pair>(line)) {
            label_width = std::fmax(label_width, ui.fonts_.measure(pair->label, tokens::Caption).width);
        }
    }
    std::vector<float> heights;
    heights.reserve(lines.size());
    float width = 0.0f;
    for (const TooltipLine& line : lines) {
        Size size;
        float line_width = 0.0f;
        base::match(
            line,
            [&](const TooltipLine::Heading& heading) {
                size = ui.fonts_.measure_wrapped(heading.text, tokens::CaptionStrong, inner_width);
                line_width = ui.fonts_.measure(heading.text, tokens::CaptionStrong).width;
            },
            [&](const TooltipLine::Text& text) {
                size = ui.fonts_.measure_wrapped(text.text, tokens::Caption, inner_width);
                line_width = ui.fonts_.measure(text.text, tokens::Caption).width;
            },
            [&](const TooltipLine::Mono& mono) {
                size = ui.fonts_.measure_wrapped(mono.text, tokens::Mono, inner_width);
                line_width = ui.fonts_.measure(mono.text, tokens::Mono).width;
            },
            [&](const TooltipLine::Pair& pair) {
                float value_width = inner_width - label_width - tokens::SpaceS;
                size = ui.fonts_.measure_wrapped(pair.value, tokens::Caption, value_width);
                line_width = label_width + tokens::SpaceS + ui.fonts_.measure(pair.value, tokens::Caption).width;
            },
            [&](const TooltipLine::Separator&) {
                size = Size(0.0f, tokens::SpaceS + 1.0f);
                line_width = 0.0f;
            });
        heights.push_back(size.height);
        width = std::max(width, std::min(line_width, inner_width));
    }
    // From -0, as Rust's sum of floats is.
    float height = -0.0f;
    for (float line_height : heights) {
        height += line_height;
    }
    Size content(std::ceil(width), height + tokens::SpaceXs * static_cast<float>(lines.size() - 1));
    Size size(content.width + controls::TooltipPadding.horizontal(), content.height + controls::TooltipPadding.vertical());
    float x = anchor.x - size.width / 2.0f;
    float y = anchor.y - controls::TooltipOffset - size.height;
    if (y < bounds.y + tokens::SpaceS) {
        y = anchor.y + controls::TooltipOffset;
    }
    x = std::clamp(x, bounds.x + tokens::SpaceS, std::max(bounds.right() - size.width - tokens::SpaceS, bounds.x + tokens::SpaceS));
    y = std::clamp(y, bounds.y + tokens::SpaceS, std::max(bounds.bottom() - size.height - tokens::SpaceS, bounds.y + tokens::SpaceS));
    Rect rect(std::round(x), std::round(y), size.width, size.height);
    ui.shown_tooltip = std::pair<std::string, Rect>(std::string(name), rect);

    Painter painter = ui.painter();
    const Theme& theme = painter.theme();
    painter.set_layer(Layer::Tooltip);
    painter.push_clip(bounds);
    painter.shadow(rect, tokens::RadiusControl, tokens::ElevationTooltip);
    painter.shape(rect, tokens::RadiusControl, theme.acrylic, Border(theme.surface_stroke_flyout, 1.0f));
    Rect area = rect.pad(controls::TooltipPadding);
    for (size_t index = 0; index < lines.size(); index++) {
        Rect row = area.cut_top(heights[index], tokens::SpaceXs);
        base::match(
            lines[index], [&](const TooltipLine::Heading& heading) { painter.text_wrapped(row, heading.text, tokens::CaptionStrong, theme.text_primary); },
            [&](const TooltipLine::Text& text) { painter.text_wrapped(row, text.text, tokens::Caption, theme.text_primary); },
            [&](const TooltipLine::Mono& mono) { painter.text_wrapped(row, mono.text, tokens::Mono, theme.text_primary); },
            [&](const TooltipLine::Pair& pair) {
                painter.text_in(Rect(row.x, row.y, label_width, tokens::Caption.line_height), pair.label, tokens::Caption, theme.text_secondary, Align::Start);
                Rect value_rect(row.x + label_width + tokens::SpaceS, row.y, row.width - label_width - tokens::SpaceS, row.height);
                painter.text_wrapped(value_rect, pair.value, tokens::Caption, theme.text_primary);
            },
            [&](const TooltipLine::Separator&) {
                float line_y = row.y + std::round(tokens::SpaceS / 2.0f);
                painter.fill(Rect(rect.x, line_y, rect.width, 1.0f), theme.divider);
            });
    }
    painter.pop_clip();
    painter.set_layer(Layer::Base);
}

} // namespace user_interface
