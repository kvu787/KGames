#include "UserInterface/Design/Icons.h"

#include "Base/Panic.h"

namespace user_interface {

#define USER_INTERFACE_ICON_CODE(name, code)                                                                                                                                       \
    case Icon::name:                                                                                                                                                               \
        value = code;                                                                                                                                                              \
        break;

char32_t glyph(Icon icon) {
    std::uint32_t value = 0;
    switch (icon) { USER_INTERFACE_ICONS(USER_INTERFACE_ICON_CODE) }
    // A space for a code that's no character, which a surrogate isn't.
    bool is_character = value < 0xD800 || (value > 0xDFFF && value <= 0x10FFFF);
    return is_character ? static_cast<char32_t>(value) : U' ';
}

#undef USER_INTERFACE_ICON_CODE

#define USER_INTERFACE_ICON_NAME(name, code)                                                                                                                                       \
    case Icon::name:                                                                                                                                                               \
        return #name;

std::string_view name(Icon icon) {
    switch (icon) { USER_INTERFACE_ICONS(USER_INTERFACE_ICON_NAME) }
    base::unreachable();
}

#undef USER_INTERFACE_ICON_NAME

} // namespace user_interface
