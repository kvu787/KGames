#include "UserInterface/Widgets/ColorPicker.h"

namespace user_interface::color_picker_detail {

void Hsv::sync(Color color) {
    if (last == color) {
        return;
    }
    auto [new_hue, new_saturation, new_value] = color.to_hsv();
    if (new_saturation > 0.0f && new_value > 0.0f) {
        hue = new_hue;
    }
    if (new_value > 0.0f) {
        saturation = new_saturation;
    }
    value = new_value;
    last = color;
}

bool hex_digit(char32_t character) {
    return (character >= U'0' && character <= U'9') || (character >= U'a' && character <= U'f') || (character >= U'A' && character <= U'F') || character == U'#';
}

FieldSettings settings() {
    return FieldSettings{
        .max_length = 9,
        .accept = hex_digit,
        .leading = SwatchSize + 8.0f,
    };
}

} // namespace user_interface::color_picker_detail
