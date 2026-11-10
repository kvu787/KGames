// Library/Meshes: every mesh passes SimplePaint's checks, which
// simple_paint::direct3d12::Mesh makes before it uploads one.

#include "Meshes/Meshes.h"
#include "Base/Panic.h"
#include "SimplePaint/Geometry.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <array>
#include <cstdint>
#include <format>
#include <string_view>
#include <tuple>
#include <vector>

using meshes::cuboid;
using meshes::Mesh;
using meshes::plane;
using meshes::uv_sphere;
using simple_paint::validate_mesh;
namespace car = meshes::car;

namespace {

void check(const Mesh& mesh, std::string_view what) {
    if (std::expected<void, simple_paint::Error> valid = validate_mesh(mesh.vertices, mesh.indices, 1); !valid) {
        base::panic(std::format("{}: {}", what, valid.error().message));
    }
}

} // namespace

// The fewest longitudes, where the normals of a triangle are furthest apart,
// with every number of latitudes, and the others with fewer.
TEST(spheres_pass_simple_paint_s_checks) {
    for (std::uint32_t latitudes = 2; latitudes <= 512; latitudes++) {
        check(uv_sphere(4, latitudes, 0), std::format("a sphere of 4 by {}", latitudes));
    }
    for (std::uint32_t longitudes = 5; longitudes <= 16; longitudes++) {
        std::vector<std::uint32_t> counts;
        for (std::uint32_t latitudes = 2; latitudes <= 64; latitudes++) {
            counts.push_back(latitudes);
        }
        counts.push_back(512);
        for (std::uint32_t latitudes : counts) {
            check(uv_sphere(longitudes, latitudes, 0), std::format("a sphere of {} by {}", longitudes, latitudes));
        }
    }
    for (auto [longitudes, latitudes] : {std::array<std::uint32_t, 2>{64, 32}, std::array<std::uint32_t, 2>{512, 2}, std::array<std::uint32_t, 2>{512, 512}}) {
        check(uv_sphere(longitudes, latitudes, 0), std::format("a sphere of {} by {}", longitudes, latitudes));
    }
}

TEST(a_sphere_needs_4_longitudes) {
    try {
        uv_sphere(3, 4, 0);
    } catch (const base::Panic& panic) {
        CHECK(std::string_view(panic.what()).contains("a UV sphere needs 4 to 512 longitudes"), "{}", panic.what());
        return;
    }
    testing::fail("a sphere of 3 longitudes was made");
}

TEST(the_car_passes_simple_paint_s_checks_with_each_part_painted) {
    CHECK_EQ(std::tuple(car::Vertices.size(), car::Indices.size(), car::MaterialCount), std::tuple(1938, 5652, 5));
    validate_mesh(car::Vertices, car::Indices, car::MaterialCount).value();
    for (std::uint32_t material : {car::Axles, car::Body, car::Cabin, car::Headlights, car::Wheels}) {
        bool painted = std::ranges::any_of(car::Vertices, [&](const simple_paint::Vertex& vertex) { return vertex.material_index == material; });
        CHECK(painted, "no vertex of material {}", material);
    }
}

TEST(boxes_and_planes_pass_simple_paint_s_checks) {
    check(cuboid({1.0f, 2.0f, 3.0f}, 0), "a box");
    check(plane({10.0f, 4.0f}, 0), "a plane");
}
