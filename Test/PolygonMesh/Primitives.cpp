// Library/PolygonMesh's primitives: each is valid, closed where it should be,
// wound so that its faces point out, has the Euler characteristic of its
// shape, and is quads where it can be.

#include "Base/Panic.h"
#include "Common.h"
#include "Math/Math.h"
#include "PolygonMesh/PolygonMesh.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <initializer_list>
#include <limits>
#include <numbers>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <vector>

using common::assert_close;
using common::assert_valid;
using common::sides_histogram;
using common::signed_volume;
using math::dot;
using math::subtract;
using math::Vector3;
using polygon_mesh::Face;
using polygon_mesh::Mesh;
using polygon_mesh::Topology;
namespace primitives = polygon_mesh::primitives;

namespace {

constexpr float Pi = std::numbers::pi_v<float>;

// What every closed primitive must be: valid, without boundary or non-manifold
// edges, of Euler characteristic `euler`, enclosing a positive volume.
void check_closed(const Mesh& mesh, std::int64_t euler, std::string_view what) {
    assert_valid(mesh, what);
    Topology topology = mesh.topology();
    CHECK(topology.is_closed(), "{} has boundary edges: {}", what, testing::describe(topology.boundary_edges()));
    CHECK(topology.non_manifold_edges().empty(), "{} has non-manifold edges", what);
    CHECK_EQ(topology.euler_characteristic(), euler, "{}'s Euler characteristic", what);
    CHECK(signed_volume(mesh) > 0.0f, "{} faces in", what);
    bool none_loose = true;
    for (std::uint32_t vertex = 0; vertex < mesh.vertex_count(); vertex++) {
        none_loose = none_loose && topology.valence(vertex) > 0;
    }
    CHECK(none_loose, "{} has loose vertices", what);
}

// Every face of a convex primitive faces away from its center.
void check_faces_out_of(const Mesh& mesh, Vector3 center, std::string_view what) {
    for (size_t face = 0; face < mesh.faces.size(); face++) {
        Vector3 out = subtract(mesh.face_center(face), center);
        CHECK(dot(mesh.face_normal(face), out) > 0.0f, "{}'s face {} faces in", what, face);
    }
}

// How many of the mesh's faces have this many sides.
std::uint32_t faces_of(const Mesh& mesh, size_t sides) {
    return static_cast<std::uint32_t>(std::ranges::count_if(mesh.faces, [&](const Face& face) { return face.vertices.size() == sides; }));
}

} // namespace

TEST(the_cube_is_6_quads_around_8_vertices) {
    Mesh cube = primitives::cube(2.0f);
    check_closed(cube, 2, "the cube");
    check_faces_out_of(cube, Vector3{}, "the cube");
    CHECK_EQ(std::tuple(cube.vertex_count(), cube.face_count(), cube.edges().size()), (std::tuple<size_t, size_t, size_t>(8, 6, 12)));
    CHECK_EQ(sides_histogram(cube), (std::vector<size_t>{0, 6}));
    assert_close(signed_volume(cube), 8.0f, 1.0e-5f, "the cube's volume");
    CHECK(std::ranges::all_of(cube.positions, [](const Vector3& position) { return std::ranges::all_of(position, [](float p) { return std::abs(p) == 1.0f; }); }));
    CHECK_EQ(cube.material_count(), 1);
}

TEST(the_plane_is_a_grid_of_quads_facing_up) {
    Mesh plane = primitives::plane({4.0f, 2.0f}, {2, 3});
    assert_valid(plane, "the plane");
    CHECK_EQ(std::pair(plane.vertex_count(), plane.face_count()), (std::pair<size_t, size_t>(12, 6)));
    CHECK_EQ(sides_histogram(plane), (std::vector<size_t>{0, 6}));
    Topology topology = plane.topology();
    CHECK_EQ(topology.euler_characteristic(), 1);
    CHECK_EQ(topology.boundary_edges().size(), 2 * (2 + 3));
    for (size_t face = 0; face < plane.faces.size(); face++) {
        CHECK_EQ(plane.face_normal(face), (Vector3{0.0f, 1.0f, 0.0f}));
    }
    float area = 0.0f;
    for (size_t face = 0; face < plane.faces.size(); face++) {
        area += plane.face_area(face);
    }
    assert_close(area, 8.0f, 1.0e-5f, "the plane's area");
    std::array<float, 2> bounds{std::numeric_limits<float>::max(), std::numeric_limits<float>::lowest()};
    for (const Vector3& position : plane.positions) {
        bounds = {std::min(bounds[0], position[0]), std::max(bounds[1], position[0])};
    }
    CHECK_EQ(bounds, (std::array{-2.0f, 2.0f}));
}

TEST(cylinders_are_quads_between_two_caps) {
    for (auto [sides, segments] : std::initializer_list<std::pair<std::uint32_t, std::uint32_t>>{{3, 1}, {8, 2}, {32, 5}}) {
        std::string what = std::format("a cylinder of {} sides and {} segments", sides, segments);
        Mesh cylinder = primitives::cylinder(1.5f, 3.0f, sides, segments);
        check_closed(cylinder, 2, what);
        check_faces_out_of(cylinder, Vector3{}, what);
        CHECK_EQ(static_cast<std::uint32_t>(cylinder.vertex_count()), sides * (segments + 1), "{}'s vertices", what);
        CHECK_EQ(static_cast<std::uint32_t>(cylinder.face_count()), sides * segments + 2, "{}'s faces", what);
        std::uint32_t quads = faces_of(cylinder, 4);
        CHECK_EQ(quads, sides * segments + (sides == 4 ? 2 : 0), "{}'s quads", what);
        float polygon_area = 0.5f * static_cast<float>(sides) * 1.5f * 1.5f * std::sin(2.0f * Pi / static_cast<float>(sides));
        assert_close(signed_volume(cylinder), polygon_area * 3.0f, 1.0e-3f, what);
    }
}

TEST(cones_end_in_tips_or_caps) {
    Mesh cone = primitives::cone(1.0f, 0.0f, 2.0f, 8);
    check_closed(cone, 2, "a cone");
    check_faces_out_of(cone, Vector3{}, "a cone");
    CHECK_EQ(std::pair(cone.vertex_count(), cone.face_count()), (std::pair<size_t, size_t>(9, 9)));
    CHECK_EQ(sides_histogram(cone), (std::vector<size_t>{8, 0, 0, 0, 0, 1}));
    float base_area = 0.5f * 8.0f * std::sin(2.0f * Pi / 8.0f);
    assert_close(signed_volume(cone), base_area * 2.0f / 3.0f, 1.0e-4f, "a cone's volume");

    Mesh upside_down = primitives::cone(0.0f, 1.0f, 2.0f, 8);
    check_closed(upside_down, 2, "an upside-down cone");
    check_faces_out_of(upside_down, Vector3{}, "an upside-down cone");
    assert_close(signed_volume(upside_down), base_area * 2.0f / 3.0f, 1.0e-4f, "an upside-down cone's volume");

    Mesh truncated = primitives::cone(1.0f, 0.5f, 2.0f, 6);
    check_closed(truncated, 2, "a truncated cone");
    check_faces_out_of(truncated, Vector3{}, "a truncated cone");
    CHECK_EQ(std::pair(truncated.vertex_count(), truncated.face_count()), (std::pair<size_t, size_t>(12, 8)));
    CHECK_EQ(sides_histogram(truncated), (std::vector<size_t>{0, 6, 0, 2}));
}

TEST(uv_spheres_are_quads_with_triangles_at_the_poles) {
    for (auto [segments, rings] : std::initializer_list<std::pair<std::uint32_t, std::uint32_t>>{{3, 2}, {8, 4}, {32, 16}}) {
        std::string what = std::format("a UV sphere of {} by {}", segments, rings);
        Mesh sphere = primitives::uv_sphere(2.0f, segments, rings);
        check_closed(sphere, 2, what);
        check_faces_out_of(sphere, Vector3{}, what);
        CHECK_EQ(static_cast<std::uint32_t>(sphere.vertex_count()), 2 + segments * (rings - 1), "{}'s vertices", what);
        CHECK_EQ(static_cast<std::uint32_t>(sphere.face_count()), segments * rings, "{}'s faces", what);
        std::uint32_t triangles = faces_of(sphere, 3);
        CHECK_EQ(triangles, 2 * segments, "{}'s triangles", what);
        for (const Vector3& position : sphere.positions) {
            assert_close(std::sqrt(dot(position, position)), 2.0f, 1.0e-5f, what);
        }
    }
    Mesh fine = primitives::uv_sphere(1.0f, 64, 32);
    assert_close(signed_volume(fine), 4.0f / 3.0f * Pi, 0.03f, "a fine sphere's volume");
}

TEST(tori_are_all_quads) {
    for (auto [major, minor] : std::initializer_list<std::pair<std::uint32_t, std::uint32_t>>{{3, 3}, {12, 6}, {48, 12}}) {
        std::string what = std::format("a torus of {} by {}", major, minor);
        Mesh torus = primitives::torus(1.0f, 0.25f, major, minor);
        check_closed(torus, 0, what);
        CHECK_EQ(static_cast<std::uint32_t>(torus.vertex_count()), major * minor, "{}'s vertices", what);
        CHECK_EQ(static_cast<std::uint32_t>(torus.face_count()), major * minor, "{}'s faces", what);
        CHECK_EQ(sides_histogram(torus), (std::vector<size_t>{0, static_cast<size_t>(major * minor)}));
        // Out of the tube: away from the circle through its middle, which a
        // torus of 3 segments around is too coarse to have its faces near.
        for (size_t face = 0; face < torus.faces.size() && major >= 12; face++) {
            Vector3 center = torus.face_center(face);
            float around = std::sqrt(center[0] * center[0] + center[2] * center[2]);
            Vector3 middle{center[0] / around, 0.0f, center[2] / around};
            CHECK(dot(torus.face_normal(face), subtract(center, middle)) > 0.0f, "{}'s face {} faces in", what, face);
        }
    }
    Mesh fine = primitives::torus(1.0f, 0.25f, 96, 48);
    assert_close(signed_volume(fine), 2.0f * Pi * Pi * 0.25f * 0.25f, 0.01f, "a fine torus's volume");
}

TEST(a_cylinder_needs_3_sides) {
    std::string message;
    try {
        primitives::cylinder(1.0f, 1.0f, 2, 1);
    } catch (const base::Panic& panic) {
        message = panic.what();
    }
    CHECK(message.contains("a cylinder needs at least 3 sides"), "it panicked with \"{}\"", message);
}
