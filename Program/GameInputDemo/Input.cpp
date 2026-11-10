#include "GameInputDemo/Input.h"

#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Base/Windows.h"

#include <algorithm>
#include <cmath>
#include <format>

#pragma comment(lib, "user32.lib")

namespace game_input_demo {

using namespace game_input;
using input_detail::LastReading;
using input_detail::Masked;
using input_detail::Rumble;

namespace {

constexpr size_t LogCapacity = 200;
// Virtual-key codes of the demo's own shortcuts. F10, which switches vertical
// sync, and F11, which switches the window to and from fullscreen, are
// Library/UserInterface's, which reads them from the same readings.
constexpr std::uint8_t KeyToggleRumble = 0x70; // F1
constexpr std::uint8_t KeyClearLog = 0x71;     // F2

constexpr std::array<std::pair<Source, GameInputKind>, 3> ReadingKinds{{
    {Source::Gamepad, GameInputKindGamepad},
    {Source::Keyboard, GameInputKindKeyboard},
    {Source::Mouse, GameInputKindMouse},
}};

constexpr std::array<std::pair<GameInputKind, std::string_view>, 14> KindNames{{
    {GameInputKindGamepad, "gamepad"},
    {GameInputKindKeyboard, "keyboard"},
    {GameInputKindMouse, "mouse"},
    {GameInputKindArcadeStick, "arcade stick"},
    {GameInputKindFlightStick, "flight stick"},
    {GameInputKindRacingWheel, "racing wheel"},
    {GameInputKindControllerAxis, "axes"},
    {GameInputKindControllerButton, "buttons"},
    {GameInputKindControllerSwitch, "switches"},
    {GameInputKindSensors, "sensors"},
    {GameInputKindTouch, "touch"},
    {GameInputKindLighting, "lighting"},
    {GameInputKindSystemButton, "system buttons"},
    {GameInputKindRawDeviceReport, "raw reports"},
}};

// The buttons held in the reading, of this kind of input. Keys aren't
// buttons.
std::int32_t buttons(GameInputKind kind, const Reading& reading) {
    if (kind == GameInputKindGamepad) {
        return reading.gamepad.buttons;
    } else if (kind == GameInputKindMouse) {
        return reading.mouse.buttons;
    } else {
        return 0;
    }
}

// Whether the choice listens to the device.
bool includes(const Listen& listen, const Device& device) {
    return base::match(
        listen, [&](const Listen::Gamepad& chosen) { return chosen.device.id == device.id || !device.is_gamepad(); },
        [&](const Listen::NoGamepad&) { return !device.is_gamepad(); });
}

// How far a stick is from its center: 0 at rest, 1 at its edge. Toward the
// corners, many sticks read a little past 1, so that's cut off at 1.
float deflection(float x, float y) {
    return std::fmin(std::hypot(x, y), 1.0f);
}

// A stick's deflection rescaled to 0 at the dead zone's edge and 1 at the
// stick's, and 0 inside the dead zone, so there's no jump where it starts.
float past_dead_zone(float deflection, float dead_zone) {
    return std::fmax((deflection - dead_zone) / (1.0f - dead_zone), 0.0f);
}

std::string pair(float x, float y) {
    return std::format("({:+.3f}, {:+.3f})", x, y);
}

} // namespace

std::string_view name(Source source) {
    switch (source) {
    case Source::Gamepad:
        return "Gamepad";
    case Source::Keyboard:
        return "Keyboard";
    case Source::Mouse:
        return "Mouse";
    case Source::Device:
        return "Device";
    case Source::Demo:
        return "Demo";
    }
    base::unreachable();
}

LogEntry::LogEntry(double time, Source source, std::string text, std::optional<std::pair<Source, size_t>> merge_key) :
    time(time), source(source), text(std::move(text)), count(1), source_at(0), text_at(0), merge_key(merge_key) {}

void LogEntry::format_line() {
    std::string time_text = std::format("{:>8.3f}  ", time);
    std::string source_text = std::format("{:<9}", name(source));
    source_at = time_text.size();
    text_at = time_text.size() + source_text.size();
    line = std::move(time_text);
    line += source_text;
    line += text;
    if (count > 1) {
        line += std::format("  (x{})", count);
    }
}

void RateCounter::add(double now) {
    tick(now);
    count += 1;
}

void RateCounter::tick(double now) {
    if (now - window_start >= 1.0) {
        // A second with no readings at all reads as 0, not the last rate.
        per_second = now - window_start >= 2.0 ? 0 : count;
        count = 0;
        window_start = now;
    }
}

namespace input_detail {

Masked Masked::held(GameInputKind kind, const Reading& reading) {
    Masked masked;
    masked.buttons = game_input_demo::buttons(kind, reading);
    if (kind == GameInputKindKeyboard) {
        masked.keys_.reserve(reading.keys.size());
        for (const GameInputKeyState& key : reading.keys) {
            masked.keys_.push_back(key.scanCode);
        }
    }
    return masked;
}

Masked Masked::still_held(GameInputKind kind, const Reading& reading) const {
    Masked masked;
    masked.buttons = buttons & game_input_demo::buttons(kind, reading);
    for (std::uint32_t code : keys_) {
        if (std::ranges::contains(reading.keys, code, &GameInputKeyState::scanCode)) {
            masked.keys_.push_back(code);
        }
    }
    return masked;
}

GameInputGamepadState Masked::gamepad(GameInputGamepadState state) const {
    state.buttons = static_cast<GameInputGamepadButtons>(state.buttons & ~buttons);
    return state;
}

GameInputMouseState Masked::mouse(GameInputMouseState state) const {
    state.buttons = static_cast<GameInputMouseButtons>(state.buttons & ~buttons);
    return state;
}

std::vector<GameInputKeyState> Masked::keys(std::span<const GameInputKeyState> held_keys) const {
    std::vector<GameInputKeyState> kept;
    for (const GameInputKeyState& key : held_keys) {
        if (!std::ranges::contains(keys_, key.scanCode)) {
            kept.push_back(key);
        }
    }
    return kept;
}

} // namespace input_detail

std::string_view Listen::label() const {
    if (const Gamepad* chosen = base::as<Gamepad>(*this)) {
        return chosen->device.label;
    }
    return "None";
}

bool Listen::operator==(const Listen& other) const {
    const Gamepad* chosen = base::as<Gamepad>(*this);
    const Gamepad* other_chosen = base::as<Gamepad>(other);
    if (chosen != nullptr && other_chosen != nullptr) {
        return chosen->device.id == other_chosen->device.id;
    }
    return chosen == nullptr && other_chosen == nullptr;
}

std::string describe_value(const Listen& listen) {
    if (const Listen::Gamepad* chosen = base::as<Listen::Gamepad>(listen)) {
        return std::format("Gamepad({}, {})", chosen->device.id, base::debug(chosen->device.label));
    }
    return "NoGamepad";
}

Input::Input(std::unique_ptr<InputSource> input_source, float rumble_dead_zone) :
    source(std::move(input_source)), start_timestamp(source->now()), rumble_dead_zone(rumble_dead_zone) {}

Input::~Input() {
    // An Input that was moved from has no source, and runs no motors.
    if (source == nullptr) {
        return;
    }
    for (const Rumble& rumble : rumbles) {
        source->set_rumble(rumble.device, nullptr);
    }
}

Input::Input(Input&& other) noexcept = default;

const Runtime* Input::runtime() const {
    return source->runtime();
}

ScriptedSource* Input::scripted() {
    return source->as_scripted();
}

double Input::seconds(std::uint64_t timestamp) const {
    return static_cast<double>(base::saturating_sub(timestamp, start_timestamp)) / 1'000'000.0;
}

void Input::update(bool window_focused) {
    if (focused && !window_focused) {
        // Keys, buttons and sticks let go of after the window lost focus
        // are never seen, so stop showing them as held.
        keyboard.keys.clear();
        keyboard.descriptions.clear();
        mouse.state.buttons = GameInputMouseNone;
        std::optional<size_t> device = gamepad.device;
        view_gamepad(device);
        for (Rumble& rumble : rumbles) {
            rumble.sticks = {0.0f, 0.0f};
        }
    }
    std::uint64_t timestamp = source->now();
    if (!focused && window_focused) {
        focus_timestamp = timestamp;
        // What's held now was pressed without focus.
        for (LastReading& last : last_readings) {
            last.masked = Masked::held(last.kind, last.reading);
        }
    }
    focused = window_focused;
    now = seconds(timestamp);
    for (auto& [event, device] : source->device_events()) {
        handle_device_event(event, std::move(device));
    }
    listen_by_default();

    // Gather every new reading of each kind, then handle them all in
    // timestamp order, so the log interleaves devices correctly.
    std::vector<std::tuple<Source, GameInputKind, Reading>> readings;
    for (const auto& [from, kind] : ReadingKinds) {
        for (Reading& reading : source->readings(kind)) {
            readings.emplace_back(from, kind, std::move(reading));
        }
    }
    std::ranges::stable_sort(readings, {}, [](const std::tuple<Source, GameInputKind, Reading>& item) { return std::get<2>(item).timestamp; });
    for (auto& [from, kind, reading] : readings) {
        take_reading(from, kind, std::move(reading));
    }

    gamepad.rate.tick(now);
    keyboard.rate.tick(now);
    mouse.rate.tick(now);
    // Without focus, the sticks don't drive the motors.
    if (window_focused) {
        update_rumble();
    } else {
        stop_rumble();
    }
}

void Input::handle_device_event(GameInputDeviceEvent event, Device device) {
    auto existing = std::ranges::find(devices, device.id, &Device::id);
    std::string label = device.label;
    std::string text;
    switch (event) {
    case GameInputDeviceEventConnected: {
        text = std::format("Connected: {} [{}]", device.label, device.kinds());
        if (existing != devices.end()) {
            *existing = std::move(device);
        } else {
            devices.push_back(std::move(device));
        }
        // Gamepads first, then keyboards, then mice.
        std::ranges::stable_sort(devices, {}, [](const Device& info) {
            GameInputKind kinds = info.supported_input;
            return std::pair((kinds & GameInputKindGamepad) == 0, (kinds & GameInputKindKeyboard) == 0);
        });
        break;
    }
    case GameInputDeviceEventDisconnected: {
        if (existing != devices.end()) {
            devices.erase(existing);
        }
        if (gamepad.device == device.id) {
            gamepad = GamepadView{};
        }
        // Its keys and buttons let go of as it went are never seen.
        if (keyboard.device == device.id) {
            keyboard = KeyboardView{};
        }
        if (mouse.device == device.id) {
            mouse = MouseView{};
        }
        std::erase_if(rumbles, [&](const Rumble& rumble) { return rumble.device == device.id; });
        std::erase_if(last_readings, [&](const LastReading& last) { return last.reading.device == device.id; });
        text = std::format("Disconnected: {}", label);
        break;
    }
    // The source describes the device again, with its new layout.
    case GameInputDeviceEventKeyboardLayoutUpdated:
        if (existing != devices.end()) {
            *existing = std::move(device);
        }
        text = std::format("Keyboard layout changed: {}", label);
        break;
    case GameInputDeviceEventHapticInfoUpdated:
        text = std::format("Haptic info changed: {}", label);
        break;
    case GameInputDeviceEventXInputIndexUpdated:
        if (existing != devices.end()) {
            *existing = std::move(device);
        }
        text = std::format("XInput index changed: {}", label);
        break;
    default:
        text = std::format("Device event {}: {}", static_cast<int>(event), label);
        break;
    }
    log(now, Source::Device, std::move(text), std::nullopt);
}

void Input::view_gamepad(std::optional<size_t> device) {
    auto found = std::ranges::find_if(devices, [&](const Device& candidate) { return device == candidate.id; });
    const Device* info = found != devices.end() ? &*found : nullptr;
    gamepad = GamepadView{
        .device = device,
        .layout = info != nullptr && info->gamepad ? std::optional(info->gamepad->supportedLayout) : std::nullopt,
        .rumble_motors = info != nullptr ? info->rumble_motors : GameInputRumbleNone,
    };
}

bool Input::listens_to(const Device& info) const {
    return includes(listen, info);
}

void Input::pick_listen(Listen choice) {
    listen_picked = true;
    set_listen(std::move(choice));
}

void Input::listen_by_default() {
    if (listen_picked) {
        return;
    }
    if (const Listen::Gamepad* chosen = base::as<Listen::Gamepad>(listen); chosen != nullptr && std::ranges::contains(gamepads(), chosen->device.id, &Device::id)) {
        return;
    }
    auto connected = gamepads();
    auto first = connected.begin();
    if (first != connected.end()) {
        set_listen(Listen::Gamepad{*first});
    }
}

void Input::align_clock(double seconds) {
    now = std::max(now, seconds);
}

void Input::set_listen(Listen choice) {
    if (choice == listen) {
        return;
    }
    listen = std::move(choice);
    // Gamepads no longer listened to stop rumbling. Only gamepads rumble.
    const Listen::Gamepad* chosen = base::as<Listen::Gamepad>(listen);
    std::optional<size_t> listened = chosen != nullptr ? std::optional(chosen->device.id) : std::nullopt;
    std::erase_if(rumbles, [&](const Rumble& rumble) {
        bool keep = listened == rumble.device;
        if (!keep) {
            source->set_rumble(rumble.device, nullptr);
        }
        return !keep;
    });
    std::string text;
    if (chosen != nullptr) {
        const Device& device = chosen->device;
        if (gamepad.device != device.id) {
            view_gamepad(device.id);
            if (std::optional<GameInputGamepadState> state = gamepad_state(device.id)) {
                gamepad.state = *state;
            }
        }
        text = std::format("Listening to {} only", device.label);
    } else {
        gamepad = GamepadView{};
        text = "Ignoring all gamepads";
    }
    // The Keyboard and Mouse cards stop showing a gamepad that's also a
    // keyboard or a mouse, once it isn't listened to, as its keys and
    // buttons let go of after that are never seen.
    auto ignored = [&](std::optional<size_t> device) {
        if (!device) {
            return false;
        }
        auto info = std::ranges::find(devices, *device, &Device::id);
        return info != devices.end() && !includes(listen, *info);
    };
    bool keyboard_ignored = ignored(keyboard.device);
    bool mouse_ignored = ignored(mouse.device);
    if (keyboard_ignored) {
        keyboard = KeyboardView{};
    }
    if (mouse_ignored) {
        mouse = MouseView{};
    }
    log(now, Source::Demo, std::move(text), std::nullopt);
}

std::optional<GameInputGamepadState> Input::gamepad_state(size_t device) const {
    if (!focused) {
        return std::nullopt;
    }
    auto last = std::ranges::find_if(last_readings, [&](const LastReading& candidate) { return candidate.kind == GameInputKindGamepad && candidate.reading.device == device; });
    if (last == last_readings.end()) {
        return std::nullopt;
    }
    if (last->reading.timestamp >= focus_timestamp) {
        return last->masked.gamepad(last->reading.gamepad);
    }
    return std::nullopt;
}

void Input::take_reading(Source from, GameInputKind kind, Reading reading) {
    // A reading of a device that isn't connected is dropped: the device
    // has disconnected, like when its reading came just before its
    // disconnect was handled, or its connection hasn't been reported
    // yet. Handled, the reading would bring back the device's view and
    // its rumble, and kept, a device that reconnects would start from it
    // instead of an idle state.
    if (reading.device && !std::ranges::contains(devices, *reading.device, &Device::id)) {
        return;
    }
    auto found = std::ranges::find_if(last_readings, [&](const LastReading& last) { return last.kind == kind && last.reading.device == reading.device; });
    std::optional<Reading> previous;
    Masked masked_before;
    if (found != last_readings.end()) {
        // Taken out, with the last one put in its place.
        LastReading last = std::move(*found);
        if (found != last_readings.end() - 1) {
            *found = std::move(last_readings.back());
        }
        last_readings.pop_back();
        previous = std::move(last.reading);
        masked_before = std::move(last.masked);
    }
    // KGames programs ignore all input while their window doesn't have
    // focus. The library turns on GameInput's background input, so that
    // it follows the devices without focus too, so the demo drops what
    // came without focus itself, along with input from before the window
    // got focus. Otherwise it would show and log what is typed
    // into other windows, and run its shortcuts for them.
    bool had_focus = focused && reading.timestamp >= focus_timestamp;
    Masked masked = had_focus ? masked_before.still_held(kind, reading) : Masked::held(kind, reading);
    if (had_focus) {
        handle_reading(from, kind, reading, previous ? &*previous : nullptr, masked_before, masked);
    }
    last_readings.push_back(LastReading{kind, std::move(reading), std::move(masked)});
}

void Input::handle_reading(Source from, GameInputKind kind, const Reading& reading, const Reading* previous, const Masked& masked_before, const Masked& masked) {
    // Connected, as take_reading drops readings of other devices.
    const Device* device = nullptr;
    if (reading.device) {
        auto info = std::ranges::find(devices, *reading.device, &Device::id);
        device = info != devices.end() ? &*info : nullptr;
    }
    if (device != nullptr && !includes(listen, *device)) {
        return;
    }
    std::string label = device != nullptr ? device->label : std::string("Unknown device");
    // A reading with several kinds of input comes up once for each, so
    // it's counted only for the first.
    std::optional<GameInputKind> first_kind;
    for (const auto& [_, reading_kind] : ReadingKinds) {
        if ((reading.kinds & reading_kind) != 0) {
            first_kind = reading_kind;
            break;
        }
    }
    if (first_kind == kind) {
        reading_count += 1;
    }
    double time = seconds(reading.timestamp);
    std::optional<size_t> id = reading.device;

    // Both readings without what's masked, so it's neither reported nor
    // shown.
    Handled handled;
    if (from == Source::Gamepad) {
        GameInputGamepadState state = masked.gamepad(reading.gamepad);
        GameInputGamepadState old = previous != nullptr ? masked_before.gamepad(previous->gamepad) : GameInputGamepadState{};
        handled = handle_gamepad(time, state, old, id);
    } else if (from == Source::Keyboard) {
        std::vector<GameInputKeyState> keys = masked.keys(reading.keys);
        std::vector<GameInputKeyState> old = previous != nullptr ? masked_before.keys(previous->keys) : std::vector<GameInputKeyState>();
        handled = handle_keyboard(time, std::move(keys), std::move(old), id, label);
    } else {
        GameInputMouseState state = masked.mouse(reading.mouse);
        std::optional<GameInputMouseState> old = previous != nullptr ? std::optional(masked_before.mouse(previous->mouse)) : std::nullopt;
        handled = handle_mouse(time, state, old, id, label);
    }
    auto& [changes, state, motion_only] = handled;
    // A reading can repeat the previous state, or change only another
    // kind of input on a device with several kinds. Those are counted but
    // not reported.
    if (changes.empty()) {
        return;
    }

    // Analog motion arrives up to thousands of times a second, so a run
    // of motion-only readings from one device shares one log entry.
    std::optional<std::pair<Source, size_t>> merge_key = motion_only ? std::optional(std::pair(from, id.value_or(0))) : std::nullopt;
    log(time, from, std::format("{}: {}", label, base::join(changes, ", ")), merge_key);

    latest = LatestEvent{
        .number = reading_count,
        .time = time,
        .source = from,
        .device = std::move(label),
        .kinds = kind_names(reading.kinds),
        .changes = std::move(changes),
        .state = std::move(state),
        .timestamp = reading.timestamp,
        .interval = previous != nullptr ? std::optional(static_cast<double>(base::saturating_sub(reading.timestamp, previous->timestamp)) / 1000.0) : std::nullopt,
    };

    // After logging the key press, so the log shows cause before effect.
    for (std::uint8_t key : std::exchange(shortcuts, {})) {
        switch (key) {
        case KeyToggleRumble:
            toggle_rumble();
            break;
        case KeyClearLog:
            log_.clear();
            break;
        default:
            break;
        }
    }
}

Input::Handled Input::handle_gamepad(double time, GameInputGamepadState state, GameInputGamepadState old, std::optional<size_t> device) {
    gamepad.rate.add(time);
    if (device) {
        std::array<float, 2> sticks{
            deflection(state.leftThumbstickX, state.leftThumbstickY),
            deflection(state.rightThumbstickX, state.rightThumbstickY),
        };
        auto rumble = std::ranges::find(rumbles, *device, &Rumble::device);
        if (rumble != rumbles.end()) {
            rumble->sticks = sticks;
        } else {
            rumbles.push_back(Rumble{.device = *device, .sticks = sticks, .sent = GameInputRumbleParams{}});
        }
    }

    // The card shows the gamepad listened to. Its view is reset when it
    // disconnects, so this sets it up again after it reconnects.
    if (gamepad.device != device) {
        view_gamepad(device);
    }
    GamepadView& view = gamepad;
    view.state = state;

    std::vector<std::string> changes;
    for (const auto& [button, name] : GamepadButtons) {
        bool held = (state.buttons & button) != 0;
        bool before = (old.buttons & button) != 0;
        if (held && !before) {
            changes.push_back(std::format("{} pressed", name));
        } else if (before && !held) {
            changes.push_back(std::format("{} released", name));
        }
    }
    bool motion_only = changes.empty();
    if (state.leftThumbstickX != old.leftThumbstickX || state.leftThumbstickY != old.leftThumbstickY) {
        changes.push_back(std::format("left stick {}", pair(state.leftThumbstickX, state.leftThumbstickY)));
    }
    if (state.rightThumbstickX != old.rightThumbstickX || state.rightThumbstickY != old.rightThumbstickY) {
        changes.push_back(std::format("right stick {}", pair(state.rightThumbstickX, state.rightThumbstickY)));
    }
    if (state.leftTrigger != old.leftTrigger) {
        changes.push_back(std::format("left trigger {:.3f}", state.leftTrigger));
    }
    if (state.rightTrigger != old.rightTrigger) {
        changes.push_back(std::format("right trigger {:.3f}", state.rightTrigger));
    }

    std::vector<std::string_view> held;
    for (const auto& [button, name] : GamepadButtons) {
        if ((state.buttons & button) != 0) {
            held.push_back(name);
        }
    }
    std::vector<std::string> summary;
    summary.reserve(4);
    summary.push_back(std::format("Left stick {}   Right stick {}", pair(state.leftThumbstickX, state.leftThumbstickY), pair(state.rightThumbstickX, state.rightThumbstickY)));
    summary.push_back(std::format("Left trigger {:.3f}   Right trigger {:.3f}", state.leftTrigger, state.rightTrigger));
    summary.push_back(std::format("Buttons held: {}", held.empty() ? std::string("none") : base::join(held, " ")));
    summary.push_back(std::format("buttons = 0x{:08X}", static_cast<std::uint32_t>(state.buttons)));
    return {std::move(changes), std::move(summary), motion_only};
}

Input::Handled Input::handle_keyboard(double time, std::vector<GameInputKeyState> keys, std::vector<GameInputKeyState> old, std::optional<size_t> device, std::string_view label) {
    std::vector<std::string> changes;
    for (const GameInputKeyState& key : keys) {
        if (!std::ranges::contains(old, key.scanCode, &GameInputKeyState::scanCode)) {
            changes.push_back(std::format("{} down", key_name(key)));
            if (key.virtualKey == KeyToggleRumble || key.virtualKey == KeyClearLog) {
                shortcuts.push_back(key.virtualKey);
            }
        }
    }
    for (const GameInputKeyState& key : old) {
        if (!std::ranges::contains(keys, key.scanCode, &GameInputKeyState::scanCode)) {
            changes.push_back(std::format("{} up", key_name(key)));
        }
    }

    std::vector<std::pair<std::string, std::string>> descriptions;
    descriptions.reserve(keys.size());
    for (const GameInputKeyState& key : keys) {
        descriptions.emplace_back(key_name(key), key_details(key));
    }
    std::vector<std::string> summary;
    summary.reserve(descriptions.size());
    for (const auto& [name, details] : descriptions) {
        summary.push_back(std::format("{}  {}", name, details));
    }
    if (summary.empty()) {
        summary.push_back("No keys down");
    }
    KeyboardView& view = keyboard;
    if (!changes.empty() || view.device == device) {
        view.rate.add(time);
        view.device = device;
        view.label = label;
        view.keys = std::move(keys);
        view.descriptions = std::move(descriptions);
    }
    return {std::move(changes), std::move(summary), false};
}

Input::Handled Input::handle_mouse(double time, GameInputMouseState state, std::optional<GameInputMouseState> old, std::optional<size_t> device, std::string_view label) {
    std::vector<std::string> changes;
    GameInputMouseButtons old_buttons = old ? old->buttons : GameInputMouseNone;
    for (const auto& [button, name] : MouseButtons) {
        bool held = (state.buttons & button) != 0;
        bool before = (old_buttons & button) != 0;
        if (held && !before) {
            changes.push_back(std::format("{} pressed", name));
        } else if (before && !held) {
            changes.push_back(std::format("{} released", name));
        }
    }
    bool motion_only = changes.empty();

    // Position and wheel are running totals, so the change since the
    // previous reading is the motion. The positions flags say which of
    // relative and absolute position the device reports.
    std::array<std::int64_t, 2> delta{0, 0};
    if (old) {
        GameInputMousePositions relative = GameInputMouseRelativePosition;
        if ((state.positions & relative) != 0 && (old->positions & relative) != 0) {
            delta = {state.positionX - old->positionX, state.positionY - old->positionY};
        }
        if (delta[0] != 0 || delta[1] != 0) {
            changes.push_back(std::format("moved ({:+}, {:+})", delta[0], delta[1]));
        }
        std::array<std::int64_t, 2> wheel{state.wheelX - old->wheelX, state.wheelY - old->wheelY};
        if (wheel[1] != 0) {
            changes.push_back(std::format("wheel {:+}", wheel[1]));
        }
        if (wheel[0] != 0) {
            changes.push_back(std::format("horizontal wheel {:+}", wheel[0]));
        }
        if ((state.positions & GameInputMouseAbsolutePosition) != 0 && (state.absolutePositionX != old->absolutePositionX || state.absolutePositionY != old->absolutePositionY)) {
            changes.push_back(std::format("absolute position ({}, {})", state.absolutePositionX, state.absolutePositionY));
        }
    }

    auto info = std::ranges::find_if(devices, [&](const Device& candidate) { return device == candidate.id; });
    std::optional<std::uint32_t> sample_rate = info != devices.end() ? info->mouse_sample_rate() : std::nullopt;
    MouseView& view = mouse;
    if (!changes.empty() || view.device == device) {
        view.rate.add(time);
        view.device = device;
        view.label = label;
        view.state = state;
        view.delta = delta;
        view.sample_rate = sample_rate;
        // 400 counts cross half the pad. Raw counts have no pointer
        // acceleration, unlike the cursor.
        for (size_t axis = 0; axis < view.pad.size(); axis++) {
            view.pad[axis] = std::clamp(view.pad[axis] + static_cast<float>(delta[axis]) / 400.0f, -1.0f, 1.0f);
        }
    }

    std::vector<std::string_view> held;
    for (const auto& [button, name] : MouseButtons) {
        if ((state.buttons & button) != 0) {
            held.push_back(name);
        }
    }
    std::vector<std::string> summary;
    summary.reserve(2);
    summary.push_back(std::format("Total motion ({}, {})   wheel ({}, {})", state.positionX, state.positionY, state.wheelX, state.wheelY));
    summary.push_back(std::format("Buttons held: {}", held.empty() ? std::string("none") : base::join(held, ", ")));
    if ((state.positions & GameInputMouseAbsolutePosition) != 0) {
        summary.push_back(std::format("Absolute position ({}, {})", state.absolutePositionX, state.absolutePositionY));
    }
    return {std::move(changes), std::move(summary), motion_only};
}

void Input::log(double time, Source from, std::string text, std::optional<std::pair<Source, size_t>> merge_key) {
    if (LogEntry* last = log_.front(); merge_key && last != nullptr && last->merge_key == merge_key) {
        last->time = time;
        last->text = std::move(text);
        last->count += 1;
        last->format_line();
        return;
    }
    LogEntry entry(time, from, std::move(text), merge_key);
    entry.format_line();
    log_.push_front(std::move(entry));
    log_.truncate(LogCapacity);
}

void Input::toggle_rumble() {
    rumble_enabled = !rumble_enabled;
    std::string_view text = rumble_enabled ? "Rumble test on: the gamepad's sticks drive its motors" : "Rumble test off";
    log(now, Source::Demo, std::string(text), std::nullopt);
}

void Input::update_rumble() {
    for (Rumble& rumble : rumbles) {
        auto [left, right] = rumble.sticks;
        GameInputRumbleParams params{};
        if (rumble_enabled) {
            params.lowFrequency = past_dead_zone(left, rumble_dead_zone);
            params.highFrequency = past_dead_zone(right, rumble_dead_zone);
        }
        if (params != rumble.sent) {
            source->set_rumble(rumble.device, &params);
            rumble.sent = params;
        }
    }
}

void Input::stop_rumble() {
    GameInputRumbleParams off{};
    for (Rumble& rumble : rumbles) {
        if (rumble.sent != off) {
            source->set_rumble(rumble.device, nullptr);
            rumble.sent = off;
        }
    }
}

json::Value Input::state() const {
    auto label = [&](std::optional<size_t> device) -> std::optional<std::string> {
        if (!device) {
            return std::nullopt;
        }
        auto info = std::ranges::find(devices, *device, &Device::id);
        return info != devices.end() ? std::optional(info->label) : std::nullopt;
    };
    auto names = [](std::int32_t flags, const auto& buttons) {
        std::vector<std::string> held;
        for (const auto& [button, name] : buttons) {
            if ((flags & button) != 0) {
                held.emplace_back(name);
            }
        }
        return held;
    };
    GameInputGamepadState pad = gamepad.state;
    std::vector<std::string> device_labels;
    device_labels.reserve(devices.size());
    for (const Device& info : devices) {
        device_labels.push_back(info.label);
    }
    std::vector<std::string> keys;
    keys.reserve(keyboard.keys.size());
    for (const GameInputKeyState& key : keyboard.keys) {
        keys.push_back(key_name(key));
    }
    std::vector<std::string> newest;
    for (size_t index = 0; index < std::min<size_t>(log_.size(), 10); index++) {
        newest.push_back(log_[index].text);
    }
    json::Value::Array running;
    running.values.reserve(rumbles.size());
    for (const Rumble& rumble : rumbles) {
        running.values.push_back(json::object({
            {"Device", label(rumble.device)},
            {"LowFrequency", rumble.sent.lowFrequency},
            {"HighFrequency", rumble.sent.highFrequency},
        }));
    }
    return json::object({
        {"ReadingCount", reading_count},
        {"Listen", listen.label()},
        {"Devices", std::move(device_labels)},
        {"Gamepad", json::object({
                        {"Device", label(gamepad.device)},
                        {"LeftStick", std::array{pad.leftThumbstickX, pad.leftThumbstickY}},
                        {"RightStick", std::array{pad.rightThumbstickX, pad.rightThumbstickY}},
                        {"LeftTrigger", pad.leftTrigger},
                        {"RightTrigger", pad.rightTrigger},
                        {"Buttons", names(pad.buttons, GamepadButtons)},
                    })},
        {"Keyboard", json::object({
                         {"Device", label(keyboard.device)},
                         {"Keys", std::move(keys)},
                     })},
        {"Mouse", json::object({
                      {"Device", label(mouse.device)},
                      {"Buttons", names(mouse.state.buttons, MouseButtons)},
                      {"Delta", mouse.delta},
                      {"Wheel", std::array{mouse.state.wheelX, mouse.state.wheelY}},
                  })},
        {"Latest", latest ? std::optional(latest->changes) : std::nullopt},
        {"Log", std::move(newest)},
        {"RumbleTest", rumble_enabled},
        {"Rumble", std::move(running)},
    });
}

std::string kind_names(GameInputKind kinds) {
    std::vector<std::string_view> names;
    for (const auto& [kind, name] : KindNames) {
        if ((kinds & kind) != 0) {
            names.push_back(name);
        }
    }
    return names.empty() ? std::string("none") : base::join(names, ", ");
}

std::string key_name(const GameInputKeyState& key) {
    // GetKeyNameTextW takes the scan code the way WM_KEYDOWN's lParam holds
    // it: in bits 16-23, with bit 24 set for keys that send an 0xE0 prefix.
    bool extended = (key.scanCode & 0xFF00) == 0xE000;
    std::uint32_t lparam = ((key.scanCode & 0xFF) << 16) | (extended ? 1u << 24 : 0u);
    wchar_t buffer[64]{};
    int length = GetKeyNameTextW(static_cast<LONG>(lparam), buffer, 64);
    if (length > 0) {
        return base::narrow(std::wstring_view(buffer, static_cast<size_t>(length)));
    } else {
        return std::format("Key 0x{:02X}", static_cast<std::uint32_t>(key.virtualKey));
    }
}

std::string key_details(const GameInputKeyState& key) {
    std::string text = std::format("scan 0x{:04X}  VK 0x{:02X}", key.scanCode, static_cast<std::uint32_t>(key.virtualKey));
    // A character that isn't a control character: Unicode's Cc, which is
    // U+0000 to U+001F and U+007F to U+009F.
    std::uint32_t code = key.codePoint;
    bool is_character = code <= 0x10FFFF && !(code >= 0xD800 && code <= 0xDFFF);
    bool is_control = code <= 0x1F || (code >= 0x7F && code <= 0x9F);
    if (is_character && !is_control) {
        text += "  '";
        base::push_char(text, static_cast<char32_t>(code));
        text += std::format("'  U+{:04X}", code);
    }
    if (key.isDeadKey) {
        text += "  dead key";
    }
    return text;
}

} // namespace game_input_demo
