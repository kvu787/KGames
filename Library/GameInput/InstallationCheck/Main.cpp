// Fails the build unless GameInput 4.0.306's installed runtime DLL is in
// System32, which every program needs to start (see ../Installation.h).
// The user installs it, once. The build never does.
//
// GameInput.vcxproj runs this before it compiles. It prints what's wrong, and
// how to install GameInput, on one line, and exits with 1, and the build
// fails with that line.

#include "Base/Panic.h"
#include "GameInput/Installation.h"

#include <print>

int main() {
    base::fail_quietly();
    std::expected<std::filesystem::path, std::string> installed = game_input::installation::installed_runtime();
    if (!installed) {
        std::println("{}", installed.error());
        return 1;
    }
    return 0;
}
