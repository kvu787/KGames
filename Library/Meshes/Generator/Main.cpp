// Makes the car, Car.h in Meshes' generated folder, which Meshes.cpp
// includes, from the model that Blender exported for Veehiicuul,
// Program/Veehiicuul/Models/SlopeCar.glb: its vertices, its triangles'
// indices, and a check of the constant for each of its parts' materials. So
// the car here is always the one in that file.
//
//     MeshesGenerator.exe <the Car.h to write>
//
// Meshes.vcxproj runs it as Meshes builds, when the model or this program
// has changed.

#include "Base/Files.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Gltf/Gltf.h"

#include "Folders.h"

#include <algorithm>
#include <array>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <format>
#include <print>
#include <string>
#include <string_view>
#include <vector>

namespace {

// From the repository's root.
constexpr std::string_view Model = "Program/Veehiicuul/Models/SlopeCar.glb";

// Each part's constant, and its material's name in the file. A constant's
// value is its place here, which the vertices' material indices use. All
// four wheels are one part, and so are both headlights.
struct Part {
    std::string_view constant;
    std::string_view material;
};
constexpr std::array<Part, 5> Parts{{
    {"Axles", "SlopeCarAxles"},
    {"Body", "SlopeCarBodyBlue"},
    {"Cabin", "SlopeCarCabinBlue"},
    {"Headlights", "SlopeCarHeadlights"},
    {"Wheels", "SlopeCarWheels"},
}};

// Fails the build with the message, which MSBuild shows as an error.
[[noreturn]] void fail(std::string_view message) {
    std::println(stderr, "MeshesGenerator: error: {}", message);
    std::exit(1);
}

// A number as C++ reads back as the same float: the fewest digits that do,
// which is how Rust's {:?} writes it, and f.
std::string literal(float value) {
    return base::debug(value) + "f";
}

std::string literal(const std::array<float, 3>& values) {
    return std::format("{{{}, {}, {}}}", literal(values[0]), literal(values[1]), literal(values[2]));
}

} // namespace

int main() {
    base::fail_quietly();
    std::vector<std::string> arguments = base::arguments();
    if (arguments.size() != 1) {
        fail("MeshesGenerator takes one argument, the file to write, like Car.h");
    }
    std::filesystem::path output = base::path(arguments[0]);
    std::filesystem::path path = base::path(folders::Repository);
    for (std::string_view part : base::split(Model, '/')) {
        path /= base::path(part);
    }
    std::expected<std::vector<std::uint8_t>, std::string> bytes = base::read(path);
    if (!bytes) {
        fail(std::format("Couldn't read {}: {}", Model, bytes.error()));
    }
    // Git LFS keeps the model, and leaves a small text file in its place until
    // it checks the model out, as in a new worktree.
    std::string_view pointer = "version https://git-lfs.github.com";
    if (std::string_view(reinterpret_cast<const char*>(bytes->data()), bytes->size()).starts_with(pointer)) {
        fail(std::format("{} is a Git LFS pointer file, not the model. Run `git lfs checkout` to put the model there, then build again.", Model));
    }
    std::expected<gltf::Model, std::string> model = gltf::parse(*bytes);
    if (!model) {
        fail(std::format("{}: {}", Model, model.error()));
    }

    // Each of the file's materials' part.
    std::vector<std::uint32_t> parts;
    for (const gltf::Material& material : model->material_list) {
        auto part = std::ranges::find_if(Parts, [&](const Part& candidate) { return candidate.material == material.name; });
        if (part != Parts.end()) {
            parts.push_back(static_cast<std::uint32_t>(part - Parts.begin()));
        } else {
            fail(std::format("{} has a material, {}, that isn't one of the car's parts", Model, material.name));
        }
    }
    for (size_t part = 0; part < Parts.size(); part++) {
        if (!std::ranges::contains(parts, static_cast<std::uint32_t>(part))) {
            fail(std::format("{} has no material {}", Model, Parts[part].material));
        }
    }

    std::string code;
    auto line = [&](std::string_view text) {
        code += text;
        code.push_back('\n');
    };
    line("// Made by Library/Meshes/Generator from Program/Veehiicuul/Models/SlopeCar.glb.");
    line("");
    line("#pragma once");
    line("");
    line("#include \"Meshes/Meshes.h\"");
    line("");
    line("#include <array>");
    line("#include <cstdint>");
    line("");
    line("namespace meshes::car::generated {");
    line("");
    line("// Each part's place among the generator's parts, which the vertices'");
    line("// material indices use, and which Meshes.h must give its constant.");
    for (size_t part = 0; part < Parts.size(); part++) {
        line(std::format("static_assert({} == {});", Parts[part].constant, part));
    }
    line(std::format("static_assert(MaterialCount == {});", Parts.size()));
    line("");
    // Each number has the fewest digits that read back as the same float.
    line(std::format("inline constexpr std::array<simple_paint::Vertex, {}> Vertices{{{{", model->positions.size()));
    for (size_t vertex = 0; vertex < model->positions.size() && vertex < model->normals.size() && vertex < model->materials.size(); vertex++) {
        line(std::format("    {{{}, {}, {}}},", literal(model->positions[vertex]), literal(model->normals[vertex]), parts[model->materials[vertex]]));
    }
    line("}};");
    line("");
    line(std::format("inline constexpr std::array<std::uint32_t, {}> Indices{{", model->indices.size()));
    for (size_t row = 0; row < model->indices.size(); row += 16) {
        std::string text = "   ";
        for (size_t index = row; index < std::min(row + 16, model->indices.size()); index++) {
            text += std::format(" {},", model->indices[index]);
        }
        line(text);
    }
    line("};");
    line("");
    line("} // namespace meshes::car::generated");

    if (std::expected<void, std::string> written = base::write(output, code); !written) {
        fail(std::format("Couldn't write {}: {}", base::display(output), written.error()));
    }
    return 0;
}
