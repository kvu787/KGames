#include "UserInterface/Input/Events.h"

#include "Base/Text.h"

#include <algorithm>
#include <format>

namespace user_interface {

std::string describe_value(const Event& event) {
    return base::match(
        event, [](const Event::PointerMoved& moved) { return std::format("PointerMoved({})", describe_value(moved.point)); },
        [](const Event::PointerLeft&) { return std::string("PointerLeft"); },
        [](const Event::ButtonPressed& pressed) { return std::format("ButtonPressed({})", describe_value(pressed.button)); },
        [](const Event::ButtonReleased& released) { return std::format("ButtonReleased({})", describe_value(released.button)); },
        [](const Event::Wheel& wheel) { return std::format("Wheel({})", describe_value(wheel.notches)); },
        [](const Event::KeyPressed& pressed) { return std::format("KeyPressed {{ key: {}, repeat: {} }}", describe_value(pressed.key), pressed.repeat); },
        [](const Event::KeyReleased& released) { return std::format("KeyReleased({})", describe_value(released.key)); },
        [](const Event::Text& typed) {
            std::string text = "Text('";
            base::push_char(text, typed.character);
            return text + "')";
        },
        [](const Event::FocusGained& gained) {
            std::string text = "FocusGained([";
            for (size_t index = 0; index < gained.held.size(); index++) {
                text += index > 0 ? ", " : "";
                text += describe_value(gained.held[index]);
            }
            return text + "])";
        },
        [](const Event::FocusLost&) { return std::string("FocusLost"); },
        [](const Event::Gamepad& gamepad) { return std::format("Gamepad({})", describe_value(gamepad.reading)); });
}

void Input::begin(double now, std::span<const Event> events) {
    delta = static_cast<float>(std::max(now - time, 0.0));
    time = now;
    for (ButtonState& button : buttons) {
        button.pressed = false;
        button.released = false;
    }
    wheel = Point::Zero;
    text.clear();
    keys_pressed_.clear();
    keys_released.clear();
    focus_lost = false;
    previous_gamepad = gamepad;
    std::optional<Point> before = pointer;
    for (const Event& event : events) {
        apply(event);
    }
    pointer_delta = pointer && before ? *pointer - *before : Point::Zero;
}

void Input::apply(const Event& event) {
    if (const auto* gained = base::as<Event::FocusGained>(event)) {
        focused = true;
        for (Key key : gained->held) {
            ignored_keys[key.code] = true;
        }
        ignore_next_gamepad = true;
    } else if (base::is<Event::FocusLost>(event)) {
        focused = false;
        focus_lost = true;
        for (ButtonState& button : buttons) {
            if (button.down) {
                button.down = false;
                button.released = true;
            }
        }
        keys_down.fill(false);
        ignored_keys.fill(false);
        pointer = std::nullopt;
        gamepad = Gamepad{.connected = gamepad.connected};
    } else if (!focused) {
        // Nothing else counts without focus.
    } else if (const auto* moved = base::as<Event::PointerMoved>(event)) {
        pointer = moved->point;
    } else if (base::is<Event::PointerLeft>(event)) {
        if (!any_button_down()) {
            pointer = std::nullopt;
        }
    } else if (const auto* press = base::as<Event::ButtonPressed>(event)) {
        double press_time = time;
        Point position = pointer.value_or(Point());
        ButtonState& state = buttons[index(press->button)];
        bool again = state.clicks > 0 && press_time - state.press_time <= DoubleClickTime && position.distance(state.press_position) <= DoubleClickDistance;
        state.clicks = again ? state.clicks + 1 : 1;
        state.down = true;
        state.pressed = true;
        state.press_time = press_time;
        state.press_position = position;
    } else if (const auto* release = base::as<Event::ButtonReleased>(event)) {
        ButtonState& state = buttons[index(release->button)];
        if (state.down) {
            state.down = false;
            state.released = true;
        }
    } else if (const auto* turned = base::as<Event::Wheel>(event)) {
        wheel += turned->notches;
    } else if (const auto* pressed = base::as<Event::KeyPressed>(event)) {
        if (ignored_keys[pressed->key.code]) {
            return;
        }
        keys_down[pressed->key.code] = true;
        keys_pressed_.emplace_back(pressed->key, pressed->repeat);
    } else if (const auto* released = base::as<Event::KeyReleased>(event)) {
        size_t code = released->key.code;
        if (std::exchange(ignored_keys[code], false)) {
            return;
        }
        if (keys_down[code]) {
            keys_down[code] = false;
            keys_released.push_back(released->key);
        }
    } else if (const auto* typed = base::as<Event::Text>(event)) {
        base::push_char(text, typed->character);
    } else if (const auto* read = base::as<Event::Gamepad>(event)) {
        if (std::exchange(ignore_next_gamepad, false)) {
            ignored_gamepad_buttons = read->reading.buttons.bits;
        }
        // A button that's let go of counts again from then on.
        ignored_gamepad_buttons &= read->reading.buttons.bits;
        Gamepad reading = read->reading;
        reading.buttons.bits &= ~ignored_gamepad_buttons;
        gamepad = reading;
    }
}

bool Input::button_down(MouseButton button) const {
    return buttons[index(button)].down;
}

bool Input::button_pressed(MouseButton button) const {
    return buttons[index(button)].pressed;
}

bool Input::button_released(MouseButton button) const {
    return buttons[index(button)].released;
}

std::uint32_t Input::clicks(MouseButton button) const {
    return buttons[index(button)].clicks;
}

Point Input::press_position(MouseButton button) const {
    return buttons[index(button)].press_position;
}

bool Input::any_button_down() const {
    return std::ranges::any_of(buttons, [](const ButtonState& button) { return button.down; });
}

bool Input::key_down(Key key) const {
    return keys_down[key.code];
}

bool Input::key_pressed(Key key) const {
    return std::ranges::any_of(keys_pressed_, [key](const std::pair<Key, bool>& press) { return press.first == key; });
}

bool Input::key_pressed_once(Key key) const {
    return std::ranges::any_of(keys_pressed_, [key](const std::pair<Key, bool>& press) { return press.first == key && !press.second; });
}

bool Input::key_released(Key key) const {
    return std::ranges::contains(keys_released, key);
}

std::span<const std::pair<Key, bool>> Input::keys_pressed() const {
    return keys_pressed_;
}

std::vector<Key> Input::keys_held() const {
    std::vector<Key> held;
    for (unsigned code = 0; code <= 255; code++) {
        Key key(static_cast<std::uint8_t>(code));
        if (key_down(key)) {
            held.push_back(key);
        }
    }
    return held;
}

Modifiers Input::modifiers() const {
    return Modifiers(key_down(Key::Control), key_down(Key::Shift), key_down(Key::Alt));
}

bool Input::shortcut_pressed(Shortcut shortcut) const {
    return modifiers() == shortcut.modifiers && key_pressed(shortcut.key_);
}

bool Input::gamepad_pressed(std::uint32_t button) const {
    return gamepad.buttons.contains(button) && !previous_gamepad.buttons.contains(button);
}

bool Input::gamepad_released(std::uint32_t button) const {
    return !gamepad.buttons.contains(button) && previous_gamepad.buttons.contains(button);
}

} // namespace user_interface
