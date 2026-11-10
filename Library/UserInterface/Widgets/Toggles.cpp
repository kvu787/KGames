#include "UserInterface/Widgets/Toggles.h"

#include <cmath>
#include <utility>

namespace user_interface {

void toggles_detail::draw_radio(Painter& painter, const Theme& theme, Rect rect, ControlState state, bool checked, std::string_view label) {
    // RadioButtonOuterEllipse(Checked)Fill and ...Stroke.
    StateColors accent(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled);
    auto [fill, stroke] =
        checked ? std::pair(accent, accent)
                : std::pair(StateColors(theme.control_alt_fill_secondary, theme.control_alt_fill_tertiary, theme.control_alt_fill_quarternary, theme.control_alt_fill_disabled),
                            StateColors(theme.control_strong_stroke, theme.control_strong_stroke, theme.control_strong_stroke_disabled, theme.control_strong_stroke_disabled));
    Color text = state == ControlState::Disabled ? theme.text_disabled : theme.text_primary;
    // The checked dot is bigger with the pointer over it, and smaller while
    // it's pressed.
    float glyph_size = controls::RadioButtonGlyph;
    switch (state) {
    case ControlState::PointerOver:
        glyph_size = controls::RadioButtonGlyphPointerOver;
        break;
    case ControlState::Pressed:
        glyph_size = controls::RadioButtonGlyphPressed;
        break;
    case ControlState::Disabled:
        glyph_size = controls::RadioButtonGlyphDisabled;
        break;
    case ControlState::Normal:
        glyph_size = controls::RadioButtonGlyph;
        break;
    }
    Point center(rect.x + controls::RadioButtonSize / 2.0f, std::round(rect.center().y));
    Rect circle = Rect::from_center(center, Size::square(controls::RadioButtonSize));
    painter.shape(circle, controls::RadioButtonSize / 2.0f, fill.get(state), Border(stroke.get(state), 1.0f));
    if (checked) {
        Rect glyph = Rect::from_center(center, Size::square(glyph_size));
        Elevation elevation = state == ControlState::Disabled ? Elevation::Control : Elevation::Accent;
        painter.shape(glyph, glyph_size / 2.0f, theme.text_on_accent, Border(brush(elevation, theme, glyph), 1.0f));
    }
    if (!label.empty()) {
        float left = circle.right() + controls::RadioButtonPadding.left;
        painter.text_fit(Rect(left, rect.y, rect.right() - left, rect.height), label, tokens::Body, text, Align::Start);
    }
}

} // namespace user_interface
