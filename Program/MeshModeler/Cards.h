// Describes the modeler's interface from its state, with
// Library/UserInterface's widgets: the app bar with its menu bar, the status
// bar, and the cards, which have fixed places in pixels for the two client
// areas that programs support, 2560 x 1369 maximized and 2560 x 1440 in
// borderless fullscreen. The view and the cards at the bottom of the side
// columns get the 71 pixels that fullscreen adds.
//
// Left: Select, with the selection mode and the tool; Edit, with the edits;
// Add, with primitives; and Model, with what the model has. Middle: the
// view, which the modeler's render fills with the mesh. Right: Shading, with
// the shader and subdivision, and Paints, with the paints and SimplePaint's
// settings for the one picked.

#pragma once

#include "MeshModeler/Modeler.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Drawing/Geometry.h"

namespace mesh_modeler::cards {

inline constexpr float LeftWidth = 328.0f;
inline constexpr float RightWidth = 400.0f;

// Where the view card's content is, from the window's bounds alone, with no
// notices under the app bar: Prepare places the scene with it before the
// frame's view runs, and the view card lays out the same way.
user_interface::Rect view_content(user_interface::Rect bounds);

user_interface::Tree<modeler::Message> view(const modeler::Modeler& modeler, user_interface::Ui& ui);

// The size of the paint list's swatches, for tests.
extern const user_interface::Size SwatchSize;

} // namespace mesh_modeler::cards
