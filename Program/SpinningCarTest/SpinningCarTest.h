// SpinningCarTest's parts, as a library, so that Test/SpinningCarTest can
// test them without a window: where the car is, the keys, the card, the
// settings, and frames drawn offscreen. Main.cpp runs the program.

#pragma once

#include "SpinningCarTest/Controls.h"
#include "SpinningCarTest/Scene.h"
#include "SpinningCarTest/Settings.h"
#include "SpinningCarTest/SpinningCar.h"
#include "UserInterface/Application/Program.h"

namespace spinning_car_test {

// How Library/UserInterface runs the test.
user_interface::Options options();

} // namespace spinning_car_test
