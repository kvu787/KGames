// The showcase of every widget of Library/UserInterface. See README.md.

#include "UiShowcase/UiShowcase.h"
#include "UserInterface/UserInterface.h"

#include "Folders.h"

int main() {
    user_interface::Options options = user_interface::Options("UiShowcase", folders::Project).gamepad();
    return user_interface::run(options, [](user_interface::Startup& startup) { return ui_showcase::showcase::Showcase(startup); });
}
