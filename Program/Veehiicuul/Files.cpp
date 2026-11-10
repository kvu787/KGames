#include "Veehiicuul/Files.h"

#include "Base/Files.h"
#include "Base/Sum.h"
#include "Base/Text.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <utility>

namespace veehiicuul::files {

using json::Value;

namespace {

// The names, with ", " between them.
std::string joined(std::span<const std::string_view> names) {
    std::string text;
    for (std::string_view name : names) {
        if (!text.empty()) {
            text += ", ";
        }
        text += name;
    }
    return text;
}

bool is_ascii_hexdigit(char character) {
    return (character >= '0' && character <= '9') || (character >= 'A' && character <= 'F') || (character >= 'a' && character <= 'f');
}

} // namespace

std::expected<Value, std::string> parse(std::string_view text) {
    // Without the byte order mark, U+FEFF, that some editors start a file
    // with.
    if (text.starts_with("\xEF\xBB\xBF")) {
        text.remove_prefix(3);
    }
    std::expected<Value, json::Error> value = json::parse(text);
    if (!value) {
        const json::Error& error = value.error();
        return std::unexpected(std::format("line {}, column {}: {}", error.line, error.column, error.message));
    }
    return std::move(*value);
}

std::expected<std::string, std::string> read(const std::filesystem::path& path) {
    std::expected<std::string, std::string> text = base::read_to_string(path);
    if (!text) {
        return std::unexpected(std::format("Couldn't read {}: {}", base::display(path), text.error()));
    }
    return text;
}

std::expected<void, std::string> write(const std::filesystem::path& path, std::string_view text) {
    // A path that ends in a root, or is empty, has no folder that it's in,
    // and making the folder that has no name does nothing.
    if (path.has_relative_path()) {
        std::filesystem::path folder = path.parent_path();
        if (!folder.empty()) {
            if (std::expected<void, std::string> made = base::create_dir_all(folder); !made) {
                return std::unexpected(std::format("Couldn't make {}: {}", base::display(folder), made.error()));
            }
        }
    }
    if (std::expected<void, std::string> written = base::write(path, text); !written) {
        return std::unexpected(std::format("Couldn't write {}: {}", base::display(path), written.error()));
    }
    return {};
}

std::expected<void, std::string> only(const Value& value, std::span<const std::string_view> names, std::string_view what) {
    const Value::Object* object = base::as<Value::Object>(value);
    if (object == nullptr) {
        return std::unexpected(std::format("{} must be an object, not {}", what, value.describe()));
    }
    for (const std::pair<std::string, Value>& entry : object->members) {
        if (std::ranges::find(names, std::string_view(entry.first)) == names.end()) {
            return std::unexpected(std::format("{} isn't part of {}. They're {}", entry.first, what, joined(names)));
        }
    }
    return {};
}

std::expected<const Value*, std::string> member(const Value& value, std::string_view name, std::string_view path) {
    if (const Value* found = value.get(name)) {
        return found;
    }
    return std::unexpected(std::format("needs {}{}", path, name));
}

std::expected<double, std::string> number(const Value& value, std::string_view name) {
    if (const Value::Number* found = base::as<Value::Number>(value); found != nullptr && std::isfinite(found->value)) {
        return found->value;
    }
    return std::unexpected(std::format("{} must be a number, not {}", name, value.describe()));
}

std::expected<double, std::string> positive(const Value& value, std::string_view name) {
    std::expected<double, std::string> found = number(value, name);
    if (!found) {
        return found;
    }
    if (*found > 0.0) {
        return found;
    }
    return std::unexpected(std::format("{} must be above 0, not {}", name, base::display(*found)));
}

std::expected<std::string, std::string> text(const Value& value, std::string_view name) {
    if (const Value::String* found = base::as<Value::String>(value); found != nullptr && !base::trim(found->value).empty()) {
        return found->value;
    }
    return std::unexpected(std::format("{} must be a name, not {}", name, value.describe()));
}

std::expected<std::array<double, 2>, std::string> point(const Value& value, std::string_view name) {
    if (const Value::Array* items = base::as<Value::Array>(value); items != nullptr && items->values.size() == 2) {
        const Value::Number* x = base::as<Value::Number>(items->values[0]);
        const Value::Number* z = base::as<Value::Number>(items->values[1]);
        if (x != nullptr && z != nullptr && std::isfinite(x->value) && std::isfinite(z->value)) {
            return std::array<double, 2>{x->value, z->value};
        }
    }
    return std::unexpected(std::format("{} must be a point, [X, Z], not {}", name, value.describe()));
}

std::expected<std::vector<std::array<double, 2>>, std::string> points(const Value& value, std::string_view name, size_t fewest) {
    const Value::Array* items = base::as<Value::Array>(value);
    if (items == nullptr) {
        return std::unexpected(std::format("{} must be a list of points, not {}", name, value.describe()));
    }
    std::vector<std::array<double, 2>> list;
    list.reserve(items->values.size());
    for (size_t index = 0; index < items->values.size(); index++) {
        std::expected<std::array<double, 2>, std::string> item = point(items->values[index], std::format("{}[{}]", name, index));
        if (!item) {
            return std::unexpected(std::move(item.error()));
        }
        list.push_back(*item);
    }
    if (list.size() < fewest) {
        return std::unexpected(std::format("{} needs at least {} points, not {}", name, fewest, list.size()));
    }
    return list;
}

std::expected<Color, std::string> color(const Value& value, std::string_view name) {
    std::optional<Color> parsed;
    if (const Value::String* found = base::as<Value::String>(value)) {
        parsed = parse_color(found->value);
    }
    if (!parsed) {
        return std::unexpected(std::format("{} must be a color like \"#1CDA61\", not {}", name, value.describe()));
    }
    return *parsed;
}

std::optional<Color> parse_color(std::string_view text) {
    if (text.size() != 7 || !text.starts_with('#') || !std::ranges::all_of(text.substr(1), is_ascii_hexdigit)) {
        return std::nullopt;
    }
    Color parsed{};
    for (size_t channel = 0; channel < 3; channel++) {
        std::optional<std::uint64_t> value = base::parse_u64(text.substr(1 + 2 * channel, 2), 16);
        if (!value) {
            return std::nullopt;
        }
        parsed[channel] = static_cast<std::uint8_t>(*value);
    }
    return parsed;
}

std::string color_text(Color color) {
    return std::format("#{:02X}{:02X}{:02X}", static_cast<unsigned>(color[0]), static_cast<unsigned>(color[1]), static_cast<unsigned>(color[2]));
}

std::array<float, 3> srgb(Color color) {
    return {static_cast<float>(color[0]) / 255.0f, static_cast<float>(color[1]) / 255.0f, static_cast<float>(color[2]) / 255.0f};
}

Value rounded(double value) {
    double nearest = std::round(value * 10'000.0) / 10'000.0;
    // Not -0.
    return Value(nearest + 0.0);
}

Value point_value(std::array<double, 2> point) {
    Value::Array array;
    array.values.reserve(2);
    array.values.push_back(rounded(point[0]));
    array.values.push_back(rounded(point[1]));
    return array;
}

Value points_value(std::span<const std::array<double, 2>> points) {
    Value::Array array;
    array.values.reserve(points.size());
    for (std::array<double, 2> point : points) {
        array.values.push_back(point_value(point));
    }
    return array;
}

} // namespace veehiicuul::files
