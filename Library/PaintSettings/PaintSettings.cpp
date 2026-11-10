#include "PaintSettings/PaintSettings.h"

#include "Base/Sum.h"
#include "Base/Text.h"

#include <array>
#include <format>
#include <string_view>
#include <utility>

namespace paint_settings {

using json::Value;
using simple_paint::Parameters;

namespace {

constexpr std::array<std::string_view, 6> Names{
    "BaseColor", "Brightness", "Shift", "RotationDegrees", "DarkPoint", "LightPoint",
};

// The names, with ", " between them.
std::string joined_names() {
    std::string joined;
    for (std::string_view name : Names) {
        if (!joined.empty()) {
            joined += ", ";
        }
        joined += name;
    }
    return joined;
}

} // namespace

std::expected<Parameters, std::string> parse(const Value& value, const Parameters& defaults) {
    const Value::Object* object = base::as<Value::Object>(value);
    if (object == nullptr) {
        return std::unexpected(std::format("must be an object with {}, not {}", joined_names(), value.describe()));
    }
    Parameters parameters = defaults;
    for (const std::pair<std::string, Value>& entry : object->members) {
        const std::string& name = entry.first;
        const Value& member = entry.second;
        auto number = [&](const Value& item) -> std::expected<double, std::string> {
            if (const Value::Number* found = base::as<Value::Number>(item)) {
                return found->value;
            }
            return std::unexpected(std::format("{} must be a number, not {}", name, item.describe()));
        };
        if (name == "BaseColor") {
            const Value::Array* channels = base::as<Value::Array>(member);
            if (channels == nullptr || channels->values.size() != 3) {
                return std::unexpected(std::format("BaseColor must be a list of 3 numbers, red, green and blue, not {}", member.describe()));
            }
            for (size_t channel = 0; channel < 3; channel++) {
                std::expected<double, std::string> color = number(channels->values[channel]);
                if (!color) {
                    return std::unexpected(color.error());
                }
                parameters.base_color_srgb[channel] = *color;
            }
        } else if (name == "Brightness") {
            std::expected<double, std::string> brightness = number(member);
            if (!brightness) {
                return std::unexpected(brightness.error());
            }
            parameters.brightness = *brightness;
        } else if (name == "Shift") {
            std::expected<double, std::string> shift = number(member);
            if (!shift) {
                return std::unexpected(shift.error());
            }
            parameters.shift = *shift;
        } else if (name == "RotationDegrees") {
            std::expected<double, std::string> rotation_degrees = number(member);
            if (!rotation_degrees) {
                return std::unexpected(rotation_degrees.error());
            }
            parameters.rotation_degrees = *rotation_degrees;
        } else if (name == "DarkPoint") {
            std::expected<double, std::string> dark_point = number(member);
            if (!dark_point) {
                return std::unexpected(dark_point.error());
            }
            parameters.dark_point = *dark_point;
        } else if (name == "LightPoint") {
            std::expected<double, std::string> light_point = number(member);
            if (!light_point) {
                return std::unexpected(light_point.error());
            }
            parameters.light_point = *light_point;
        } else {
            return std::unexpected(std::format("{} isn't a SimplePaint setting. They're {}.", name, joined_names()));
        }
    }
    if (std::expected<void, simple_paint::Error> valid = simple_paint::validate_parameters(parameters); !valid) {
        return std::unexpected(valid.error().message);
    }
    return parameters;
}

Value to_value(const Parameters& parameters) {
    return json::object({
        {"BaseColor", parameters.base_color_srgb},
        {"Brightness", parameters.brightness},
        {"Shift", parameters.shift},
        {"RotationDegrees", parameters.rotation_degrees},
        {"DarkPoint", parameters.dark_point},
        {"LightPoint", parameters.light_point},
    });
}

std::string format(const Parameters& parameters, size_t indent) {
    std::string outer(indent, ' ');
    std::string text;
    std::string json = to_value(parameters).to_json();
    size_t index = 0;
    for (std::string_view line : base::lines(json)) {
        if (index > 0) {
            text += "\n";
            text += outer;
        }
        text += line;
        index += 1;
    }
    return text;
}

} // namespace paint_settings
