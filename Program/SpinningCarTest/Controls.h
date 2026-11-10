// What the test's own key does. The card handles F1, and Library/UserInterface
// F11.

#pragma once

#include "UserInterface/Input/Events.h"

namespace spinning_car_test::controls {

// Whether S went down this frame, which keeps the car in the middle, or lets
// it move. Holding it down repeats it, which does nothing more.
bool toggles_stationary(const user_interface::Input& input);

} // namespace spinning_car_test::controls
