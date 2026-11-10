#include "UserInterface/Widgets/Widget.h"

#include "Base/Panic.h"

#include <algorithm>

namespace user_interface {

Color StateColors::get(ControlState state) const {
    switch (state) {
    case ControlState::Normal:
        return normal;
    case ControlState::PointerOver:
        return pointer_over;
    case ControlState::Pressed:
        return pressed;
    case ControlState::Disabled:
        return disabled;
    }
    base::unreachable();
}

Brush brush(Elevation elevation, const Theme& theme, Rect rect) {
    float x = rect.x;
    float top = rect.y;
    float bottom = rect.bottom();
    auto line = [&](float from, float to, Color start, Color end) {
        return Brush(Brush::Linear{
            .from = Point(x, from),
            .to = Point(x, to),
            .start = start,
            .end = end,
        });
    };
    switch (elevation) {
    // From 0 to 3 pixels, mirrored to the bottom in the light theme.
    // The dark theme's isn't mirrored, so its lighter edge is the top.
    case Elevation::Control:
        switch (theme.kind) {
        case ThemeKind::Light:
            return line(bottom - 1.0f, bottom - 3.0f, theme.control_stroke_secondary, theme.control_stroke);
        case ThemeKind::Dark:
            return line(top + 1.0f, top + 3.0f, theme.control_stroke_secondary, theme.control_stroke);
        }
        base::unreachable();
    case Elevation::Accent:
        return line(bottom - 1.0f, bottom - 3.0f, theme.control_stroke_on_accent_secondary, theme.control_stroke_on_accent);
    case Elevation::Circle:
        return line(top + rect.height * 0.5f, top + rect.height * 0.7f, theme.control_stroke, theme.control_stroke_secondary);
    case Elevation::TextControl:
        return line(bottom - 1.0f, bottom - 2.0f, theme.control_strong_stroke, theme.control_stroke);
    case Elevation::TextControlFocused:
        return line(bottom - 2.0f, bottom - 2.01f, theme.accent, theme.control_stroke);
    }
    base::unreachable();
}

Border border(Elevation elevation, const Theme& theme, Rect rect) {
    Insets widths = elevation == Elevation::TextControlFocused ? Insets(1.0f, 1.0f, 1.0f, 2.0f) : Insets::all(1.0f);
    return Border::sides(brush(elevation, theme, rect), widths);
}

void focus_visual(Painter& painter, const Theme& theme, Rect rect, float radius, Insets margin) {
    Rect outer(rect.x - margin.left, rect.y - margin.top, rect.width + margin.horizontal(), rect.height + margin.vertical());
    float outer_radius = radius > 0.0f ? radius + (margin.left + margin.top) / 2.0f : 0.0f;
    painter.shape(outer, outer_radius, Color::Transparent, Border(theme.focus_outer, 2.0f));
    painter.shape(outer.inset(2.0f), std::max(outer_radius - 2.0f, 0.0f), Color::Transparent, Border(theme.focus_inner, 1.0f));
}

} // namespace user_interface
