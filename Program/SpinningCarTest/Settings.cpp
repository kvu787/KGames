#include "SpinningCarTest/Settings.h"

#include "Base/Numbers.h"
#include "Base/Sum.h"
#include "PaintSettings/PaintSettings.h"
#include "Json/Json.h"

#include <cmath>
#include <cstddef>
#include <expected>
#include <format>
#include <string>
#include <utility>

namespace spinning_car_test::settings {

using json::Value;
using simple_paint::Parameters;

namespace {

// The sphere's resolution, whose members that are missing keep theirs from
// `defaults`. It's the defaults unless every member is right.
std::expected<std::array<std::uint32_t, 2>, std::string> parse_resolution(const Value& value, std::array<std::uint32_t, 2> defaults) {
    const Value::Object* object = base::as<Value::Object>(value);
    if (object == nullptr) {
        return std::unexpected(std::format("must be an object with UResolution and VResolution, not {}", value.describe()));
    }
    std::array<std::uint32_t, 2> resolution = defaults;
    for (const auto& [member, member_value] : object->members) {
        size_t index = 0;
        // The fewest and the most that the member may be.
        std::pair<std::int64_t, std::int64_t> range;
        if (member == "UResolution") {
            index = 0;
            range = {4, 512};
        } else if (member == "VResolution") {
            index = 1;
            range = {2, 512};
        } else {
            return std::unexpected(std::format("has {}, which isn't one of UResolution and VResolution", member));
        }
        const Value::Number* number = base::as<Value::Number>(member_value);
        if (number != nullptr && number->value - std::trunc(number->value) == 0.0 && range.first <= base::saturating_cast<std::int64_t>(number->value) &&
            base::saturating_cast<std::int64_t>(number->value) <= range.second) {
            resolution[index] = base::saturating_cast<std::uint32_t>(number->value);
        } else {
            return std::unexpected(std::format("{} must be a whole number from {} to {}, not {}", member, range.first, range.second, member_value.describe()));
        }
    }
    return resolution;
}

} // namespace

SceneSettings SceneSettings::read(user_interface::Settings& settings) {
    SceneSettings defaults;
    SceneSettings scene;
    for (size_t index = 0; index < scene.materials.size(); index++) {
        const Parameters& default_ = defaults.materials[index];
        scene.materials[index] = settings.value(MaterialNames[index], paint_settings::to_value(default_), [&](const Value& value) {
            // The settings go on after what's wrong, so it ends without
            // the sentence's period.
            return paint_settings::parse(value, default_).transform_error([](std::string error) {
                while (error.ends_with('.')) {
                    error.pop_back();
                }
                return error;
            });
        });
    }
    auto [u, v] = defaults.sphere_resolution;
    Value sphere = json::object({{"UResolution", u}, {"VResolution", v}});
    scene.sphere_resolution = settings.value("Sphere", std::move(sphere), [&](const Value& value) { return parse_resolution(value, defaults.sphere_resolution); });
    return scene;
}

} // namespace spinning_car_test::settings
