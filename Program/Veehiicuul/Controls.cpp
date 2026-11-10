#include "Veehiicuul/Controls.h"

#include "UserInterface/Core/Interface.h"
#include "Veehiicuul/Driving.h"

#include <cmath>

namespace veehiicuul::controls {

using driving::DeadZone;
using user_interface::Gamepad;
using user_interface::GamepadButtons;
using user_interface::Input;
using user_interface::Key;
using user_interface::Ui;

namespace {

// The left stick zooms only past this, so that resting a thumb on it while
// holding RB doesn't.
constexpr DeadZone ZoomDeadZone{
    .inner = 0.05,
    .outer = 0.95,
};

} // namespace

Controls Controls::read(const Ui& ui, double wheel_notches) {
    Gamepad gamepad = ui.gamepad();
    const Input& input = ui.input();
    // Not with Ctrl, which is for the editors' shortcuts, like Ctrl+S,
    // which mustn't push the car.
    bool control = input.modifiers().control;
    auto held = [&](Key first, Key second) {
        return !control && (ui.key_down(first) || ui.key_down(second));
    };
    // A key's press, not its repeats.
    auto key = [&](Key pressed_key) {
        return ui.key_pressed(pressed_key) && input.key_pressed_once(pressed_key) && !control;
    };
    auto pressed = [&](std::uint32_t button, Key other) {
        return input.gamepad_pressed(button) || key(other);
    };
    // Keys push all the way, and a diagonal as far as a straight push.
    std::array<std::int32_t, 2> keys{
        static_cast<std::int32_t>(held(Key::Right, Key::D)) - static_cast<std::int32_t>(held(Key::Left, Key::A)),
        static_cast<std::int32_t>(held(Key::Up, Key::W)) - static_cast<std::int32_t>(held(Key::Down, Key::S)),
    };
    std::array<double, 2> pushed{};
    if (keys == std::array<std::int32_t, 2>{0, 0}) {
        pushed = {static_cast<double>(gamepad.right_stick.x), static_cast<double>(gamepad.right_stick.y)};
    } else {
        double length = std::sqrt(static_cast<double>(keys[0] * keys[0] + keys[1] * keys[1]));
        pushed = {static_cast<double>(keys[0]) / length, static_cast<double>(keys[1]) / length};
    }
    double braking = ui.key_down(Key::Space) ? 1.0 : static_cast<double>(gamepad.left_trigger);
    double zooming = gamepad.buttons.contains(GamepadButtons::RightShoulder) ? ZoomDeadZone.apply(static_cast<double>(gamepad.left_stick.y)) : 0.0;
    auto step = [](bool previous, bool next) {
        return static_cast<std::int32_t>(next) - static_cast<std::int32_t>(previous);
    };
    return Controls{
        .push = pushed,
        .brake = braking,
        .vehicle = step(pressed(GamepadButtons::DpadLeft, Key::Q), pressed(GamepadButtons::DpadRight, Key::E)),
        .scene = step(pressed(GamepadButtons::DpadDown, Key::PageDown), pressed(GamepadButtons::DpadUp, Key::PageUp)),
        .restart = pressed(GamepadButtons::X, Key::R),
        .toggle_camera = pressed(GamepadButtons::Menu, Key::C),
        .zoom = zooming,
        .notches = wheel_notches,
        .default_zoom = pressed(GamepadButtons::Y, Key::Home),
        .toggle_card = pressed(GamepadButtons::View, Key::F1),
    };
}

} // namespace veehiicuul::controls
