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

void push_char_beyond_ascii(std::string& text, char32_t character) {
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

Decoded decode_char_beyond_ascii(std::string_view text) {
    constexpr Decoded Invalid{Replacement, 1, false};
    auto byte = [&](size_t index) {
        return static_cast<unsigned char>(text[index]);
    };
    auto continues = [&](size_t index) {
        return index < text.size() && (byte(index) & 0xC0) == 0x80;
    };
    // Whether the byte at the index is there and in the range.
    auto within = [&](size_t index, unsigned char low, unsigned char high) {
        return index < text.size() && byte(index) >= low && byte(index) <= high;
    };
    // Bytes that aren't UTF-8 are one U+FFFD for the longest run of them that
    // starts a character, as String::from_utf8_lossy gives.
    auto invalid = [](size_t length) {
        return Decoded{Replacement, length, false};
    };
    unsigned char first = byte(0);
    if (first < 0x80) {
        return {first, 1};
    }
    if (first >= 0xC2 && first <= 0xDF) {
        if (!continues(1)) {
            return Invalid;
        }
        return {(static_cast<char32_t>(first & 0x1F) << 6) | (byte(1) & 0x3F), 2};
    }
    if (first >= 0xE0 && first <= 0xEF) {
        // Not the long form of a shorter character, and not a surrogate.
        unsigned char low = first == 0xE0 ? 0xA0 : 0x80;
        unsigned char high = first == 0xED ? 0x9F : 0xBF;
        if (!within(1, low, high)) {
            return Invalid;
        }
        if (!continues(2)) {
            return invalid(2);
        }
        return {(static_cast<char32_t>(first & 0x0F) << 12) | (static_cast<char32_t>(byte(1) & 0x3F) << 6) | (byte(2) & 0x3F), 3};
    }
    if (first >= 0xF0 && first <= 0xF4) {
        // Not the long form of a shorter character, and no more than U+10FFFF.
        unsigned char low = first == 0xF0 ? 0x90 : 0x80;
        unsigned char high = first == 0xF4 ? 0x8F : 0xBF;
        if (!within(1, low, high)) {
            return Invalid;
        }
        if (!continues(2)) {
            return invalid(2);
        }
        if (!continues(3)) {
            return invalid(3);
        }
        return {(static_cast<char32_t>(first & 0x07) << 18) | (static_cast<char32_t>(byte(1) & 0x3F) << 12) | (static_cast<char32_t>(byte(2) & 0x3F) << 6) | (byte(3) & 0x3F), 4};
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

// char::is_whitespace: Unicode's White_Space.
bool is_space(char32_t character) {
    return character == U' ' || (character >= U'\t' && character <= U'\r') || character == 0x85 || character == 0xA0 || character == 0x1680 ||
           (character >= 0x2000 && character <= 0x200A) || character == 0x2028 || character == 0x2029 || character == 0x202F || character == 0x205F || character == 0x3000;
}

} // namespace

std::string_view trim(std::string_view text) {
    while (!text.empty()) {
        Decoded first = decode_char(text);
        if (!is_space(first.character)) {
            break;
        }
        text.remove_prefix(first.length);
    }
    while (!text.empty()) {
        // The last character starts at the last byte that isn't a
        // continuation byte.
        size_t start = text.size() - 1;
        while (start > 0 && (static_cast<unsigned char>(text[start]) & 0xC0) == 0x80) {
            start -= 1;
        }
        Decoded last = decode_char(text.substr(start));
        if (!is_space(last.character) || start + last.length != text.size()) {
            break;
        }
        text.remove_suffix(last.length);
    }
    return text;
}

bool eq_ignore_ascii_case(std::string_view a, std::string_view b) {
    if (a.size() != b.size()) {
        return false;
    }
    for (size_t index = 0; index < a.size(); index++) {
        auto lower = [](char character) {
            return character >= 'A' && character <= 'Z' ? static_cast<char>(character - 'A' + 'a') : character;
        };
        if (lower(a[index]) != lower(b[index])) {
            return false;
        }
    }
    return true;
}

bool is_utf8(std::string_view text) {
    while (!text.empty()) {
        Decoded decoded = decode_char(text);
        // U+FFFD in three bytes is the character itself. In fewer, it stands
        // for bytes that aren't UTF-8.
        if (!decoded.valid) {
            return false;
        }
        text.remove_prefix(decoded.length);
    }
    return true;
}

std::string from_utf8_lossy(std::string_view bytes) {
    if (is_utf8(bytes)) {
        return std::string(bytes);
    }
    std::string text;
    text.reserve(bytes.size());
    for (char32_t character : chars(bytes)) {
        push_char(text, character);
    }
    return text;
}

std::string replace(std::string_view text, std::string_view from, std::string_view to) {
    std::string replaced;
    if (from.empty()) {
        return std::string(text);
    }
    while (true) {
        size_t at = text.find(from);
        replaced += text.substr(0, at);
        if (at == std::string_view::npos) {
            return replaced;
        }
        replaced += to;
        text.remove_prefix(at + from.size());
    }
}

std::optional<std::string> environment_variable(std::string_view name) {
    std::wstring wide_name = widen(name);
    DWORD size = GetEnvironmentVariableW(wide_name.c_str(), nullptr, 0);
    if (size == 0) {
        // Not set, or set to nothing, which reads as nothing either way.
        return GetLastError() == ERROR_ENVVAR_NOT_FOUND ? std::nullopt : std::optional<std::string>(std::string());
    }
    std::wstring value(size, L'\0');
    DWORD length = GetEnvironmentVariableW(wide_name.c_str(), value.data(), size);
    value.resize(length);
    return narrow(value);
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

std::optional<std::uint64_t> parse_u64(std::string_view text, int radix) {
    if (text.starts_with('+')) {
        text.remove_prefix(1);
    }
    if (text.empty() || text.starts_with('-') || text.starts_with('+')) {
        return std::nullopt;
    }
    std::uint64_t number = 0;
    auto result = std::from_chars(text.data(), text.data() + text.size(), number, radix);
    if (result.ec != std::errc() || result.ptr != text.data() + text.size()) {
        return std::nullopt;
    }
    return number;
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
