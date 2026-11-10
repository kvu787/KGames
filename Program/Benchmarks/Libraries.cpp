#include "Benchmarks/Libraries.h"

#include "Base/Files.h"
#include "Base/Panic.h"
#include "Gltf/Gltf.h"
#include "Math/Math.h"
#include "PolygonMesh/PolygonMesh.h"
#include "PresentTiming/PresentTiming.h"
#include "Json/Json.h"

#include "Folders.h"

#include <array>
#include <cstdint>
#include <filesystem>
#include <format>
#include <numbers>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>

namespace benchmarks::libraries {

// With the two colons in front: json, math and present_timing are also the
// names of this file's groups.
using ::polygon_mesh::Mesh;
using ::polygon_mesh::Shading;
using ::polygon_mesh::Triangles;
using ::present_timing::events::Event;
using ::present_timing::events::InFrame;
using ::present_timing::events::Kind;
using ::present_timing::events::Runtime;
using ::present_timing::tracker::Tracker;
namespace primitives = ::polygon_mesh::primitives;

namespace {

// Veehiicuul's folder, whose models and scenes are the inputs.
std::filesystem::path veehiicuul() {
    return base::path(std::string(folders::Project) + "/../Veehiicuul");
}

// A torus of 1152 quads, like a model a modeler would subdivide.
Mesh torus() {
    return primitives::torus(1.0f, 0.3f, 48, 24);
}

} // namespace

std::vector<Measurement> meshes(Budget budget) {
    std::vector<Measurement> all;
    Mesh cube = primitives::cube(2.0f);
    for (std::uint32_t levels = 1; levels <= 4; levels++) {
        size_t faces = cube.catmull_clark(levels).mesh.face_count();
        all.push_back(measure(std::format("mesh/catmull-clark/cube/{}", levels), "subdivide", budget, 1.0, [&] {
                          black_box(cube.catmull_clark(levels));
                      }).with_note(std::format("{} faces out", faces)));
    }
    Mesh torus_mesh = torus();
    for (std::uint32_t levels = 1; levels <= 2; levels++) {
        size_t faces = torus_mesh.catmull_clark(levels).mesh.face_count();
        all.push_back(measure(std::format("mesh/catmull-clark/torus/{}", levels), "subdivide", budget, 1.0, [&] {
                          black_box(torus_mesh.catmull_clark(levels));
                      }).with_note(std::format("{} faces out", faces)));
    }
    Mesh smooth = torus_mesh.catmull_clark(2).mesh;
    size_t faces = smooth.face_count();
    all.push_back(measure("mesh/triangulate/smooth", "triangulate", budget, 1.0, [&] {
                      black_box(smooth.triangulate(Shading::Smooth{std::numbers::pi_v<float>}));
                  }).with_note(std::format("{} quads", faces)));
    all.push_back(measure("mesh/triangulate/flat", "triangulate", budget, 1.0, [&] { black_box(smooth.triangulate(Shading::Flat{})); }).with_note(std::format("{} quads", faces)));
    all.push_back(measure("mesh/topology", "build", budget, 1.0, [&] { black_box(smooth.topology()); }).with_note(std::format("{} quads", faces)));
    Triangles triangles = smooth.triangulate(Shading::Flat{});
    size_t count = triangles.indices.size() / 3;
    all.push_back(measure("mesh/ray-cast/triangles", "cast", budget, 1.0, [&] {
                      black_box(triangles.ray_cast({0.3f, 5.0f, 0.2f}, {0.0f, -1.0f, 0.0f}));
                  }).with_note(std::format("{} triangles", count)));
    all.push_back(
        measure("mesh/ray-cast/mesh", "cast", budget, 1.0, [&] { black_box(smooth.ray_cast({0.3f, 5.0f, 0.2f}, {0.0f, -1.0f, 0.0f})); }).with_note(std::format("{} quads", faces)));
    all.push_back(measure("mesh/primitive/uv-sphere", "make", budget, 1.0, [&] { black_box(primitives::uv_sphere(1.0f, 64, 32)); }));
    all.push_back(measure("mesh/edges", "collect", budget, 1.0, [&] { black_box(smooth.edges()); }).with_note(std::format("{} quads", faces)));
    return all;
}

std::vector<Measurement> models(Budget budget) {
    std::vector<Measurement> all;
    for (std::string_view name : {"SlopeCar", "BigCone"}) {
        std::filesystem::path path = veehiicuul() / "Models" / std::format("{}.glb", name);
        std::expected<std::vector<std::uint8_t>, std::string> bytes = base::read(path);
        if (!bytes) {
            base::panic(std::format("Couldn't read {}: {}", base::display(path), bytes.error()));
        }
        size_t triangles = gltf::parse(*bytes).value().indices.size() / 3;
        all.push_back(measure(std::format("gltf/parse/{}", name), "parse", budget, 1.0, [&] {
                          black_box(gltf::parse(*bytes).value());
                      }).with_note(std::format("{} KB, {} triangles", bytes->size() / 1024, triangles)));
    }
    return all;
}

std::vector<Measurement> json(Budget budget) {
    std::vector<Measurement> all;
    std::filesystem::path path = veehiicuul() / "Scenes" / "Ribeye.json";
    std::expected<std::string, std::string> text = base::read_to_string(path);
    if (!text) {
        base::panic(std::format("Couldn't read {}: {}", base::display(path), text.error()));
    }
    ::json::Value value = ::json::parse(*text).value();
    all.push_back(measure("json/parse/scene", "parse", budget, 1.0, [&] { black_box(::json::parse(*text).value()); }).with_note(std::format("{} KB", text->size() / 1024)));
    all.push_back(measure("json/write/scene", "write", budget, 1.0, [&] { black_box(value.to_json()); }).with_note(std::format("{} KB", text->size() / 1024)));
    return all;
}

std::vector<Measurement> math(Budget budget) {
    using ::math::Matrix;
    std::vector<Measurement> all;
    Matrix a = Matrix::rotation_y(0.3f) * Matrix::translation(1.0f, 2.0f, 3.0f);
    Matrix b = Matrix::scaling(2.0f, 2.0f, 2.0f) * Matrix::rotation_x(0.1f);
    all.push_back(measure("math/matrix/multiply", "multiply", budget, 1000.0, [&] {
        Matrix m = a;
        for (int index = 0; index < 1000; index++) {
            m = black_box(m * b);
        }
        black_box(m);
    }));
    std::vector<std::array<float, 3>> points;
    points.reserve(10'000);
    for (int index = 0; index < 10'000; index++) {
        points.push_back({static_cast<float>(index), static_cast<float>(index % 7), static_cast<float>(index % 13)});
    }
    all.push_back(measure("math/matrix/transform-points", "point", budget, 10'000.0, [&] {
                      for (const std::array<float, 3>& point : points) {
                          black_box(a.transform_point(point));
                      }
                  }).with_note("10000 points"));
    return all;
}

namespace {

constexpr std::uint32_t Process = 100;
constexpr std::uint32_t Thread = 7;
constexpr std::uint64_t SwapChain = 0xABC;
constexpr std::uint64_t Window = 0x99;
constexpr std::uint32_t AllowTearing = 0x200;
constexpr std::uint32_t FlipWaitComplete = 11;

Event event(std::int64_t time, std::uint32_t process, std::uint32_t thread, Kind kind) {
    return Event{.time = time, .process = process, .thread = thread, .kind = std::move(kind)};
}

// The events of one present that the display took at once, as the tracker's
// tests make them: the Present call, its token and queue packet, the token
// going in frame, the flip, and the display's interrupt.
void torn(Tracker& tracker, std::int64_t start, std::uint32_t sequence) {
    std::tuple<std::uint64_t, std::uint64_t, std::uint64_t> key{1, std::uint64_t{sequence}, 1};
    for (auto& [offset, kind] : std::array<std::pair<std::int64_t, Kind>, 6>{{
             {0, Kind::PresentStart{.runtime = Runtime::Dxgi, .swap_chain = SwapChain, .flags = AllowTearing, .sync_interval = 0}},
             {1, Kind::TokenCompositionSurfaceObject{.key = key}},
             {2, Kind::PresentHistoryStart{.token = 1000 + std::uint64_t{sequence}, .model = 3, .token_data = 0}},
             {3, Kind::QueuePacketStart{.packet_type = 0, .submit_sequence = sequence, .context = 0x5, .present = true}},
             {4, Kind::PresentInfo{.window = Window}},
             {5, Kind::PresentStop{.runtime = Runtime::Dxgi, .result = 0}},
         }}) {
        tracker.handle(event(start + offset, Process, Thread, std::move(kind)));
    }
    tracker.handle(event(start + 6, 4, 4, Kind::TokenStateChanged{.key = key, .state = InFrame, .independent_flip = true}));
    tracker.handle(event(start + 10, 0, 0, Kind::MmioFlipMultiPlaneOverlay{.submit_sequence = sequence, .status = FlipWaitComplete}));
    tracker.handle(event(start + 20, 0, 0,
                         Kind::SyncDpcMultiPlane{
                             .planes = {0x1000},
                             .flip_sequences = {std::uint64_t{sequence} << 32},
                         }));
}

} // namespace

std::vector<Measurement> present_timing(Budget budget) {
    Tracker tracker(Process);
    std::uint32_t sequence = 0;
    std::vector<Measurement> all;
    all.push_back(measure("present-timing/tracker/present", "present", budget, 1.0, [&] {
                      sequence += 1;
                      torn(tracker, std::int64_t{sequence} * 100, sequence);
                      black_box(tracker.take());
                  }).with_note("9 events a present"));
    return all;
}

} // namespace benchmarks::libraries
