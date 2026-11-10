// The models that vehicles and decorations are made of: the .glb files in
// Models, which Blender exports, with Library/Gltf. A model's materials are
// its slots, each painted with a SimplePaint material. What Blender says a
// slot looks like is its first paint: the SimplePaint material in its
// custom property "SimplePaint", in Library/PaintSettings' format, if it has
// one, or else one with its base color. The game can paint over that, for
// the model, in Models/<Name>.json, and for a vehicle, in Vehicles.json.
//
//     {
//         "Paint": {
//             "SlopeCarAxles": { "BaseColor": [0.2159, 0.2159, 0.2159], ... }
//         }
//     }

#pragma once

#include "Gltf/Gltf.h"
#include "SimplePaint/Geometry.h"
#include "SimplePaint/Material.h"
#include "Veehiicuul/Collision.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace veehiicuul::models {

// The most slots a model has, as SIMPLE_PAINT_MATERIAL_COUNT in
// VeehiicuulLibrary.vcxproj says.
inline constexpr size_t MostSlots = 16;

// A model's paint: some of its slots' SimplePaint materials, by the slots'
// names. The slots it leaves out keep their paint from before.
using Paint = std::vector<std::pair<std::string, simple_paint::Parameters>>;

struct Model {
    std::string name;
    std::vector<std::string> slots;
    // Each slot's paint, as Blender says.
    std::vector<simple_paint::Parameters> blender_paints;
    std::vector<simple_paint::Vertex> vertices;
    std::vector<std::uint32_t> indices;
    // The rectangle around it seen from above.
    collision::Footprint footprint;

    Model(std::string model_name, std::vector<std::string> model_slots, std::vector<simple_paint::Parameters> paints, std::vector<simple_paint::Vertex> model_vertices,
          std::vector<std::uint32_t> model_indices, collision::Footprint model_footprint);
    // A model is read once, and never copied.
    Model(const Model&) = delete;
    Model& operator=(const Model&) = delete;
    Model(Model&&) = default;
    Model& operator=(Model&&) = default;

    static std::expected<Model, std::string> read(std::string_view model_name, const gltf::Model& model);

    // Its slots' materials, with these paints over Blender's, each over
    // the ones before it.
    std::vector<simple_paint::Parameters> materials(std::span<const Paint* const> paints) const;
};

// A slot's paint from its base color, which glTF says as linear RGB: a
// little shaded, with the color itself where the surface faces sideways.
simple_paint::Parameters paint_from_color(std::array<float, 4> color);

struct Models {
    std::vector<Model> list;

    explicit Models(std::vector<Model> models);
    Models(const Models&) = delete;
    Models& operator=(const Models&) = delete;
    Models(Models&&) = default;
    Models& operator=(Models&&) = default;

    // Every .glb file in a folder, by name.
    static std::expected<Models, std::string> load(const std::filesystem::path& folder);

    std::optional<size_t> find(std::string_view name) const;

    std::vector<std::string_view> names() const;
};

// A paint, from its object of slots' materials.
std::expected<Paint, std::string> parse_paint(const json::Value& value, std::string_view name);

json::Value paint_value(const Paint& paint);

// A model's paint file's text, like Models/SlopeCar.json.
std::string paint_file(const Paint& paint);

std::expected<Paint, std::string> parse_paint_file(std::string_view text);

// Sets a slot's material in a paint, in the slots' order.
void set_slot(Paint& paint, std::span<const std::string> slots, std::string_view slot, const simple_paint::Parameters& parameters);

} // namespace veehiicuul::models
