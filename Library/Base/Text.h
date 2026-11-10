// Text: every string in the repository is UTF-8, in a std::string, as Rust's
// are. These are what Rust's standard library has for them and C++'s doesn't:
// UTF-16 for the Windows API, a string's characters, numbers written the way
// Rust writes them, so that files and scripts' states read the same, and
// the program's arguments.

#pragma once

#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

namespace base {

// UTF-8 to UTF-16 and back, for the Windows API. Text that isn't valid gets
// U+FFFD where it's wrong, as String::from_utf16_lossy gives.
std::wstring widen(std::string_view text);
std::string narrow(std::wstring_view text);

// How many bytes the character takes in UTF-8: char::len_utf8.
constexpr size_t len_utf8(char32_t character) {
    return character < 0x80 ? 1 : character < 0x800 ? 2 : character < 0x10000 ? 3 : 4;
}

// Adds the character to the string: String::push.
void push_char(std::string& text, char32_t character);

// The character at the start of the text, which mustn't be empty, and how
// many bytes it takes. U+FFFD, and one byte, for bytes that aren't UTF-8.
struct Decoded {
    char32_t character;
    size_t length;
};
Decoded decode_char(std::string_view text);

// The text's characters, for a range-based for: str::chars.
//
//     for (char32_t character : base::chars(text)) { ... }
class Chars {
public:
    class Iterator {
    public:
        using difference_type = std::ptrdiff_t;
        using value_type = char32_t;

        Iterator() = default;
        explicit Iterator(std::string_view rest) : rest_(rest) {}

        char32_t operator*() const {
            return decode_char(rest_).character;
        }

        Iterator& operator++() {
            rest_.remove_prefix(decode_char(rest_).length);
            return *this;
        }

        Iterator operator++(int) {
            Iterator before = *this;
            ++*this;
            return before;
        }

        bool operator==(const Iterator& other) const {
            return rest_.data() == other.rest_.data() && rest_.size() == other.rest_.size();
        }

    private:
        std::string_view rest_;
    };

    explicit Chars(std::string_view text) : text_(text) {}

    Iterator begin() const {
        return Iterator(text_);
    }

    Iterator end() const {
        return Iterator(text_.substr(text_.size()));
    }

private:
    std::string_view text_;
};

inline Chars chars(std::string_view text) {
    return Chars(text);
}

// How many characters the text has: str::chars().count().
size_t char_count(std::string_view text);

// Whether a character starts at the byte offset, or the offset is the
// text's end: str::is_char_boundary.
bool is_char_boundary(std::string_view text, size_t offset);

// Without the whitespace at its ends: str::trim.
std::string_view trim(std::string_view text);

// The parts between the separators: str::split.
std::vector<std::string_view> split(std::string_view text, char separator);

// The text's lines, without their line ends: str::lines.
std::vector<std::string_view> lines(std::string_view text);

// ASCII letters in lower or upper case: str::to_ascii_lowercase.
std::string to_ascii_lowercase(std::string_view text);
std::string to_ascii_uppercase(std::string_view text);

// A number as Rust's {} writes it: the shortest decimal that reads back as
// the same number, never with an exponent, and without ".0" for a whole
// number. "inf", "-inf" and "NaN" for those.
std::string display(double value);
std::string display(float value);

// A number as Rust's {:?} writes it: the same digits, with ".0" for a whole
// number, and an exponent, like 1e-7 or 1.5e16, below 0.0001 and from 1e16.
std::string debug(double value);
std::string debug(float value);

// The text in double quotes, with what a Rust string literal escapes
// escaped, as {:?} writes a string.
std::string debug(std::string_view text);

// The number that the text is, as str::parse::<f64> reads it, or nothing if
// it isn't one.
std::optional<double> parse_f64(std::string_view text);
std::optional<float> parse_f32(std::string_view text);

// A whole number in decimal, with an optional sign: str::parse::<i64>.
std::optional<std::int64_t> parse_i64(std::string_view text);
std::optional<std::uint64_t> parse_u64(std::string_view text);

// The program's arguments, after its own name: std::env::args().skip(1).
std::vector<std::string> arguments();

} // namespace base
