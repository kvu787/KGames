// Library/PolygonMesh's picking: ray casts against faces, from either side and
// through concave faces only where they are, the vertex and edge nearest a
// ray, and the bounds and centers of meshes and selections.

#include "Common.h"
#include "PolygonMesh/PolygonMesh.h"
#include "Testing/Testing.h"

#include <array>
#include <cmath>
#include <initializer_list>
#include <optional>

using common::assert_close;
using common::assert_near;
using math::Vector3;
using polygon_mesh::Edge;
using polygon_mesh::Hit;
using polygon_mesh::Mesh;
using polygon_mesh::SubdivisionSurface;
namespace primitives = polygon_mesh::primitives;

namespace {

void assert_hit(std::optional<Hit> hit, size_t face, float distance, Vector3 point) {
    CHECK(hit.has_value(), "the ray missed");
    CHECK_EQ(hit->face, face);
    assert_close(hit->distance, distance, 1.0e-5f, "the distance to the hit");
    assert_near(hit->point, point, 1.0e-5f, "the point hit");
}

} // namespace

TEST(rays_hit_the_nearest_face) {
    Mesh cube = primitives::cube(2.0f);
    // The front, +Z, before the back, even along the edge between its
    // triangles.
    assert_hit(cube.ray_cast({0.0f, 0.0f, 5.0f}, {0.0f, 0.0f, -1.0f}), 1, 4.0f, {0.0f, 0.0f, 1.0f});
    assert_hit(cube.ray_cast({0.5f, -0.5f, 5.0f}, {0.0f, 0.0f, -1.0f}), 1, 4.0f, {0.5f, -0.5f, 1.0f});
    // The direction's length doesn't matter.
    assert_hit(cube.ray_cast({0.25f, -0.5f, 5.0f}, {0.0f, 0.0f, -10.0f}), 1, 4.0f, {0.25f, -0.5f, 1.0f});
    // From inside, a face's back.
    assert_hit(cube.ray_cast(Vector3{}, {1.0f, 0.0f, 0.0f}), 5, 1.0f, {1.0f, 0.0f, 0.0f});
    assert_hit(cube.ray_cast({0.5f, 3.0f, 0.5f}, {0.0f, -1.0f, 0.0f}), 3, 2.0f, {0.5f, 1.0f, 0.5f});
    // At an angle.
    assert_hit(cube.ray_cast({-3.0f, 0.0f, 0.0f}, {1.0f, 0.0f, 0.25f}), 4, std::sqrt(4.25f), {-1.0f, 0.0f, 0.5f});

    CHECK_EQ(cube.ray_cast({5.0f, 5.0f, 5.0f}, {1.0f, 0.0f, 0.0f}), std::nullopt);
    CHECK_EQ(cube.ray_cast({0.0f, 0.0f, 5.0f}, {0.0f, 0.0f, 1.0f}), std::nullopt);
    CHECK_EQ(cube.ray_cast({0.0f, 0.0f, 5.0f}, Vector3{}), std::nullopt);
    CHECK_EQ(Mesh().ray_cast(Vector3{}, {1.0f, 0.0f, 0.0f}), std::nullopt);

    // Which quad of a grid, from below.
    Mesh grid = primitives::plane({3.0f, 3.0f}, {3, 3});
    assert_hit(grid.ray_cast({0.4f, -2.0f, 0.6f}, {0.0f, 1.0f, 0.0f}), 7, 2.0f, {0.4f, 0.0f, 0.6f});
}

TEST(rays_through_a_concave_face_hit_only_where_it_is) {
    // An L, missing its corner at x and z from 1 to 2.
    Mesh mesh;
    for (auto [x, z] : std::initializer_list<std::array<float, 2>>{{0.0f, 0.0f}, {0.0f, 2.0f}, {1.0f, 2.0f}, {1.0f, 1.0f}, {2.0f, 1.0f}, {2.0f, 0.0f}}) {
        mesh.add_vertex({x, 0.0f, z});
    }
    mesh.add_face({0, 1, 2, 3, 4, 5}, 0);
    CHECK_EQ(mesh.face_normal(0), (Vector3{0.0f, 1.0f, 0.0f}));
    Vector3 down{0.0f, -1.0f, 0.0f};
    CHECK_EQ(mesh.ray_cast({1.5f, 1.0f, 1.5f}, down), std::nullopt);
    for (auto [x, z] : std::initializer_list<std::array<float, 2>>{{0.5f, 0.5f}, {0.5f, 1.5f}, {1.5f, 0.5f}, {0.9f, 1.9f}}) {
        assert_hit(mesh.ray_cast({x, 1.0f, z}, down), 0, 1.0f, {x, 0.0f, z});
    }
}

TEST(the_nearest_vertex_and_edge_to_a_ray) {
    Mesh cube = primitives::cube(2.0f);
    Vector3 along{0.0f, 0.0f, -1.0f};
    // Corners 6, at 1, 1, 1, and 2, at 1, 1, -1, are as near the ray, and 6
    // is nearer its start.
    CHECK_EQ(cube.nearest_vertex({1.05f, 1.0f, 5.0f}, along, 0.1f), std::optional<std::uint32_t>(6));
    CHECK_EQ(cube.nearest_vertex({0.95f, 0.98f, 5.0f}, along, 0.1f), std::optional<std::uint32_t>(6));
    CHECK_EQ(cube.nearest_vertex({1.05f, 1.0f, 5.0f}, along, 0.01f), std::nullopt);
    CHECK_EQ(cube.nearest_vertex({1.0f, 1.0f, -5.0f}, along, 0.1f), std::nullopt);
    // The front top edge, 6 to 7, before the back one, 2 to 3.
    CHECK_EQ(cube.nearest_edge({0.0f, 1.02f, 5.0f}, along, 0.1f), std::optional(Edge(6, 7)));
    CHECK_EQ(cube.nearest_edge({0.3f, 1.02f, 5.0f}, {0.0f, 0.0f, 1.0f}, 0.1f), std::nullopt);
    // Across the middle of the right top edge, 2 to 6, before the left one.
    CHECK_EQ(cube.nearest_edge({5.0f, 1.02f, 0.2f}, {-1.0f, 0.0f, 0.0f}, 0.05f), std::optional(Edge(2, 6)));
    CHECK_EQ(cube.nearest_edge({0.0f, 0.0f, 5.0f}, along, 0.5f), std::nullopt);
}

TEST(bounds_and_centers) {
    Mesh cube = primitives::cube(2.0f);
    CHECK_EQ(cube.bounds(), std::optional(std::array{Vector3{-1.0f, -1.0f, -1.0f}, Vector3{1.0f, 1.0f, 1.0f}}));
    Mesh plane = primitives::plane({4.0f, 2.0f}, {1, 1});
    CHECK_EQ(plane.bounds(), std::optional(std::array{Vector3{-2.0f, 0.0f, -1.0f}, Vector3{2.0f, 0.0f, 1.0f}}));
    CHECK_EQ(Mesh().bounds(), std::nullopt);
    CHECK_EQ(cube.center_of({3, 7, 6, 2}), std::optional(Vector3{0.0f, 1.0f, 0.0f}));
    CHECK_EQ(cube.center_of({6, 6, 99}), std::optional(Vector3{1.0f, 1.0f, 1.0f}));
    CHECK_EQ(cube.center_of({}), std::nullopt);
}

TEST(rays_hit_a_subdivision_surface_s_base_faces) {
    Mesh cube = primitives::cube(2.0f);
    SubdivisionSurface surface = cube.catmull_clark(2);
    Hit hit = surface.ray_cast({0.3f, 0.2f, 5.0f}, {0.0f, 0.0f, -1.0f}).value();
    CHECK_EQ(hit.face, 1);
    // The smooth surface is inside the cube.
    CHECK(hit.distance > 4.0f && hit.distance < 4.5f);
    Hit side = surface.ray_cast({5.0f, 0.1f, -0.2f}, {-1.0f, 0.0f, 0.0f}).value();
    CHECK_EQ(side.face, 5);
}
