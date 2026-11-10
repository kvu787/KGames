#include "UserInterface/Widgets/NumberBox.h"

namespace user_interface::number_box_detail {

bool numeric(char32_t character) {
    return (character >= U'0' && character <= U'9') || character == U'.' || character == U'-' || character == U'+' || character == U'e' || character == U'E';
}

} // namespace user_interface::number_box_detail
