#include "SpinningCarTest/Controls.h"

namespace spinning_car_test::controls {

bool toggles_stationary(const user_interface::Input& input) {
    return input.key_pressed_once(user_interface::Key::S);
}

} // namespace spinning_car_test::controls
