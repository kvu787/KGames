#include "GameInputDemo/ScriptedSource.h"

#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Base/Windows.h"
#include "GameInputDemo/DeviceTree.h"
#include "GameInputDemo/Input.h"
#include "UserInterface/Input/Devices.h"

#include <array>
#include <cmath>

#pragma comment(lib, "user32.lib")

namespace game_input_demo {

using namespace game_input;

namespace {

std::expected<size_t, std::string> id(const json::Value& value) {
    if (const json::Value* member = value.get("Id")) {
        if (const json::Value::Number* number = base::as<json::Value::Number>(*member);
            number != nullptr && number->value >= 1.0 && number->value - std::trunc(number->value) == 0.0) {
            return base::saturating_cast<size_t>(number->value);
        }
    }
    return std::unexpected(std::string("a device's Id must be a whole number from 1"));
}

// A member that's a number, or 0 if it's left out.
std::expected<double, std::string> number(const json::Value& value, std::string_view name) {
    const json::Value* member = value.get(name);
    if (member == nullptr) {
        return 0.0;
    }
    if (const json::Value::Number* found = base::as<json::Value::Number>(*member)) {
        return found->value;
    }
    return std::unexpected(std::format("{} must be a number, not {}", name, member->describe()));
}

// A member that's [x, y], or [0, 0] if it's left out.
std::expected<std::array<double, 2>, std::string> pair(const json::Value& value, std::string_view name) {
    const json::Value* member = value.get(name);
    if (member == nullptr) {
        return std::array{0.0, 0.0};
    }
    if (const json::Value::Array* array = base::as<json::Value::Array>(*member)) {
        const std::vector<json::Value>& items = array->values;
        if (items.size() == 2) {
            const json::Value::Number* x = base::as<json::Value::Number>(items[0]);
            const json::Value::Number* y = base::as<json::Value::Number>(items[1]);
            if (x != nullptr && y != nullptr) {
                return std::array{x->value, y->value};
            }
        }
        return std::unexpected(std::format("{} must be [x, y], not a list of {}", name, items.size()));
    }
    return std::unexpected(std::format("{} must be [x, y], not {}", name, member->describe()));
}

// A member that's a list of names, or none if it's left out.
std::expected<std::vector<std::string>, std::string> strings(const json::Value& value, std::string_view name) {
    auto not_names = [&](const json::Value& other) {
        return std::format("{} must be a list of names, not {}", name, other.describe());
    };
    const json::Value* member = value.get(name);
    if (member == nullptr) {
        return std::vector<std::string>();
    }
    if (const json::Value::Array* array = base::as<json::Value::Array>(*member)) {
        std::vector<std::string> names;
        names.reserve(array->values.size());
        for (const json::Value& item : array->values) {
            const json::Value::String* text = base::as<json::Value::String>(item);
            if (text == nullptr) {
                return std::unexpected(not_names(item));
            }
            names.push_back(text->value);
        }
        return names;
    }
    return std::unexpected(not_names(*member));
}

// The flags named in a member that's a list of names, from `choices`.
template <class Flag, size_t Count>
std::expected<Flag, std::string> flags(const json::Value& value, std::string_view name, const std::array<std::pair<Flag, std::string_view>, Count>& choices) {
    std::expected<std::vector<std::string>, std::string> items = strings(value, name);
    if (!items) {
        return std::unexpected(std::move(items.error()));
    }
    std::int32_t named = 0;
    for (const std::string& item : *items) {
        auto choice = std::ranges::find(choices, std::string_view(item), &std::pair<Flag, std::string_view>::second);
        if (choice == choices.end()) {
            return std::unexpected(std::format("{} can't have {}", name, item));
        }
        named |= choice->first;
    }
    return static_cast<Flag>(named);
}

} // namespace

void ScriptedSource::connect(Device device) {
    std::erase_if(connected, [&](const Device& known) { return known.id == device.id; });
    connected.push_back(device);
    events.emplace_back(GameInputDeviceEventConnected, std::move(device));
}

void ScriptedSource::disconnect(size_t id) {
    auto device = std::ranges::find(connected, id, &Device::id);
    if (device != connected.end()) {
        Device removed = std::move(*device);
        connected.erase(device);
        events.emplace_back(GameInputDeviceEventDisconnected, std::move(removed));
    }
}

std::optional<GameInputRumbleParams> ScriptedSource::rumble(size_t id) const {
    auto found = std::ranges::find(rumble_, id, &std::pair<size_t, std::optional<GameInputRumbleParams>>::first);
    return found != rumble_.end() ? found->second : std::nullopt;
}

std::expected<void, std::string> ScriptedSource::apply(const json::Value& step) {
    const json::Value::Object* object = base::as<json::Value::Object>(step);
    if (object == nullptr) {
        return std::unexpected(std::format("a step for the demo must be an object, not {}", step.describe()));
    }
    if (object->members.size() != 1) {
        return std::unexpected(std::string("a step for the demo has one of Connect, Disconnect, Gamepad, Keyboard and Mouse"));
    }
    const auto& [name, value] = object->members.front();
    if (name == "Connect") {
        constexpr std::array<std::pair<GameInputKind, std::string_view>, 3> Inputs{{
            {GameInputKindGamepad, "Gamepad"},
            {GameInputKindKeyboard, "Keyboard"},
            {GameInputKindMouse, "Mouse"},
        }};
        std::expected<GameInputKind, std::string> supported_input = flags(value, "Input", Inputs);
        if (!supported_input) {
            return std::unexpected(std::move(supported_input.error()));
        }
        constexpr std::array<std::pair<GameInputRumbleMotors, std::string_view>, 4> Motors{{
            {GameInputRumbleLowFrequency, "LowFrequency"},
            {GameInputRumbleHighFrequency, "HighFrequency"},
            {GameInputRumbleLeftTrigger, "LeftTrigger"},
            {GameInputRumbleRightTrigger, "RightTrigger"},
        }};
        std::expected<GameInputRumbleMotors, std::string> rumble_motors = flags(value, "Motors", Motors);
        if (!rumble_motors) {
            return std::unexpected(std::move(rumble_motors.error()));
        }
        const json::Value* named = value.get("Name");
        const json::Value::String* label = named != nullptr ? base::as<json::Value::String>(*named) : nullptr;
        if (label == nullptr) {
            return std::unexpected(std::string("Connect needs a Name"));
        }
        std::expected<size_t, std::string> device = id(value);
        if (!device) {
            return std::unexpected(std::move(device.error()));
        }
        connect(Device{
            .id = *device,
            .label = label->value,
            .vendor_id = 0,
            .product_id = 0,
            .revision = 0,
            .usage = GameInputUsage{},
            .hardware_version = GameInputVersion{},
            .firmware_version = GameInputVersion{},
            .family = GameInputFamilyVirtual,
            .supported_input = *supported_input,
            .rumble_motors = *rumble_motors,
            .system_buttons = GameInputSystemButtonNone,
            .container_id = GUID{},
            .pnp_path = std::string(),
            .xinput_index = std::nullopt,
            .keyboard = std::nullopt,
            .mouse = std::nullopt,
            .gamepad = (*supported_input & GameInputKindGamepad) != 0 ? std::optional(GameInputGamepadInfo{}) : std::nullopt,
            .tree = Tree::known(std::unexpected(std::string("the device comes from a script"))),
        });
        return {};
    } else if (name == "Disconnect") {
        if (const json::Value::Number* device = base::as<json::Value::Number>(value);
            device != nullptr && device->value >= 1.0 && device->value - std::trunc(device->value) == 0.0) {
            disconnect(base::saturating_cast<size_t>(device->value));
            return {};
        }
        return std::unexpected(std::format("Disconnect needs a device's Id, not {}", value.describe()));
    } else if (name == "Gamepad") {
        std::expected<std::array<double, 2>, std::string> left = pair(value, "LeftStick");
        if (!left) {
            return std::unexpected(std::move(left.error()));
        }
        std::expected<std::array<double, 2>, std::string> right = pair(value, "RightStick");
        if (!right) {
            return std::unexpected(std::move(right.error()));
        }
        std::expected<GameInputGamepadButtons, std::string> buttons = flags(value, "Buttons", GamepadButtons);
        if (!buttons) {
            return std::unexpected(std::move(buttons.error()));
        }
        std::expected<double, std::string> left_trigger = number(value, "LeftTrigger");
        if (!left_trigger) {
            return std::unexpected(std::move(left_trigger.error()));
        }
        std::expected<double, std::string> right_trigger = number(value, "RightTrigger");
        if (!right_trigger) {
            return std::unexpected(std::move(right_trigger.error()));
        }
        GameInputGamepadState state{
            .buttons = *buttons,
            .leftTrigger = static_cast<float>(*left_trigger),
            .rightTrigger = static_cast<float>(*right_trigger),
            .leftThumbstickX = static_cast<float>((*left)[0]),
            .leftThumbstickY = static_cast<float>((*left)[1]),
            .rightThumbstickX = static_cast<float>((*right)[0]),
            .rightThumbstickY = static_cast<float>((*right)[1]),
        };
        std::expected<size_t, std::string> device = id(value);
        if (!device) {
            return std::unexpected(std::move(device.error()));
        }
        return read(*device, [&](Reading& reading) { reading.gamepad = state; });
    } else if (name == "Keyboard") {
        std::expected<std::vector<std::string>, std::string> down = strings(value, "Down");
        if (!down) {
            return std::unexpected(std::move(down.error()));
        }
        std::vector<GameInputKeyState> keys;
        keys.reserve(down->size());
        for (const std::string& held : *down) {
            std::optional<GameInputKeyState> key = key_state(held);
            if (!key) {
                return std::unexpected(std::format("{} isn't a key", held));
            }
            keys.push_back(*key);
        }
        std::expected<size_t, std::string> device = id(value);
        if (!device) {
            return std::unexpected(std::move(device.error()));
        }
        return read(*device, [&](Reading& reading) { reading.keys = std::move(keys); });
    } else if (name == "Mouse") {
        std::expected<std::array<double, 2>, std::string> motion = pair(value, "Motion");
        if (!motion) {
            return std::unexpected(std::move(motion.error()));
        }
        std::expected<std::array<double, 2>, std::string> wheel = pair(value, "Wheel");
        if (!wheel) {
            return std::unexpected(std::move(wheel.error()));
        }
        std::expected<GameInputMouseButtons, std::string> buttons = flags(value, "Buttons", MouseButtons);
        if (!buttons) {
            return std::unexpected(std::move(buttons.error()));
        }
        std::expected<size_t, std::string> device = id(value);
        if (!device) {
            return std::unexpected(std::move(device.error()));
        }
        return read(*device, [&](Reading& reading) {
            GameInputMouseState& mouse = reading.mouse;
            mouse.buttons = *buttons;
            mouse.positions = GameInputMouseRelativePosition;
            mouse.positionX += base::saturating_cast<std::int64_t>((*motion)[0]);
            mouse.positionY += base::saturating_cast<std::int64_t>((*motion)[1]);
            mouse.wheelX += base::saturating_cast<std::int64_t>((*wheel)[0]);
            mouse.wheelY += base::saturating_cast<std::int64_t>((*wheel)[1]);
        });
    } else {
        return std::unexpected(std::format("{} isn't a step for the demo", name));
    }
}

std::uint64_t ScriptedSource::now() {
    return now_;
}

std::vector<std::pair<GameInputDeviceEvent, Device>> ScriptedSource::device_events() {
    return std::exchange(events, {});
}

std::vector<Reading> ScriptedSource::readings(GameInputKind kind) {
    auto queue = std::ranges::find(queues, kind, &std::pair<GameInputKind, std::vector<Reading>>::first);
    return queue != queues.end() ? std::exchange(queue->second, {}) : std::vector<Reading>();
}

void ScriptedSource::set_rumble(size_t device, const GameInputRumbleParams* params) {
    std::erase_if(rumble_, [&](const std::pair<size_t, std::optional<GameInputRumbleParams>>& known) { return known.first == device; });
    rumble_.emplace_back(device, params != nullptr ? std::optional(*params) : std::nullopt);
}

ScriptedSource* ScriptedSource::as_scripted() {
    return this;
}

std::optional<GameInputKeyState> key_state(std::string_view name) {
    std::optional<user_interface::Key> parsed = user_interface::Key::parse(name);
    if (!parsed) {
        return std::nullopt;
    }
    std::uint32_t key = parsed->code;
    std::uint32_t scan_code = MapVirtualKeyW(key, MAPVK_VK_TO_VSC_EX);
    // Windows gives the arrow keys and Delete the scan codes of their twins
    // on the numeric keypad, without the prefix that the keys themselves send.
    if ((key >= 0x25 && key <= 0x28) || key == 0x2E) {
        scan_code |= 0xE000;
    }
    std::uint32_t character = 0;
    if (key >= 0x41 && key <= 0x5A) {
        character = key + 0x20;
    } else if ((key >= 0x30 && key <= 0x39) || key == 0x20) {
        character = key;
    }
    return GameInputKeyState{
        .scanCode = scan_code,
        .codePoint = character,
        .virtualKey = static_cast<std::uint8_t>(key),
        .isDeadKey = false,
    };
}

} // namespace game_input_demo
