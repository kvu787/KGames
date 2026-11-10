// Turns a frame's GameInput readings (Platform/InputDevices.h) into the
// interface's events (Input/Events.h), in the order that they happened,
// with the window's focus changes between them:
//
// - Keys come from keyboards' readings, which are the keys held down. Shift,
//   Ctrl and Alt are the same keys on the left and the right. Typed text is
//   the keys, translated with Windows' keyboard layout, with Shift, AltGr and
//   dead keys. A held key repeats at Windows' keyboard delay and rate.
// - The mouse's buttons and wheels come from mice's readings, and the pointer
//   is where Windows' cursor is, so that it moves as Windows moves it.
// - The gamepad is the newest reading of any gamepad.
//
// Without focus, the window gets no input, though its devices' state is
// still followed. Keys and buttons held when focus comes count once they're
// let go of, like the click that gives the window focus, and a press while
// the pointer isn't on the window is another window's.

#pragma once

#include "GameInput/GameInput.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Platform/InputDevices.h"
#include "UserInterface/Platform/Window.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <span>
#include <tuple>
#include <utility>
#include <vector>

namespace user_interface {

// What a frame read.
struct FrameInput {
    std::vector<Event> events;
    // The time of the newest reading that the frame took, as the counter's
    // ticks, if it took any: where its input latency starts.
    std::optional<std::int64_t> input_time;
};

// What the window says for a frame: where Windows' cursor is in it, whether
// it's on it, and when it got or lost focus.
struct WindowState {
    Point cursor;
    bool inside = false;
    std::vector<FocusChange> focus_changes;
};

class InputReader {
public:
    // Held keys repeat at Windows' keyboard delay and rate. `gamepad` is
    // whether the gamepad goes to the interface.
    explicit InputReader(bool gamepad);

    // Held keys repeat after the delay, in seconds, at the rate, in repeats a
    // second.
    static InputReader with_repeat(bool gamepad, double delay, double rate);

    InputReader(const InputReader&) = delete;
    InputReader& operator=(const InputReader&) = delete;
    InputReader(InputReader&&) = default;
    InputReader& operator=(InputReader&&) = default;

    // The frame's events, from the devices' readings since the last frame and
    // the window's focus changes, in time order. Captures the mouse while a
    // press that the interface got is held. `devices` is null for a program
    // without GameInput.
    FrameInput frame(const Devices* devices, Window& window, std::span<const FocusChange> focus_changes);

    // Whether a press that the interface got is held.
    bool capturing() const;

    // The frame's events from readings, each with its time on the counter and
    // its kind of input, from the devices that disconnected, and from what the
    // window says, at a time on the counter.
    FrameInput read(std::span<const std::tuple<std::int64_t, game_input::GameInputKind, Reading>> readings, std::span<const size_t> disconnected, const WindowState& window,
                    std::int64_t now);

private:
    // A keyboard's held keys, with their scan codes.
    using HeldKeys = std::vector<std::pair<Key, std::uint32_t>>;

    struct Repeat {
        Key key;
        std::uint32_t scan_code;
        // When it next repeats, as the counter's ticks.
        std::int64_t next;
    };

    InputReader(bool gamepad, double delay, double rate);

    // With the delay and the rate as a pair, as Windows' settings give them.
    InputReader(bool gamepad, std::pair<double, double> repeat);

    // Tells the interface where the pointer is, if that changed: where the
    // cursor is, while it's on the window or a press that the interface got
    // is held, or else that it left.
    void point(Point cursor, bool inside, std::vector<Event>& events);

    void keyboard(const Reading& reading, std::int64_t time, std::vector<Event>& events);

    // The presses and releases since the last time, from what every keyboard
    // holds now, at a time on the counter.
    void update_keys(std::int64_t time, std::vector<Event>& events);

    void mouse(const Reading& reading, Point cursor, bool inside, std::vector<Event>& events);

    // The presses and releases since the last time, from what every mouse
    // holds now.
    void update_buttons(Point cursor, bool inside, std::vector<Event>& events);

    // The text that a key types, translated with Windows' keyboard layout,
    // like WM_CHAR. Alt's keys type nothing, unless Ctrl is held with it,
    // which is AltGr.
    void text(Key key, std::uint32_t scan_code, std::vector<Event>& events) const;

    // Whether the gamepad goes to the interface, as Options::gamepad says.
    bool gamepad;
    bool focused = false;
    // Each keyboard's held keys, with their scan codes, by device.
    std::vector<std::pair<std::optional<size_t>, HeldKeys>> keyboards;
    std::array<bool, 256> keys_down{};
    std::array<std::uint32_t, 256> scan_codes{};
    // Each mouse's last reading, by device.
    std::vector<std::pair<std::optional<size_t>, game_input::GameInputMouseState>> mice;
    // The mouse buttons held, as bits by MouseButton, on any mouse.
    std::uint32_t buttons_down = 0;
    // Buttons whose presses the interface didn't get, whose releases it
    // doesn't either.
    std::uint32_t ignored_buttons = 0;
    Gamepad gamepad_state;
    std::optional<size_t> gamepad_device;
    std::optional<Repeat> repeat;
    std::int64_t repeat_delay;
    std::int64_t repeat_interval;
    // Where the interface last heard the pointer was.
    std::optional<Point> pointer;
};

// The interface's gamepad for a gamepad's state as GameInput reads it.
Gamepad gamepad(const game_input::GameInputGamepadState& state);

} // namespace user_interface
