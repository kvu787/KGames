// Library/Gltf: reading .glb files made here, with nodes, transforms,
// indices of each size, and missing normals and materials, and the models
// that Veehiicuul reads, which Blender exported.

#include "Base/Files.h"
#include "Base/Panic.h"
#include "Gltf/Gltf.h"
#include "Testing/Testing.h"

#include "Folders.h"

#include <algorithm>
#include <array>
#include <bit>
#include <cmath>
#include <cstdint>
#include <filesystem>
#include <format>
#include <initializer_list>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

using gltf::Model;
using gltf::parse;

namespace {

void push_word(std::vector<std::uint8_t>& bytes, std::uint32_t word) {
    for (int byte = 0; byte < 4; byte++) {
        bytes.push_back(static_cast<std::uint8_t>(word >> (8 * byte)));
    }
}

// A .glb file with this JSON, and this binary chunk.
std::vector<std::uint8_t> glb(std::string_view json_text, std::span<const std::uint8_t> binary_chunk) {
    std::vector<std::uint8_t> json(json_text.begin(), json_text.end());
    while (json.size() % 4 != 0) {
        json.push_back(' ');
    }
    std::vector<std::uint8_t> binary(binary_chunk.begin(), binary_chunk.end());
    while (binary.size() % 4 != 0) {
        binary.push_back(0);
    }
    size_t length = 12 + 8 + json.size() + 8 + binary.size();
    std::vector<std::uint8_t> bytes;
    for (std::uint32_t word :
         {std::uint32_t{0x46546C67}, std::uint32_t{2}, static_cast<std::uint32_t>(length), static_cast<std::uint32_t>(json.size()), std::uint32_t{0x4E4F534A}}) {
        push_word(bytes, word);
    }
    bytes.append_range(json);
    push_word(bytes, static_cast<std::uint32_t>(binary.size()));
    push_word(bytes, 0x004E4942);
    bytes.append_range(binary);
    return bytes;
}

std::vector<std::uint8_t> floats(std::initializer_list<float> values) {
    std::vector<std::uint8_t> bytes;
    for (float value : values) {
        push_word(bytes, std::bit_cast<std::uint32_t>(value));
    }
    return bytes;
}

// One triangle in the XY plane, facing +Z, with its positions, normals and
// 16-bit indices, and the nodes and materials that the test gives.
std::vector<std::uint8_t> triangle(std::string_view nodes, std::string_view materials, std::string_view primitive) {
    std::vector<std::uint8_t> binary = floats({0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f});
    binary.append_range(floats({0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 1.0f}));
    for (std::uint16_t index : {std::uint16_t{0}, std::uint16_t{1}, std::uint16_t{2}}) {
        binary.push_back(static_cast<std::uint8_t>(index));
        binary.push_back(static_cast<std::uint8_t>(index >> 8));
    }
    std::string json = std::format(
        R"({{
            "asset": {{ "version": "2.0" }},
            "scene": 0,
            "scenes": [{{ "nodes": [0] }}],
            "nodes": {},
            "materials": {},
            "meshes": [{{ "primitives": [{}] }}],
            "buffers": [{{ "byteLength": {} }}],
            "bufferViews": [
                {{ "buffer": 0, "byteOffset": 0, "byteLength": 36 }},
                {{ "buffer": 0, "byteOffset": 36, "byteLength": 36 }},
                {{ "buffer": 0, "byteOffset": 72, "byteLength": 6 }}
            ],
            "accessors": [
                {{ "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3" }},
                {{ "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3" }},
                {{ "bufferView": 2, "componentType": 5123, "count": 3, "type": "SCALAR" }}
            ]
        }})",
        nodes, materials, primitive, binary.size());
    return glb(json, binary);
}

constexpr std::string_view Primitive = R"({ "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2, "material": 0 })";
constexpr std::string_view Materials = R"([{ "name": "Red", "pbrMetallicRoughness": { "baseColorFactor": [1, 0, 0, 1] }, "extras": { "Shiny": 1 } }])";

bool near_(std::array<float, 3> actual, std::array<float, 3> expected) {
    for (size_t axis = 0; axis < 3; axis++) {
        if (!(std::abs(actual[axis] - expected[axis]) < 1e-5f)) {
            return false;
        }
    }
    return true;
}

// The text with every `from` in it replaced by `to`: str::replace.
std::string replace(std::string_view text, std::string_view from, std::string_view to) {
    std::string replaced;
    while (true) {
        size_t at = text.find(from);
        replaced += text.substr(0, at);
        if (at == std::string_view::npos) {
            return replaced;
        }
        replaced += to;
        text.remove_prefix(at + from.size());
    }
}

// What's wrong with a file that parse must not read.
std::string problem_of(std::span<const std::uint8_t> bytes) {
    std::expected<Model, std::string> model = parse(bytes);
    CHECK(!model.has_value());
    return model.error();
}

Model model(std::string_view name) {
    std::filesystem::path path = base::path(folders::Project) / base::path(std::format("../../Program/Veehiicuul/Models/{}.glb", name));
    std::expected<Model, std::string> loaded = gltf::load(path);
    if (!loaded) {
        base::panic(loaded.error());
    }
    return std::move(*loaded);
}

} // namespace

TEST(reads_a_triangle_with_its_material) {
    Model model = parse(triangle(R"([{ "mesh": 0 }])", Materials, Primitive)).value();
    CHECK_EQ(model.positions, (std::vector<std::array<float, 3>>{{0.0f, 0.0f, 0.0f}, {1.0f, 0.0f, 0.0f}, {0.0f, 1.0f, 0.0f}}));
    CHECK_EQ(model.normals, std::vector(3, std::array{0.0f, 0.0f, 1.0f}));
    CHECK_EQ(model.indices, (std::vector<std::uint32_t>{0, 1, 2}));
    CHECK_EQ(model.materials, (std::vector<std::uint32_t>{0, 0, 0}));
    CHECK_EQ(model.material_list[0].name, "Red");
    CHECK_EQ(model.material_list[0].base_color, (std::array{1.0f, 0.0f, 0.0f, 1.0f}));
    CHECK_EQ(model.material_list[0].extras, std::optional(json::object({{"Shiny", 1.0}})));
}

// A child node's transform is its parent's, then its own: here a parent
// moved along X, and a child turned 90 degrees around Y and scaled by 2.
// Its normal turns with it, and keeps its length.
TEST(applies_each_node_s_transform_and_its_parents) {
    std::string_view nodes = R"([
        { "translation": [10, 0, 0], "children": [1] },
        { "mesh": 0, "rotation": [0, 0.70710678, 0, 0.70710678], "scale": [2, 2, 2] }
    ])";
    Model model = parse(triangle(nodes, Materials, Primitive)).value();
    CHECK(near_(model.positions[1], {10.0f, 0.0f, -2.0f}), "{}", testing::describe(model.positions));
    CHECK(near_(model.positions[2], {10.0f, 2.0f, 0.0f}), "{}", testing::describe(model.positions));
    CHECK(near_(model.normals[0], {1.0f, 0.0f, 0.0f}), "{}", testing::describe(model.normals));
    // A scale that mirrors reverses the winding, so the front stays out.
    Model mirrored = parse(triangle(R"([{ "mesh": 0, "scale": [-1, 1, 1] }])", Materials, Primitive)).value();
    CHECK_EQ(mirrored.indices, (std::vector<std::uint32_t>{0, 2, 1}));
    CHECK(near_(mirrored.normals[0], {0.0f, 0.0f, 1.0f}));
}

// Without normals, each triangle is flat, and without a material, it gets
// one called Default.
TEST(makes_flat_normals_and_a_default_material) {
    std::string_view primitive = R"({ "attributes": { "POSITION": 0 }, "indices": 2 })";
    Model model = parse(triangle(R"([{ "mesh": 0 }])", "[]", primitive)).value();
    CHECK_EQ(model.normals, std::vector(3, std::array{0.0f, 0.0f, 1.0f}));
    CHECK_EQ(model.material_list.size(), 1);
    CHECK_EQ(model.material_list[0].name, "Default");
    CHECK_EQ(model.material_list[0].base_color, (std::array{1.0f, 1.0f, 1.0f, 1.0f}));
}

TEST(says_what_is_wrong_with_a_file) {
    std::string_view lines = R"({ "attributes": { "POSITION": 0 }, "mode": 1 })";
    std::vector<std::pair<std::vector<std::uint8_t>, std::string_view>> cases{
        {{'g', 'l', 'T', 'F'}, "is cut short"},
        {std::vector<std::uint8_t>(20, 0), "isn't a binary glTF file"},
        {triangle(R"([{ "mesh": 0 }])", Materials, lines), "has a primitive that isn't triangles"},
        {triangle(R"([{ "mesh": 0 }])", "[]", Primitive), "has no material 0"},
        {triangle(R"([{ "mesh": 0, "scale": [0, 1, 1] }])", Materials, Primitive), "has a node whose transform flattens its mesh"},
        {triangle(R"([{ "children": [0] }])", Materials, Primitive), "has node 0 in more than one place, or in a loop"},
    };
    for (const auto& [bytes, problem] : cases) {
        CHECK_EQ(problem_of(bytes), problem);
    }
}

// A character cut short after its second or its third byte isn't UTF-8, any
// more than one cut short after its first, as std::str::from_utf8 has it in
// Rust: a material's name would be bytes that aren't text.
TEST(says_that_json_with_a_character_cut_short_isn_t_utf_8) {
    for (std::string_view name : {std::string_view("Red\xC3"), std::string_view("Red\xE2\x82"), std::string_view("Red\xF0\x9F\x98")}) {
        std::string materials = std::format(R"([{{ "name": "{}" }}])", name);
        CHECK_EQ(problem_of(triangle(R"([{ "mesh": 0 }])", materials, Primitive)), "has JSON that isn't UTF-8", "after {} bytes", name.size() - 3);
    }
    // The whole character is.
    std::string_view whole = "[{ \"name\": \"Red\xE2\x82\xAC\" }]";
    CHECK_EQ(parse(triangle(R"([{ "mesh": 0 }])", whole, Primitive)).value().material_list[0].name, "Red\xE2\x82\xAC");
}

// The car, as Blender exported it: one mesh with five materials, 3 units
// wide and about 6 long, facing +Z, with its wheels on the ground.
TEST(reads_the_car_that_blender_exported) {
    Model car = model("SlopeCar");
    std::vector<std::string_view> names;
    for (const gltf::Material& material : car.material_list) {
        names.push_back(material.name);
    }
    CHECK_EQ(names, (std::vector<std::string_view>{"SlopeCarAxles", "SlopeCarBodyBlue", "SlopeCarCabinBlue", "SlopeCarHeadlights", "SlopeCarWheels"}));
    auto [min_, max_] = car.bounds();
    CHECK(near_(min_, {-1.5f, 0.0f, -3.0f}) && near_(max_, {1.5f, 2.3902397f, 3.157522f}), "{} {}", testing::describe(min_), testing::describe(max_));
    CHECK_EQ(car.indices.size(), 5652);
    for (const std::array<float, 3>& normal : car.normals) {
        float length = std::sqrt(normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]);
        CHECK(std::abs(length - 1.0f) < 1e-4f);
    }
}

// The cone, whose parts are under one empty node, each moved up from it.
TEST(reads_the_cone_s_parts_where_their_nodes_put_them) {
    Model cone = model("BigCone");
    CHECK_EQ(cone.material_list.size(), 4);
    auto [min_, max_] = cone.bounds();
    CHECK(std::abs(min_[1]) < 1e-4f, "its base is on the ground: {}", testing::describe(min_));
    CHECK(max_[1] >= 5.9f && max_[1] < 6.1f, "{}", testing::describe(max_));
    // The beak is at the front, toward +Z.
    auto found = std::ranges::find_if(cone.material_list, [](const gltf::Material& material) { return material.name == "BigConeBeak"; });
    CHECK(found != cone.material_list.end());
    std::uint32_t beak = static_cast<std::uint32_t>(found - cone.material_list.begin());
    bool front = true;
    for (size_t vertex = 0; vertex < cone.positions.size() && vertex < cone.materials.size(); vertex++) {
        if (cone.materials[vertex] == beak) {
            front = front && cone.positions[vertex][2] > 1.0f;
        }
    }
    CHECK(front);
}

// A file can say anything about its accessors and nodes, and the reader
// says what's wrong instead of allocating, reading past the chunk, or
// overflowing the stack.
TEST(says_what_is_wrong_with_a_file_s_sizes_and_nodes) {
    auto accessors = [](std::string_view list) {
        std::string json = std::format(
            R"({{
                "asset": {{ "version": "2.0" }},
                "scenes": [{{ "nodes": [0] }}],
                "nodes": [{{ "mesh": 0 }}],
                "materials": {},
                "meshes": [{{ "primitives": [{}] }}],
                "buffers": [{{ "byteLength": 8 }}],
                "bufferViews": [{{ "buffer": 0, "byteOffset": 0, "byteLength": 8 }}],
                "accessors": {}
            }})",
            Materials, Primitive, list);
        return glb(json, std::vector<std::uint8_t>(8, 0));
    };
    auto three = [](std::string_view first) {
        return std::format(R"([{}, {}, {{ "bufferView": 0, "componentType": 5123, "count": 3, "type": "SCALAR" }}])", first, first);
    };
    struct Case {
        std::string_view accessor;
        std::string_view problem;
    };
    for (const Case& expected : {
             Case{R"({ "componentType": 5126, "count": 4294967295, "type": "VEC3" })", "has accessor 0 with 4294967295 elements, more than 16777216"},
             Case{R"({ "componentType": 5126, "count": 1e18, "type": "VEC3" })", "has accessor 0 without a count"},
             Case{R"({ "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3" })", "has accessor 0 past the end of the binary chunk"},
             Case{R"({ "bufferView": 0, "byteOffset": 4294967295, "componentType": 5126, "count": 1, "type": "VEC3" })", "has accessor 0 past the end of the binary chunk"},
         }) {
        CHECK_EQ(problem_of(accessors(three(expected.accessor))), expected.problem);
    }
    std::string_view strided = R"([{ "buffer": 0, "byteOffset": 0, "byteLength": 8, "byteStride": 4294967295 }])";
    std::vector<std::uint8_t> file = accessors(three(R"({ "bufferView": 0, "componentType": 5126, "count": 2, "type": "VEC3" })"));
    std::string json = replace(std::string_view(reinterpret_cast<const char*>(file.data()), file.size()), R"([{ "buffer": 0, "byteOffset": 0, "byteLength": 8 }])", strided);
    CHECK_EQ(problem_of(glb(std::string_view(json).substr(20, json.size() - 16 - 20), std::vector<std::uint8_t>(8, 0))), "has accessor 0 past the end of the binary chunk");

    // A chain of nodes as long as a stack would be deep.
    size_t count = 100'000;
    std::string nodes = "[";
    for (size_t node = 0; node < count; node++) {
        if (node > 0) {
            nodes.push_back(',');
        }
        if (node + 1 < count) {
            nodes += std::format(R"({{ "children": [{}] }})", node + 1);
        } else {
            nodes += R"({ "mesh": 0 })";
        }
    }
    nodes.push_back(']');
    Model model = parse(triangle(nodes, Materials, Primitive)).value();
    CHECK_EQ(model.indices.size(), 3);
    std::string_view looped = R"([{ "children": [1] }, { "children": [0], "mesh": 0 }])";
    CHECK_EQ(problem_of(triangle(looped, Materials, Primitive)), "has node 0 in more than one place, or in a loop");
}
