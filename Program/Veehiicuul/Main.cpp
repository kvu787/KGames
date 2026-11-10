// Veehiicuul, the racing game and its editors. See README.md.

#include "UserInterface/UserInterface.h"
#include "Veehiicuul/Veehiicuul.h"

int main() {
    user_interface::Options options = user_interface::Options(veehiicuul::Name, veehiicuul::Folder).gamepad();
    return user_interface::run(options, veehiicuul::program::Veehiicuul::start);
}
