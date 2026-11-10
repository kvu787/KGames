// Library/PolygonMesh's Catmull-Clark subdivision against known results: the
// counts at each level, a cube's first level against the textbook values,
// infinitely sharp and semi-sharp creases, fixed boundary corners, n-gons and
// triangles becoming quads, and meshes that aren't manifolds.

#include "Common.h"
#include "Math/Math.h"
#include "PolygonMesh/PolygonMesh.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <cmath>
#include <initializer_list>
#include <set>
#include <string_view>
#include <tuple>
#include <utility>
#include <vector>

using common::assert_close;
using common::assert_near;
using common::assert_valid;
using common::sides_histogram;
using common::signed_volume;
using common::unwrap;
using math::Vector3;
using polygon_mesh::Edge;
using polygon_mesh::Face;
using polygon_mesh::InfinitelySharp;
using polygon_mesh::Mesh;
using polygon_mesh::SubdivisionSurface;
using polygon_mesh::Topology;
namespace primitives = polygon_mesh::primitives;

namespace {

// How many vertices and faces.
using Counts = std::pair<size_t, size_t>;

Counts counts(const Mesh& mesh) {
    return {mesh.vertex_count(), mesh.face_count()};
}

bool all_quads(const Mesh& mesh) {
    return std::ranges::all_of(mesh.faces, [](const Face& face) { return face.vertices.size() == 4; });
}

Mesh creased_cube(float sharpness) {
    Mesh cube = primitives::cube(2.0f);
    std::set<Edge> edges = cube.edges();
    cube.crease_edges(edges, sharpness);
    return cube;
}

// The sharpness of each of the mesh's creases, in the edges' order.
std::vector<float> sharpnesses(const Mesh& mesh) {
    std::vector<float> values;
    for (const auto& crease : mesh.creases) {
        values.push_back(crease.second);
    }
    return values;
}

} // namespace

TEST(a_cube_s_levels_have_the_known_counts) {
    Mesh cube = primitives::cube(2.0f);
    CHECK_EQ(cube.catmull_clark(0).mesh, cube);
    for (auto [level, faces] : std::initializer_list<std::pair<std::uint32_t, size_t>>{{1, 24}, {2, 96}, {3, 384}}) {
        SubdivisionSurface surface = cube.catmull_clark(level);
        CHECK_EQ(counts(surface.mesh), Counts(faces + 2, faces), "level {}", level);
        CHECK(all_quads(surface.mesh));
        assert_valid(surface.mesh, "a subdivided cube");
        Topology topology = surface.mesh.topology();
        CHECK(topology.is_closed());
        CHECK_EQ(topology.euler_characteristic(), 2);
        CHECK(signed_volume(surface.mesh) > 0.0f);
        // Each base face's children are together, in order.
        size_t per_face = faces / 6;
        bool together = true;
        for (size_t face = 0; face < surface.base_faces.size(); face++) {
            together = together && surface.base_faces[face] == face / per_face;
        }
        CHECK(together);
    }
}

TEST(a_cube_s_first_level_has_the_textbook_points) {
    Mesh cube = primitives::cube(2.0f);
    Mesh surface = cube.catmull_clark(1).mesh;
    Topology topology = cube.topology();
    // Vertices: the average of the 3 face points, twice the average of the 3
    // edges' midpoints, and none of the vertex, over 3: 5/9 of the way out.
    for (std::uint32_t vertex = 0; vertex < 8; vertex++) {
        Vector3 expected = cube.position(vertex);
        for (float& p : expected) {
            p = p * 5.0f / 9.0f;
        }
        assert_near(surface.position(vertex), expected, 1.0e-6f, "a vertex point");
    }
    // Edges: the average of the ends and the two faces' points, 3/4 out on
    // the two axes that the edge is at the side of.
    for (size_t index = 0; index < topology.edges().size(); index++) {
        Edge edge = topology.edges()[index];
        Vector3 a = cube.position(edge.vertices()[0]);
        Vector3 b = cube.position(edge.vertices()[1]);
        Vector3 expected{};
        for (size_t axis = 0; axis < 3; axis++) {
            expected[axis] = a[axis] == b[axis] ? 0.75f * a[axis] : 0.0f;
        }
        assert_near(surface.position(8 + static_cast<std::uint32_t>(index)), expected, 1.0e-6f, "an edge point");
    }
    // Faces: their centers.
    for (size_t face = 0; face < 6; face++) {
        assert_near(surface.position(20 + static_cast<std::uint32_t>(face)), cube.face_center(face), 1.0e-6f, "a face point");
    }
}

TEST(an_infinitely_sharp_cube_stays_a_cube) {
    Mesh cube = creased_cube(InfinitelySharp);
    for (std::uint32_t level = 1; level <= 3; level++) {
        Mesh surface = cube.catmull_clark(level).mesh;
        for (std::uint32_t vertex = 0; vertex < 8; vertex++) {
            CHECK_EQ(surface.position(vertex), cube.position(vertex), "a corner at level {}", level);
        }
        for (const Vector3& position : surface.positions) {
            float outermost = 0.0f;
            for (float p : position) {
                outermost = std::max(outermost, std::abs(p));
            }
            assert_close(outermost, 1.0f, 1.0e-6f, "a vertex on the cube's surface");
        }
        assert_close(signed_volume(surface), 8.0f, 1.0e-4f, "the sharp cube's volume");
        // The creases stay infinitely sharp, on both halves of each edge.
        CHECK_EQ(surface.creases.size(), size_t{12} << level);
        CHECK(std::ranges::all_of(surface.creases, [](const auto& crease) { return crease.second == InfinitelySharp; }));
    }
}

TEST(semi_sharp_creases_lose_1_each_level_and_blend) {
    // Sharpness 1 is a crease for the first level, and smooth after, so its
    // first level is exactly the infinitely sharp one's, and its second
    // isn't.
    Mesh smooth = primitives::cube(2.0f);
    Mesh semi = creased_cube(1.0f);
    Mesh sharp = creased_cube(InfinitelySharp);
    std::vector<Vector3> semi_first = semi.catmull_clark(1).mesh.positions;
    std::vector<Vector3> sharp_first = sharp.catmull_clark(1).mesh.positions;
    for (size_t index = 0; index < std::min(semi_first.size(), sharp_first.size()); index++) {
        assert_near(semi_first[index], sharp_first[index], 1.0e-6f, "a point of the first level");
    }
    auto corner = [](const Mesh& mesh, std::uint32_t level) {
        return mesh.catmull_clark(level).mesh.position(6);
    };
    Vector3 smooth_corner = corner(smooth, 2);
    Vector3 semi_corner = corner(semi, 2);
    Vector3 sharp_corner = corner(sharp, 2);
    CHECK_EQ(sharp_corner, (Vector3{1.0f, 1.0f, 1.0f}));
    for (auto [a, b] : {std::pair(smooth_corner, semi_corner), std::pair(semi_corner, sharp_corner), std::pair(smooth_corner, sharp_corner)}) {
        CHECK(math::length(math::subtract(a, b)) > 0.05f, "{} and {} are too close", testing::describe(a), testing::describe(b));
    }
    CHECK(semi_corner[0] > smooth_corner[0] && semi_corner[0] < sharp_corner[0]);

    // Sharpness 0.5 is halfway between a crease and smooth: the corner is
    // halfway between where it is and 5/9, and the edges between their
    // midpoints and 3/4.
    Mesh half = creased_cube(0.5f).catmull_clark(1).mesh;
    assert_near(half.position(6), {7.0f / 9.0f, 7.0f / 9.0f, 7.0f / 9.0f}, 1.0e-6f, "a half-sharp corner");
    std::uint32_t edge = static_cast<std::uint32_t>(primitives::cube(2.0f).topology().edge_index(Edge(2, 6)).value());
    assert_near(half.position(8 + edge), {0.875f, 0.875f, 0.0f}, 1.0e-6f, "a half-sharp edge");

    // The crease's sharpness goes down a level at a time, on both halves.
    Mesh cube = primitives::cube(2.0f);
    cube.set_crease(Edge(2, 6), 2.5f);
    std::vector<std::vector<float>> levels;
    for (std::uint32_t level = 1; level <= 3; level++) {
        levels.push_back(sharpnesses(cube.catmull_clark(level).mesh));
    }
    CHECK_EQ(levels, (std::vector<std::vector<float>>{std::vector<float>(2, 1.5f), std::vector<float>(4, 0.5f), std::vector<float>{}}));
}

TEST(darts_creases_and_corners) {
    // One sharp edge at a vertex is a dart, which moves like a smooth vertex,
    // though the edge's point is its midpoint.
    Mesh cube = primitives::cube(2.0f);
    cube.set_crease(Edge(2, 6), InfinitelySharp);
    std::uint32_t edge = static_cast<std::uint32_t>(cube.topology().edge_index(Edge(2, 6)).value());
    Mesh surface = cube.catmull_clark(1).mesh;
    assert_near(surface.position(6), {5.0f / 9.0f, 5.0f / 9.0f, 5.0f / 9.0f}, 1.0e-6f, "a dart");
    assert_near(surface.position(8 + edge), {1.0f, 1.0f, 0.0f}, 1.0e-6f, "a sharp edge's point");
    // Two make a crease, which moves along them: 6/8 of the vertex and 1/8 of
    // each of their other ends.
    cube.set_crease(Edge(6, 7), InfinitelySharp);
    surface = cube.catmull_clark(1).mesh;
    assert_near(surface.position(6), {0.75f, 1.0f, 0.75f}, 1.0e-6f, "a crease");
    // Three make a corner, which stays.
    cube.set_crease(Edge(5, 6), InfinitelySharp);
    CHECK_EQ(cube.catmull_clark(1).mesh.position(6), (Vector3{1.0f, 1.0f, 1.0f}));

    // Sharpness 2 is a crease for two levels, the second fading with weight
    // 1, so it matches the infinitely sharp cube for two levels and not the
    // third.
    Mesh twice = creased_cube(2.0f);
    Mesh sharp = creased_cube(InfinitelySharp);
    for (std::uint32_t level = 1; level <= 2; level++) {
        std::vector<Vector3> a = twice.catmull_clark(level).mesh.positions;
        std::vector<Vector3> b = sharp.catmull_clark(level).mesh.positions;
        for (size_t index = 0; index < std::min(a.size(), b.size()); index++) {
            assert_near(a[index], b[index], 1.0e-6f, "a point of sharpness 2");
        }
    }
    Vector3 corner = twice.catmull_clark(3).mesh.position(6);
    CHECK(corner[0] < 1.0f - 1.0e-3f, "sharpness 2 is smooth by the third level");
}

TEST(a_plane_s_boundary_corners_stay_put) {
    Mesh plane = primitives::plane({2.0f, 2.0f}, {2, 2});
    for (std::uint32_t level = 1; level <= 3; level++) {
        Mesh surface = plane.catmull_clark(level).mesh;
        assert_valid(surface, "a subdivided plane");
        for (std::uint32_t corner : {0u, 2u, 6u, 8u}) {
            CHECK_EQ(surface.position(corner), plane.position(corner), "a corner at level {}", level);
        }
        CHECK_EQ(surface.position(1), (Vector3{0.0f, 0.0f, -1.0f}), "the middle of an edge at level {}", level);
        CHECK(std::ranges::all_of(surface.positions, [](const Vector3& position) { return position[1] == 0.0f; }));
        // The boundary stays on the square's edges.
        Topology topology = surface.topology();
        for (Edge edge : topology.boundary_edges()) {
            for (std::uint32_t vertex : edge.vertices()) {
                Vector3 position = surface.position(vertex);
                assert_close(std::max(std::abs(position[0]), std::abs(position[2])), 1.0f, 1.0e-6f, "a boundary vertex");
            }
        }
        CHECK_EQ(topology.boundary_edges().size(), size_t{8} << level);
    }
    // A lone quad becomes a grid of the same square.
    Mesh quad = primitives::plane({2.0f, 2.0f}, {1, 1}).catmull_clark(1).mesh;
    CHECK_EQ(counts(quad), Counts(9, 4));
    std::vector<Vector3> corners = quad.positions;
    std::ranges::stable_sort(corners);
    std::vector<Vector3> grid = primitives::plane({2.0f, 2.0f}, {2, 2}).positions;
    std::ranges::stable_sort(grid);
    CHECK_EQ(corners, grid);
}

TEST(n_gons_and_triangles_become_quads) {
    Mesh cone = primitives::cone(1.0f, 0.0f, 2.0f, 8);
    SubdivisionSurface surface = cone.catmull_clark(1);
    CHECK_EQ(counts(surface.mesh), Counts(9 + 16 + 9, 8 * 3 + 8));
    CHECK(all_quads(surface.mesh));
    assert_valid(surface.mesh, "a subdivided cone");
    CHECK(surface.mesh.topology().is_closed());
    CHECK_EQ(std::ranges::count(surface.base_faces, size_t{8}), 8);

    Mesh tetrahedron = primitives::cone(1.0f, 0.0f, 1.0f, 3);
    Mesh subdivided = tetrahedron.catmull_clark(2).mesh;
    CHECK_EQ(counts(subdivided), Counts(12 * 4 + 2, 12 * 4));
    CHECK_EQ(sides_histogram(subdivided), (std::vector<size_t>{0, 48}));
}

TEST(children_keep_their_face_s_material) {
    Mesh cylinder = primitives::cylinder(1.0f, 2.0f, 6, 2);
    cylinder.assign_material({3}, 2);
    cylinder.assign_material({12, 13}, 5);
    SubdivisionSurface surface = cylinder.catmull_clark(2);
    for (size_t face = 0; face < std::min(surface.mesh.faces.size(), surface.base_faces.size()); face++) {
        CHECK_EQ(surface.mesh.faces[face].material, cylinder.faces[surface.base_faces[face]].material);
    }
    CHECK_EQ(surface.mesh.material_count(), 6);
    CHECK_EQ(std::ranges::count_if(surface.mesh.faces, [](const Face& face) { return face.material == 5; }), 2 * 6 * 4);
}

TEST(closed_surfaces_stay_closed_and_wound_out) {
    std::vector<std::tuple<Mesh, std::int64_t, std::string_view>> cases;
    cases.emplace_back(primitives::torus(1.0f, 0.25f, 12, 6), 0, "a torus");
    cases.emplace_back(primitives::uv_sphere(1.0f, 8, 4), 2, "a sphere");
    cases.emplace_back(primitives::cylinder(1.0f, 2.0f, 5, 1), 2, "a cylinder");
    for (const auto& [mesh, euler, what] : cases) {
        Mesh surface = mesh.catmull_clark(2).mesh;
        assert_valid(surface, what);
        Topology topology = surface.topology();
        CHECK(topology.is_closed(), "{}", what);
        CHECK_EQ(topology.euler_characteristic(), euler, "{}", what);
        CHECK(signed_volume(surface) > 0.0f, "{}", what);
        CHECK(all_quads(surface), "{}", what);
    }
    // A sphere's subdivision shrinks inside it, toward a smooth limit.
    Mesh sphere = primitives::uv_sphere(1.0f, 16, 8).catmull_clark(3).mesh;
    CHECK(std::ranges::all_of(sphere.positions, [](const Vector3& position) { return math::length(position) < 1.0f + 1.0e-6f; }));
    CHECK(std::ranges::all_of(sphere.positions, [](const Vector3& position) { return math::length(position) > 0.9f; }));
}

TEST(non_manifold_meshes_subdivide_without_panicking) {
    // Three quads on one edge, whose ends stay put, as corners.
    Mesh mesh;
    for (Vector3 position : {
             Vector3{0.0f, 0.0f, 0.0f},
             Vector3{1.0f, 0.0f, 0.0f},
             Vector3{1.0f, 1.0f, 0.0f},
             Vector3{0.0f, 1.0f, 0.0f},
             Vector3{1.0f, 0.0f, 1.0f},
             Vector3{0.0f, 0.0f, 1.0f},
             Vector3{1.0f, -1.0f, 0.0f},
             Vector3{0.0f, -1.0f, 0.0f},
         }) {
        mesh.add_vertex(position);
    }
    mesh.add_face({0, 1, 2, 3}, 0);
    mesh.add_face({1, 0, 5, 4}, 0);
    mesh.add_face({1, 0, 7, 6}, 0);
    // A loose vertex stays where it is.
    mesh.add_vertex({5.0f, 5.0f, 5.0f});
    Mesh surface = mesh.catmull_clark(3).mesh;
    unwrap(surface.validate());
    CHECK_EQ(surface.face_count(), 3 * 64);
    CHECK_EQ(surface.position(0), (Vector3{0.0f, 0.0f, 0.0f}));
    CHECK_EQ(surface.position(1), (Vector3{1.0f, 0.0f, 0.0f}));
    CHECK_EQ(surface.position(8), (Vector3{5.0f, 5.0f, 5.0f}));
    // The spine is infinitely sharp, so its points stay on it.
    Topology topology = surface.topology();
    for (Edge edge : topology.non_manifold_edges()) {
        for (std::uint32_t vertex : edge.vertices()) {
            Vector3 position = surface.position(vertex);
            CHECK(position[1] == 0.0f && position[2] == 0.0f, "vertex {} left the spine", vertex);
        }
    }
}
