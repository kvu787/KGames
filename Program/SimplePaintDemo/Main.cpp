// SimplePaintDemo. See README.md. It has no console window in any build, as
// SimplePaintDemo.vcxproj doesn't set KGamesConsole.

#include "SimplePaintDemo/Demo.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Application/Run.h"

#include "Folders.h"

int main() {
    return user_interface::run(user_interface::Options("SimplePaintDemo", folders::Project),
                               [](user_interface::Startup& startup) { return simple_paint_demo::demo::Demo::start(startup); });
}
