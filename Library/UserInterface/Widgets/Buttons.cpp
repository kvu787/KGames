#include "UserInterface/Widgets/Buttons.h"

#include "Base/Panic.h"
#include "UserInterface/Core/Interface.h"

#include <algorithm>
#include <cmath>

namespace user_interface {

namespace {

// The brushes of a button in each state, per style.
struct ButtonColors {
    StateColors background;
    StateColors foreground;
    // An elevation border, and a flat border for the pressed and disabled
    // states.
    std::optional<std::pair<Elevation, StateColors>> border;
};

ButtonColors colors(const Theme& theme, ButtonStyle style, bool checked) {
    if (checked) {
        return ButtonColors{
            .background = StateColors(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled),
            .foreground = StateColors(theme.text_on_accent, theme.text_on_accent, theme.text_on_accent_secondary, theme.text_on_accent_disabled),
            .border = std::pair(Elevation::Accent,
                                StateColors(theme.control_stroke_on_accent, theme.control_stroke_on_accent, theme.control_fill_transparent, theme.control_fill_transparent)),
        };
    }
    switch (style) {
    case ButtonStyle::Standard:
        return ButtonColors{
            .background = StateColors(theme.control_fill, theme.control_fill_secondary, theme.control_fill_tertiary, theme.control_fill_disabled),
            .foreground = StateColors(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled),
            .border = std::pair(Elevation::Control, StateColors(theme.control_stroke, theme.control_stroke, theme.control_stroke, theme.control_stroke)),
        };
    case ButtonStyle::Accent:
        return ButtonColors{
            .background = StateColors(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled),
            .foreground = StateColors(theme.text_on_accent, theme.text_on_accent, theme.text_on_accent_secondary, theme.text_on_accent_disabled),
            .border = std::pair(Elevation::Accent,
                                StateColors(theme.control_stroke_on_accent, theme.control_stroke_on_accent, theme.control_fill_transparent, theme.control_fill_transparent)),
        };
    case ButtonStyle::Subtle:
        return ButtonColors{
            .background = StateColors(theme.subtle_fill_transparent, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, theme.subtle_fill_disabled),
            .foreground = StateColors(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled),
            .border = std::nullopt,
        };
    }
    base::unreachable();
}

} // namespace

Fonts& buttons_detail::fonts_of(Ui& ui) {
    return ui.fonts();
}

void draw_button(Draw& ctx, Rect rect, std::string_view text, std::optional<Icon> icon, ButtonStyle style, bool checked, bool chevron, ControlState state) {
    ButtonColors button_colors = colors(ctx.theme(), style, checked);
    Color background = button_colors.background.get(state);
    Color foreground = button_colors.foreground.get(state);
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    // At rest and with the pointer over, the border is the elevation
    // gradient; pressed and disabled, it's flat.
    std::optional<Border> border;
    if (button_colors.border) {
        const auto& [elevation, flat] = *button_colors.border;
        switch (state) {
        case ControlState::Normal:
        case ControlState::PointerOver:
            border = user_interface::border(elevation, theme, rect);
            break;
        case ControlState::Pressed:
        case ControlState::Disabled:
            border = Border(flat.get(state), 1.0f);
            break;
        }
    }
    painter.shape(rect, tokens::RadiusControl, background, border);
    Rect content = rect.pad(controls::ButtonPadding);
    float icon_width = icon ? (text.empty() ? tokens::Icon : tokens::Icon + controls::ButtonIconGap) : 0.0f;
    float text_width = text.empty() ? 0.0f : std::min(painter.measure(text, tokens::Body).width, content.width - icon_width);
    float chevron_width = chevron ? controls::DropDownChevronGap + controls::DropDownChevronSize : 0.0f;
    float total = icon_width + text_width + chevron_width;
    float x = std::round(std::max(content.x + (content.width - total) / 2.0f, content.x));
    if (icon) {
        painter.icon(Point(x + tokens::Icon / 2.0f, rect.center().y), *icon, tokens::Icon, foreground);
        x += icon_width;
    }
    if (!text.empty()) {
        Rect text_rect(x, rect.y, text_width, rect.height);
        painter.text_fit(text_rect, text, tokens::Body, foreground, Align::Start);
        x += text_width;
    }
    if (chevron) {
        painter.icon(Point(x + controls::DropDownChevronGap + controls::DropDownChevronSize / 2.0f, rect.center().y), Icon::ChevronDown, controls::DropDownChevronSize, foreground);
    }
}

void draw_text_control_button(Draw& ctx, const Part& part, Rect rect, Icon icon, float glyph) {
    ControlState state = ctx.look().part_state(part);
    const Theme& theme = ctx.theme();
    Color background = StateColors(Color::Transparent, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, Color::Transparent).get(state);
    Color foreground = StateColors(theme.text_secondary, theme.text_secondary, theme.text_tertiary, theme.text_disabled).get(state);
    Painter painter = ctx.painter();
    painter.rounded(rect, tokens::RadiusControl, background);
    painter.icon(rect.center(), icon, glyph, foreground);
}

} // namespace user_interface
