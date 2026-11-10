#include "Veehiicuul/Settings.h"

#include "Base/Sum.h"
#include "Base/Text.h"
#include "UserInterface/Application/Settings.h"
#include "Json/Json.h"

#include <array>
#include <expected>
#include <format>
#include <string_view>
#include <utility>

namespace veehiicuul::settings {

using driving::DeadZone;
using driving::DeadZones;
using json::Value;

namespace {

std::expected<std::string, std::string> name(const Value& value) {
    if (const Value::String* text = base::as<Value::String>(value); text != nullptr && !base::trim(text->value).empty()) {
        return text->value;
    }
    return std::unexpected(std::format("must be a name, not {}", value.describe()));
}

Value dead_zones_value(const DeadZones& dead_zones) {
    auto zone = [](DeadZone of) {
        return Value(std::array<double, 2>{of.inner, of.outer});
    };
    return json::object({
        {"Sideways", zone(dead_zones.sideways)},
        {"ForwardAndBack", zone(dead_zones.forward_and_back)},
    });
}

std::string dead_zone_problem(std::string_view member, const Value& value) {
    return std::format("has {}, which must be [inner, outer], from 0 to 1 with inner below outer, not {}", member, value.describe());
}

// Like {"Sideways": [0.1, 0.95], "ForwardAndBack": [0.05, 0.95]}. A member
// that's left out keeps its default, and one that's wrong makes both keep
// theirs.
std::expected<DeadZones, std::string> dead_zones(const Value& value) {
    const Value::Object* object = base::as<Value::Object>(value);
    if (object == nullptr) {
        return std::unexpected(std::format("must be an object with Sideways and ForwardAndBack, not {}", value.describe()));
    }
    DeadZones zones;
    for (const std::pair<std::string, Value>& entry : object->members) {
        const std::string& member = entry.first;
        DeadZone* zone = nullptr;
        if (member == "Sideways") {
            zone = &zones.sideways;
        } else if (member == "ForwardAndBack") {
            zone = &zones.forward_and_back;
        } else {
            return std::unexpected(std::format("has {}, which isn't a dead zone. They're Sideways and ForwardAndBack", member));
        }
        const Value::Array* items = base::as<Value::Array>(entry.second);
        if (items == nullptr || items->values.size() != 2) {
            return std::unexpected(dead_zone_problem(member, entry.second));
        }
        const Value::Number* inner = base::as<Value::Number>(items->values[0]);
        const Value::Number* outer = base::as<Value::Number>(items->values[1]);
        if (inner == nullptr || outer == nullptr || !(0.0 <= inner->value && inner->value < outer->value && outer->value <= 1.0)) {
            return std::unexpected(dead_zone_problem(member, entry.second));
        }
        *zone = DeadZone{
            .inner = inner->value,
            .outer = outer->value,
        };
    }
    return zones;
}

} // namespace

GameSettings GameSettings::read(user_interface::Settings& settings) {
    GameSettings defaults;
    std::string scene_name = settings.value("Scene", Value(defaults.scene), name);
    std::string vehicle_name = settings.value("Vehicle", Value(defaults.vehicle), name);
    bool following = settings.choice("Camera", "Fixed", {{"Fixed", "Follow"}}) == "Follow";
    // The function above, not the settings' dead zones.
    DeadZones zones = settings.value("DeadZones", dead_zones_value(defaults.dead_zones), veehiicuul::settings::dead_zones);
    return GameSettings{
        .scene = std::move(scene_name),
        .vehicle = std::move(vehicle_name),
        .follow = following,
        .dead_zones = zones,
    };
}

std::string describe_value(const GameSettings& settings) {
    return std::format("GameSettings {{ scene: {}, vehicle: {}, follow: {}, dead_zones: {} }}", base::debug(settings.scene), base::debug(settings.vehicle), settings.follow,
                       describe_value(settings.dead_zones));
}

} // namespace veehiicuul::settings
