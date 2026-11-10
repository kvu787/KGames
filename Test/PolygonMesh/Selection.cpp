// Library/PolygonMesh's selections: what selecting vertices, edges and faces
// selects of the others, switching modes, all, none, inverting, loops and
// rings, as in Blender.

#include "PolygonMesh/PolygonMesh.h"
#include "Testing/Testing.h"

#include <array>
#include <initializer_list>
#include <optional>
#include <set>
#include <span>
#include <tuple>

using math::Vector3;
using polygon_mesh::Edge;
using polygon_mesh::Mesh;
using polygon_mesh::Selection;
using polygon_mesh::SelectMode;
namespace primitives = polygon_mesh::primitives;

namespace {

// The cube's faces are -Z, +Z, -Y, +Y, -X and +X. Its top, +Y, is 3, 7, 6, 2.
constexpr size_t Top = 3;
constexpr size_t Front = 1;

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

// How many vertices, edges and faces are selected.
using Counts = std::tuple<size_t, size_t, size_t>;

Counts counts(const Selection& selection) {
    return {selection.vertices.size(), selection.edges.size(), selection.faces.size()};
}

} // namespace

TEST(vertices_select_the_edges_and_faces_between_them) {
    Mesh cube = primitives::cube(2.0f);
    Selection top = Selection::from_vertices(cube, {3, 7, 6, 2});
    CHECK_EQ(top.edges.size(), 4);
    CHECK_EQ(top.faces, set<size_t>({Top}));
    // Two edges, but no face, and no diagonal, which isn't an edge.
    Selection three = Selection::from_vertices(cube, {3, 7, 6});
    CHECK_EQ(three.edges, set({Edge(3, 7), Edge(6, 7)}));
    CHECK(three.faces.empty());
    // Vertices that aren't in the mesh are left out.
    CHECK_EQ(Selection::from_vertices(cube, {1, 99}).vertices, set<std::uint32_t>({1}));
}

TEST(edges_select_their_vertices_and_the_faces_they_go_around) {
    Mesh cube = primitives::cube(2.0f);
    std::array<Edge, 4> around{Edge(3, 7), Edge(6, 7), Edge(2, 6), Edge(2, 3)};
    Selection top = Selection::from_edges(cube, around);
    CHECK_EQ(top.vertices, set<std::uint32_t>({2, 3, 6, 7}));
    CHECK_EQ(top.faces, set<size_t>({Top}));
    Selection three = Selection::from_edges(cube, std::span(around).first(3));
    CHECK(three.faces.empty());
    CHECK_EQ(three.vertices, set<std::uint32_t>({2, 3, 6, 7}));
    CHECK(Selection::from_edges(cube, {Edge(0, 6)}).is_empty());
}

TEST(faces_select_their_edges_and_vertices) {
    Mesh cube = primitives::cube(2.0f);
    Selection two = Selection::from_faces(cube, {Top, Front, 17});
    CHECK_EQ(two.faces, set<size_t>({Front, Top}));
    CHECK_EQ(std::pair(two.vertices.size(), two.edges.size()), (std::pair<size_t, size_t>(6, 7)));
}

TEST(switching_modes_keeps_what_s_selected_of_the_new_mode_s_elements) {
    Mesh cube = primitives::cube(2.0f);
    // The top's vertices and one of the bottom's.
    Selection vertices = Selection::from_vertices(cube, {3, 7, 6, 2, 0});
    CHECK_EQ(vertices.faces, set<size_t>({Top}));
    CHECK_EQ(vertices.edges.size(), 5);
    Selection edges = vertices.in_mode(cube, SelectMode::Edges);
    CHECK_EQ(edges.edges.size(), 5);
    CHECK_EQ(edges.faces, set<size_t>({Top}));
    Selection faces = edges.in_mode(cube, SelectMode::Faces);
    CHECK_EQ(faces, Selection::from_faces(cube, {Top}));
    // Back to vertices from two faces: their six vertices, and what they
    // select, which is the same two faces.
    Selection two = Selection::from_faces(cube, {Top, Front}).in_mode(cube, SelectMode::Vertices);
    CHECK_EQ(two.vertices.size(), 6);
    CHECK_EQ(two.faces, set<size_t>({Front, Top}));
    CHECK_EQ(two.in_mode(cube, SelectMode::Vertices), two);
}

TEST(all_none_and_inverting) {
    Mesh cube = primitives::cube(2.0f);
    Selection all = Selection::all(cube);
    CHECK_EQ(counts(all), Counts(8, 12, 6));
    CHECK(Selection().is_empty());
    CHECK_EQ(all.inverted(cube, SelectMode::Faces), Selection());
    CHECK_EQ(Selection().inverted(cube, SelectMode::Edges), all);

    Selection top = Selection::from_faces(cube, {Top});
    CHECK_EQ(top.inverted(cube, SelectMode::Faces).faces, set<size_t>({0, 1, 2, 4, 5}));
    // The other vertices are the bottom's, which select the bottom face.
    Selection bottom = top.inverted(cube, SelectMode::Vertices);
    CHECK_EQ(bottom.vertices, set<std::uint32_t>({0, 1, 4, 5}));
    CHECK_EQ(bottom.faces, set<size_t>({2}));
    // The other edges are the 8 not around the top, which go all the way
    // around the bottom.
    Selection others = top.inverted(cube, SelectMode::Edges);
    CHECK_EQ(others.edges.size(), 8);
    CHECK_EQ(others.faces, set<size_t>({2}));

    Selection both = top.union_(bottom, cube, SelectMode::Faces);
    CHECK_EQ(both.faces, set<size_t>({2, Top}));
}

TEST(loops_and_rings) {
    // Vertex (ring, side) is ring * 8 + side.
    Mesh cylinder = primitives::cylinder(1.0f, 2.0f, 8, 3);
    Selection around = Selection::edge_loop(cylinder, Edge(8, 9));
    CHECK_EQ(counts(around), Counts(8, 8, 0));
    Selection ring = Selection::edge_ring(cylinder, Edge(8, 16));
    CHECK_EQ(counts(ring), Counts(16, 8, 0));
    Selection faces = Selection::face_loop(cylinder, Edge(8, 16));
    CHECK_EQ(faces.faces, range_set<size_t>(8, 16));
    CHECK_EQ(faces.vertices, range_set<std::uint32_t>(8, 24));
}

TEST(a_selection_s_center_is_its_vertices_average) {
    Mesh cube = primitives::cube(2.0f);
    CHECK_EQ(Selection::from_faces(cube, {Top}).center(cube), std::optional(Vector3{0.0f, 1.0f, 0.0f}));
    CHECK_EQ(Selection::all(cube).center(cube), std::optional(Vector3{0.0f, 0.0f, 0.0f}));
    CHECK_EQ(Selection().center(cube), std::nullopt);
}
