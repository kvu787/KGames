// MinimalSceneTest's parts, as a library, so that Test/MinimalSceneTest can
// test them without a window: the sphere's motion, the keys, the card, the
// settings, and frames drawn offscreen. Main.cpp runs the program.

#pragma once

#include "MinimalSceneTest/MovingSphere.h"
#include "MinimalSceneTest/Scene.h"
#include "MinimalSceneTest/Settings.h"
#include "UserInterface/Application/Program.h"

namespace minimal_scene_test {

// How Library/UserInterface runs the test.
user_interface::Options options();

} // namespace minimal_scene_test
