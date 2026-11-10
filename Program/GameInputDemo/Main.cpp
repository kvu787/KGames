// GameInputDemo: shows what gamepads, keyboards and mice send through
// GameInput. See README.md.

#include "GameInputDemo/Demo.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Application/Run.h"

#include "Folders.h"

int main() {
    // The demo shows every gamepad's readings, from the library's devices,
    // so they're input that its frames take, like its keys and mice.
    return user_interface::run(user_interface::Options("GameInputDemo", folders::Project).gamepad(),
                               [](user_interface::Startup& startup) { return game_input_demo::Demo(startup); });
}
