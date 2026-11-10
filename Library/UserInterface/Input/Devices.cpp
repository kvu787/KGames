#include "UserInterface/Input/Devices.h"

#include "Base/Panic.h"
#include "Base/Text.h"

#include <charconv>
#include <format>
#include <system_error>

namespace user_interface {

namespace {

// Whether the texts are the same but for the case of their ASCII letters.
bool eq_ignore_ascii_case(std::string_view a, std::string_view b) {
    auto lower = [](char character) {
        return character >= 'A' && character <= 'Z' ? static_cast<char>(character - 'A' + 'a') : character;
    };
    if (a.size() != b.size()) {
        return false;
    }
    for (size_t index = 0; index < a.size(); index++) {
        if (lower(a[index]) != lower(b[index])) {
            return false;
        }
    }
    return true;
}

// A whole number from 0 to 255 in the radix, with nothing before it but an
// optional +, or nothing if the text isn't one.
std::optional<std::uint8_t> parse_u8(std::string_view digits, int radix) {
    if (digits.starts_with('+')) {
        digits.remove_prefix(1);
    }
    if (digits.empty()) {
        return std::nullopt;
    }
    std::uint8_t value = 0;
    const char* end = digits.data() + digits.size();
    std::from_chars_result result = std::from_chars(digits.data(), end, value, radix);
    if (result.ec != std::errc() || result.ptr != end) {
        return std::nullopt;
    }
    return value;
}

} // namespace

constexpr std::array<std::pair<Key, std::string_view>, 36> Key::Names{{
    {Key::Backspace, "Backspace"},
    {Key::Tab, "Tab"},
    {Key::Enter, "Enter"},
    {Key::Shift, "Shift"},
    {Key::Control, "Ctrl"},
    {Key::Alt, "Alt"},
    {Key::Pause, "Pause"},
    {Key::CapsLock, "CapsLock"},
    {Key::Escape, "Esc"},
    {Key::Space, "Space"},
    {Key::PageUp, "PageUp"},
    {Key::PageDown, "PageDown"},
    {Key::End, "End"},
    {Key::Home, "Home"},
    {Key::Left, "Left"},
    {Key::Up, "Up"},
    {Key::Right, "Right"},
    {Key::Down, "Down"},
    {Key::Insert, "Insert"},
    {Key::Delete, "Delete"},
    {Key::LeftWindows, "Windows"},
    {Key::RightWindows, "RightWindows"},
    {Key::LeftShift, "LeftShift"},
    {Key::RightShift, "RightShift"},
    {Key::LeftControl, "LeftCtrl"},
    {Key::RightControl, "RightCtrl"},
    {Key::LeftAlt, "LeftAlt"},
    {Key::RightAlt, "RightAlt"},
    {Key(0xBA), "Semicolon"},
    {Key(0xBB), "Plus"},
    {Key(0xBC), "Comma"},
    {Key(0xBD), "Minus"},
    {Key(0xBE), "Period"},
    {Key(0xBF), "Slash"},
    {Key(0xC0), "Backtick"},
    {Key(0xDE), "Quote"},
}};

std::optional<Key> Key::character(char32_t character) {
    char32_t upper = character >= U'a' && character <= U'z' ? static_cast<char32_t>(character - U'a' + U'A') : character;
    bool alphanumeric = (upper >= U'0' && upper <= U'9') || (upper >= U'A' && upper <= U'Z') || (upper >= U'a' && upper <= U'z');
    return alphanumeric ? std::optional(Key(static_cast<std::uint8_t>(upper))) : std::nullopt;
}

std::string Key::name() const {
    for (const auto& [key, known] : Names) {
        if (key == *this) {
            return std::string(known);
        }
    }
    if ((code >= '0' && code <= '9') || (code >= 'A' && code <= 'Z')) {
        return std::string(1, static_cast<char>(code));
    }
    if (code >= 0x70 && code <= 0x87) {
        return std::format("F{}", code - 0x6F);
    }
    if (code >= 0x60 && code <= 0x69) {
        return std::format("Numpad{}", code - 0x60);
    }
    return std::format("Key{:02X}", static_cast<unsigned>(code));
}

std::optional<Key> Key::parse(std::string_view name) {
    for (const auto& [key, known] : Names) {
        if (eq_ignore_ascii_case(known, name)) {
            return key;
        }
    }
    std::string upper = base::to_ascii_uppercase(name);
    if (upper == "CONTROL") {
        return Key::Control;
    }
    if (upper == "ESCAPE") {
        return Key::Escape;
    }
    if (upper == "RETURN") {
        return Key::Enter;
    }
    if (base::char_count(upper) == 1) {
        return Key::character(base::decode_char(upper).character);
    }
    std::string_view rest = upper;
    if (rest.starts_with('F')) {
        if (std::optional<std::uint8_t> number = parse_u8(rest.substr(1), 10); number && *number >= 1 && *number <= 24) {
            return Key(static_cast<std::uint8_t>(0x6F + *number));
        }
    }
    if (rest.starts_with("NUMPAD")) {
        if (std::optional<std::uint8_t> number = parse_u8(rest.substr(6), 10); number && *number <= 9) {
            return Key(static_cast<std::uint8_t>(0x60 + *number));
        }
    }
    if (rest.starts_with("KEY")) {
        if (std::optional<std::uint8_t> number = parse_u8(rest.substr(3), 16)) {
            return Key(*number);
        }
    }
    return std::nullopt;
}

bool Key::is_modifier() const {
    return (code >= 0x10 && code <= 0x12) || code == 0x5B || code == 0x5C || (code >= 0xA0 && code <= 0xA5);
}

bool Key::is_function() const {
    return code >= 0x70 && code <= 0x87;
}

bool Key::is_navigation() const {
    return *this == Key::Left || *this == Key::Right || *this == Key::Up || *this == Key::Down || *this == Key::Home || *this == Key::End || *this == Key::PageUp ||
           *this == Key::PageDown;
}

std::string describe_value(Key key) {
    return std::format("Key({})", static_cast<unsigned>(key.code));
}

std::string describe_value(Modifiers modifiers) {
    return std::format("Modifiers {{ control: {}, shift: {}, alt: {} }}", modifiers.control, modifiers.shift, modifiers.alt);
}

std::optional<Shortcut> Shortcut::parse(std::string_view text) {
    std::vector<std::string_view> parts = base::split(text, '+');
    for (std::string_view& part : parts) {
        part = base::trim(part);
    }
    if (parts.empty()) {
        return std::nullopt;
    }
    std::string_view last = parts.back();
    Modifiers modifiers = Modifiers::None;
    for (size_t index = 0; index + 1 < parts.size(); index++) {
        std::string held = base::to_ascii_lowercase(parts[index]);
        if (held == "ctrl" || held == "control") {
            modifiers.control = true;
        } else if (held == "shift") {
            modifiers.shift = true;
        } else if (held == "alt") {
            modifiers.alt = true;
        } else {
            return std::nullopt;
        }
    }
    std::optional<Key> key = Key::parse(last);
    if (!key) {
        return std::nullopt;
    }
    return Shortcut(modifiers, *key);
}

std::string Shortcut::name() const {
    std::string name;
    for (auto [held, word] : {std::pair(modifiers.control, "Ctrl+"), std::pair(modifiers.shift, "Shift+"), std::pair(modifiers.alt, "Alt+")}) {
        if (held) {
            name += word;
        }
    }
    name += key_.name();
    return name;
}

std::string describe_value(Shortcut shortcut) {
    return std::format("Shortcut {{ modifiers: {}, key: {} }}", describe_value(shortcut.modifiers), describe_value(shortcut.key_));
}

std::string_view name(MouseButton button) {
    switch (button) {
    case MouseButton::Left:
        return "Left";
    case MouseButton::Right:
        return "Right";
    case MouseButton::Middle:
        return "Middle";
    }
    base::unreachable();
}

std::optional<MouseButton> mouse_button_parse(std::string_view name) {
    for (MouseButton button : AllMouseButtons) {
        if (eq_ignore_ascii_case(user_interface::name(button), name)) {
            return button;
        }
    }
    return std::nullopt;
}

std::string describe_value(MouseButton button) {
    return std::string(name(button));
}

std::vector<std::string_view> GamepadButtons::names() const {
    std::vector<std::string_view> names;
    for (const auto& [bit, name] : Names) {
        if (contains(bit)) {
            names.push_back(name);
        }
    }
    return names;
}

std::optional<std::uint32_t> GamepadButtons::parse(std::string_view name) {
    for (const auto& [bit, known] : Names) {
        if (eq_ignore_ascii_case(known, name)) {
            return bit;
        }
    }
    return std::nullopt;
}

std::string describe_value(GamepadButtons buttons) {
    return std::format("GamepadButtons({})", buttons.bits);
}

std::string describe_value(const Gamepad& gamepad) {
    return std::format("Gamepad {{ connected: {}, left_stick: {}, right_stick: {}, left_trigger: {}, right_trigger: {}, buttons: {} }}", gamepad.connected,
                       describe_value(gamepad.left_stick), describe_value(gamepad.right_stick), base::debug(gamepad.left_trigger), base::debug(gamepad.right_trigger),
                       describe_value(gamepad.buttons));
}

} // namespace user_interface
