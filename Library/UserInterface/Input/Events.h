// A frame's input: the events since the last frame, from the window, or
// made up by a script or a test, gathered into what happened this frame and
// what's held. Programs and widgets read it the same way whatever made it.
//
// Without window focus there's no input at all. Keys and gamepad buttons
// held when focus comes count only once they're let go of, so that the Alt
// of an Alt+Tab, or a button pressed in another program, does nothing here.

#pragma once

#include "Base/Sum.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Input/Devices.h"

#include <array>
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace user_interface {

struct Event {
    struct PointerMoved {
        Point point;
        bool operator==(const PointerMoved&) const = default;
    };
    // The pointer left the window with no button held.
    struct PointerLeft {
        bool operator==(const PointerLeft&) const = default;
    };
    struct ButtonPressed {
        MouseButton button;
        bool operator==(const ButtonPressed&) const = default;
    };
    struct ButtonReleased {
        MouseButton button;
        bool operator==(const ButtonReleased&) const = default;
    };
    // In notches of the wheel, +y away from the user, +x to the right.
    struct Wheel {
        Point notches;
        bool operator==(const Wheel&) const = default;
    };
    struct KeyPressed {
        Key key;
        bool repeat;
        bool operator==(const KeyPressed&) const = default;
    };
    struct KeyReleased {
        Key key;
        bool operator==(const KeyReleased&) const = default;
    };
    struct Text {
        char32_t character;
        bool operator==(const Text&) const = default;
    };
    // With the keys held as focus came, which count once they're let go of.
    struct FocusGained {
        std::vector<Key> held;
        bool operator==(const FocusGained&) const = default;
    };
    struct FocusLost {
        bool operator==(const FocusLost&) const = default;
    };
    struct Gamepad {
        user_interface::Gamepad reading;
        bool operator==(const Gamepad&) const = default;
    };
    std::variant<PointerMoved, PointerLeft, ButtonPressed, ButtonReleased, Wheel, KeyPressed, KeyReleased, Text, FocusGained, FocusLost, Gamepad> value;

    template <base::AlternativeOf<Event> T> Event(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Event&) const = default;
};

// How a failed check in a test shows an event.
std::string describe_value(const Event& event);

// How close in time and place a press must be to the last one to make a
// double-click: Windows' defaults.
inline constexpr double DoubleClickTime = 0.5;
inline constexpr float DoubleClickDistance = 4.0f;

class Input {
public:
    // Seconds since the program started, and since the last frame.
    double time = 0.0;
    float delta = 0.0f;
    bool focused = false;
    // Focus went away this frame, which ends presses and drags.
    bool focus_lost = false;
    std::optional<Point> pointer;
    // How far the pointer moved this frame.
    Point pointer_delta = Point::Zero;
    // Notches of the wheel this frame.
    Point wheel = Point::Zero;
    // What was typed this frame.
    std::string text;
    Gamepad gamepad;
    Gamepad previous_gamepad;

    // Starts the frame at the time: the last frame's presses are forgotten,
    // and the events apply in order.
    void begin(double now, std::span<const Event> events);

    bool button_down(MouseButton button) const;

    bool button_pressed(MouseButton button) const;

    bool button_released(MouseButton button) const;

    // 1 for this frame's press if it's a click, 2 for a double-click, and on.
    std::uint32_t clicks(MouseButton button) const;

    // Where the button last went down.
    Point press_position(MouseButton button) const;

    bool any_button_down() const;

    bool key_down(Key key) const;

    // Pressed this frame, counting a held key's repeats.
    bool key_pressed(Key key) const;

    // Pressed this frame, not counting repeats.
    bool key_pressed_once(Key key) const;

    bool key_released(Key key) const;

    // This frame's presses, with whether each is a repeat.
    std::span<const std::pair<Key, bool>> keys_pressed() const;

    std::vector<Key> keys_held() const;

    Modifiers modifiers() const;

    // Whether the shortcut's key was pressed this frame, with exactly its
    // modifiers held.
    bool shortcut_pressed(Shortcut shortcut) const;

    // Whether a gamepad button went down this frame.
    bool gamepad_pressed(std::uint32_t button) const;

    bool gamepad_released(std::uint32_t button) const;

private:
    struct ButtonState {
        bool down = false;
        bool pressed = false;
        bool released = false;
        // Presses in a row, close in time and place: 2 for a double-click.
        std::uint32_t clicks = 0;
        double press_time = 0.0;
        Point press_position;
    };

    void apply(const Event& event);

    std::array<ButtonState, 3> buttons{};
    std::array<bool, 256> keys_down{};
    // This frame's presses, and whether each is a held key's repeat.
    std::vector<std::pair<Key, bool>> keys_pressed_;
    std::vector<Key> keys_released;
    std::array<bool, 256> ignored_keys{};
    std::uint32_t ignored_gamepad_buttons = 0;
    bool ignore_next_gamepad = false;
};

} // namespace user_interface
