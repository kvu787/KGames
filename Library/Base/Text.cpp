#include "Base/Text.h"

#include "Base/Windows.h"

#include <charconv>
#include <cmath>
#include <format>
#include <limits>
#include <shellapi.h>

#pragma comment(lib, "shell32.lib")

namespace base {

std::wstring widen(std::string_view text) {
    if (text.empty()) {
        return {};
    }
    int length = MultiByteToWideChar(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), nullptr, 0);
    std::wstring wide(static_cast<size_t>(length), L'\0');
    MultiByteToWideChar(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), wide.data(), length);
    return wide;
}

std::string narrow(std::wstring_view text) {
    if (text.empty()) {
        return {};
    }
    int length = WideCharToMultiByte(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), nullptr, 0, nullptr, nullptr);
    std::string narrow(static_cast<size_t>(length), '\0');
    WideCharToMultiByte(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), narrow.data(), length, nullptr, nullptr);
    return narrow;
}

void push_char(std::string& text, char32_t character) {
    if (character < 0x80) {
        text.push_back(static_cast<char>(character));
    } else if (character < 0x800) {
        text.push_back(static_cast<char>(0xC0 | (character >> 6)));
        text.push_back(static_cast<char>(0x80 | (character & 0x3F)));
    } else if (character < 0x10000) {
        text.push_back(static_cast<char>(0xE0 | (character >> 12)));
        text.push_back(static_cast<char>(0x80 | ((character >> 6) & 0x3F)));
        text.push_back(static_cast<char>(0x80 | (character & 0x3F)));
    } else {
        text.push_back(static_cast<char>(0xF0 | (character >> 18)));
        text.push_back(static_cast<char>(0x80 | ((character >> 12) & 0x3F)));
        text.push_back(static_cast<char>(0x80 | ((character >> 6) & 0x3F)));
        text.push_back(static_cast<char>(0x80 | (character & 0x3F)));
    }
}

Decoded decode_char(std::string_view text) {
    constexpr Decoded Invalid{U'�', 1};
    auto byte = [&](size_t index) {
        return static_cast<unsigned char>(text[index]);
    };
    auto continues = [&](size_t index) {
        return index < text.size() && (byte(index) & 0xC0) == 0x80;
    };
    unsigned char first = byte(0);
    if (first < 0x80) {
        return {first, 1};
    }
    if ((first & 0xE0) == 0xC0 && continues(1)) {
        char32_t character = (static_cast<char32_t>(first & 0x1F) << 6) | (byte(1) & 0x3F);
        return character >= 0x80 ? Decoded{character, 2} : Invalid;
    }
    if ((first & 0xF0) == 0xE0 && continues(1) && continues(2)) {
        char32_t character = (static_cast<char32_t>(first & 0x0F) << 12) | (static_cast<char32_t>(byte(1) & 0x3F) << 6) | (byte(2) & 0x3F);
        bool surrogate = character >= 0xD800 && character < 0xE000;
        return character >= 0x800 && !surrogate ? Decoded{character, 3} : Invalid;
    }
    if ((first & 0xF8) == 0xF0 && continues(1) && continues(2) && continues(3)) {
        char32_t character =
            (static_cast<char32_t>(first & 0x07) << 18) | (static_cast<char32_t>(byte(1) & 0x3F) << 12) | (static_cast<char32_t>(byte(2) & 0x3F) << 6) | (byte(3) & 0x3F);
        return character >= 0x10000 && character < 0x110000 ? Decoded{character, 4} : Invalid;
    }
    return Invalid;
}

size_t char_count(std::string_view text) {
    size_t count = 0;
    for (char byte : text) {
        // Every byte but a continuation byte starts a character.
        count += (static_cast<unsigned char>(byte) & 0xC0) != 0x80;
    }
    return count;
}

bool is_char_boundary(std::string_view text, size_t offset) {
    if (offset >= text.size()) {
        return offset == text.size();
    }
    return (static_cast<unsigned char>(text[offset]) & 0xC0) != 0x80;
}

namespace {

// What char::is_whitespace is for ASCII, which is all that the files here
// have at their ends.
bool is_space(char character) {
    return character == ' ' || (character >= '\t' && character <= '\r');
}

} // namespace

std::string_view trim(std::string_view text) {
    while (!text.empty() && is_space(text.front())) {
        text.remove_prefix(1);
    }
    while (!text.empty() && is_space(text.back())) {
        text.remove_suffix(1);
    }
    return text;
}

std::vector<std::string_view> split(std::string_view text, char separator) {
    std::vector<std::string_view> parts;
    while (true) {
        size_t at = text.find(separator);
        parts.push_back(text.substr(0, at));
        if (at == std::string_view::npos) {
            return parts;
        }
        text.remove_prefix(at + 1);
    }
}

std::vector<std::string_view> lines(std::string_view text) {
    std::vector<std::string_view> lines;
    while (!text.empty()) {
        size_t at = text.find('\n');
        std::string_view line = text.substr(0, at);
        if (at != std::string_view::npos && line.ends_with('\r')) {
            line.remove_suffix(1);
        }
        lines.push_back(line);
        if (at == std::string_view::npos) {
            break;
        }
        text.remove_prefix(at + 1);
    }
    return lines;
}

std::string to_ascii_lowercase(std::string_view text) {
    std::string lower(text);
    for (char& character : lower) {
        if (character >= 'A' && character <= 'Z') {
            character = static_cast<char>(character - 'A' + 'a');
        }
    }
    return lower;
}

std::string to_ascii_uppercase(std::string_view text) {
    std::string upper(text);
    for (char& character : upper) {
        if (character >= 'a' && character <= 'z') {
            character = static_cast<char>(character - 'a' + 'A');
        }
    }
    return upper;
}

namespace {

// std::to_chars without a precision writes the shortest digits that read
// back as the same number, which is what Rust writes too.
template <class Float> std::string shortest(Float value, std::chars_format format) {
    char buffer[400];
    auto result = std::to_chars(buffer, buffer + sizeof(buffer), value, format);
    return std::string(buffer, result.ptr);
}

template <class Float> std::optional<std::string> special(Float value) {
    if (std::isnan(value)) {
        return "NaN";
    }
    if (std::isinf(value)) {
        return value < 0 ? "-inf" : "inf";
    }
    return std::nullopt;
}

template <class Float> std::string display_float(Float value) {
    if (auto text = special(value)) {
        return *text;
    }
    return shortest(value, std::chars_format::fixed);
}

template <class Float> std::string debug_float(Float value) {
    if (auto text = special(value)) {
        return *text;
    }
    Float size = std::abs(value);
    if (size != 0 && (size < static_cast<Float>(1e-4) || size >= static_cast<Float>(1e16))) {
        // "1.5e-07" and "1e+16" are 1.5e-7 and 1e16 in Rust.
        std::string text = shortest(value, std::chars_format::scientific);
        size_t exponent = text.find('e') + 1;
        std::string digits = text.substr(exponent + 1);
        digits.erase(0, std::min(digits.find_first_not_of('0'), digits.size() - 1));
        return text.substr(0, exponent) + (text[exponent] == '-' ? "-" : "") + digits;
    }
    std::string text = shortest(value, std::chars_format::fixed);
    if (text.find('.') == std::string::npos) {
        text += ".0";
    }
    return text;
}

template <class Number> std::optional<Number> parse_number(std::string_view text) {
    Number number{};
    auto result = std::from_chars(text.data(), text.data() + text.size(), number);
    if (result.ec != std::errc() || result.ptr != text.data() + text.size()) {
        return std::nullopt;
    }
    return number;
}

template <class Float> std::optional<Float> parse_float(std::string_view text) {
    // Rust takes a + before a number, and std::from_chars doesn't.
    bool plus = text.starts_with('+');
    if (plus) {
        text.remove_prefix(1);
        if (text.starts_with('-') || text.starts_with('+')) {
            return std::nullopt;
        }
    }
    if (text.empty()) {
        return std::nullopt;
    }
    Float number{};
    auto result = std::from_chars(text.data(), text.data() + text.size(), number);
    if (result.ptr != text.data() + text.size() || result.ec == std::errc::invalid_argument) {
        return std::nullopt;
    }
    if (result.ec == std::errc::result_out_of_range) {
        // Rust reads a number too large as infinity, and one too small as 0.
        bool negative = text.starts_with('-');
        bool tiny = text.find("e-") != std::string_view::npos || text.find("E-") != std::string_view::npos;
        Float size = tiny ? Float{0} : std::numeric_limits<Float>::infinity();
        return negative ? -size : size;
    }
    return number;
}

} // namespace

std::string display(double value) {
    return display_float(value);
}

std::string display(float value) {
    return display_float(value);
}

std::string debug(double value) {
    return debug_float(value);
}

std::string debug(float value) {
    return debug_float(value);
}

std::string debug(std::string_view text) {
    std::string quoted = "\"";
    for (char32_t character : chars(text)) {
        switch (character) {
        case U'"':
            quoted += "\\\"";
            break;
        case U'\\':
            quoted += "\\\\";
            break;
        case U'\n':
            quoted += "\\n";
            break;
        case U'\r':
            quoted += "\\r";
            break;
        case U'\t':
            quoted += "\\t";
            break;
        case U'\0':
            quoted += "\\0";
            break;
        default:
            if (character < 0x20 || character == 0x7F) {
                quoted += std::format("\\u{{{:x}}}", static_cast<std::uint32_t>(character));
            } else {
                push_char(quoted, character);
            }
        }
    }
    quoted += '"';
    return quoted;
}

std::optional<double> parse_f64(std::string_view text) {
    return parse_float<double>(text);
}

std::optional<float> parse_f32(std::string_view text) {
    return parse_float<float>(text);
}

std::optional<std::int64_t> parse_i64(std::string_view text) {
    if (text.starts_with('+')) {
        text.remove_prefix(1);
        if (text.starts_with('-') || text.starts_with('+')) {
            return std::nullopt;
        }
    }
    return parse_number<std::int64_t>(text);
}

std::optional<std::uint64_t> parse_u64(std::string_view text) {
    if (text.starts_with('+')) {
        text.remove_prefix(1);
    }
    if (text.starts_with('-') || text.starts_with('+')) {
        return std::nullopt;
    }
    return parse_number<std::uint64_t>(text);
}

std::vector<std::string> arguments() {
    int count = 0;
    wchar_t** wide = CommandLineToArgvW(GetCommandLineW(), &count);
    std::vector<std::string> arguments;
    for (int index = 1; wide != nullptr && index < count; index++) {
        arguments.push_back(narrow(wide[index]));
    }
    LocalFree(wide);
    return arguments;
}

} // namespace base
