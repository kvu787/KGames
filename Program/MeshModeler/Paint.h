// The paints that color the mesh: each face's material is a paint, which is a
// name and SimplePaint's parameters. The flat shader colors with the same
// paints' base colors. A document has up to MaximumPaints, which is how many
// the shaders are compiled for.

#pragma once

#include "SimplePaint/Material.h"
#include "UserInterface/Drawing/Geometry.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <string>
#include <string_view>
#include <vector>

namespace mesh_modeler::paint {

inline constexpr size_t MaximumPaints = 16;

struct Paint {
    std::string name;
    simple_paint::Parameters parameters;

    // A paint of SimplePaint's default parameters, in the color, 0xRRGGBB.
    Paint(std::string_view name, std::uint32_t color);

    Paint(std::string name, const simple_paint::Parameters& parameters);

    // The base color, as the interface shows it.
    user_interface::Color color() const;

    // A base color, kept inside what SimplePaint accepts: each channel at
    // least Margin and at most InteriorMaximum.
    void set_color(user_interface::Color color);

    // SimplePaint's constants for the paint. The interface keeps every
    // parameter in its range, so it compiles; if one didn't, the paint shows
    // as a neutral gray instead of failing the frame.
    simple_paint::GpuMaterial material() const;

    // The base color in linear light, for the flat shader.
    std::array<float, 4> linear_color() const;

    // As a document has it: { "Name": ..., "Paint": PaintSettings' object }.
    json::Value to_value() const;

    static std::expected<Paint, std::string> from_value(const json::Value& value);

    bool operator==(const Paint&) const = default;
};

// The paints that a new document starts with.
std::vector<Paint> default_paints();

// A color for a new paint, after the ones before it, from Fluent's shared
// colors.
std::uint32_t next_color(size_t count);

} // namespace mesh_modeler::paint
