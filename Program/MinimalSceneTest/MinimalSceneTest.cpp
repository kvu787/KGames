#include "MinimalSceneTest/MinimalSceneTest.h"

// Where this project is, which is the test's folder.
#include "Folders.h"

namespace minimal_scene_test {

user_interface::Options options() {
    return user_interface::Options("MinimalSceneTest", folders::Project);
}

} // namespace minimal_scene_test
