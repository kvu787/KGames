// What the keys and the pointer do in the modeler, as in Blender's edit mode:
//
// - Keys run the commands whose shortcuts they are, unless a field with focus
//   takes them, a menu or a dialog is open, or a modal edit is under way.
// - In the view, the middle button orbits, with Shift pans, and the wheel
//   zooms at the pointer. The left button selects with the Select tool: a
//   click picks, Shift adds or takes away, Alt picks a loop, and a drag
//   selects a box, with Ctrl to take it away. With the Paint tool it paints
//   the faces it presses and drags over, and with the Loop cut tool it cuts
//   the loop under the pointer. The right button opens a menu of edits.
// - A modal edit, like moving, follows the pointer anywhere until a click or
//   Enter keeps it, or a right-click or Esc takes it back. X, Y and Z limit it
//   to an axis, N to the selection's normal, and Ctrl snaps it.
//
// `messages` turns the frame's keys into messages; `view` and `operation`
// apply the pointer's input, as update runs them.

#pragma once

#include "MeshModeler/Modeler.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Widgets/Layout.h"

#include <vector>

namespace mesh_modeler::interaction {

// The frame's messages from the keys: the modal edit's, or the commands
// whose shortcuts were pressed, without repeats, unless something else has
// the keys. Prepare goes last, after everything the frame does.
std::vector<modeler::Message> messages(const modeler::Modeler& modeler, const user_interface::Ui& ui);

// What the pointer did in the view this frame.
void view(modeler::Modeler& modeler, user_interface::ViewInput input);

// A modal edit: what the pointer does on the cover over the whole window,
// which takes it from everything else while the edit is under way.
void operation(modeler::Modeler& modeler, user_interface::ViewInput input);

} // namespace mesh_modeler::interaction
