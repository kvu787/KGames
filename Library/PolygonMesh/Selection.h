// What's selected, in all three of a modeler's modes at once, as Blender
// keeps it: selecting vertices selects the edges whose ends are both
// selected and the faces whose vertices all are, selecting edges selects
// their vertices and the faces whose edges all are, and selecting faces
// selects their edges and vertices. The functions that make selections make
// ones that agree that way, and switching modes keeps the selection of the
// new mode's elements, like Blender.

#pragma once

#include "PolygonMesh/Mesh.h"

#include <cstddef>
#include <cstdint>
#include <initializer_list>
#include <optional>
#include <set>
#include <string>
#include <utility>

namespace polygon_mesh {

enum class SelectMode {
    Vertices,
    Edges,
    Faces,
};

std::string describe_value(SelectMode mode);

struct Selection {
    std::set<std::uint32_t> vertices;
    std::set<Edge> edges;
    std::set<size_t> faces;

    // The vertices, the edges between two of them, and the faces with all of
    // their vertices among them. Vertices that aren't in the mesh are left
    // out.
    template <RangeOf<std::uint32_t> Vertices = std::initializer_list<std::uint32_t>> static Selection from_vertices(const Mesh& mesh, Vertices&& vertices) {
        return from_vertices_impl(mesh, mesh.vertex_set(vertices));
    }

    // The edges, their vertices, and the faces with all of their edges among
    // them. Edges that aren't in the mesh are left out.
    template <RangeOf<Edge> Edges = std::initializer_list<Edge>> static Selection from_edges(const Mesh& mesh, Edges&& edges) {
        return from_edges_impl(mesh, mesh.edge_set(edges));
    }

    // The faces, and their edges and vertices. Faces that aren't in the mesh
    // are left out.
    template <RangeOf<size_t> Faces = std::initializer_list<size_t>> static Selection from_faces(const Mesh& mesh, Faces&& faces) {
        return from_faces_impl(mesh, mesh.face_set(faces));
    }

    // Everything, loose vertices too.
    static Selection all(const Mesh& mesh);

    bool is_empty() const;

    // The selection that switching to `mode` leaves, like Blender: what's
    // selected of the mode's own elements, and what that selects.
    Selection in_mode(const Mesh& mesh, SelectMode mode) const;

    // The mode's elements that aren't selected, and what they select.
    Selection inverted(const Mesh& mesh, SelectMode mode) const;

    // Both selections' elements, each selection in `mode`, as when adding to
    // a selection with Shift.
    Selection union_(const Selection& other, const Mesh& mesh, SelectMode mode) const;

    // The edge loop through an edge, as Alt+click selects it in Blender's
    // vertex and edge modes.
    static Selection edge_loop(const Mesh& mesh, Edge edge);

    // The edge ring through an edge, as Ctrl+Alt+click selects it.
    static Selection edge_ring(const Mesh& mesh, Edge edge);

    // The quads across the edge ring through an edge, as Alt+click selects
    // them in face mode.
    static Selection face_loop(const Mesh& mesh, Edge edge);

    // The average of the selected vertices' positions, the median point that
    // Blender rotates and scales about, or nothing if none are selected.
    std::optional<math::Vector3> center(const Mesh& mesh) const;

    bool operator==(const Selection&) const = default;

private:
    // What from_vertices, from_edges and from_faces select, from those of
    // their elements that are in the mesh, each once.
    static Selection from_vertices_impl(const Mesh& mesh, std::set<std::uint32_t> vertices);
    static Selection from_edges_impl(const Mesh& mesh, std::set<Edge> edges);
    static Selection from_faces_impl(const Mesh& mesh, std::set<size_t> faces);
};

std::string describe_value(const Selection& selection);

} // namespace polygon_mesh
