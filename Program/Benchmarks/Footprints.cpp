#include "Benchmarks/Footprints.h"

#include "Base/Files.h"
#include "Base/Panic.h"
#include "Gltf/Gltf.h"
#include "PolygonMesh/PolygonMesh.h"
#include "Json/Json.h"

#include "Folders.h"

#include <cstdint>
#include <expected>
#include <filesystem>
#include <format>
#include <numbers>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace benchmarks::footprints {

using memory::Report;
using memory::Usage;
using polygon_mesh::Mesh;
using polygon_mesh::Shading;
namespace primitives = polygon_mesh::primitives;

namespace {

// How many runs a benchmark counts at most. A memory benchmark needs only
// enough runs for the per-run numbers to be whole; it isn't looking for a
// minimum the way a timing benchmark is.
constexpr size_t Runs = 3;

// Veehiicuul's folder, whose models and scenes are the inputs, as in
// Libraries.cpp.
std::filesystem::path veehiicuul() {
    return base::path(std::string(folders::Project) + "/../Veehiicuul");
}

// A torus of 1,152 quads, like a model a modeler would subdivide.
Mesh torus() {
    return primitives::torus(1.0f, 0.3f, 48, 24);
}

} // namespace

Report libraries(Budget budget) {
    std::vector<Usage> all;
    Mesh torus_mesh = torus();
    auto [mesh, subdivided] = memory::held("memory/mesh/torus/2", "mesh", [&] { return torus_mesh.catmull_clark(2).mesh; });
    size_t faces = subdivided.face_count();
    size_t vertices = subdivided.positions.size();
    auto a_quad = [&](Usage usage, std::string_view what) {
        double bytes = static_cast<double>(usage.kept) / static_cast<double>(faces);
        return std::move(usage).with_note(std::format("{} quads, {}, {:.0f} B a quad", faces, what, bytes));
    };
    all.push_back(a_quad(std::move(mesh), std::format("{} vertices", vertices)));
    all.push_back(memory::measure("memory/mesh/catmull-clark/torus/2", "subdivide", budget, Runs, [&] {
                      black_box(torus_mesh.catmull_clark(2));
                  }).with_note(std::format("{} quads in, {} out", torus_mesh.face_count(), faces)));
    {
        auto [topology, built] = memory::held("memory/mesh/topology", "topology", [&] { return subdivided.topology(); });
        size_t edges = built.edges().size();
        all.push_back(a_quad(std::move(topology), std::format("{} edges", edges)));
    }
    all.push_back(memory::measure("memory/mesh/triangulate/smooth", "triangulate", budget, Runs, [&] {
                      black_box(subdivided.triangulate(Shading::Smooth{std::numbers::pi_v<float>}));
                  }).with_note(std::format("{} quads, with the topology built again inside", faces)));
    {
        auto [triangles, drawn] = memory::held("memory/mesh/triangles", "triangles", [&] { return subdivided.triangulate(Shading::Flat{}); });
        all.push_back(std::move(triangles).with_note(std::format("{} triangles, {} vertices", drawn.indices.size() / 3, drawn.vertices.size())));
    }

    for (std::string_view name : {"SlopeCar", "BigCone"}) {
        std::filesystem::path path = veehiicuul() / "Models" / std::format("{}.glb", name);
        std::expected<std::vector<std::uint8_t>, std::string> bytes = base::read(path);
        if (!bytes) {
            base::panic(std::format("Couldn't read {}: {}", base::display(path), bytes.error()));
        }
        auto [usage, model] = memory::held(std::format("memory/gltf/{}", name), "model", [&] { return gltf::parse(*bytes).value(); });
        all.push_back(std::move(usage).with_note(std::format("{} KB on disk, {} triangles", bytes->size() / 1024, model.indices.size() / 3)));
    }

    std::filesystem::path path = veehiicuul() / "Scenes" / "Ribeye.json";
    std::expected<std::string, std::string> text = base::read_to_string(path);
    if (!text) {
        base::panic(std::format("Couldn't read {}: {}", base::display(path), text.error()));
    }
    {
        auto [usage, value] = memory::held("memory/json/scene", "parse", [&] { return json::parse(*text).value(); });
        all.push_back(std::move(usage).with_note(std::format("{} KB of text", text->size() / 1024)));
    }
    all.push_back(memory::measure("memory/json/parse/scene", "parse", budget, Runs, [&] {
                      black_box(json::parse(*text).value());
                  }).with_note(std::format("{} KB of text", text->size() / 1024)));
    return Report::of(std::move(all));
}

} // namespace benchmarks::footprints
