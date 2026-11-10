// Library/PolygonMesh's topology: edges and their faces, boundaries,
// valences, and edge loops and rings, on closed, open and non-manifold
// meshes.

#include "Common.h"
#include "PolygonMesh/PolygonMesh.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <array>
#include <initializer_list>
#include <limits>
#include <span>
#include <utility>
#include <vector>

using common::unwrap;
using common::unwrap_err;
using math::Vector3;
using polygon_mesh::Edge;
using polygon_mesh::Mesh;
using polygon_mesh::Ring;
using polygon_mesh::Topology;
namespace primitives = polygon_mesh::primitives;

namespace {

std::vector<Edge> edges(std::initializer_list<std::array<std::uint32_t, 2>> pairs) {
    std::vector<Edge> made;
    for (auto [a, b] : pairs) {
        made.push_back(Edge(a, b));
    }
    return made;
}

std::vector<Edge> sorted(std::vector<Edge> unsorted) {
    std::ranges::stable_sort(unsorted);
    return unsorted;
}

// The numbers as a std::vector, which a check can compare and show.
std::vector<size_t> numbers(std::span<const size_t> items) {
    return std::vector<size_t>(items.begin(), items.end());
}

// Three quads on one edge, like a page between two others.
Mesh three_pages() {
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
    return mesh;
}

} // namespace

TEST(edges_are_undirected_with_the_smaller_vertex_first) {
    Edge edge(7, 3);
    CHECK_EQ(edge, Edge(3, 7));
    CHECK_EQ(edge.vertices(), (std::array<std::uint32_t, 2>{3, 7}));
    CHECK_EQ(std::pair(edge.other(3), edge.other(7)), (std::pair<std::uint32_t, std::uint32_t>(7, 3)));
    CHECK(edge.contains(7) && !edge.contains(5));
}

TEST(a_cube_s_edges_each_have_two_faces_and_its_vertices_three) {
    Mesh cube = primitives::cube(2.0f);
    Topology topology = cube.topology();
    CHECK_EQ(topology.edges().size(), 12);
    for (size_t edge = 0; edge < 12; edge++) {
        CHECK_EQ(topology.edge_faces(edge).size(), 2);
        CHECK(!topology.is_boundary(edge) && !topology.is_non_manifold(edge));
    }
    for (std::uint32_t vertex = 0; vertex < 8; vertex++) {
        CHECK_EQ(topology.valence(vertex), 3);
        CHECK_EQ(topology.vertex_faces(vertex).size(), 3);
        CHECK(!topology.is_boundary_vertex(vertex));
    }
    // Side i of a face goes from its vertex i to vertex i + 1.
    for (size_t face = 0; face < cube.faces.size(); face++) {
        size_t side = 0;
        for (auto [a, b] : cube.faces[face].sides()) {
            CHECK_EQ(topology.edge(topology.face_edges(face)[side]), Edge(a, b));
            side += 1;
        }
    }
    Edge top(2, 3);
    CHECK_EQ(numbers(topology.faces_of_edge(top)), (std::vector<size_t>{0, 3}));
    CHECK_EQ(numbers(topology.faces_of_edge(Edge(0, 6))), std::vector<size_t>{});
    CHECK(topology.is_closed() && topology.is_consistently_wound());
}

TEST(a_plane_s_boundary_and_valences) {
    // Vertex (column, row) is row * 4 + column.
    Mesh plane = primitives::plane({3.0f, 3.0f}, {3, 3});
    Topology topology = plane.topology();
    CHECK_EQ(topology.edges().size(), 24);
    CHECK_EQ(topology.boundary_edges().size(), 12);
    std::vector<size_t> valences;
    std::vector<bool> on_boundary;
    for (std::uint32_t vertex : {0u, 1u, 5u, 15u}) {
        valences.push_back(topology.valence(vertex));
        on_boundary.push_back(topology.is_boundary_vertex(vertex));
    }
    CHECK_EQ(valences, (std::vector<size_t>{2, 3, 4, 2}));
    CHECK_EQ(on_boundary, (std::vector<bool>{true, true, false, true}));
    CHECK_EQ(numbers(topology.vertex_faces(5)), (std::vector<size_t>{0, 1, 3, 4}));
    CHECK(!topology.is_closed());
}

TEST(edge_loops_cross_vertices_of_four_edges) {
    Mesh plane = primitives::plane({3.0f, 3.0f}, {3, 3});
    Topology topology = plane.topology();
    CHECK_EQ(topology.edge_loop(Edge(5, 9)), edges({{1, 5}, {5, 9}, {9, 13}}));
    // Along the boundary, around the corners, back to the start.
    std::vector<Edge> boundary = topology.edge_loop(Edge(0, 1));
    CHECK_EQ(boundary.size(), 12);
    CHECK_EQ(sorted(boundary), sorted(topology.boundary_edges()));
    CHECK_EQ(topology.edge_loop(Edge(0, 5)), std::vector<Edge>{});

    // Vertex (ring, side) is ring * 8 + side.
    Mesh cylinder = primitives::cylinder(1.0f, 2.0f, 8, 3);
    topology = cylinder.topology();
    std::vector<Edge> around = topology.edge_loop(Edge(8, 9));
    std::vector<Edge> ring;
    for (std::uint32_t side = 0; side < 8; side++) {
        ring.push_back(Edge(8 + side, 8 + (side + 1) % 8));
    }
    CHECK_EQ(sorted(around), sorted(ring));
    // Up the side, stopping at the caps, where three edges meet.
    CHECK_EQ(topology.edge_loop(Edge(8, 16)), edges({{0, 8}, {8, 16}, {16, 24}}));

    // Vertex (major, minor) is major * 6 + minor.
    Mesh torus = primitives::torus(1.0f, 0.25f, 12, 6);
    topology = torus.topology();
    CHECK_EQ(topology.edge_loop(Edge(0, 1)).size(), 6);
    CHECK_EQ(topology.edge_loop(Edge(0, 6)).size(), 12);
}

TEST(edge_rings_cross_quads) {
    Mesh plane = primitives::plane({3.0f, 3.0f}, {3, 3});
    Topology topology = plane.topology();
    CHECK_EQ(topology.edge_ring(Edge(5, 9)), edges({{7, 11}, {6, 10}, {5, 9}, {4, 8}}));
    Ring ring = topology.ring(Edge(5, 9));
    CHECK_EQ(std::pair(ring.faces, ring.closed), std::pair(std::vector<size_t>{5, 4, 3}, false));

    Mesh cylinder = primitives::cylinder(1.0f, 2.0f, 8, 3);
    topology = cylinder.topology();
    ring = topology.ring(Edge(8, 16));
    CHECK(ring.closed);
    CHECK_EQ(std::pair(ring.edges.size(), ring.faces.size()), (std::pair<size_t, size_t>(8, 8)));
    CHECK_EQ(topology.face_loop(Edge(8, 16)).size(), 8);
    // From one cap to the other, which aren't quads.
    CHECK_EQ(topology.edge_ring(Edge(8, 9)), edges({{24, 25}, {16, 17}, {8, 9}, {0, 1}}));

    Mesh torus = primitives::torus(1.0f, 0.25f, 12, 6);
    topology = torus.topology();
    CHECK_EQ(topology.edge_ring(Edge(0, 1)).size(), 12);
    CHECK_EQ(topology.edge_ring(Edge(0, 6)).size(), 6);

    // Between the triangles at the poles.
    Mesh sphere = primitives::uv_sphere(1.0f, 8, 4);
    topology = sphere.topology();
    CHECK_EQ(topology.edge_ring(Edge(1, 2)), edges({{17, 18}, {9, 10}, {1, 2}}));
    CHECK_EQ(topology.edge_ring(Edge(0, 1)), edges({{0, 1}}));
}

TEST(non_manifold_meshes_are_described_without_panicking) {
    Mesh mesh = three_pages();
    unwrap(mesh.validate());
    Topology topology = mesh.topology();
    size_t spine = topology.edge_index(Edge(0, 1)).value();
    CHECK_EQ(numbers(topology.edge_faces(spine)), (std::vector<size_t>{0, 1, 2}));
    CHECK(topology.is_non_manifold(spine) && !topology.is_boundary(spine));
    CHECK_EQ(topology.non_manifold_edges(), edges({{0, 1}}));
    CHECK_EQ(topology.boundary_edges().size(), 9);
    CHECK_EQ(topology.edge_ring(Edge(0, 1)), edges({{0, 1}}));
    CHECK_EQ(topology.edge_loop(Edge(0, 1)), edges({{0, 1}}));
    CHECK_EQ(topology.edge_loop(Edge(2, 3)), edges({{1, 2}, {2, 3}, {0, 3}}));
    CHECK_EQ(topology.edge_ring(Edge(1, 2)), edges({{1, 2}, {0, 3}}));
    CHECK_EQ(topology.valence(0), 4);
    CHECK_EQ(topology.euler_characteristic(), 8 - 10 + 3);
}

TEST(faces_that_go_the_same_way_along_an_edge_are_inconsistent) {
    Mesh mesh;
    for (Vector3 position : {Vector3{0.0f, 0.0f, 0.0f}, Vector3{1.0f, 0.0f, 0.0f}, Vector3{0.0f, 1.0f, 0.0f}, Vector3{0.0f, -1.0f, 0.0f}}) {
        mesh.add_vertex(position);
    }
    mesh.add_face({0, 1, 2}, 0);
    mesh.add_face({0, 1, 3}, 0);
    Topology topology = mesh.topology();
    CHECK(!topology.is_consistently_wound());
    CHECK_EQ(topology.non_manifold_edges(), edges({{0, 1}}));
    mesh.faces[1].flip();
    CHECK_EQ(mesh.faces[1].vertices, (std::vector<std::uint32_t>{0, 3, 1}));
    CHECK(mesh.topology().is_consistently_wound());
}

TEST(validation_says_what_s_wrong) {
    Mesh mesh = primitives::cube(1.0f);
    unwrap(mesh.validate());
    mesh.faces[2].vertices[1] = 9;
    CHECK_EQ(unwrap_err(mesh.validate()), "Face 2 uses vertex 9, but there are only 8 vertices.");
    mesh.faces[2].vertices = {0, 1, 0, 4};
    CHECK_EQ(unwrap_err(mesh.validate()), "Face 2 uses a vertex more than once.");
    mesh.faces[2].vertices = {0, 1};
    CHECK_EQ(unwrap_err(mesh.validate()), "Face 2 has 2 vertices, but a face needs at least 3.");
    mesh = primitives::cube(1.0f);
    mesh.positions[3][1] = std::numeric_limits<float>::quiet_NaN();
    CHECK_EQ(unwrap_err(mesh.validate()), "Vertex 3's position isn't finite.");
    mesh = primitives::cube(1.0f);
    mesh.creases.insert_or_assign(Edge(0, 1), 11.0f);
    CHECK(unwrap_err(mesh.validate()).contains("has sharpness 11"));
    mesh.set_crease(Edge(0, 1), 11.0f);
    CHECK_EQ(mesh.crease(Edge(0, 1)), 10.0f);
    unwrap(mesh.validate());
    mesh.set_crease(Edge(0, 1), 0.0f);
    CHECK(mesh.creases.empty());
}
