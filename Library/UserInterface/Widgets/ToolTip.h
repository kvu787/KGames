// WinUI's ToolTip: what a control is for, after the pointer rests on it for
// TooltipDelay. It's in-app acrylic with a flyout's stroke, raised like a
// tooltip, above the pointer, or below it if there's no room above, and
// always in the window. A press or the wheel puts it away until the pointer
// moves to another control. Elements get one with Element::tooltip; this
// draws it.

#pragma once

#include "UserInterface/Core/Tree.h"
#include "UserInterface/Drawing/Geometry.h"

#include <string_view>

namespace user_interface {

// The library's own: draws the tooltip of the target with the name, anchored
// where the pointer rested on it.
void draw_tooltip(Interface& ui, std::string_view name, const Tooltip& tooltip, Point anchor);

} // namespace user_interface
