// What input comes from: keys and their names, shortcuts like Ctrl+S, mouse
// buttons, and the gamepad. Scripts name keys and buttons the way these do.

#pragma once

#include "UserInterface/Drawing/Geometry.h"

#include <array>
#include <compare>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace user_interface {

// A key, by its Windows virtual-key code.
struct Key {
    std::uint8_t code;

    static const Key Backspace;
    static const Key Tab;
    static const Key Enter;
    static const Key Shift;
    static const Key Control;
    static const Key Alt;
    static const Key Pause;
    static const Key CapsLock;
    static const Key Escape;
    static const Key Space;
    static const Key PageUp;
    static const Key PageDown;
    static const Key End;
    static const Key Home;
    static const Key Left;
    static const Key Up;
    static const Key Right;
    static const Key Down;
    static const Key Insert;
    static const Key Delete;
    static const Key Digit0;
    static const Key Digit1;
    static const Key Digit2;
    static const Key Digit3;
    static const Key Digit4;
    static const Key Digit5;
    static const Key Digit6;
    static const Key Digit7;
    static const Key Digit8;
    static const Key Digit9;
    static const Key A;
    static const Key B;
    static const Key C;
    static const Key D;
    static const Key E;
    static const Key F;
    static const Key G;
    static const Key H;
    static const Key I;
    static const Key J;
    static const Key K;
    static const Key L;
    static const Key M;
    static const Key N;
    static const Key O;
    static const Key P;
    static const Key Q;
    static const Key R;
    static const Key S;
    static const Key T;
    static const Key U;
    static const Key V;
    static const Key W;
    static const Key X;
    static const Key Y;
    static const Key Z;
    static const Key LeftWindows;
    static const Key RightWindows;
    static const Key Numpad0;
    static const Key F1;
    static const Key F2;
    static const Key F3;
    static const Key F4;
    static const Key F5;
    static const Key F6;
    static const Key F7;
    static const Key F8;
    static const Key F9;
    static const Key F10;
    static const Key F11;
    static const Key F12;
    static const Key LeftShift;
    static const Key RightShift;
    static const Key LeftControl;
    static const Key RightControl;
    static const Key LeftAlt;
    static const Key RightAlt;

    constexpr explicit Key(std::uint8_t code) : code(code) {}

    // The key of a letter or a digit.
    static std::optional<Key> character(char32_t character);

    // Like "A", "F11", "Left" or "Space".
    std::string name() const;

    // A key from its name, without regard to case.
    static std::optional<Key> parse(std::string_view name);

    bool is_modifier() const;

    bool is_function() const;

    // The arrows, Home, End, Page Up and Page Down.
    bool is_navigation() const;

    auto operator<=>(const Key&) const = default;

private:
    // Names that aren't the key's character or F-number.
    static const std::array<std::pair<Key, std::string_view>, 36> Names;
};

inline constexpr Key Key::Backspace{0x08};
inline constexpr Key Key::Tab{0x09};
inline constexpr Key Key::Enter{0x0D};
inline constexpr Key Key::Shift{0x10};
inline constexpr Key Key::Control{0x11};
inline constexpr Key Key::Alt{0x12};
inline constexpr Key Key::Pause{0x13};
inline constexpr Key Key::CapsLock{0x14};
inline constexpr Key Key::Escape{0x1B};
inline constexpr Key Key::Space{0x20};
inline constexpr Key Key::PageUp{0x21};
inline constexpr Key Key::PageDown{0x22};
inline constexpr Key Key::End{0x23};
inline constexpr Key Key::Home{0x24};
inline constexpr Key Key::Left{0x25};
inline constexpr Key Key::Up{0x26};
inline constexpr Key Key::Right{0x27};
inline constexpr Key Key::Down{0x28};
inline constexpr Key Key::Insert{0x2D};
inline constexpr Key Key::Delete{0x2E};
inline constexpr Key Key::Digit0{'0'};
inline constexpr Key Key::Digit1{'1'};
inline constexpr Key Key::Digit2{'2'};
inline constexpr Key Key::Digit3{'3'};
inline constexpr Key Key::Digit4{'4'};
inline constexpr Key Key::Digit5{'5'};
inline constexpr Key Key::Digit6{'6'};
inline constexpr Key Key::Digit7{'7'};
inline constexpr Key Key::Digit8{'8'};
inline constexpr Key Key::Digit9{'9'};
inline constexpr Key Key::A{'A'};
inline constexpr Key Key::B{'B'};
inline constexpr Key Key::C{'C'};
inline constexpr Key Key::D{'D'};
inline constexpr Key Key::E{'E'};
inline constexpr Key Key::F{'F'};
inline constexpr Key Key::G{'G'};
inline constexpr Key Key::H{'H'};
inline constexpr Key Key::I{'I'};
inline constexpr Key Key::J{'J'};
inline constexpr Key Key::K{'K'};
inline constexpr Key Key::L{'L'};
inline constexpr Key Key::M{'M'};
inline constexpr Key Key::N{'N'};
inline constexpr Key Key::O{'O'};
inline constexpr Key Key::P{'P'};
inline constexpr Key Key::Q{'Q'};
inline constexpr Key Key::R{'R'};
inline constexpr Key Key::S{'S'};
inline constexpr Key Key::T{'T'};
inline constexpr Key Key::U{'U'};
inline constexpr Key Key::V{'V'};
inline constexpr Key Key::W{'W'};
inline constexpr Key Key::X{'X'};
inline constexpr Key Key::Y{'Y'};
inline constexpr Key Key::Z{'Z'};
inline constexpr Key Key::LeftWindows{0x5B};
inline constexpr Key Key::RightWindows{0x5C};
inline constexpr Key Key::Numpad0{0x60};
inline constexpr Key Key::F1{0x70};
inline constexpr Key Key::F2{0x71};
inline constexpr Key Key::F3{0x72};
inline constexpr Key Key::F4{0x73};
inline constexpr Key Key::F5{0x74};
inline constexpr Key Key::F6{0x75};
inline constexpr Key Key::F7{0x76};
inline constexpr Key Key::F8{0x77};
inline constexpr Key Key::F9{0x78};
inline constexpr Key Key::F10{0x79};
inline constexpr Key Key::F11{0x7A};
inline constexpr Key Key::F12{0x7B};
inline constexpr Key Key::LeftShift{0xA0};
inline constexpr Key Key::RightShift{0xA1};
inline constexpr Key Key::LeftControl{0xA2};
inline constexpr Key Key::RightControl{0xA3};
inline constexpr Key Key::LeftAlt{0xA4};
inline constexpr Key Key::RightAlt{0xA5};

// How a failed check in a test shows a key.
std::string describe_value(Key key);

struct Modifiers {
    bool control = false;
    bool shift = false;
    bool alt = false;

    static const Modifiers None;
    static const Modifiers Control;
    static const Modifiers Shift;
    static const Modifiers Alt;
    static const Modifiers ControlShift;

    constexpr Modifiers() = default;
    constexpr Modifiers(bool control, bool shift, bool alt) : control(control), shift(shift), alt(alt) {}

    constexpr bool any() const {
        return control || shift || alt;
    }

    bool operator==(const Modifiers&) const = default;
};

inline constexpr Modifiers Modifiers::None = Modifiers(false, false, false);
inline constexpr Modifiers Modifiers::Control = Modifiers(true, false, false);
inline constexpr Modifiers Modifiers::Shift = Modifiers(false, true, false);
inline constexpr Modifiers Modifiers::Alt = Modifiers(false, false, true);
inline constexpr Modifiers Modifiers::ControlShift = Modifiers(true, true, false);

std::string describe_value(Modifiers modifiers);

// A key with the modifiers held for it, like Ctrl+S.
struct Shortcut {
    Modifiers modifiers;
    Key key_;

    constexpr Shortcut(Modifiers modifiers, Key key) : modifiers(modifiers), key_(key) {}

    static constexpr Shortcut key(Key key) {
        return Shortcut(Modifiers::None, key);
    }

    static constexpr Shortcut control(Key key) {
        return Shortcut(Modifiers::Control, key);
    }

    // From its name, like "Ctrl+Shift+S" or "F11".
    static std::optional<Shortcut> parse(std::string_view text);

    // As Windows writes it in menus, like "Ctrl+Shift+S".
    std::string name() const;

    bool operator==(const Shortcut&) const = default;
};

std::string describe_value(Shortcut shortcut);

enum class MouseButton : std::uint8_t {
    Left,
    Right,
    Middle,
};

inline constexpr std::array<MouseButton, 3> AllMouseButtons{MouseButton::Left, MouseButton::Right, MouseButton::Middle};

constexpr size_t index(MouseButton button) {
    return static_cast<size_t>(button);
}

std::string_view name(MouseButton button);

std::optional<MouseButton> mouse_button_parse(std::string_view name);

std::string describe_value(MouseButton button);

// The gamepad's buttons, as bits, which are GameInput's own.
struct GamepadButtons {
    std::uint32_t bits = 0;

    static constexpr std::uint32_t Menu = 1 << 0;
    static constexpr std::uint32_t View = 1 << 1;
    static constexpr std::uint32_t A = 1 << 2;
    static constexpr std::uint32_t B = 1 << 3;
    static constexpr std::uint32_t X = 1 << 4;
    static constexpr std::uint32_t Y = 1 << 5;
    static constexpr std::uint32_t DpadUp = 1 << 6;
    static constexpr std::uint32_t DpadDown = 1 << 7;
    static constexpr std::uint32_t DpadLeft = 1 << 8;
    static constexpr std::uint32_t DpadRight = 1 << 9;
    static constexpr std::uint32_t LeftShoulder = 1 << 10;
    static constexpr std::uint32_t RightShoulder = 1 << 11;
    static constexpr std::uint32_t LeftThumbstick = 1 << 12;
    static constexpr std::uint32_t RightThumbstick = 1 << 13;
    static constexpr std::uint32_t All = (1 << 14) - 1;

    static constexpr std::array<std::pair<std::uint32_t, std::string_view>, 14> Names{{
        {A, "A"},
        {B, "B"},
        {X, "X"},
        {Y, "Y"},
        {LeftShoulder, "LeftShoulder"},
        {RightShoulder, "RightShoulder"},
        {View, "View"},
        {Menu, "Menu"},
        {LeftThumbstick, "LeftThumbstick"},
        {RightThumbstick, "RightThumbstick"},
        {DpadUp, "Up"},
        {DpadDown, "Down"},
        {DpadLeft, "Left"},
        {DpadRight, "Right"},
    }};

    constexpr GamepadButtons() = default;
    constexpr explicit GamepadButtons(std::uint32_t bits) : bits(bits) {}

    constexpr bool contains(std::uint32_t button) const {
        return (bits & button) != 0;
    }

    std::vector<std::string_view> names() const;

    static std::optional<std::uint32_t> parse(std::string_view name);

    bool operator==(const GamepadButtons&) const = default;
};

std::string describe_value(GamepadButtons buttons);

// A gamepad's reading: sticks from -1 to 1 with +y up, and triggers from 0
// to 1, raw, without dead zones.
struct Gamepad {
    bool connected = false;
    Point left_stick;
    Point right_stick;
    float left_trigger = 0.0f;
    float right_trigger = 0.0f;
    GamepadButtons buttons;

    bool operator==(const Gamepad&) const = default;
};

std::string describe_value(const Gamepad& gamepad);

} // namespace user_interface
