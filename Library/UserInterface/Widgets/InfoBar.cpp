#include "UserInterface/Widgets/InfoBar.h"

#include "Base/Text.h"
#include "UserInterface/Core/Interface.h"

#include <cstddef>

namespace user_interface {

Fonts& info_bar_detail::fonts_of(Ui& ui) {
    return ui.fonts();
}

std::string_view info_bar_detail::trim_end(std::string_view text) {
    // What's left without the whitespace at both ends is in the text, and
    // ends where the text without the whitespace at its end does.
    std::string_view trimmed = base::trim(text);
    if (trimmed.empty()) {
        return text.substr(0, 0);
    }
    return text.substr(0, static_cast<size_t>(trimmed.data() + trimmed.size() - text.data()));
}

} // namespace user_interface
