#include "IdentifyDisplays/IdentifyDisplays.h"

#include "UserInterface/Design/Theme.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"

#include <algorithm>
#include <cmath>
#include <cstddef>

namespace identify_displays {

using displays::Display;
using user_interface::Align;
using user_interface::Border;
using user_interface::Color;
using user_interface::Corners;
using user_interface::Draw;
using user_interface::Font;
using user_interface::Fonts;
using user_interface::Painter;
using user_interface::Rect;
using user_interface::TextStyle;
using user_interface::Theme;
using user_interface::Weight;
namespace tokens = user_interface::tokens;

namespace {

// The identifier is in Cascadia Mono, whose 0 has a slash through it.
constexpr TextStyle Identifier = TextStyle(Font::Mono, Weight::Semibold, 112.0f, 132.0f);
constexpr float AccentBar = 4.0f;
constexpr float MinimumWidth = 360.0f;

} // namespace

Identity::Identity(std::span<const Display* const> displays) {
    for (size_t index = 0; index < displays.size(); index++) {
        const Display& display = *displays[index];
        if (index > 0) {
            lines.push_back(Line{
                .text = std::string(),
                .style = tokens::Body,
                .kind = Kind::Gap,
            });
        }
        lines.push_back(Line{
            .text = display.identifier,
            .style = Identifier,
            .kind = Kind::Identifier,
        });
        lines.push_back(Line{
            .text = display.name,
            .style = tokens::Title,
            .kind = Kind::Name,
        });
        std::string details = display.details();
        if (display.primary) {
            details += ", primary";
        }
        lines.push_back(Line{
            .text = std::move(details),
            .style = tokens::BodyLarge,
            .kind = Kind::Details,
        });
    }
    lines.push_back(Line{
        .text = std::string(),
        .style = tokens::Body,
        .kind = Kind::Gap,
    });
    lines.push_back(Line{
        .text = "Click or press Esc to close",
        .style = tokens::Body,
        .kind = Kind::Hint,
    });
    scale = static_cast<float>(displays[0]->dpi) / 96.0f;
}

TextStyle Identity::style(TextStyle style) const {
    return TextStyle(style.font, style.weight_, style.size * scale, std::round(style.line_height * scale));
}

float Identity::padding() const {
    return std::round(tokens::SpaceXxxl * scale);
}

std::pair<float, float> Identity::size(Fonts& fonts) const {
    float width = MinimumWidth * scale;
    float height = 0.0f;
    for (const Line& line : lines) {
        TextStyle scaled = style(line.style);
        width = std::max(width, std::ceil(fonts.measure(line.text, scaled).width));
        height += scaled.line_height;
    }
    float around = padding();
    return {width + around * 2.0f, height + around * 2.0f + AccentBar * scale};
}

std::string_view Identity::kind() const {
    return "Identity";
}

void Identity::draw(Draw& ctx) const {
    Rect bounds = ctx.rect();
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    painter.fill(bounds, theme.acrylic);
    painter.fill(bounds.top_strip(std::round(AccentBar * scale)), theme.accent);
    painter.shape(bounds, Corners::Zero, Color::Transparent, Border(theme.surface_stroke_flyout, 1.0f));
    float around = padding();
    float y = AccentBar * scale + around;
    for (const Line& line : lines) {
        TextStyle scaled = style(line.style);
        Color color = theme.text_tertiary;
        switch (line.kind) {
        case Kind::Identifier:
        case Kind::Name:
            color = theme.text_primary;
            break;
        case Kind::Details:
            color = theme.text_secondary;
            break;
        case Kind::Hint:
        case Kind::Gap:
            color = theme.text_tertiary;
            break;
        }
        Rect row(0.0f, y, bounds.width, scaled.line_height);
        painter.text_in(row, line.text, scaled, color, Align::Center);
        y += scaled.line_height;
    }
}

} // namespace identify_displays
