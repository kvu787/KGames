// The test. See README.md. It has no console window in any build, as
// SpinningCarTest.vcxproj doesn't ask for one.

#include "SpinningCarTest/SpinningCarTest.h"
#include "UserInterface/Application/Run.h"

using spinning_car_test::spinning_car::SpinningCar;

int main() {
    return user_interface::run(spinning_car_test::options(), [](user_interface::Startup& startup) { return SpinningCar::start(startup); });
}
