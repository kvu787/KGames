#include "GameInputDemo/Devices.h"

#include "GameInputDemo/Input.h"

#include <array>
#include <format>

namespace game_input_demo {

using namespace game_input;

namespace {

// Windows gives devices built into the computer this container.
constexpr GUID BuiltInContainer{0x00000000, 0x0000, 0x0000, {0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff}};

constexpr std::array<std::pair<GameInputSystemButtons, std::string_view>, 2> SystemButtons{{
    {GameInputSystemButtonGuide, "Guide"},
    {GameInputSystemButtonShare, "Share"},
}};

// The items with the separator between them, as a slice's join gives them in
// Rust.
template <class Items> std::string join(const Items& items, std::string_view separator) {
    std::string joined;
    bool first = true;
    for (const auto& item : items) {
        if (!first) {
            joined += separator;
        }
        joined += item;
        first = false;
    }
    return joined;
}

// The names of the flags that are set, from `choices`, or "none".
template <class Choices> std::string names(std::int32_t flags, const Choices& choices) {
    std::vector<std::string_view> names;
    for (const auto& [flag, name] : choices) {
        if ((flags & flag) != 0) {
            names.push_back(name);
        }
    }
    if (names.empty()) {
        return "none";
    } else {
        return join(names, ", ");
    }
}

std::string or_none(std::string_view text) {
    if (text.empty()) {
        return "none";
    } else {
        return std::string(text);
    }
}

std::string version(GameInputVersion version) {
    return std::format("{}.{}.{}.{}", version.major, version.minor, version.build, version.revision);
}

// The usage page and usage, with their names from the HID Usage Tables for
// the ones that input devices have.
std::string usage_name(GameInputUsage usage) {
    auto is = [&](std::uint16_t page, std::uint16_t id) {
        return usage.page == page && usage.id == id;
    };
    std::string_view name;
    if (is(0, 0)) {
        return "none";
    } else if (is(0x01, 0x01)) {
        name = "Generic Desktop: Pointer";
    } else if (is(0x01, 0x02)) {
        name = "Generic Desktop: Mouse";
    } else if (is(0x01, 0x04)) {
        name = "Generic Desktop: Joystick";
    } else if (is(0x01, 0x05)) {
        name = "Generic Desktop: Gamepad";
    } else if (is(0x01, 0x06)) {
        name = "Generic Desktop: Keyboard";
    } else if (is(0x01, 0x07)) {
        name = "Generic Desktop: Keypad";
    } else if (is(0x01, 0x08)) {
        name = "Generic Desktop: Multi-axis Controller";
    } else if (is(0x01, 0x80)) {
        name = "Generic Desktop: System Control";
    } else if (is(0x0C, 0x01)) {
        name = "Consumer: Consumer Control";
    } else if (is(0x0D, 0x05)) {
        name = "Digitizers: Touch Pad";
    } else {
        name = "";
    }
    std::string numbers = std::format("page 0x{:02X}, usage 0x{:02X}", usage.page, usage.id);
    if (name.empty()) {
        return numbers;
    } else {
        return std::format("{} ({})", name, numbers);
    }
}

std::string keyboard_info(const GameInputKeyboardInfo& info) {
    std::string_view kind;
    switch (info.kind) {
    case GameInputAnsiKeyboard:
        kind = "ANSI";
        break;
    case GameInputIsoKeyboard:
        kind = "ISO";
        break;
    case GameInputKsKeyboard:
        kind = "KS (Korean)";
        break;
    case GameInputAbntKeyboard:
        kind = "ABNT (Brazilian)";
        break;
    case GameInputJisKeyboard:
        kind = "JIS (Japanese)";
        break;
    default:
        kind = "unknown kind";
        break;
    }
    return std::format("{}, layout 0x{:08X}, {} keys, {} function keys, {} at once, type {}, subtype {}", kind, info.layout, info.keyCount, info.functionKeyCount,
                       info.maxSimultaneousKeys, info.platformType, info.platformSubtype);
}

std::string mouse_info(const GameInputMouseInfo& info) {
    std::string text = std::format("buttons {}", names(info.supportedButtons, MouseButtons));
    if (info.hasWheelY) {
        text += ", wheel";
    }
    if (info.hasWheelX) {
        text += ", horizontal wheel";
    }
    if (info.sampleRate > 0) {
        text += std::format(", {} Hz", info.sampleRate);
    }
    return text;
}

std::string gamepad_info(const GameInputGamepadInfo& info) {
    return std::format("buttons {}, {} extra buttons, {} extra axes", names(info.supportedLayout, GamepadButtons), info.extraButtonCount, info.extraAxisCount);
}

// A GUID as Windows writes one, without the braces: its numbers in
// hexadecimal, in upper case.
std::string guid(const GUID& id) {
    return std::format("{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}", id.Data1, id.Data2, id.Data3, id.Data4[0], id.Data4[1], id.Data4[2], id.Data4[3],
                       id.Data4[4], id.Data4[5], id.Data4[6], id.Data4[7]);
}

} // namespace

std::string_view Device::family_name() const {
    switch (family) {
    case GameInputFamilyVirtual:
        return "virtual";
    case GameInputFamilyXboxOne:
        return "Xbox One";
    case GameInputFamilyXbox360:
        return "Xbox 360";
    case GameInputFamilyHid:
        return "HID";
    case GameInputFamilyI8042:
        return "i8042 (PS/2)";
    case GameInputFamilyAggregate:
        return "aggregate";
    default:
        return "unknown";
    }
}

std::string Device::kinds() const {
    return kind_names(supported_input);
}

bool Device::is_gamepad() const {
    return (supported_input & GameInputKindGamepad) != 0;
}

std::optional<std::uint32_t> Device::mouse_sample_rate() const {
    if (mouse && mouse->sampleRate > 0) {
        return mouse->sampleRate;
    }
    return std::nullopt;
}

std::vector<Section> Device::details() const {
    std::vector<std::pair<std::string, std::string>> rows;
    rows.emplace_back("Family", family_name());
    rows.emplace_back("Input", kinds());
    rows.emplace_back("IDs", std::format("vendor 0x{:04X}, product 0x{:04X}, revision 0x{:04X}", vendor_id, product_id, revision));
    rows.emplace_back("HID usage", usage_name(usage));
    rows.emplace_back("Versions", std::format("hardware {}, firmware {}", version(hardware_version), version(firmware_version)));
    if (keyboard) {
        rows.emplace_back("Keyboard", keyboard_info(*keyboard));
    }
    if (mouse) {
        rows.emplace_back("Mouse", mouse_info(*mouse));
    }
    if (gamepad) {
        rows.emplace_back("Gamepad", gamepad_info(*gamepad));
    }
    if (xinput_index) {
        rows.emplace_back("XInput index", std::format("{}", *xinput_index));
    }
    if (rumble_motors != 0) {
        rows.emplace_back("Rumble", rumble_motor_names(rumble_motors));
    }
    if (system_buttons != 0) {
        rows.emplace_back("System buttons", names(system_buttons, SystemButtons));
    }
    std::string container = std::format("{{{}}}", guid(container_id));
    if (container_id == BuiltInContainer) {
        container += ", built into this computer";
    } else if (container_id == GUID{}) {
        container = "none";
    }
    rows.emplace_back("Container", std::move(container));
    rows.emplace_back("PnP path", or_none(pnp_path));
    std::vector<Section> sections;
    sections.push_back(Section{.title = "GameInput", .rows = std::move(rows)});

    const std::expected<std::vector<Node>, std::string>* found = tree.get();
    if (found == nullptr) {
        sections.push_back(Section{.title = "Windows", .rows = {{"Device tree", "looking it up"}}});
        return sections;
    }
    if (!found->has_value()) {
        sections.push_back(Section{.title = "Windows", .rows = {{"Not found", found->error()}}});
        return sections;
    }
    const std::vector<Node>& nodes = found->value();
    if (!nodes.empty()) {
        const Node& node = nodes.front();
        std::vector<std::pair<std::string, std::string>> node_rows;
        node_rows.emplace_back("Instance ID", node.instance_id);
        node_rows.emplace_back("Name", or_none(node.name));
        if (!node.bus_name.empty()) {
            node_rows.emplace_back("Bus name", node.bus_name);
        }
        node_rows.emplace_back("Manufacturer", or_none(node.manufacturer));
        node_rows.emplace_back("Class", or_none(node.class_));
        node_rows.emplace_back("Driver", or_none(node.service));
        if (!node.inf.empty()) {
            node_rows.emplace_back("INF file", node.inf);
        }
        if (!node.location.empty()) {
            node_rows.emplace_back("Location", node.location);
        }
        node_rows.emplace_back("Hardware IDs", or_none(join(node.hardware_ids, "\n")));
        sections.push_back(Section{.title = "Windows", .rows = std::move(node_rows)});
    }
    // Each node above it by its class, then its names and where it is,
    // and its instance ID under them.
    if (nodes.size() > 1) {
        std::vector<std::pair<std::string, std::string>> above;
        above.reserve(nodes.size() - 1);
        for (size_t index = 1; index < nodes.size(); index++) {
            const Node& node = nodes[index];
            std::string text = node.name;
            if (!node.bus_name.empty() && node.bus_name != node.name) {
                text += std::format(", \"{}\"", node.bus_name);
            }
            if (!node.location.empty()) {
                text += std::format(", at {}", node.location);
            }
            above.emplace_back(node.class_, std::format("{}\n{}", text, node.instance_id));
        }
        sections.push_back(Section{.title = "Connected through, nearest first", .rows = std::move(above)});
    }
    return sections;
}

std::string rumble_motor_names(GameInputRumbleMotors motors) {
    constexpr std::array<std::pair<GameInputRumbleMotors, std::string_view>, 4> Motors{{
        {GameInputRumbleLowFrequency, "low frequency"},
        {GameInputRumbleHighFrequency, "high frequency"},
        {GameInputRumbleLeftTrigger, "left trigger"},
        {GameInputRumbleRightTrigger, "right trigger"},
    }};
    return names(motors, Motors);
}

} // namespace game_input_demo
