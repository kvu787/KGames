// What every widget shares: the brushes of WinUI's visual states, its focus
// visual, and its elevation borders, the gradients that make controls look
// raised.

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"

#include <cstdint>
#include <functional>
#include <utility>

namespace user_interface {

// A brush for each of WinUI's visual states, like ButtonBackground,
// ButtonBackgroundPointerOver, ButtonBackgroundPressed and
// ButtonBackgroundDisabled.
struct StateColors {
    Color normal;
    Color pointer_over;
    Color pressed;
    Color disabled;

    constexpr StateColors(Color normal, Color pointer_over, Color pressed, Color disabled) : normal(normal), pointer_over(pointer_over), pressed(pressed), disabled(disabled) {}

    static constexpr StateColors all(Color color) {
        return StateColors(color, color, color, color);
    }

    Color get(ControlState state) const;

    bool operator==(const StateColors&) const = default;
};

// WinUI's elevation border brushes: gradients across a control's border
// that shade one edge, so that it looks raised.
enum class Elevation : std::uint8_t {
    // ControlElevationBorderBrush, of buttons, combo boxes and the like.
    Control,
    // AccentControlElevationBorderBrush, of accent buttons.
    Accent,
    // CircleElevationBorderBrush, of round things like a slider's thumb.
    Circle,
    // TextControlElevationBorderBrush, of text fields at rest.
    TextControl,
    // TextControlElevationBorderFocusedBrush: the accent along the bottom.
    TextControlFocused,
};

// The border's brush over a control in the rectangle.
Brush brush(Elevation elevation, const Theme& theme, Rect rect);

Border border(Elevation elevation, const Theme& theme, Rect rect);

// WinUI's focus visual: a 2-pixel outer stroke in FocusStrokeColorOuter and
// a 1-pixel inner one in FocusStrokeColorInner, around the control by its
// FocusVisualMargin, with its corners as much rounder as the margin is wide.
void focus_visual(Painter& painter, const Theme& theme, Rect rect, float radius, Insets margin);

// A reaction to a widget's value: the message that the value makes.
template <class V, class M> using Reaction = std::function<M(V)>;

// The library's own: the reaction that anything callable with a V, which
// returns an M, is.
template <class V, class M, class F> Reaction<V, M> react(F&& reaction) {
    return Reaction<V, M>(std::forward<F>(reaction));
}

} // namespace user_interface
