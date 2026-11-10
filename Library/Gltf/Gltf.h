// Reads models from binary glTF files, .glb, like the ones that Blender
// exports with File > Export > glTF 2.0: every mesh of the default scene, as
// one list of triangles, with each node's transform applied, and the
// materials that its triangles use.
//
// glTF is Y up, right-handed, with a model's front toward +Z, as Blender
// exports it. Only what KGames uses is read: positions, normals, triangle
// indices, materials' names, base colors and extras, and the nodes' trees
// and transforms. Texture coordinates, textures, animations, skins, cameras
// and lights are ignored. A primitive that isn't triangles, or data in
// another file, is an error.

#pragma once

#include "Json/Json.h"

#include <array>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <optional>
#include <span>
#include <string>
#include <utility>
#include <vector>

namespace gltf {

// A material, as glTF says it: its name, its base color as linear RGBA, and
// its extras, which Blender fills with the material's custom properties.
struct Material {
    std::string name;
    std::array<float, 4> base_color{};
    std::optional<json::Value> extras;

    bool operator==(const Material&) const = default;
};

// A model's triangles: each vertex's position, normal, and the index of its
// triangle's material. Triangles are wound counterclockwise when seen from
// their front, as glTF's are.
struct Model {
    std::vector<std::array<float, 3>> positions;
    std::vector<std::array<float, 3>> normals;
    std::vector<std::uint32_t> materials;
    std::vector<std::uint32_t> indices;
    // Each material's index is its place here. A primitive without one gets
    // one called "Default", white.
    std::vector<Material> material_list;

    // The box around its positions: the smallest and the largest of each
    // coordinate.
    std::pair<std::array<float, 3>, std::array<float, 3>> bounds() const;

    bool operator==(const Model&) const = default;
};

std::expected<Model, std::string> load(const std::filesystem::path& path);

// Reads a .glb file's bytes.
std::expected<Model, std::string> parse(std::span<const std::uint8_t> bytes);

} // namespace gltf
