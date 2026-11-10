// The vehicles, from Vehicles.json. Each is a model, painted its own way,
// so that it looks like no other, with its own acceleration and footprint,
// the same on every track. Its paint goes over its model's, slot by slot.
//
//     {
//         "Vehicles": [
//             {
//                 "Name": "Green",
//                 "Model": "SlopeCar",
//                 "Acceleration": { "Forward": 10, "Reverse": 100, "Left": 40, "Right": 40 },
//                 "Footprint": { "Left": 1.5, "Right": 1.5, "Front": 2.9925, "Back": 3 },
//                 "Paint": { "SlopeCarBodyBlue": { ... } }
//             }
//         ]
//     }

#pragma once

#include "Veehiicuul/Collision.h"
#include "Veehiicuul/Driving.h"
#include "Veehiicuul/Models.h"

#include <cstddef>
#include <expected>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

namespace veehiicuul::vehicles {

struct Vehicle {
    std::string name;
    std::string model;
    driving::Acceleration acceleration;
    collision::Footprint footprint;
    models::Paint paint;

    bool operator==(const Vehicle&) const = default;
};

struct Vehicles {
    std::vector<Vehicle> list;

    static std::expected<Vehicles, std::string> parse(std::string_view text);

    std::string to_json() const;

    std::optional<size_t> find(std::string_view name) const;

    // A name that no vehicle has yet, like "Vehicle 7".
    std::string new_name() const;

    bool operator==(const Vehicles&) const = default;
};

} // namespace veehiicuul::vehicles
