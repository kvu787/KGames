#include "UserInterface/Testing/Headless.h"

#include "Base/Windows.h"

namespace user_interface::headless_detail {

Fonts fonts() {
    try {
        return Fonts();
    } catch (const base::WindowsError& error) {
        base::panic(std::format("DirectWrite couldn't start: {}", error.what()));
    }
}

Rect target(const Interface& ui, std::string_view name) {
    if (const Target* found = ui.target(name)) {
        return found->rect;
    }
    std::string names;
    bool first = true;
    for (const Target& target : ui.targets()) {
        if (!target.listed) {
            continue;
        }
        if (!first) {
            names += ", ";
        }
        names += target.name;
        first = false;
    }
    base::panic(std::format("The last frame didn't show {}. It showed: {}", name, names));
}

} // namespace user_interface::headless_detail
