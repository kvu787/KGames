// The showcase's page: an app bar with its icon and title, a menu row under
// it with a menu bar in line with the icon and the cards, five columns of cards
// that show every widget, and a status bar. The cards at the bottom of their
// columns take the 71 pixels that borderless fullscreen adds.
//
// `view` describes the page from the showcase's state, and changes nothing:
// each widget carries the message that the input on it sends.

#pragma once

#include "UiShowcase/Showcase.h"
#include "UserInterface/UserInterface.h"

#include <array>
#include <cstdint>
#include <string_view>

namespace ui_showcase::cards {

// The theme's colors that the combo box offers, and what each is for.
inline constexpr std::array<std::string_view, 5> Roles{"Accent", "Success", "Caution", "Critical", "Attention"};
inline constexpr std::array<std::uint32_t, 8> Palette{0xD13438, 0xCA5010, 0xC19C00, 0x107C10, 0x038387, 0x0078D4, 0x8764B8, 0xE3008C};

user_interface::Tree<showcase::Message> view(const showcase::Showcase& showcase, user_interface::Ui& ui);

} // namespace ui_showcase::cards
