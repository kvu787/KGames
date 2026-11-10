// The platformer's parts, as a library, so that Test/Platformer can test
// them without a window: the levels, the physics, the events and the game
// they drive, and the progress that persists. Main.cpp is the program.
//
// The cards and the program that Library/UserInterface runs aren't here
// yet: they come with that library's widgets and main loop.

#pragma once

#include "Platformer/Controls.h"
#include "Platformer/Events.h"
#include "Platformer/Game.h"
#include "Platformer/Level.h"
#include "Platformer/Physics.h"
#include "Platformer/Progress.h"

#include <string_view>

namespace platformer {

inline constexpr std::string_view Name = "Platformer";
// The program's folder, whose Save.json holds the progress.
extern const std::string_view Folder;

} // namespace platformer
