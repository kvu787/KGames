#include "PolygonMesh/Selection.h"

#include "PolygonMesh/Support.h"
#include "PolygonMesh/Topology.h"

#include <algorithm>
#include <format>
#include <ranges>

namespace polygon_mesh {

std::string describe_value(SelectMode mode) {
    switch (mode) {
    case SelectMode::Vertices:
        return "Vertices";
    case SelectMode::Edges:
        return "Edges";
    case SelectMode::Faces:
        return "Faces";
    }
    return "SelectMode";
}

Selection Selection::from_vertices_impl(const Mesh& mesh, std::set<std::uint32_t> vertices) {
    Selection selection{.vertices = std::move(vertices)};
    for (size_t index = 0; index < mesh.faces.size(); index++) {
        const Face& face = mesh.faces[index];
        for (Edge edge : face.edges()) {
            auto [a, b] = edge.vertices();
            if (selection.vertices.contains(a) && selection.vertices.contains(b)) {
                selection.edges.insert(edge);
            }
        }
        if (std::ranges::all_of(face.vertices, [&](std::uint32_t vertex) { return selection.vertices.contains(vertex); })) {
            selection.faces.insert(index);
        }
    }
    return selection;
}

Selection Selection::from_edges_impl(const Mesh& mesh, std::set<Edge> edges) {
    Selection selection;
    for (Edge edge : edges) {
        for (std::uint32_t vertex : edge.vertices()) {
            selection.vertices.insert(vertex);
        }
    }
    for (size_t face = 0; face < mesh.faces.size(); face++) {
        if (std::ranges::all_of(mesh.faces[face].edges(), [&](Edge edge) { return edges.contains(edge); })) {
            selection.faces.insert(face);
        }
    }
    selection.edges = std::move(edges);
    return selection;
}

Selection Selection::from_faces_impl(const Mesh& mesh, std::set<size_t> faces) {
    Selection selection{.faces = std::move(faces)};
    for (size_t index : selection.faces) {
        const Face& face = mesh.faces[index];
        selection.vertices.insert(face.vertices.begin(), face.vertices.end());
        for (Edge edge : face.edges()) {
            selection.edges.insert(edge);
        }
    }
    return selection;
}

Selection Selection::all(const Mesh& mesh) {
    return from_vertices(mesh, std::views::iota(std::uint32_t{0}, static_cast<std::uint32_t>(mesh.positions.size())));
}

bool Selection::is_empty() const {
    return vertices.empty() && edges.empty() && faces.empty();
}

Selection Selection::in_mode(const Mesh& mesh, SelectMode mode) const {
    switch (mode) {
    case SelectMode::Vertices:
        return from_vertices(mesh, vertices);
    case SelectMode::Edges:
        return from_edges(mesh, edges);
    case SelectMode::Faces:
        break;
    }
    return from_faces(mesh, faces);
}

Selection Selection::inverted(const Mesh& mesh, SelectMode mode) const {
    switch (mode) {
    case SelectMode::Vertices:
        return from_vertices(mesh, std::views::iota(std::uint32_t{0}, static_cast<std::uint32_t>(mesh.positions.size())) |
                                       std::views::filter([&](std::uint32_t vertex) { return !vertices.contains(vertex); }));
    case SelectMode::Edges: {
        std::set<Edge> every = mesh.edges();
        return from_edges(mesh, every | std::views::filter([&](Edge edge) { return !edges.contains(edge); }));
    }
    case SelectMode::Faces:
        break;
    }
    return from_faces(mesh, std::views::iota(size_t{0}, mesh.faces.size()) | std::views::filter([&](size_t face) { return !faces.contains(face); }));
}

Selection Selection::union_(const Selection& other, const Mesh& mesh, SelectMode mode) const {
    Selection both = *this;
    both.vertices.insert(other.vertices.begin(), other.vertices.end());
    both.edges.insert(other.edges.begin(), other.edges.end());
    both.faces.insert(other.faces.begin(), other.faces.end());
    return both.in_mode(mesh, mode);
}

Selection Selection::edge_loop(const Mesh& mesh, Edge edge) {
    return from_edges(mesh, Topology(mesh).edge_loop(edge));
}

Selection Selection::edge_ring(const Mesh& mesh, Edge edge) {
    return from_edges(mesh, Topology(mesh).edge_ring(edge));
}

Selection Selection::face_loop(const Mesh& mesh, Edge edge) {
    return from_faces(mesh, Topology(mesh).face_loop(edge));
}

std::optional<math::Vector3> Selection::center(const Mesh& mesh) const {
    return average(vertices | std::views::filter([&](std::uint32_t vertex) { return vertex < mesh.positions.size(); }) |
                   std::views::transform([&](std::uint32_t vertex) { return mesh.positions[vertex]; }));
}

std::string describe_value(const Selection& selection) {
    return std::format("Selection {{ vertices: {}, edges: {}, faces: {} }}", describe_numbers(selection.vertices, "{}"),
                       describe_list(selection.edges, [](Edge edge) { return describe_value(edge); }, "{}"), describe_numbers(selection.faces, "{}"));
}

} // namespace polygon_mesh
