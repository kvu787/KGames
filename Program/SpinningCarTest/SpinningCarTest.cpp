#include "SpinningCarTest/SpinningCarTest.h"

// Where this project is, which is the test's folder.
#include "Folders.h"

namespace spinning_car_test {

user_interface::Options options() {
    return user_interface::Options("SpinningCarTest", folders::Project);
}

} // namespace spinning_car_test
