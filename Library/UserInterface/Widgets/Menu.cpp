#include "UserInterface/Widgets/Menu.h"

#include "UserInterface/Core/Interface.h"

#include <format>

namespace user_interface {

namespace menu_detail {

std::string child_name(std::string_view item_name) {
    return std::format("{}/Menu", item_name);
}

std::string context_menu_name(std::string_view element) {
    return std::format("{}/ContextMenu", element);
}

Fonts& fonts_of(Ui& ui) {
    return ui.fonts();
}

} // namespace menu_detail

void open_menu(Dispatch<void>& ctx, Id owner, std::string_view name, Point origin) {
    ctx.open_popup_of(owner, name);
    menu_detail::MenuState& state = ctx.memory_of<menu_detail::MenuState>(id(name));
    state.origin = origin;
    state.highlighted = std::nullopt;
    state.open_submenu = std::nullopt;
}

} // namespace user_interface
