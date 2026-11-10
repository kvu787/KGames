// Library/PolygonMesh's editing operations, on small meshes whose results are
// known exactly: what each makes, what it selects, and that it keeps the mesh
// valid, consistently wound, and closed where it was.

#include "Common.h"
#include "Math/Math.h"
#include "PolygonMesh/PolygonMesh.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <cmath>
#include <initializer_list>
#include <map>
#include <numbers>
#include <optional>
#include <ranges>
#include <set>
#include <span>
#include <utility>
#include <vector>

using common::assert_close;
using common::assert_near;
using common::assert_valid;
using common::sides_histogram;
using common::signed_volume;
using common::unwrap;
using common::unwrap_err;
using math::length;
using math::Matrix;
using math::subtract;
using math::Vector3;
using polygon_mesh::Edge;
using polygon_mesh::Face;
using polygon_mesh::Mesh;
using polygon_mesh::Selection;
using polygon_mesh::Topology;
namespace primitives = polygon_mesh::primitives;

namespace {

constexpr float FracPi2 = std::numbers::pi_v<float> / 2.0f;

// The cube's faces are -Z, +Z, -Y, +Y, -X and +X, and its top, +Y, is 3, 7,
// 6, 2. Vertex 0 is at -1, -1, -1, and 6 at 1, 1, 1.
constexpr size_t Front = 1;
constexpr size_t Top = 3;

template <class T> std::set<T> set(std::initializer_list<T> items) {
    return std::set<T>(items);
}

// The whole numbers from `start` up to `end`, without it, as a set.
template <class T> std::set<T> range_set(T start, T end) {
    std::set<T> numbers;
    for (T number = start; number < end; number++) {
        numbers.insert(number);
    }
    return numbers;
}

// How many vertices and faces.
using Counts = std::pair<size_t, size_t>;

Counts counts(const Mesh& mesh) {
    return {mesh.vertex_count(), mesh.face_count()};
}

// Valid, consistently wound and without holes.
void assert_closed(const Mesh& mesh, std::string_view what) {
    assert_valid(mesh, what);
    Topology topology = mesh.topology();
    CHECK(topology.is_closed(), "{} has boundary edges: {}", what, testing::describe(topology.boundary_edges()));
    CHECK_EQ(topology.euler_characteristic(), 2, "{}'s Euler characteristic", what);
}

template <polygon_mesh::RangeOf<size_t> Faces = std::initializer_list<size_t>> float area(const Mesh& mesh, Faces&& faces) {
    float sum = 0.0f;
    for (auto&& face : faces) {
        sum += mesh.face_area(static_cast<size_t>(face));
    }
    return sum;
}

} // namespace

TEST(moving_vertices) {
    Mesh cube = primitives::cube(2.0f);
    Selection top = Selection::from_faces(cube, {Top});
    cube.translate(top.vertices, {0.0f, 1.0f, 0.0f});
    CHECK_EQ(cube.position(6), (Vector3{1.0f, 2.0f, 1.0f}));
    CHECK_EQ(cube.position(0), (Vector3{-1.0f, -1.0f, -1.0f}));

    cube = primitives::cube(2.0f);
    cube.scale(top.vertices, {0.0f, 1.0f, 0.0f}, {2.0f, 1.0f, 0.5f});
    CHECK_EQ(cube.position(6), (Vector3{2.0f, 1.0f, 0.5f}));
    CHECK_EQ(cube.position(5), (Vector3{1.0f, -1.0f, 1.0f}));

    // Like Math's rotations, and like the same rotation as a matrix.
    Mesh original = primitives::cube(2.0f);
    Mesh rotated = original;
    rotated.rotate(std::views::iota(0, 8), Vector3{}, {0.0f, 2.0f, 0.0f}, FracPi2);
    Mesh transformed = original;
    transformed.transform(std::views::iota(0, 8), Vector3{}, Matrix::rotation_y(FracPi2));
    for (std::uint32_t vertex = 0; vertex < 8; vertex++) {
        Vector3 expected = Matrix::rotation_y(FracPi2).transform_point(original.position(vertex));
        assert_near(rotated.position(vertex), expected, 1.0e-6f, "a rotated vertex");
        assert_near(transformed.position(vertex), expected, 1.0e-6f, "a transformed vertex");
    }
    assert_near(rotated.position(6), {1.0f, 1.0f, -1.0f}, 1.0e-6f, "a corner turned a quarter about Y");
    // About a pivot: the pivot stays.
    Mesh turned = original;
    turned.rotate({6}, {1.0f, 1.0f, 0.0f}, {0.0f, 0.0f, 1.0f}, FracPi2);
    assert_near(turned.position(6), {1.0f, 1.0f, 1.0f}, 1.0e-6f, "a vertex on the axis");
    turned.rotate({5}, {1.0f, 1.0f, 1.0f}, {0.0f, 0.0f, 1.0f}, FracPi2);
    assert_near(turned.position(5), {3.0f, 1.0f, 1.0f}, 1.0e-6f, "a vertex turned about a pivot");
    Mesh unmoved = original;
    unmoved.rotate(std::views::iota(0, 8), Vector3{}, Vector3{}, 1.0f);
    unmoved.translate({42}, {1.0f, 0.0f, 0.0f});
    CHECK_EQ(unmoved, original);
}

TEST(extruding_a_cube_s_face) {
    Mesh cube = primitives::cube(2.0f);
    cube.assign_material({Top}, 2);
    cube.set_crease(Edge(2, 3), 5.0f);
    Selection selection = cube.extrude_faces({Top});
    CHECK_EQ(counts(cube), Counts(12, 10));
    assert_closed(cube, "a cube with a face extruded");
    CHECK_EQ(selection.faces, set<size_t>({Top}));
    CHECK_EQ(selection.vertices, range_set<std::uint32_t>(8, 12));
    CHECK_EQ(selection.edges.size(), 4);
    // The walls are quads of the face's material, and the crease is on both
    // the old edge and the new one.
    CHECK_EQ(sides_histogram(cube), (std::vector<size_t>{0, 10}));
    CHECK(std::ranges::all_of(std::span(cube.faces).subspan(6), [](const Face& face) { return face.material == 2; }));
    CHECK_EQ(cube.crease(Edge(2, 3)), 5.0f);
    CHECK_EQ(cube.creases.size(), 2);
    cube.translate(selection.vertices, {0.0f, 1.0f, 0.0f});
    assert_close(signed_volume(cube), 12.0f, 1.0e-5f, "the extruded cube's volume");
    CHECK_EQ(cube.face_normal(Top), (Vector3{0.0f, 1.0f, 0.0f}));
}

TEST(extruding_regions) {
    // Two faces that share an edge: one region, with walls along its 6
    // boundary edges, and none along the edge between them.
    Mesh cube = primitives::cube(2.0f);
    Selection selection = cube.extrude_faces({Top, Front});
    CHECK_EQ(counts(cube), Counts(14, 12));
    assert_closed(cube, "a cube with two faces extruded");
    CHECK_EQ(selection.faces, set<size_t>({Front, Top}));
    CHECK_EQ(selection.vertices.size(), 6);
    cube.translate(selection.vertices, {0.0f, 1.0f, 1.0f});
    assert_valid(cube, "a cube with two faces extruded and moved");
    CHECK(signed_volume(cube) > 8.0f);

    // A lone quad becomes a closed box, with the quad left behind, flipped.
    Mesh quad = primitives::plane({2.0f, 2.0f}, {1, 1});
    selection = quad.extrude_faces({0});
    CHECK_EQ(counts(quad), Counts(8, 6));
    assert_closed(quad, "an extruded quad");
    quad.translate(selection.vertices, {0.0f, 1.0f, 0.0f});
    assert_close(signed_volume(quad), 4.0f, 1.0e-5f, "the extruded quad's volume");

    // A whole open grid does too.
    Mesh grid = primitives::plane({2.0f, 2.0f}, {2, 2});
    selection = grid.extrude_faces(std::views::iota(0, 4));
    CHECK_EQ(counts(grid), Counts(18, 16));
    assert_closed(grid, "an extruded grid");
    grid.translate(selection.vertices, {0.0f, 0.5f, 0.0f});
    assert_close(signed_volume(grid), 2.0f, 1.0e-5f, "the extruded grid's volume");

    // A face in the middle of a grid: the walls join the grid, and nothing is
    // left behind.
    grid = primitives::plane({3.0f, 3.0f}, {3, 3});
    grid.extrude_faces({4});
    CHECK_EQ(counts(grid), Counts(20, 13));
    assert_valid(grid, "a grid with its middle extruded");
    CHECK_EQ(grid.topology().boundary_edges().size(), 12);

    // A vertex inside the region, which nothing else uses, moves with it.
    grid = primitives::plane({3.0f, 3.0f}, {3, 3});
    selection = grid.extrude_faces({0, 1, 3, 4});
    CHECK_EQ(counts(grid), Counts(24, 17));
    assert_valid(grid, "a grid with a corner extruded");
    CHECK(selection.vertices.contains(5) && selection.vertices.size() == 9);
}

TEST(extruding_edges) {
    // The quad is 0, 2, 3, 1, so it goes from 1 to 0, and the new quad from 0
    // to 1, then back along the copies, 5 of 1 and 4 of 0.
    Mesh quad = primitives::plane({2.0f, 2.0f}, {1, 1});
    quad.set_crease(Edge(0, 1), 1.5f);
    Selection selection = quad.extrude_edges({Edge(0, 1), Edge(0, 3)});
    CHECK_EQ(counts(quad), Counts(6, 2));
    assert_valid(quad, "a quad with an edge extruded");
    CHECK_EQ(quad.faces[1].vertices, (std::vector<std::uint32_t>{0, 1, 5, 4}));
    CHECK_EQ(selection.edges, set({Edge(4, 5)}));
    CHECK_EQ(selection.vertices, set<std::uint32_t>({4, 5}));
    CHECK(selection.faces.empty());
    CHECK_EQ(quad.crease(Edge(4, 5)), 1.5f);

    quad = primitives::plane({2.0f, 2.0f}, {1, 1});
    std::vector<Edge> boundary = quad.topology().boundary_edges();
    selection = quad.extrude_edges(boundary);
    CHECK_EQ(counts(quad), Counts(8, 5));
    assert_valid(quad, "a quad with its boundary extruded");
    CHECK_EQ(selection.edges.size(), 4);
    CHECK_EQ(selection.faces, std::set<size_t>{});
    quad.scale(selection.vertices, Vector3{}, {2.0f, 1.0f, 2.0f});
    assert_close(area(quad, std::views::iota(0, 5)), 16.0f, 1.0e-5f, "the widened quad's area");
}

TEST(insetting_a_region) {
    Mesh cube = primitives::cube(2.0f);
    Selection selection = cube.inset_region({Top}, 0.25f);
    CHECK_EQ(counts(cube), Counts(12, 10));
    assert_closed(cube, "a cube with a face inset");
    CHECK_EQ(selection.faces, set<size_t>({Top}));
    for (std::uint32_t vertex : selection.vertices) {
        auto [x, y, z] = cube.position(vertex);
        CHECK(std::abs(std::abs(x) - 0.75f) < 1.0e-6f && y == 1.0f && std::abs(std::abs(z) - 0.75f) < 1.0e-6f);
    }
    assert_close(cube.face_area(Top), 2.25f, 1.0e-5f, "the inset face's area");

    // Two faces that share an edge: the copies of its ends stay on it.
    cube = primitives::cube(2.0f);
    selection = cube.inset_region({Top, Front}, 0.25f);
    CHECK_EQ(counts(cube), Counts(14, 12));
    assert_closed(cube, "a cube with two faces inset");
    CHECK_EQ(selection.faces, set<size_t>({Front, Top}));
    std::vector<Vector3> positions;
    for (std::uint32_t vertex : selection.vertices) {
        positions.push_back(cube.position(vertex));
    }
    CHECK(std::ranges::contains(positions, Vector3{0.75f, 1.0f, 1.0f}) && std::ranges::contains(positions, Vector3{-0.75f, 1.0f, 1.0f}));

    // Flat: the copies are the thickness from each edge, farther at corners.
    Mesh strip = primitives::plane({2.0f, 1.0f}, {2, 1});
    selection = strip.inset_region({0, 1}, 0.25f);
    CHECK_EQ(counts(strip), Counts(12, 8));
    assert_valid(strip, "an inset strip");
    assert_close(area(strip, selection.faces), 0.75f, 1.0e-5f, "the inset strip's area");
    assert_close(area(strip, std::views::iota(0, 8)), 2.0f, 1.0e-5f, "the whole strip's area");
    for (Vector3 expected : {Vector3{0.0f, 0.0f, -0.25f}, Vector3{-0.75f, 0.0f, -0.25f}, Vector3{0.75f, 0.0f, 0.25f}}) {
        bool found = std::ranges::any_of(selection.vertices, [&](std::uint32_t vertex) { return length(subtract(strip.position(vertex), expected)) < 1.0e-6f; });
        CHECK(found, "no copy at {}", testing::describe(expected));
    }
}

TEST(insetting_faces_on_their_own) {
    Mesh cube = primitives::cube(2.0f);
    Selection selection = cube.inset_individual({Top, Front}, 0.25f);
    CHECK_EQ(counts(cube), Counts(16, 14));
    assert_closed(cube, "a cube with two faces inset on their own");
    CHECK_EQ(selection.faces, set<size_t>({Front, Top}));
    CHECK_EQ(selection.vertices.size(), 8);

    Mesh quad = primitives::plane({2.0f, 2.0f}, {1, 1});
    quad.inset_individual({0}, 0.5f);
    CHECK_EQ(counts(quad), Counts(8, 5));
    assert_valid(quad, "an inset quad");
    assert_close(quad.face_area(0), 1.0f, 1.0e-5f, "the inset quad's area");
}

TEST(deleting) {
    Mesh cube = primitives::cube(2.0f);
    cube.set_crease(Edge(4, 5), 2.0f);
    cube.set_crease(Edge(0, 1), 2.0f);
    Selection selection = cube.delete_vertices({0});
    CHECK(selection.is_empty());
    CHECK_EQ(counts(cube), Counts(7, 3));
    assert_valid(cube, "a cube without a corner");
    CHECK_EQ(cube.topology().boundary_edges().size(), 6);
    CHECK_EQ(cube.position(0), (Vector3{1.0f, -1.0f, -1.0f}));
    CHECK_EQ(cube.faces[0].vertices, (std::vector<std::uint32_t>{3, 4, 5, 6}));
    // The crease follows its edge's renumbering, and goes with its edge.
    CHECK_EQ(cube.creases, (std::map<Edge, float>{{Edge(3, 4), 2.0f}}));

    // An edge takes its faces, and its vertices if nothing else uses them.
    cube = primitives::cube(2.0f);
    cube.delete_edges({Edge(2, 3)});
    CHECK_EQ(counts(cube), Counts(8, 4));
    Mesh quad = primitives::plane({2.0f, 2.0f}, {1, 1});
    quad.delete_edges({Edge(0, 1)});
    CHECK_EQ(counts(quad), Counts(2, 0));

    // Faces take the vertices that nothing else uses.
    cube = primitives::cube(2.0f);
    cube.delete_faces({Top});
    CHECK_EQ(counts(cube), Counts(8, 5));
    CHECK_EQ(cube.topology().boundary_edges().size(), 4);
    cube.delete_faces(std::views::iota(0, 5));
    CHECK_EQ(counts(cube), Counts(0, 0));
    Mesh grid = primitives::plane({3.0f, 3.0f}, {3, 3});
    grid.delete_faces({4});
    CHECK_EQ(counts(grid), Counts(16, 8));
    grid.delete_faces({0});
    CHECK_EQ(counts(grid), Counts(15, 7));

    // Only faces leaves the vertices, loose.
    cube = primitives::cube(2.0f);
    cube.delete_only_faces(std::views::iota(0, 6));
    CHECK_EQ(counts(cube), Counts(8, 0));
    CHECK_EQ(cube.remove_unused_vertices(), std::vector<std::optional<std::uint32_t>>(8));
    CHECK_EQ(counts(cube), Counts(0, 0));
}

TEST(removing_unused_vertices_renumbers_the_rest) {
    Mesh grid = primitives::plane({3.0f, 3.0f}, {3, 3});
    Face second = grid.faces[1];
    grid.delete_only_faces({0});
    CHECK_EQ(counts(grid), Counts(16, 8));
    std::vector<std::optional<std::uint32_t>> renumbered = grid.remove_unused_vertices();
    std::vector<std::optional<std::uint32_t>> in_order;
    for (std::uint32_t vertex = 0; vertex < 15; vertex++) {
        in_order.push_back(vertex);
    }
    CHECK_EQ(renumbered[0], std::nullopt);
    CHECK_EQ(std::vector(renumbered.begin() + 1, renumbered.end()), in_order);
    std::vector<std::uint32_t> one_less;
    for (std::uint32_t vertex : second.vertices) {
        one_less.push_back(vertex - 1);
    }
    CHECK_EQ(grid.faces[0].vertices, one_less);
    CHECK_EQ(grid.remove_unused_vertices(), in_order);
}

TEST(making_faces) {
    // Filling the cube's top back in follows the hole's boundary.
    Mesh cube = primitives::cube(2.0f);
    cube.assign_material(std::views::iota(0, 6), 4);
    cube.delete_only_faces({Top});
    Selection selection = unwrap(cube.make_face({2, 3, 6, 7}));
    assert_closed(cube, "a cube with its top made again");
    CHECK_EQ(selection.faces, set<size_t>({5}));
    CHECK_EQ(cube.faces[5].material, 4);
    CHECK_EQ(cube.face_normal(5), (Vector3{0.0f, 1.0f, 0.0f}));
    assert_close(signed_volume(cube), 8.0f, 1.0e-5f, "the cube's volume");

    // Three of the hole's four vertices: around their middle, wound like the
    // faces around them.
    cube = primitives::cube(2.0f);
    cube.delete_only_faces({Top});
    unwrap(cube.make_face({2, 3, 6}));
    assert_valid(cube, "a cube with a triangle in its top");
    CHECK_EQ(cube.topology().boundary_edges().size(), 3);
    CHECK_EQ(cube.face_normal(5), (Vector3{0.0f, 1.0f, 0.0f}));

    cube = primitives::cube(2.0f);
    CHECK_EQ(unwrap_err(cube.make_face({2, 3})), "Making a face takes at least 3 vertices.");
    CHECK_EQ(unwrap_err(cube.make_face({2, 3, 6, 7})), "There's already a face with these vertices.");

    // Loose vertices, with nothing to agree with.
    Mesh quad = primitives::plane({2.0f, 2.0f}, {1, 1});
    quad.delete_only_faces({0});
    unwrap(quad.make_face({0, 1, 2, 3}));
    assert_valid(quad, "a quad made from loose vertices");
    CHECK_EQ(std::abs(quad.face_normal(0)[1]), 1.0f);
    assert_close(quad.face_area(0), 4.0f, 1.0e-5f, "the made quad's area");
}

TEST(filling_holes) {
    Mesh cube = primitives::cube(2.0f);
    cube.delete_only_faces({Top});
    std::vector<Edge> boundary = cube.topology().boundary_edges();
    Selection selection = unwrap(cube.fill(boundary));
    assert_closed(cube, "a cube with its top filled");
    CHECK_EQ(selection.faces, set<size_t>({5}));
    assert_close(signed_volume(cube), 8.0f, 1.0e-5f, "the filled cube's volume");

    // Both ends of a tube.
    Mesh tube = primitives::cylinder(1.0f, 2.0f, 8, 1);
    tube.delete_only_faces({8, 9});
    boundary = tube.topology().boundary_edges();
    selection = unwrap(tube.fill(boundary));
    assert_closed(tube, "a tube with its ends filled");
    CHECK_EQ(selection.faces, set<size_t>({8, 9}));
    CHECK_EQ(sides_histogram(tube), (std::vector<size_t>{0, 8, 0, 0, 0, 2}));
    CHECK(signed_volume(tube) > 0.0f);

    cube = primitives::cube(2.0f);
    std::set<Edge> edges = cube.edges();
    CHECK_EQ(unwrap_err(cube.fill(edges)), "The selection has no loop of boundary edges around a hole to fill.");
}

TEST(merging_at_the_center) {
    // The top collapses into a point: a pyramid.
    Mesh cube = primitives::cube(2.0f);
    Selection selection = cube.merge_at_center({3, 7, 6, 2});
    CHECK_EQ(counts(cube), Counts(5, 5));
    assert_closed(cube, "a pyramid");
    CHECK_EQ(selection.vertices, set<std::uint32_t>({2}));
    CHECK_EQ(cube.position(2), (Vector3{0.0f, 1.0f, 0.0f}));
    CHECK_EQ(sides_histogram(cube), (std::vector<size_t>{4, 1}));

    // An edge collapses: its two faces become triangles.
    cube = primitives::cube(2.0f);
    cube.set_crease(Edge(3, 7), 3.0f);
    cube.merge_at_center({2, 3});
    CHECK_EQ(counts(cube), Counts(7, 6));
    assert_closed(cube, "a cube with an edge merged");
    CHECK_EQ(sides_histogram(cube), (std::vector<size_t>{2, 4}));
    CHECK_EQ(cube.position(2), (Vector3{0.0f, 1.0f, -1.0f}));
    // Vertex 7 is 6 now, and the crease went with it.
    CHECK_EQ(cube.crease(Edge(2, 6)), 3.0f);
}

TEST(merging_by_distance) {
    // Two quads side by side, each with its own copy of the edge between them.
    Mesh mesh;
    for (Vector3 position : {
             Vector3{0.0f, 0.0f, 0.0f},
             Vector3{1.0f, 0.0f, 0.0f},
             Vector3{1.0f, 0.0f, 1.0f},
             Vector3{0.0f, 0.0f, 1.0f},
             Vector3{1.0f, 0.0f, 0.00001f},
             Vector3{2.0f, 0.0f, 0.0f},
             Vector3{2.0f, 0.0f, 1.0f},
             Vector3{1.0f, 0.0f, 1.0f},
         }) {
        mesh.add_vertex(position);
    }
    mesh.add_face({0, 3, 2, 1}, 0);
    mesh.add_face({4, 7, 6, 5}, 0);
    Selection selection = mesh.merge_by_distance(std::views::iota(0, 8), 0.001f);
    CHECK_EQ(counts(mesh), Counts(6, 2));
    assert_valid(mesh, "two quads merged");
    CHECK_EQ(mesh.topology().boundary_edges().size(), 6);
    CHECK_EQ(selection.vertices, range_set<std::uint32_t>(0, 6));
    // The first of each pair stays where it was.
    CHECK_EQ(mesh.position(1), (Vector3{1.0f, 0.0f, 0.0f}));

    // Nothing within the distance: nothing changes.
    Mesh cube = primitives::cube(2.0f);
    cube.merge_by_distance(std::views::iota(0, 8), 0.5f);
    CHECK_EQ(cube, primitives::cube(2.0f));

    // An extrusion that hasn't moved merges back into the cube.
    cube = primitives::cube(2.0f);
    cube.extrude_faces({Top});
    cube.merge_by_distance(std::views::iota(0, 12), 0.0001f);
    CHECK_EQ(counts(cube), Counts(8, 6));
    assert_closed(cube, "a cube merged back");
    assert_close(signed_volume(cube), 8.0f, 1.0e-5f, "the merged cube's volume");
}

TEST(subdividing_faces) {
    Mesh cube = primitives::cube(2.0f);
    cube.set_crease(Edge(2, 3), 3.0f);
    Selection selection = cube.subdivide_faces({Top});
    CHECK_EQ(counts(cube), Counts(13, 9));
    assert_closed(cube, "a cube with its top subdivided");
    CHECK_EQ(selection.faces, set<size_t>({Top, 6, 7, 8}));
    // The faces next to it get the midpoints of the edges that they share.
    CHECK_EQ(sides_histogram(cube), (std::vector<size_t>{0, 5, 4}));
    assert_close(area(cube, selection.faces), 4.0f, 1.0e-5f, "the top's area");
    std::vector<Vector3> positions;
    for (std::uint32_t vertex : selection.vertices) {
        positions.push_back(cube.position(vertex));
    }
    CHECK(std::ranges::contains(positions, Vector3{0.0f, 1.0f, 0.0f}) && std::ranges::contains(positions, Vector3{0.0f, 1.0f, -1.0f}));
    // Both halves of the creased edge are creased.
    CHECK_EQ(cube.creases.size(), 2);
    CHECK(std::ranges::all_of(cube.creases, [](const auto& crease) { return crease.second == 3.0f; }));

    cube = primitives::cube(2.0f);
    cube.subdivide_faces(std::views::iota(0, 6));
    CHECK_EQ(counts(cube), Counts(26, 24));
    assert_closed(cube, "a subdivided cube");
    assert_close(signed_volume(cube), 8.0f, 1.0e-5f, "the subdivided cube's volume");

    // A triangle splits into 4, and a pentagon into 5 quads.
    Mesh triangle;
    for (Vector3 position : {Vector3{0.0f, 0.0f, 0.0f}, Vector3{2.0f, 0.0f, 0.0f}, Vector3{0.0f, 2.0f, 0.0f}}) {
        triangle.add_vertex(position);
    }
    triangle.add_face({0, 1, 2}, 7);
    triangle.subdivide_faces({0});
    CHECK_EQ(counts(triangle), Counts(6, 4));
    CHECK_EQ(sides_histogram(triangle), (std::vector<size_t>{4}));
    assert_valid(triangle, "a subdivided triangle");
    assert_close(area(triangle, std::views::iota(0, 4)), 2.0f, 1.0e-6f, "the triangle's area");
    CHECK(std::ranges::all_of(triangle.faces, [](const Face& face) { return face.material == 7; }));
    Mesh pentagon = primitives::cone(1.0f, 0.0f, 1.0f, 5);
    pentagon.subdivide_faces({5});
    CHECK_EQ(counts(pentagon), Counts(6 + 5 + 1, 6 + 4));
    assert_closed(pentagon, "a pyramid with its base subdivided");
}

TEST(loop_cuts) {
    // Around the cube's sides, a quarter of the way up from vertex 0.
    Mesh cube = primitives::cube(2.0f);
    Selection selection = unwrap(cube.loop_cut(Edge(0, 3), 0.25f));
    CHECK_EQ(counts(cube), Counts(12, 10));
    assert_closed(cube, "a cube with a loop cut");
    CHECK_EQ(selection.vertices, range_set<std::uint32_t>(8, 12));
    CHECK_EQ(selection.edges.size(), 4);
    CHECK(std::ranges::all_of(selection.vertices, [&](std::uint32_t vertex) { return cube.position(vertex)[1] == -0.5f; }));
    CHECK_EQ(sides_histogram(cube), (std::vector<size_t>{0, 10}));
    assert_close(signed_volume(cube), 8.0f, 1.0e-5f, "the cut cube's volume");

    // Across a grid, from boundary to boundary.
    Mesh grid = primitives::plane({3.0f, 3.0f}, {3, 3});
    selection = unwrap(grid.loop_cut(Edge(5, 9), 0.5f));
    CHECK_EQ(counts(grid), Counts(20, 12));
    assert_valid(grid, "a cut grid");
    CHECK_EQ(grid.topology().boundary_edges().size(), 14);
    CHECK_EQ(selection.edges.size(), 3);

    // Ending at faces that aren't quads, which get the new vertices.
    Mesh cylinder = primitives::cylinder(1.0f, 2.0f, 8, 1);
    unwrap(cylinder.loop_cut(Edge(0, 1), 0.5f));
    CHECK_EQ(counts(cylinder), Counts(18, 11));
    assert_closed(cylinder, "a cut cylinder");
    CHECK_EQ(sides_histogram(cylinder), (std::vector<size_t>{0, 9, 0, 0, 0, 0, 2}));
    Mesh sphere = primitives::uv_sphere(1.0f, 8, 4);
    unwrap(sphere.loop_cut(Edge(1, 2), 0.5f));
    CHECK_EQ(counts(sphere), Counts(29, 34));
    assert_closed(sphere, "a cut sphere");

    Mesh pyramid = primitives::cone(1.0f, 0.0f, 1.0f, 3);
    CHECK_EQ(unwrap_err(pyramid.loop_cut(Edge(0, 1), 0.5f)), "A loop cut goes across quads, and this edge has none next to it.");
    CHECK_EQ(unwrap_err(pyramid.loop_cut(Edge(0, 9), 0.5f)), "The mesh has no edge from vertex 0 to 9.");
}

TEST(flipping_and_recalculating_normals) {
    Mesh cube = primitives::cube(2.0f);
    cube.flip_faces(std::views::iota(0, 6));
    assert_valid(cube, "an inside-out cube");
    assert_close(signed_volume(cube), -8.0f, 1.0e-5f, "the inside-out cube's volume");
    cube.recalculate_normals_outside(std::views::iota(0, 6));
    assert_close(signed_volume(cube), 8.0f, 1.0e-5f, "the cube's volume");

    cube = primitives::cube(2.0f);
    cube.flip_faces({Front, Top});
    CHECK(!cube.topology().is_consistently_wound());
    cube.recalculate_normals_outside(std::views::iota(0, 6));
    CHECK_EQ(cube, primitives::cube(2.0f));

    Mesh torus = primitives::torus(1.0f, 0.25f, 12, 6);
    torus.flip_faces(std::views::iota(0, 72) | std::views::filter([](int face) { return face % 3 != 0; }));
    torus.recalculate_normals_outside(std::views::iota(0, 72));
    assert_valid(torus, "a torus with its normals recalculated");
    CHECK(signed_volume(torus) > 0.0f);

    // An open grid agrees with its first face.
    Mesh grid = primitives::plane({3.0f, 3.0f}, {3, 3});
    grid.flip_faces({4, 7});
    grid.recalculate_normals_outside(std::views::iota(0, 9));
    CHECK_EQ(grid, primitives::plane({3.0f, 3.0f}, {3, 3}));
}

TEST(creases_and_materials) {
    Mesh cube = primitives::cube(2.0f);
    Selection top = Selection::from_faces(cube, {Top});
    std::vector<Edge> edges(top.edges.begin(), top.edges.end());
    edges.push_back(Edge(0, 6));
    cube.crease_edges(edges, 2.5f);
    CHECK_EQ(cube.creases.size(), 4);
    CHECK_EQ(cube.crease(Edge(2, 3)), 2.5f);
    cube.crease_edges({Edge(2, 3)}, 0.0f);
    CHECK_EQ(cube.creases.size(), 3);
    cube.crease_edges({Edge(3, 7)}, 25.0f);
    CHECK_EQ(cube.crease(Edge(3, 7)), 10.0f);

    cube.assign_material({1, 2, 99}, 3);
    std::vector<std::uint32_t> materials;
    for (const Face& face : cube.faces) {
        materials.push_back(face.material);
    }
    CHECK_EQ(materials, (std::vector<std::uint32_t>{0, 3, 3, 0, 0, 0}));
    CHECK_EQ(cube.material_count(), 4);
}

// Two holes that touch at a vertex: two triangles that share only vertex
// 0, filled from their boundary. Each gets a face of its own, instead of
// one face that goes through vertex 0 twice.
TEST(filling_holes_that_touch_at_a_vertex) {
    Mesh mesh;
    for (Vector3 position : {Vector3{0.0f, 0.0f, 0.0f}, Vector3{1.0f, 0.0f, 0.0f}, Vector3{-1.0f, 0.0f, 0.0f}, Vector3{1.0f, 0.0f, 1.0f}, Vector3{-1.0f, 0.0f, 1.0f}}) {
        mesh.add_vertex(position);
    }
    mesh.add_face({2, 0, 4}, 0);
    mesh.add_face({0, 1, 3}, 0);
    std::set<Edge> edges = mesh.edges();
    Selection selection = unwrap(mesh.fill(edges));
    CHECK_EQ(selection.faces, set<size_t>({2, 3}));
    assert_valid(mesh, "two touching triangles with their backs filled");
    CHECK_EQ(sides_histogram(mesh), (std::vector<size_t>{4}));
    CHECK_EQ(mesh.topology().boundary_edges().size(), 0);
}
