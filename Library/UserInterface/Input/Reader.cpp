#include "UserInterface/Input/Reader.h"

#include "Base/Numbers.h"
#include "Base/Sum.h"
#include "Base/Windows.h"
#include "UserInterface/Platform/Clock.h"

#include <algorithm>
#include <variant>

#pragma comment(lib, "user32.lib")

namespace user_interface {

namespace {

// What a frame's events come from, in the order of their times: the window
// got or lost focus, or a reading came, of a kind of input.
struct Item {
    struct Focus {
        bool gained;
    };
    struct Reading {
        game_input::GameInputKind kind;
        const user_interface::Reading* reading;
    };
    std::variant<Focus, Reading> value;

    template <base::AlternativeOf<Item> T> Item(T&& alternative) : value(std::forward<T>(alternative)) {}
};

// The interface's key for a key that GameInput reads: its virtual key, with
// Shift, Ctrl and Alt the same on the left and the right.
std::optional<Key> key(const game_input::GameInputKeyState& state) {
    std::uint32_t code = state.virtualKey;
    if (code == 0) {
        code = MapVirtualKeyW(state.scanCode, MAPVK_VSC_TO_VK_EX);
    }
    switch (code) {
    case 0xA0:
    case 0xA1:
        return Key::Shift;
    case 0xA2:
    case 0xA3:
        return Key::Control;
    case 0xA4:
    case 0xA5:
        return Key::Alt;
    default:
        if (code >= 1 && code <= 0xFE) {
            return Key(static_cast<std::uint8_t>(code));
        }
        return std::nullopt;
    }
}

// GameInput's mouse buttons as bits by MouseButton.
std::uint32_t button_bits(game_input::GameInputMouseButtons buttons) {
    std::uint32_t bits = 0;
    const std::array<std::pair<MouseButton, game_input::GameInputMouseButtons>, 3> flags{{
        {MouseButton::Left, game_input::GameInputMouseLeftButton},
        {MouseButton::Right, game_input::GameInputMouseRightButton},
        {MouseButton::Middle, game_input::GameInputMouseMiddleButton},
    }};
    for (auto [button, flag] : flags) {
        if ((buttons & flag) != 0) {
            bits |= std::uint32_t{1} << static_cast<std::uint32_t>(button);
        }
    }
    return bits;
}

// Windows' keyboard delay, in seconds, and repeat rate, in repeats a second,
// from its settings: the delay's 0 to 3 is 250 ms to 1 s, and the rate's 0 to
// 31 is about 2.5 to 30 a second.
std::pair<double, double> keyboard_repeat() {
    std::uint32_t delay = 1;
    std::uint32_t speed = 31;
    SystemParametersInfoW(SPI_GETKEYBOARDDELAY, 0, &delay, 0);
    SystemParametersInfoW(SPI_GETKEYBOARDSPEED, 0, &speed, 0);
    double delay_seconds = static_cast<double>(std::min<std::uint32_t>(delay, 3) + 1) * 0.25;
    double rate = 2.5 + static_cast<double>(std::min<std::uint32_t>(speed, 31)) * (30.0 - 2.5) / 31.0;
    return {delay_seconds, rate};
}

} // namespace

InputReader::InputReader(bool gamepad) : InputReader(gamepad, keyboard_repeat()) {}

InputReader::InputReader(bool gamepad, std::pair<double, double> repeat) : InputReader(gamepad, repeat.first, repeat.second) {}

InputReader::InputReader(bool gamepad, double delay, double rate) :
    gamepad(gamepad),
    repeat_delay(base::saturating_cast<std::int64_t>(delay * static_cast<double>(clock::frequency()))),
    repeat_interval(base::saturating_cast<std::int64_t>(static_cast<double>(clock::frequency()) / rate)) {}

InputReader InputReader::with_repeat(bool gamepad, double delay, double rate) {
    return InputReader(gamepad, delay, rate);
}

FrameInput InputReader::frame(const Devices* devices, Window& window, std::span<const FocusChange> focus_changes) {
    auto [cursor, inside] = window.cursor();
    WindowState state{
        .cursor = cursor,
        .inside = inside,
        .focus_changes = std::vector<FocusChange>(focus_changes.begin(), focus_changes.end()),
    };
    std::vector<std::tuple<std::int64_t, game_input::GameInputKind, Reading>> readings;
    std::vector<size_t> disconnected;
    if (devices != nullptr) {
        for (const auto& [event, device] : devices->device_events()) {
            if (event == game_input::GameInputDeviceEventDisconnected) {
                disconnected.push_back(device.id);
            }
        }
        for (game_input::GameInputKind kind : Kinds) {
            for (const Reading& reading : devices->readings(kind)) {
                readings.emplace_back(devices->qpc(reading.timestamp), kind, reading);
            }
        }
    }
    FrameInput frame = read(readings, disconnected, state, clock::now());
    window.set_capture(capturing());
    return frame;
}

bool InputReader::capturing() const {
    return focused && (buttons_down & ~ignored_buttons) != 0;
}

FrameInput InputReader::read(std::span<const std::tuple<std::int64_t, game_input::GameInputKind, Reading>> readings, std::span<const size_t> disconnected,
                             const WindowState& window, std::int64_t now) {
    Point cursor = window.cursor;
    bool inside = window.inside;
    FrameInput frame;
    // A device that's unplugged lets go of what it held: a gamepad's
    // state goes, and a keyboard's keys and a mouse's buttons are
    // released, unless another keyboard or mouse holds them.
    auto unplugged = [&](const std::optional<size_t>& device) {
        return device && std::ranges::find(disconnected, *device) != disconnected.end();
    };
    if (unplugged(gamepad_device)) {
        gamepad_state = Gamepad{};
        gamepad_device = std::nullopt;
    }
    if (std::ranges::any_of(keyboards, [&](const std::pair<std::optional<size_t>, HeldKeys>& held) { return unplugged(held.first); })) {
        std::erase_if(keyboards, [&](const std::pair<std::optional<size_t>, HeldKeys>& held) { return unplugged(held.first); });
        update_keys(now, frame.events);
    }
    if (std::ranges::any_of(mice, [&](const std::pair<std::optional<size_t>, game_input::GameInputMouseState>& last) { return unplugged(last.first); })) {
        std::erase_if(mice, [&](const std::pair<std::optional<size_t>, game_input::GameInputMouseState>& last) { return unplugged(last.first); });
        update_buttons(cursor, inside, frame.events);
    }
    std::vector<std::pair<std::int64_t, Item>> items;
    items.reserve(window.focus_changes.size());
    for (const FocusChange& change : window.focus_changes) {
        items.emplace_back(change.time, Item::Focus{change.gained});
    }
    for (const auto& [time, kind, reading] : readings) {
        items.emplace_back(time, Item::Reading{kind, &reading});
    }
    // Stable, so that a reading with two kinds of input keeps its order.
    std::ranges::stable_sort(items, {}, [](const std::pair<std::int64_t, Item>& item) { return item.first; });

    for (const auto& [time, item] : items) {
        if (const Item::Focus* focus = base::as<Item::Focus>(item)) {
            if (focus->gained) {
                if (!focused) {
                    focused = true;
                    std::vector<Key> held;
                    for (size_t code = 0; code <= 255; code++) {
                        if (keys_down[code]) {
                            held.push_back(Key(static_cast<std::uint8_t>(code)));
                        }
                    }
                    frame.events.push_back(Event::FocusGained{std::move(held)});
                    ignored_buttons = buttons_down;
                    pointer = std::nullopt;
                }
            } else if (focused) {
                focused = false;
                frame.events.push_back(Event::FocusLost{});
                repeat = std::nullopt;
                pointer = std::nullopt;
            }
        } else if (const Item::Reading* taken = base::as<Item::Reading>(item)) {
            const Reading& reading = *taken->reading;
            switch (taken->kind) {
            case game_input::GameInputKindKeyboard:
                keyboard(reading, time, frame.events);
                break;
            case game_input::GameInputKindMouse:
                mouse(reading, cursor, inside, frame.events);
                break;
            default:
                gamepad_state = user_interface::gamepad(reading.gamepad);
                gamepad_device = reading.device;
                break;
            }
            if (focused && (taken->kind != game_input::GameInputKindGamepad || gamepad)) {
                frame.input_time = time;
            }
        }
    }

    // A held key repeats, at the times Windows' settings give it.
    if (focused && repeat) {
        Key repeated = repeat->key;
        std::uint32_t scan_code = repeat->scan_code;
        std::int32_t count = 0;
        while (repeat->next <= now) {
            repeat->next += std::max<std::int64_t>(repeat_interval, 1);
            count += 1;
        }
        for (std::int32_t done = 0; done < count; done++) {
            frame.events.push_back(Event::KeyPressed{.key = repeated, .repeat = true});
            text(repeated, scan_code, frame.events);
        }
    }
    point(cursor, inside, frame.events);
    if (gamepad) {
        frame.events.push_back(Event::Gamepad{gamepad_state});
    }
    return frame;
}

void InputReader::point(Point cursor, bool inside, std::vector<Event>& events) {
    if (!focused) {
        return;
    }
    std::optional<Point> where;
    if (inside || (buttons_down & ~ignored_buttons) != 0) {
        where = cursor;
    }
    if (where != pointer) {
        if (where) {
            events.push_back(Event::PointerMoved{*where});
        } else {
            events.push_back(Event::PointerLeft{});
        }
        pointer = where;
    }
}

void InputReader::keyboard(const Reading& reading, std::int64_t time, std::vector<Event>& events) {
    HeldKeys keys;
    for (const game_input::GameInputKeyState& state : reading.keys) {
        if (std::optional<Key> held = key(state)) {
            keys.emplace_back(*held, state.scanCode);
        }
    }
    auto known = std::ranges::find_if(keyboards, [&](const std::pair<std::optional<size_t>, HeldKeys>& held) { return held.first == reading.device; });
    if (known != keyboards.end()) {
        known->second = std::move(keys);
    } else {
        keyboards.emplace_back(reading.device, std::move(keys));
    }
    update_keys(time, events);
}

void InputReader::update_keys(std::int64_t time, std::vector<Event>& events) {
    std::array<bool, 256> down{};
    for (const std::pair<std::optional<size_t>, HeldKeys>& held : keyboards) {
        for (const auto& [held_key, scan_code] : held.second) {
            down[held_key.code] = true;
            scan_codes[held_key.code] = scan_code;
        }
    }
    for (size_t code = 0; code < down.size(); code++) {
        if (keys_down[code] && !down[code]) {
            keys_down[code] = false;
            if (focused) {
                events.push_back(Event::KeyReleased{Key(static_cast<std::uint8_t>(code))});
            }
            if (repeat && repeat->key.code == code) {
                repeat = std::nullopt;
            }
        }
    }
    for (size_t code = 0; code < down.size(); code++) {
        if (!keys_down[code] && down[code]) {
            keys_down[code] = true;
            Key pressed(static_cast<std::uint8_t>(code));
            if (focused) {
                events.push_back(Event::KeyPressed{.key = pressed, .repeat = false});
                text(pressed, scan_codes[code], events);
                // Windows repeats the last key pressed, and not Shift,
                // Ctrl or Alt here.
                if (!pressed.is_modifier()) {
                    repeat = Repeat{
                        .key = pressed,
                        .scan_code = scan_codes[code],
                        .next = time + repeat_delay,
                    };
                }
            }
        }
    }
}

void InputReader::mouse(const Reading& reading, Point cursor, bool inside, std::vector<Event>& events) {
    game_input::GameInputMouseState state = reading.mouse;
    std::optional<game_input::GameInputMouseState> previous;
    auto known = std::ranges::find_if(mice, [&](const std::pair<std::optional<size_t>, game_input::GameInputMouseState>& last) { return last.first == reading.device; });
    if (known != mice.end()) {
        previous = std::exchange(known->second, state);
    } else {
        mice.emplace_back(reading.device, state);
    }
    update_buttons(cursor, inside, events);
    // The wheels are running totals, in Windows' units of 120 a notch.
    if (previous && focused && inside) {
        Point turned(static_cast<float>(state.wheelX - previous->wheelX) / static_cast<float>(WHEEL_DELTA),
                     static_cast<float>(state.wheelY - previous->wheelY) / static_cast<float>(WHEEL_DELTA));
        if (turned != Point()) {
            point(cursor, inside, events);
            events.push_back(Event::Wheel{turned});
        }
    }
}

void InputReader::update_buttons(Point cursor, bool inside, std::vector<Event>& events) {
    std::uint32_t down = 0;
    for (const std::pair<std::optional<size_t>, game_input::GameInputMouseState>& last : mice) {
        down |= button_bits(last.second.buttons);
    }
    for (MouseButton button : AllMouseButtons) {
        std::uint32_t bit = std::uint32_t{1} << static_cast<std::uint32_t>(button);
        if ((down & bit) != 0 && (buttons_down & bit) == 0) {
            buttons_down |= bit;
            if (focused && inside && (ignored_buttons & bit) == 0) {
                point(cursor, inside, events);
                events.push_back(Event::ButtonPressed{button});
            } else {
                ignored_buttons |= bit;
            }
        } else if ((down & bit) == 0 && (buttons_down & bit) != 0) {
            buttons_down &= ~bit;
            if ((ignored_buttons & bit) != 0) {
                ignored_buttons &= ~bit;
            } else if (focused) {
                point(cursor, inside, events);
                events.push_back(Event::ButtonReleased{button});
            }
        }
    }
}

void InputReader::text(Key key, std::uint32_t scan_code, std::vector<Event>& events) const {
    auto held = [&](Key which) {
        return keys_down[which.code];
    };
    if (held(Key::Alt) && !held(Key::Control)) {
        return;
    }
    std::array<BYTE, 256> state{};
    for (size_t code = 0; code < state.size(); code++) {
        if (keys_down[code]) {
            state[code] = 0x80;
        }
    }
    const std::array<std::pair<Key, size_t>, 3> sides{{{Key::Shift, 0xA0}, {Key::Control, 0xA2}, {Key::Alt, 0xA4}}};
    for (auto [generic, left] : sides) {
        if (held(generic)) {
            state[left] = 0x80;
        }
    }
    // Caps Lock's and Num Lock's toggles are Windows', which keeps them as
    // the window's messages come.
    for (std::uint8_t toggle : {Key::CapsLock.code, std::uint8_t{0x90}, std::uint8_t{0x91}}) {
        if ((GetKeyState(toggle) & 1) != 0) {
            state[toggle] |= 1;
        }
    }
    std::array<wchar_t, 8> buffer{};
    int count = ToUnicodeEx(key.code, scan_code & 0xFF, state.data(), buffer.data(), static_cast<int>(buffer.size()), 0, GetKeyboardLayout(0));
    // Less than 0 for a dead key, which the next key's text has with it.
    if (count > 0) {
        // The characters, without the surrogates that aren't in pairs.
        size_t length = static_cast<size_t>(count);
        for (size_t index = 0; index < length; index++) {
            char32_t unit = buffer[index];
            if (unit < 0xD800 || unit > 0xDFFF) {
                events.push_back(Event::Text{unit});
            } else if (unit < 0xDC00 && index + 1 < length && buffer[index + 1] >= 0xDC00 && buffer[index + 1] <= 0xDFFF) {
                char32_t low = buffer[index + 1];
                events.push_back(Event::Text{0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00)});
                index += 1;
            }
        }
    }
}

Gamepad gamepad(const game_input::GameInputGamepadState& state) {
    return Gamepad{
        .connected = true,
        .left_stick = Point(state.leftThumbstickX, state.leftThumbstickY),
        .right_stick = Point(state.rightThumbstickX, state.rightThumbstickY),
        .left_trigger = state.leftTrigger,
        .right_trigger = state.rightTrigger,
        // GameInput's bits for these buttons are the interface's.
        .buttons = GamepadButtons(static_cast<std::uint32_t>(state.buttons) & 0x3FFF),
    };
}

} // namespace user_interface
