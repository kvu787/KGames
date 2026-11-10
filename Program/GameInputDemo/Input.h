// Keeps what the cards show from gamepads, keyboards and mice: the latest
// state of each, the connected devices, and a log of input events. The
// readings come from an InputSource (see Devices.h): GameInput, or a script
// or a test.
//
// Each reading is a snapshot of one device's whole state, so this compares
// each one with the same device's reading before it to say what changed. It
// keeps every device's last reading itself for that, because GameInput's
// GetPreviousReading fails once that reading has left its short history,
// like after a pause in input. Then a held key would show as pressed again.

#pragma once

#include "Base/Sum.h"
#include "Base/VecDeque.h"
#include "GameInput/GameInput.h"
#include "GameInputDemo/Devices.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <limits>
#include <memory>
#include <optional>
#include <ranges>
#include <span>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <variant>
#include <vector>

namespace game_input_demo {

enum class Source : std::uint8_t {
    Gamepad,
    Keyboard,
    Mouse,
    Device,
    Demo,
};

std::string_view name(Source source);

class LogEntry {
public:
    double time;
    Source source;
    std::string text;
    // How many readings were merged into this entry. See Input::log.
    std::uint32_t count;
    // The entry as the log card shows it, made when the entry changes
    // instead of every frame: the time, the source from `source_at`, and the
    // text and count from `text_at`.
    std::string line;
    size_t source_at;
    size_t text_at;

private:
    friend class Input;

    LogEntry(double time, Source source, std::string text, std::optional<std::pair<Source, size_t>> merge_key);

    void format_line();

    std::optional<std::pair<Source, size_t>> merge_key;
};

// The most recent reading that changed something, described in full.
struct LatestEvent {
    std::uint64_t number;
    double time;
    Source source;
    std::string device;
    std::string kinds;
    std::vector<std::string> changes;
    std::vector<std::string> state;
    // GameInput's own timestamp, in microseconds.
    std::uint64_t timestamp;
    // Since the same device's previous reading. Nothing for the first reading
    // of the device that the demo has seen, so the changes are from an idle
    // state.
    std::optional<double> interval;
};

// Counts readings per second.
class RateCounter {
public:
    std::uint32_t per_second = 0;

private:
    friend class Input;

    void add(double now);

    void tick(double now);

    double window_start = 0.0;
    std::uint32_t count = 0;
};

struct GamepadView {
    // The device's ID. See Device.
    std::optional<size_t> device;
    std::optional<game_input::GameInputGamepadButtons> layout;
    game_input::GameInputRumbleMotors rumble_motors{};
    game_input::GameInputGamepadState state{};
    RateCounter rate;
};

// Which gamepad the demo listens to, if any, chosen in the Gamepad card's
// combo box. The others are ignored, along with any keyboard or mouse input
// they have: a GameSir in Xbox 360 mode is one device with a gamepad, a
// keyboard and a mouse.
struct Listen {
    // Kept with its device's data, so that it's listened to again if it
    // reconnects, and named while it's gone.
    struct Gamepad {
        Device device;
    };
    struct NoGamepad {};
    std::variant<Gamepad, NoGamepad> value;

    template <base::AlternativeOf<Listen> T> Listen(T&& alternative) : value(std::forward<T>(alternative)) {}

    // Like "Xbox Controller (045E:0B13)", or "None".
    std::string_view label() const;

    // The same choice, whatever the device's data says now.
    bool operator==(const Listen& other) const;
};

// How a failed check in a test shows a choice: which it is, and its
// gamepad's ID and label.
std::string describe_value(const Listen& listen);

// The Keyboard and Mouse cards each show one device: the last one whose
// reading changed their kind of input. A reading can have several kinds, like
// one from a keyboard that is also a mouse, and a reading that doesn't change
// a card's kind leaves the card showing the device it was showing. The rate
// counts the readings that the card took.
struct KeyboardView {
    std::optional<size_t> device;
    std::string label;
    std::vector<game_input::GameInputKeyState> keys;
    // Each key's name and key_details, in the same order, made when the keys
    // change instead of every frame, as GetKeyNameTextW and the formatting
    // take a third of a microsecond a key.
    std::vector<std::pair<std::string, std::string>> descriptions;
    RateCounter rate;
};

struct MouseView {
    std::optional<size_t> device;
    std::string label;
    game_input::GameInputMouseState state{};
    std::array<std::int64_t, 2> delta{};
    // Relative motion added up and kept inside -1..1, for the motion pad.
    std::array<float, 2> pad{};
    std::optional<std::uint32_t> sample_rate;
    RateCounter rate;
};

// Input's own types, which nothing else uses.
namespace input_detail {

// How far a gamepad's left and right sticks last were from their centers,
// and the rumble last sent to it.
struct Rumble {
    size_t device;
    std::array<float, 2> sticks;
    game_input::GameInputRumbleParams sent;
};

// One device's keys or buttons of one kind of input that were held when the
// window got focus, or while it didn't have it, and haven't been let go of
// since. The demo ignores them until they are: it doesn't show them as held,
// and doesn't say they were pressed or let go of. So the click that gives the
// window focus, or the Alt of the Alt+Tab that switches to it, does only
// that, and a shortcut held down in another window doesn't run here.
struct Masked {
    // Gamepad or mouse buttons.
    std::int32_t buttons = 0;
    // Keyboard keys, by scan code.
    std::vector<std::uint32_t> keys_;

    // Everything held in the reading, of this kind of input.
    static Masked held(game_input::GameInputKind kind, const Reading& reading);

    // Of these, what's still held in the reading.
    Masked still_held(game_input::GameInputKind kind, const Reading& reading) const;

    // The state without what's masked.
    game_input::GameInputGamepadState gamepad(game_input::GameInputGamepadState state) const;

    game_input::GameInputMouseState mouse(game_input::GameInputMouseState state) const;

    std::vector<game_input::GameInputKeyState> keys(std::span<const game_input::GameInputKeyState> held_keys) const;
};

// A device's newest reading with one kind of input, handled or not, which
// its next one is compared with.
struct LastReading {
    game_input::GameInputKind kind;
    Reading reading;
    Masked masked;
};

} // namespace input_detail

class Input {
public:
    Input(std::unique_ptr<InputSource> input_source, float rumble_dead_zone);

    // Stops the motors that it runs.
    ~Input();
    Input(const Input&) = delete;
    Input& operator=(const Input&) = delete;
    Input(Input&& other) noexcept;
    Input& operator=(Input&&) = delete;

    // The GameInput runtime that was loaded, for the status bar, or null when
    // the input comes from a script.
    const game_input::Runtime* runtime() const;

    // The source, if it's a script's or a test's.
    ScriptedSource* scripted();

    // `window_focused` is whether the window has focus. Input is only handled
    // while it does, see take_reading. A minimized window doesn't have it, so
    // the demo calls this with false on the frame that restores the window,
    // and restoring it counts as getting focus.
    void update(bool window_focused);

    // The connected gamepads, which the combo box offers besides none.
    auto gamepads() const {
        return devices | std::views::filter(&Device::is_gamepad);
    }

    // Whether the demo handles the device's readings.
    bool listens_to(const Device& info) const;

    // For a choice picked in the combo box. The demo keeps to it from then on.
    void pick_listen(Listen choice);

    // Brings the log's clock up to a moment, in seconds, for an entry made
    // before the frame's readings, like the combo box's pick.
    void align_clock(double seconds);

    // What the cards show, for scripted runs to save and check.
    json::Value state() const;

private:
    std::unique_ptr<InputSource> source;
    // The source's timestamps are in microseconds. Shown relative to this.
    std::uint64_t start_timestamp;

public:
    double now = 0.0;
    std::vector<Device> devices;
    GamepadView gamepad;
    KeyboardView keyboard;
    MouseView mouse;
    std::optional<LatestEvent> latest;
    base::VecDeque<LogEntry> log_;
    std::uint64_t reading_count = 0;
    bool rumble_enabled = false;
    // The rumble test's inner dead zone for both sticks, as a fraction of
    // full deflection, from the settings. GameInput reports raw stick values,
    // so applying a dead zone is up to the game. The demo applies this one
    // only to the rumble test. Everything else, from the cards to the log,
    // shows the raw values, and the stick graphs draw this as a ring.
    float rumble_dead_zone;

private:
    // One for each gamepad listened to that has sent a reading.
    std::vector<input_detail::Rumble> rumbles;
    // For each device and kind of input it has sent a reading of. A device's
    // are dropped when it disconnects, so one that reconnects starts from an
    // idle state.
    std::vector<input_detail::LastReading> last_readings;

public:
    // Changed with pick_listen, or by listen_by_default until then.
    Listen listen = Listen::NoGamepad{};
    // Whether listen was picked in the combo box.
    bool listen_picked = false;

private:
    // What a reading changed, the device's whole state, and whether only
    // analog motion changed.
    using Handled = std::tuple<std::vector<std::string>, std::vector<std::string>, bool>;

    double seconds(std::uint64_t timestamp) const;

    void handle_device_event(game_input::GameInputDeviceEvent event, Device device);

    // Shows another gamepad in the card, starting from an idle state.
    void view_gamepad(std::optional<size_t> device);

    // Until a choice is picked, the demo listens to a connected gamepad if
    // there is one: the one it has while that's connected, or else the first.
    void listen_by_default();

    void set_listen(Listen choice);

    // A gamepad's state from its last reading, without what's masked, to
    // show it as soon as it's listened to. Nothing without focus, or when
    // that reading came before the window got focus, as that's input without
    // it.
    std::optional<game_input::GameInputGamepadState> gamepad_state(size_t device) const;

    // Handles a reading if the window had focus for it, and keeps it as its
    // device's last reading of this kind, which the next one is compared with.
    void take_reading(Source from, game_input::GameInputKind kind, Reading reading);

    // `masked_before` is what was masked for the previous reading, and
    // `masked` what is for this one.
    void handle_reading(Source from, game_input::GameInputKind kind, const Reading& reading, const Reading* previous, const input_detail::Masked& masked_before,
                        const input_detail::Masked& masked);

    // `old` is the state of the device's previous reading, or idle.
    Handled handle_gamepad(double time, game_input::GameInputGamepadState state, game_input::GameInputGamepadState old, std::optional<size_t> device);

    // `old` is the keys of the device's previous reading, or none.
    Handled handle_keyboard(double time, std::vector<game_input::GameInputKeyState> keys, std::vector<game_input::GameInputKeyState> old, std::optional<size_t> device,
                            std::string_view label);

    // `old` is the state of the device's previous reading, if there was one.
    Handled handle_mouse(double time, game_input::GameInputMouseState state, std::optional<game_input::GameInputMouseState> old, std::optional<size_t> device,
                         std::string_view label);

    void log(double time, Source from, std::string text, std::optional<std::pair<Source, size_t>> merge_key);

    void toggle_rumble();

    // With the rumble test on, a gamepad's left stick drives its
    // low-frequency (left grip) motor and its right stick the high-frequency
    // (right grip) one. Each motor is off while its stick is inside its dead
    // zone, which keeps a stick at rest from running it with its noise. Past
    // the dead zone, the motor's strength grows linearly with how far the
    // stick is pushed, up to full at the edge. Only those two motors, the
    // standard rumble that Xbox 360, PlayStation 4 and 5 and most other
    // gamepads with rumble have. Xbox One and Series gamepads' trigger motors
    // stay off.
    void update_rumble();

    // Stops every motor, for while the window doesn't have focus, or is
    // minimized. Otherwise they would keep running at their last strength,
    // as the sticks aren't read then. The sticks count as at rest from then
    // on, so the motors start again only once a stick moves with focus.
    void stop_rumble();

    // Virtual keys of the demo's shortcuts, pressed in the reading being handled.
    std::vector<std::uint8_t> shortcuts;
    // Whether the window had focus at the last update, so input was handled.
    bool focused = false;
    // When the window last got focus, in the source's clock. Readings from
    // before then are input without focus too, like the click that gave it.
    std::uint64_t focus_timestamp = std::numeric_limits<std::uint64_t>::max();
};

inline constexpr std::array<std::pair<game_input::GameInputGamepadButtons, std::string_view>, 30> GamepadButtons{{
    {game_input::GameInputGamepadA, "A"},
    {game_input::GameInputGamepadB, "B"},
    {game_input::GameInputGamepadX, "X"},
    {game_input::GameInputGamepadY, "Y"},
    {game_input::GameInputGamepadC, "C"},
    {game_input::GameInputGamepadZ, "Z"},
    {game_input::GameInputGamepadView, "View"},
    {game_input::GameInputGamepadMenu, "Menu"},
    {game_input::GameInputGamepadLeftShoulder, "LB"},
    {game_input::GameInputGamepadRightShoulder, "RB"},
    {game_input::GameInputGamepadLeftTriggerButton, "LT"},
    {game_input::GameInputGamepadRightTriggerButton, "RT"},
    {game_input::GameInputGamepadLeftThumbstick, "LS"},
    {game_input::GameInputGamepadRightThumbstick, "RS"},
    {game_input::GameInputGamepadDPadUp, "Up"},
    {game_input::GameInputGamepadDPadDown, "Down"},
    {game_input::GameInputGamepadDPadLeft, "Left"},
    {game_input::GameInputGamepadDPadRight, "Right"},
    {game_input::GameInputGamepadLeftThumbstickUp, "LS Up"},
    {game_input::GameInputGamepadLeftThumbstickDown, "LS Down"},
    {game_input::GameInputGamepadLeftThumbstickLeft, "LS Left"},
    {game_input::GameInputGamepadLeftThumbstickRight, "LS Right"},
    {game_input::GameInputGamepadRightThumbstickUp, "RS Up"},
    {game_input::GameInputGamepadRightThumbstickDown, "RS Down"},
    {game_input::GameInputGamepadRightThumbstickLeft, "RS Left"},
    {game_input::GameInputGamepadRightThumbstickRight, "RS Right"},
    {game_input::GameInputGamepadPaddleLeft1, "P1"},
    {game_input::GameInputGamepadPaddleRight1, "P2"},
    {game_input::GameInputGamepadPaddleLeft2, "P3"},
    {game_input::GameInputGamepadPaddleRight2, "P4"},
}};

inline constexpr std::array<std::pair<game_input::GameInputMouseButtons, std::string_view>, 7> MouseButtons{{
    {game_input::GameInputMouseLeftButton, "Left"},
    {game_input::GameInputMouseRightButton, "Right"},
    {game_input::GameInputMouseMiddleButton, "Middle"},
    {game_input::GameInputMouseButton4, "Back"},
    {game_input::GameInputMouseButton5, "Forward"},
    {game_input::GameInputMouseWheelTiltLeft, "Tilt left"},
    {game_input::GameInputMouseWheelTiltRight, "Tilt right"},
}};

std::string kind_names(game_input::GameInputKind kinds);

// The key's name in the current keyboard layout, like "Left Shift" or "Num 7".
std::string key_name(const game_input::GameInputKeyState& key);

// What the Keyboard card says about a key after its name: its codes, and the
// character it types, if any. DirectWrite draws a character that Cascadia
// Mono doesn't have with a font that does.
std::string key_details(const game_input::GameInputKeyState& key);

} // namespace game_input_demo
