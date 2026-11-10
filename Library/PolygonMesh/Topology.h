// How a mesh's vertices, edges and faces connect, computed from its faces on
// demand, since a modeler's edits change it all the time. It owns its data,
// so a program can keep it beside the mesh while the mesh doesn't change.
// Edges are numbered in Edge's order, which doesn't depend on the faces'.
// Nothing assumes a manifold: an edge can have any number of faces, and a
// vertex any fan of them.

#pragma once

#include "PolygonMesh/Mesh.h"

#include <cstddef>
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <tuple>
#include <unordered_map>
#include <utility>
#include <vector>

namespace polygon_mesh {

// An edge ring in order, and the quads that it crosses: faces[i] is between
// edges[i] and edges[i + 1], and for a closed ring, the last is between the
// last edge and the first.
struct Ring {
    std::vector<size_t> edges;
    std::vector<size_t> faces;
    bool closed = false;

    bool operator==(const Ring&) const = default;
};

std::string describe_value(const Ring& ring);

class Topology {
public:
    Topology() = default;
    explicit Topology(const Mesh& mesh);

    std::span<const Edge> edges() const {
        return edges_;
    }

    Edge edge(size_t index) const {
        return edges_[index];
    }

    // The edge's number, or nothing if no face has it.
    std::optional<size_t> edge_index(Edge edge) const;

    // The faces that have the edge, by number, in the mesh's order.
    std::span<const size_t> edge_faces(size_t edge) const {
        return edge_faces_[edge];
    }

    // The faces that have the edge, or none if it isn't one of the mesh's.
    std::span<const size_t> faces_of_edge(Edge edge) const;

    // The edge of each of the face's sides, by number: side i goes from vertex
    // i to vertex i + 1.
    std::span<const size_t> face_edges(size_t face) const {
        return face_edges_[face];
    }

    // The edges at the vertex, by number.
    std::span<const size_t> vertex_edges(std::uint32_t vertex) const {
        return vertex_edges_[vertex];
    }

    // The faces around the vertex, in the mesh's order.
    std::span<const size_t> vertex_faces(std::uint32_t vertex) const {
        return vertex_faces_[vertex];
    }

    // How many edges meet at the vertex.
    size_t valence(std::uint32_t vertex) const {
        return vertex_edges_[vertex].size();
    }

    // An edge with one face.
    bool is_boundary(size_t edge) const {
        return edge_faces_[edge].size() == 1;
    }

    // An edge with more than two faces, or with two that go along it the same
    // way.
    bool is_non_manifold(size_t edge) const {
        return edge_faces_[edge].size() > 2 || !edge_consistent[edge];
    }

    bool is_boundary_vertex(std::uint32_t vertex) const;

    std::vector<Edge> boundary_edges() const;

    std::vector<Edge> non_manifold_edges() const;

    // Whether every edge has two faces: a surface without holes.
    bool is_closed() const;

    // Whether the faces next to each edge agree on which side is their front,
    // so that no two go along an edge the same way.
    bool is_consistently_wound() const;

    // Vertices minus edges plus faces, counting every vertex: 2 for a closed
    // surface like a sphere, 0 for a torus, and 1 for a disk.
    std::int64_t euler_characteristic() const;

    // The edge loop through an edge, in order, as Blender selects it: across
    // each vertex where four edges meet, to the edge that shares no face with
    // the one before, and along the boundary from a boundary edge. It stops
    // where it can't go on, or where it closes.
    std::vector<Edge> edge_loop(Edge edge) const;

    // The edge ring through an edge, in order: from each edge to the opposite
    // side of each quad next to it, as far as the quads go, or until it closes.
    std::vector<Edge> edge_ring(Edge edge) const;

    // The quads that the edge ring through an edge crosses, which Blender
    // selects as a face loop.
    std::vector<size_t> face_loop(Edge edge) const;

    // The edge ring through an edge, with the quads between its edges. An edge
    // with more than two faces has no ring but itself.
    Ring ring(Edge edge) const;

private:
    // The edges after `start` toward `vertex`, and whether the walk came back
    // to `start`.
    std::pair<std::vector<size_t>, bool> walk_loop(size_t start, std::uint32_t vertex, bool boundary) const;

    std::optional<size_t> next_in_loop(size_t edge, std::uint32_t vertex, bool boundary) const;

    // The edges after `start` through `face`, the quads crossed on the way,
    // and whether the walk came back to `start`, when the last quad is the
    // one between the last edge and `start`.
    std::tuple<std::vector<size_t>, std::vector<size_t>, bool> walk_ring(size_t start, size_t face) const;

    std::vector<Edge> edges_;
    std::unordered_map<Edge, size_t> indices;
    std::vector<std::vector<size_t>> edge_faces_;
    // Whether no two of an edge's faces go along it the same way, which two
    // faces of a consistently wound surface never do.
    std::vector<bool> edge_consistent;
    // For each face, the edge of each side, from vertex i to i + 1.
    std::vector<std::vector<size_t>> face_edges_;
    std::vector<std::vector<size_t>> vertex_edges_;
    std::vector<std::vector<size_t>> vertex_faces_;
};

} // namespace polygon_mesh
