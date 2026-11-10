#include "Json/Json.h"

#include "Base/Text.h"

#include <algorithm>
#include <cmath>
#include <format>

namespace json {

namespace {

void newline(std::string& text, size_t depth) {
    text.push_back('\n');
    text.append(depth * 4, ' ');
}

void write_string(std::string& text, std::string_view value) {
    text.push_back('"');
    for (char32_t character : base::chars(value)) {
        switch (character) {
        case U'"':
            text += "\\\"";
            break;
        case U'\\':
            text += "\\\\";
            break;
        case U'\n':
            text += "\\n";
            break;
        case U'\r':
            text += "\\r";
            break;
        case U'\t':
            text += "\\t";
            break;
        default:
            if (character <= 0x1F) {
                text += std::format("\\u{:04X}", static_cast<unsigned>(character));
            } else {
                base::push_char(text, character);
            }
        }
    }
    text.push_back('"');
}

void write(const Value& value, std::string& text, size_t depth) {
    base::match(
        value, [&](const Value::Null&) { text += "null"; }, [&](const Value::Bool& boolean) { text += boolean.value ? "true" : "false"; },
        [&](const Value::Number& number) { text += std::isfinite(number.value) ? base::debug(number.value) : "null"; },
        [&](const Value::String& string) { write_string(text, string.value); },
        [&](const Value::Array& array) {
            const std::vector<Value>& values = array.values;
            if (values.empty()) {
                text += "[]";
                return;
            }
            bool flat = std::ranges::none_of(values, [](const Value& item) { return base::is<Value::Array>(item) || base::is<Value::Object>(item); });
            if (flat) {
                size_t last_newline = text.rfind('\n');
                size_t line_start = last_newline == std::string::npos ? 0 : last_newline + 1;
                std::string line;
                for (const Value& item : values) {
                    write(item, line, depth + 1);
                    line += ", ";
                }
                flat = text.size() - line_start + line.size() <= LineLength;
            }
            text.push_back('[');
            for (size_t index = 0; index < values.size(); index++) {
                if (index > 0) {
                    text.push_back(',');
                }
                if (!flat) {
                    newline(text, depth + 1);
                } else if (index > 0) {
                    text.push_back(' ');
                }
                write(values[index], text, depth + 1);
            }
            if (!flat) {
                newline(text, depth);
            }
            text.push_back(']');
        },
        [&](const Value::Object& object) {
            if (object.members.empty()) {
                text += "{}";
                return;
            }
            text.push_back('{');
            bool first = true;
            for (const auto& [name, member] : object.members) {
                if (!first) {
                    text.push_back(',');
                }
                first = false;
                newline(text, depth + 1);
                write_string(text, name);
                text += ": ";
                write(member, text, depth + 1);
            }
            newline(text, depth);
            text.push_back('}');
        });
}

// How deep arrays and objects can nest. The parser recurses for each level,
// so a file nested much deeper would overflow the stack instead of failing.
// No settings file or script comes close.
constexpr size_t MaxDepth = 128;

bool is_digit(char32_t character) {
    return character >= U'0' && character <= U'9';
}

bool is_hex_digit(char character) {
    return (character >= '0' && character <= '9') || (character >= 'a' && character <= 'f') || (character >= 'A' && character <= 'F');
}

struct Parser {
    std::string_view text;
    // A byte offset into text.
    size_t position = 0;
    // How many arrays and objects the position is in.
    size_t depth = 0;

    std::string_view rest() const {
        return text.substr(position);
    }

    std::optional<char32_t> peek() const {
        if (position >= text.size()) {
            return std::nullopt;
        }
        return base::decode_char(rest()).character;
    }

    // Steps past the next character if it's the expected one, which is ASCII.
    bool eat(char expected) {
        bool found = position < text.size() && text[position] == expected;
        if (found) {
            position += 1;
        }
        return found;
    }

    void skip_whitespace() {
        while (position < text.size() && (text[position] == ' ' || text[position] == '\t' || text[position] == '\n' || text[position] == '\r')) {
            position += 1;
        }
    }

    // An error at the current position.
    std::unexpected<Error> error(std::string_view message) const {
        std::string_view before = text.substr(0, position);
        size_t last_newline = before.rfind('\n');
        std::string_view last_line = last_newline == std::string_view::npos ? before : before.substr(last_newline + 1);
        return std::unexpected(Error{
            .line = static_cast<size_t>(std::ranges::count(before, '\n')) + 1,
            .column = base::char_count(last_line) + 1,
            .message = std::string(message),
        });
    }

    std::expected<Value, Error> value() {
        skip_whitespace();
        std::optional<char32_t> next = peek();
        if (!next) {
            return error("expected a value, but the file ended");
        }
        if (*next == U'{' || *next == U'[') {
            if (depth == MaxDepth) {
                return error("arrays and objects are nested too deeply");
            }
            depth += 1;
            std::expected<Value, Error> value = *next == U'{' ? object() : array();
            depth -= 1;
            return value;
        }
        if (*next == U'"') {
            return string().transform([](std::string&& string) { return Value(Value::String{std::move(string)}); });
        }
        if (*next == U'-' || is_digit(*next)) {
            return number();
        }
        if (rest().starts_with("true")) {
            position += 4;
            return Value::Bool{true};
        }
        if (rest().starts_with("false")) {
            position += 5;
            return Value::Bool{false};
        }
        if (rest().starts_with("null")) {
            position += 4;
            return Value::Null{};
        }
        return error("expected a value");
    }

    std::expected<Value, Error> object() {
        position += 1;
        Value::Object object;
        skip_whitespace();
        if (eat('}')) {
            return object;
        }
        while (true) {
            skip_whitespace();
            if (peek() == U'}') {
                return error("JSON doesn't allow a comma before '}'");
            }
            if (peek() != U'"') {
                return error("expected a name in double quotes");
            }
            std::expected<std::string, Error> name = string();
            if (!name) {
                return std::unexpected(name.error());
            }
            skip_whitespace();
            if (!eat(':')) {
                return error("expected ':'");
            }
            std::expected<Value, Error> member = value();
            if (!member) {
                return member;
            }
            object.members.emplace_back(std::move(*name), std::move(*member));
            skip_whitespace();
            if (eat('}')) {
                return object;
            }
            if (!eat(',')) {
                return error("expected ',' or '}'");
            }
        }
    }

    std::expected<Value, Error> array() {
        position += 1;
        Value::Array array;
        skip_whitespace();
        if (eat(']')) {
            return array;
        }
        while (true) {
            skip_whitespace();
            if (peek() == U']') {
                return error("JSON doesn't allow a comma before ']'");
            }
            std::expected<Value, Error> item = value();
            if (!item) {
                return item;
            }
            array.values.push_back(std::move(*item));
            skip_whitespace();
            if (eat(']')) {
                return array;
            }
            if (!eat(',')) {
                return error("expected ',' or ']'");
            }
        }
    }

    std::expected<std::string, Error> string() {
        position += 1;
        std::string string;
        while (true) {
            size_t start = position;
            if (position >= text.size()) {
                return error("expected '\"' to end the string");
            }
            base::Decoded decoded = base::decode_char(rest());
            position += decoded.length;
            char32_t character = decoded.character;
            if (character == U'"') {
                return string;
            }
            if (character == U'\\') {
                char escaped;
                switch (position < text.size() ? text[position] : '\0') {
                case '"':
                    escaped = '"';
                    break;
                case '\\':
                    escaped = '\\';
                    break;
                case '/':
                    escaped = '/';
                    break;
                case 'b':
                    escaped = '\b';
                    break;
                case 'f':
                    escaped = '\f';
                    break;
                case 'n':
                    escaped = '\n';
                    break;
                case 'r':
                    escaped = '\r';
                    break;
                case 't':
                    escaped = '\t';
                    break;
                case 'u': {
                    position += 1;
                    std::expected<char32_t, Error> unicode = unicode_escape();
                    if (!unicode) {
                        return std::unexpected(unicode.error());
                    }
                    base::push_char(string, *unicode);
                    continue;
                }
                default:
                    position = start;
                    return error("unknown escape sequence");
                }
                position += 1;
                string.push_back(escaped);
            } else if (character <= 0x1F) {
                // Like a line break. They must be escaped, like \n.
                position = start;
                return error("strings can't contain control characters");
            } else {
                string.append(text.substr(start, decoded.length));
            }
        }
    }

    // After "\u": four hex digits of UTF-16. A character outside the Basic
    // Multilingual Plane takes two, a high surrogate then a low one.
    std::expected<char32_t, Error> unicode_escape() {
        size_t start = position;
        std::expected<char32_t, Error> first = hex_digits();
        if (!first) {
            return first;
        }
        if (!(*first >= 0xD800 && *first < 0xDC00)) {
            // A low surrogate on its own is no character.
            if (!(*first >= 0xDC00 && *first < 0xE000)) {
                return *first;
            }
            position = start;
            return error("unpaired UTF-16 surrogate");
        }
        char32_t second = 0;
        if (rest().starts_with("\\u")) {
            position += 2;
            std::expected<char32_t, Error> digits = hex_digits();
            if (!digits) {
                return digits;
            }
            second = *digits;
        }
        if (!(second >= 0xDC00 && second < 0xE000)) {
            position = start;
            return error("unpaired UTF-16 surrogate");
        }
        return 0x10000 + ((*first - 0xD800) << 10) + (second - 0xDC00);
    }

    std::expected<char32_t, Error> hex_digits() {
        std::string_view digits = rest().substr(0, 4);
        if (digits.size() < 4 || !std::ranges::all_of(digits, is_hex_digit)) {
            return error("expected four hex digits after \\u");
        }
        position += 4;
        char32_t number = 0;
        for (char digit : digits) {
            number = number * 16 + static_cast<char32_t>(digit <= '9' ? digit - '0' : (digit | 0x20) - 'a' + 10);
        }
        return number;
    }

    std::expected<Value, Error> number() {
        size_t start = position;
        eat('-');
        bool leading_zero = peek() == U'0';
        size_t whole = digits();
        if (whole == 0) {
            return error("expected a digit");
        }
        if (whole > 1 && leading_zero) {
            position = start;
            return error("numbers can't start with 0, like 01");
        }
        if (eat('.') && digits() == 0) {
            return error("expected a digit after '.'");
        }
        if (eat('e') || eat('E')) {
            if (!eat('+')) {
                eat('-');
            }
            if (digits() == 0) {
                return error("expected a digit in the exponent");
            }
        }
        // Every number that JSON allows parses, to infinity past what a
        // double holds, which the file would read back as null.
        double number = base::parse_f64(text.substr(start, position - start)).value();
        if (!std::isfinite(number)) {
            position = start;
            return error("the number is too large");
        }
        return Value::Number{number};
    }

    // Steps past a run of digits, and returns how many there were.
    size_t digits() {
        size_t start = position;
        while (position < text.size() && text[position] >= '0' && text[position] <= '9') {
            position += 1;
        }
        return position - start;
    }
};

} // namespace

Value::Value(float number) : value(Number{base::parse_f64(base::display(number)).value_or(static_cast<double>(number))}) {}

std::string Value::describe() const {
    return base::match(
        *this, [](const Null&) { return std::string("null"); }, [](const Bool& boolean) { return std::string(boolean.value ? "true" : "false"); },
        [](const Number& number) { return base::display(number.value); }, [](const String& string) { return std::format("\"{}\"", string.value); },
        [](const Array& array) { return std::format("a list of {}", array.values.size()); }, [](const Object&) { return std::string("an object"); });
}

const Value* Value::get(std::string_view name) const {
    if (const Object* object = base::as<Object>(*this)) {
        for (const auto& [member, member_value] : object->members) {
            if (member == name) {
                return &member_value;
            }
        }
    }
    return nullptr;
}

std::string Value::to_json() const {
    std::string text;
    write(*this, text, 0);
    return text;
}

Value object(std::initializer_list<std::pair<std::string_view, Value>> members) {
    Value::Object object;
    object.members.reserve(members.size());
    for (const auto& [name, value] : members) {
        object.members.emplace_back(std::string(name), value);
    }
    return object;
}

std::expected<Value, Error> parse(std::string_view text) {
    Parser parser{.text = text};
    std::expected<Value, Error> value = parser.value();
    if (!value) {
        return value;
    }
    parser.skip_whitespace();
    if (parser.position < text.size()) {
        return parser.error("expected the end of the file");
    }
    return value;
}

} // namespace json
