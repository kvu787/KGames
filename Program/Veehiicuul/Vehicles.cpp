#include "Veehiicuul/Vehicles.h"

#include "Base/Sum.h"
#include "Base/Text.h"
#include "Veehiicuul/Files.h"
#include "Json/Json.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <utility>

namespace veehiicuul::vehicles {

using collision::Footprint;
using driving::Acceleration;
using json::Value;
using models::Paint;

namespace {

std::expected<Vehicle, std::string> vehicle(const Value& value) {
    if (std::expected<void, std::string> known = files::only(value, {{"Name", "Model", "Acceleration", "Footprint", "Paint"}}, "a vehicle"); !known) {
        return std::unexpected(std::move(known.error()));
    }
    std::expected<std::string, std::string> name = files::member(value, "Name", "").and_then([](const Value* found) { return files::text(*found, "Name"); });
    if (!name) {
        return std::unexpected(std::move(name.error()));
    }
    std::expected<std::string, std::string> model =
        files::member(value, "Model", std::format("{}'s ", *name)).and_then([&](const Value* found) { return files::text(*found, std::format("{}'s Model", *name)); });
    if (!model) {
        return std::unexpected(std::move(model.error()));
    }
    std::expected<const Value*, std::string> acceleration = files::member(value, "Acceleration", std::format("{}'s ", *name));
    if (!acceleration) {
        return std::unexpected(std::move(acceleration.error()));
    }
    auto rate = [&](std::string_view direction) -> std::expected<double, std::string> {
        const Value* found = (*acceleration)->get(direction);
        if (found == nullptr) {
            return std::unexpected(std::format("{} needs Acceleration.{}", *name, direction));
        }
        if (const Value::Number* number = base::as<Value::Number>(*found); number != nullptr && std::isfinite(number->value) && number->value >= 0.0) {
            return number->value;
        }
        return std::unexpected(std::format("{}'s Acceleration.{} must be 0 or more, not {}", *name, direction, found->describe()));
    };
    std::expected<const Value*, std::string> footprint = files::member(value, "Footprint", std::format("{}'s ", *name));
    if (!footprint) {
        return std::unexpected(std::move(footprint.error()));
    }
    if (std::expected<void, std::string> known = files::only(**footprint, {{"Left", "Right", "Front", "Back"}}, std::format("{}'s Footprint", *name)); !known) {
        return std::unexpected(std::move(known.error()));
    }
    auto side = [&](std::string_view which) {
        return files::member(**footprint, which, std::format("{}'s Footprint.", *name)).and_then([&](const Value* found) {
            return files::positive(*found, std::format("{}'s Footprint.{}", *name, which));
        });
    };
    std::array<double, 4> rates{};
    std::array<std::string_view, 4> directions{"Forward", "Reverse", "Left", "Right"};
    for (size_t index = 0; index < rates.size(); index++) {
        std::expected<double, std::string> found = rate(directions[index]);
        if (!found) {
            return std::unexpected(std::move(found.error()));
        }
        rates[index] = *found;
    }
    std::array<double, 4> sides{};
    std::array<std::string_view, 4> side_names{"Left", "Right", "Front", "Back"};
    for (size_t index = 0; index < sides.size(); index++) {
        std::expected<double, std::string> found = side(side_names[index]);
        if (!found) {
            return std::unexpected(std::move(found.error()));
        }
        sides[index] = *found;
    }
    Paint paint;
    if (const Value* found = value.get("Paint")) {
        std::expected<Paint, std::string> parsed = models::parse_paint(*found, std::format("{}'s Paint", *name));
        if (!parsed) {
            return std::unexpected(std::move(parsed.error()));
        }
        paint = std::move(*parsed);
    }
    return Vehicle{
        .name = std::move(*name),
        .model = std::move(*model),
        .acceleration =
            Acceleration{
                .forward = rates[0],
                .reverse = rates[1],
                .left = rates[2],
                .right = rates[3],
            },
        .footprint = Footprint::from_sides(sides[0], sides[1], sides[2], sides[3]),
        .paint = std::move(paint),
    };
}

Value vehicle_value(const Vehicle& vehicle) {
    Acceleration rates = vehicle.acceleration;
    Footprint footprint = vehicle.footprint;
    return json::object({
        {"Name", vehicle.name},
        {"Model", vehicle.model},
        {"Acceleration", json::object({
                             {"Forward", files::rounded(rates.forward)},
                             {"Reverse", files::rounded(rates.reverse)},
                             {"Left", files::rounded(rates.left)},
                             {"Right", files::rounded(rates.right)},
                         })},
        {"Footprint", json::object({
                          {"Left", files::rounded(footprint.max_[0])},
                          {"Right", files::rounded(-footprint.min_[0])},
                          {"Front", files::rounded(footprint.max_[1])},
                          {"Back", files::rounded(-footprint.min_[1])},
                      })},
        {"Paint", models::paint_value(vehicle.paint)},
    });
}

} // namespace

std::expected<Vehicles, std::string> Vehicles::parse(std::string_view text) {
    std::expected<Value, std::string> value = files::parse(text);
    if (!value) {
        return std::unexpected(std::move(value.error()));
    }
    if (std::expected<void, std::string> known = files::only(*value, {{"Vehicles"}}, "the vehicles file"); !known) {
        return std::unexpected(std::move(known.error()));
    }
    const Value* member = value->get("Vehicles");
    const Value::Array* items = member != nullptr ? base::as<Value::Array>(*member) : nullptr;
    if (items == nullptr) {
        return std::unexpected(std::string("needs Vehicles, a list of vehicles"));
    }
    std::vector<Vehicle> vehicles;
    for (size_t index = 0; index < items->values.size(); index++) {
        std::expected<Vehicle, std::string> parsed = vehicle(items->values[index]);
        if (!parsed) {
            return std::unexpected(std::format("Vehicles[{}]: {}", index, parsed.error()));
        }
        if (std::ranges::any_of(vehicles, [&](const Vehicle& other) { return other.name == parsed->name; })) {
            return std::unexpected(std::format("Vehicles[{}]: another vehicle is called {}", index, parsed->name));
        }
        vehicles.push_back(std::move(*parsed));
    }
    if (vehicles.empty()) {
        return std::unexpected(std::string("Vehicles needs at least one vehicle"));
    }
    return Vehicles{.list = std::move(vehicles)};
}

std::string Vehicles::to_json() const {
    Value::Array values;
    values.values.reserve(list.size());
    for (const Vehicle& vehicle : list) {
        values.values.push_back(vehicle_value(vehicle));
    }
    return json::object({{"Vehicles", std::move(values)}}).to_json() + "\n";
}

std::optional<size_t> Vehicles::find(std::string_view name) const {
    for (size_t index = 0; index < list.size(); index++) {
        if (base::eq_ignore_ascii_case(list[index].name, name)) {
            return index;
        }
    }
    return std::nullopt;
}

std::string Vehicles::new_name() const {
    for (size_t number = list.size() + 1;; number++) {
        std::string name = std::format("Vehicle {}", number);
        if (!find(name)) {
            return name;
        }
    }
}

} // namespace veehiicuul::vehicles
