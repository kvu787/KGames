// GameInputDemo's parts, as a library, so that Test/GameInputDemo can test
// what the demo makes of readings, and what input does to its cards, without
// a window, and without GameInput. Main.cpp is the program.

#pragma once

#include "GameInputDemo/DeviceTree.h"
#include "GameInputDemo/Devices.h"
#include "GameInputDemo/Input.h"
#include "GameInputDemo/ScriptedSource.h"

#include <string_view>

namespace game_input_demo {

inline constexpr std::string_view Name = "GameInput Demo";

} // namespace game_input_demo
