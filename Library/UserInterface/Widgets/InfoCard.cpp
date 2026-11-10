#include "UserInterface/Widgets/InfoCard.h"

namespace user_interface {

bool InfoCard::toggled(const Input& input) {
    return input.key_pressed_once(Key::F1);
}

} // namespace user_interface
