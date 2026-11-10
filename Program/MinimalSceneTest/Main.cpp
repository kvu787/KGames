// The test. See README.md. It has no console window in any build, as
// MinimalSceneTest.vcxproj doesn't ask for one.

#include "MinimalSceneTest/MinimalSceneTest.h"
#include "UserInterface/Application/Run.h"

using minimal_scene_test::moving_sphere::MovingSphere;

int main() {
    return user_interface::run(minimal_scene_test::options(), [](user_interface::Startup& startup) { return MovingSphere::start(startup); });
}
