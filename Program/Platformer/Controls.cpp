#include "Platformer/Controls.h"

#include "UserInterface/Core/Interface.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <span>

namespace platformer::controls {

using user_interface::Gamepad;
using user_interface::GamepadButtons;
using user_interface::Input;
using user_interface::Key;
using user_interface::Ui;

Controls Controls::read(const Ui& ui) {
    Gamepad gamepad = ui.gamepad();
    const Input& input = ui.input();
    // Not with Ctrl, which is for shortcuts like Ctrl+S elsewhere.
    bool control = input.modifiers().control;
    auto held = [&](std::span<const Key> keys) {
        return !control && std::ranges::any_of(keys, [&](Key key) { return ui.key_down(key); });
    };
    // A key's press, not its repeats.
    auto key = [&](Key which) {
        return ui.key_pressed(which) && input.key_pressed_once(which) && !control;
    };
    auto pressed = [&](std::uint32_t button, Key other) {
        return input.gamepad_pressed(button) || key(other);
    };
    std::int32_t keys = static_cast<std::int32_t>(held({{Key::Right, Key::F}})) - static_cast<std::int32_t>(held({{Key::Left, Key::S}}));
    double push = 0.0;
    if (keys != 0) {
        push = static_cast<double>(keys);
    } else {
        double stick = static_cast<double>(gamepad.left_stick.x);
        std::int32_t dpad =
            static_cast<std::int32_t>(gamepad.buttons.contains(GamepadButtons::DpadRight)) - static_cast<std::int32_t>(gamepad.buttons.contains(GamepadButtons::DpadLeft));
        if (dpad != 0) {
            push = static_cast<double>(dpad);
        } else if (std::abs(stick) > 0.25) {
            push = std::clamp(stick, -1.0, 1.0);
        } else {
            push = 0.0;
        }
    }
    std::array<Key, 3> jump_keys = {Key::Space, Key::E, Key::Up};
    bool jump_pressed = std::ranges::any_of(jump_keys, key) || input.gamepad_pressed(GamepadButtons::A);
    bool jump_held = held(jump_keys) || gamepad.buttons.contains(GamepadButtons::A);
    auto step = [](bool previous, bool next) {
        return static_cast<std::int32_t>(next) - static_cast<std::int32_t>(previous);
    };
    return Controls{
        .push = push,
        .jump_pressed = jump_pressed,
        .jump_held = jump_held,
        .restart = pressed(GamepadButtons::X, Key::R),
        .level = step(pressed(GamepadButtons::LeftShoulder, Key::PageDown), pressed(GamepadButtons::RightShoulder, Key::PageUp)),
        .toggle_cards = pressed(GamepadButtons::View, Key::F1),
    };
}

} // namespace platformer::controls
