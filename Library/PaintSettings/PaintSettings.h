// SimplePaint materials in programs' JSON files, in one format, so that a
// material can be copied between programs:
//
//     {
//         "BaseColor": [0.107, 0.223, 0.578],
//         "Brightness": 0.5,
//         "Shift": 0.0,
//         "RotationDegrees": 0.0,
//         "DarkPoint": 0.0,
//         "LightPoint": 1.0
//     }
//
// SimplePaint itself leaves serialization to its hosts.

#pragma once

#include "SimplePaint/Material.h"
#include "Json/Json.h"

#include <cstddef>
#include <expected>
#include <string>

namespace paint_settings {

// Reads a material from a JSON object. A member that's missing keeps its
// value from `defaults`. Fails with what's wrong, including any value that
// SimplePaint rejects.
std::expected<simple_paint::Parameters, std::string> parse(const json::Value& value, const simple_paint::Parameters& defaults);

// The material as a JSON value, in the same format.
json::Value to_value(const simple_paint::Parameters& parameters);

// The material as JSON text, as `to_value` writes it, over several lines,
// with the lines after the first indented by `indent` more spaces, for
// nesting in a settings file.
std::string format(const simple_paint::Parameters& parameters, size_t indent);

} // namespace paint_settings
