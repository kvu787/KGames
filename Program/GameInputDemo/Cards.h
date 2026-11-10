// Describes the demo's cards in fixed pixels from what the demo read, with
// Library/UserInterface's widgets and canvases: the app bar, the status bar,
// and between them three columns of cards. The first has the Gamepad card
// over the Keyboard and Mouse cards, the second the Latest input event card
// over the Devices card, and the third the Event log. The demo supports two
// window setups, maximized and borderless fullscreen (see
// Library/UserInterface's README), whose client areas differ only in
// height, so the cards have fixed places in pixels. The 71 pixels that
// fullscreen adds go to the bottom row of cards and to the log.
//
// The combo box and the Devices card's rows are the library's widgets, which
// read the mouse's buttons from GameInput, and the pointer from where Windows'
// cursor is in the window. Everything else that the cards show comes from the
// devices' readings.

#pragma once

#include "GameInputDemo/Demo.h"
#include "GameInputDemo/Input.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"

#include <expected>
#include <string>
#include <string_view>

namespace game_input_demo {

// The combo box that picks which gamepad the demo listens to. The library
// names the items of its list "Gamepad/0", "Gamepad/1" and so on.
inline constexpr std::string_view GamepadComboBox = "Gamepad";
// The Devices card's list. Its rows are "Devices/0", "Devices/1" and so on,
// in the card's order, each labeled with its device's label. Two devices can
// have the same label, so the rows are named by their place.
inline constexpr std::string_view DeviceRows = "Devices";

// The widths of the first two columns. The log takes the rest of the page.
inline constexpr float GamepadColumn = 1000.0f;
inline constexpr float LatestColumn = 660.0f;
// The stick graphs are square, and the trigger bars as tall.
inline constexpr float StickGraph = 400.0f;

// How wide a device's tooltip may be, so that its every detail fits in the
// window's height.
inline constexpr float DeviceTooltipWidth = 720.0f;

// The app bar, the status bar and the cards, from what the demo read.
user_interface::Tree<Message> view(user_interface::Ui& ui, const std::expected<Input, std::string>& input);

} // namespace game_input_demo
