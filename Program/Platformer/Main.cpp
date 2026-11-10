// The platformer, as Library/UserInterface's main loop runs it. It has no
// console window in any build: Platformer.vcxproj builds a window's program.

#include "Platformer/Platformer.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Application/Run.h"

#include <utility>

int main() {
    user_interface::Options options = user_interface::Options(platformer::Name, platformer::Folder).gamepad();
    return user_interface::run(std::move(options), [](user_interface::Startup& startup) { return platformer::program::Platformer::start(startup); });
}
