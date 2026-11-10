#include "PolygonMesh/Topology.h"

#include "PolygonMesh/Support.h"

#include <algorithm>
#include <format>
#include <set>

namespace polygon_mesh {

std::string describe_value(const Ring& ring) {
    return std::format("Ring {{ edges: {}, faces: {}, closed: {} }}", describe_numbers(ring.edges), describe_numbers(ring.faces), ring.closed);
}

Topology::Topology(const Mesh& mesh) {
    std::set<Edge> sorted = mesh.edges();
    edges_.assign(sorted.begin(), sorted.end());
    indices.reserve(edges_.size());
    for (size_t index = 0; index < edges_.size(); index++) {
        indices.emplace(edges_[index], index);
    }
    edge_faces_.resize(edges_.size());
    std::vector<std::uint32_t> forward(edges_.size(), 0);
    std::vector<std::uint32_t> backward(edges_.size(), 0);
    vertex_faces_.resize(mesh.positions.size());
    face_edges_.reserve(mesh.faces.size());
    for (size_t face_index = 0; face_index < mesh.faces.size(); face_index++) {
        const Face& face = mesh.faces[face_index];
        std::vector<size_t> sides;
        sides.reserve(face.vertices.size());
        for (auto [a, b] : face.sides()) {
            size_t edge = indices.at(Edge(a, b));
            edge_faces_[edge].push_back(face_index);
            if (a < b) {
                forward[edge] += 1;
            } else {
                backward[edge] += 1;
            }
            sides.push_back(edge);
        }
        face_edges_.push_back(std::move(sides));
        for (std::uint32_t vertex : face.vertices) {
            vertex_faces_[vertex].push_back(face_index);
        }
    }
    vertex_edges_.resize(mesh.positions.size());
    for (size_t index = 0; index < edges_.size(); index++) {
        for (std::uint32_t vertex : edges_[index].vertices()) {
            vertex_edges_[vertex].push_back(index);
        }
    }
    edge_consistent.reserve(edges_.size());
    for (size_t edge = 0; edge < edges_.size(); edge++) {
        edge_consistent.push_back(forward[edge] <= 1 && backward[edge] <= 1);
    }
}

std::optional<size_t> Topology::edge_index(Edge edge) const {
    auto found = indices.find(edge);
    if (found == indices.end()) {
        return std::nullopt;
    }
    return found->second;
}

std::span<const size_t> Topology::faces_of_edge(Edge edge) const {
    std::optional<size_t> index = edge_index(edge);
    if (!index) {
        return {};
    }
    return edge_faces_[*index];
}

bool Topology::is_boundary_vertex(std::uint32_t vertex) const {
    return std::ranges::any_of(vertex_edges(vertex), [&](size_t edge) { return is_boundary(edge); });
}

std::vector<Edge> Topology::boundary_edges() const {
    std::vector<Edge> boundary;
    for (size_t edge = 0; edge < edges_.size(); edge++) {
        if (is_boundary(edge)) {
            boundary.push_back(edges_[edge]);
        }
    }
    return boundary;
}

std::vector<Edge> Topology::non_manifold_edges() const {
    std::vector<Edge> non_manifold;
    for (size_t edge = 0; edge < edges_.size(); edge++) {
        if (is_non_manifold(edge)) {
            non_manifold.push_back(edges_[edge]);
        }
    }
    return non_manifold;
}

bool Topology::is_closed() const {
    return std::ranges::all_of(edge_faces_, [](const std::vector<size_t>& faces) { return faces.size() == 2; });
}

bool Topology::is_consistently_wound() const {
    return std::ranges::all_of(edge_consistent, [](bool consistent) { return consistent; });
}

std::int64_t Topology::euler_characteristic() const {
    return static_cast<std::int64_t>(vertex_edges_.size()) - static_cast<std::int64_t>(edges_.size()) + static_cast<std::int64_t>(face_edges_.size());
}

std::vector<Edge> Topology::edge_loop(Edge edge) const {
    std::optional<size_t> found = edge_index(edge);
    if (!found) {
        return {};
    }
    size_t start = *found;
    bool boundary = is_boundary(start);
    auto [a, b] = edges_[start].vertices();
    auto [forward, closed] = walk_loop(start, b, boundary);
    std::vector<size_t> path;
    if (!closed) {
        std::vector<size_t> backward = walk_loop(start, a, boundary).first;
        for (size_t index = backward.size(); index-- > 0;) {
            if (!std::ranges::contains(forward, backward[index])) {
                path.push_back(backward[index]);
            }
        }
    }
    path.push_back(start);
    path.insert(path.end(), forward.begin(), forward.end());
    std::vector<Edge> loop;
    loop.reserve(path.size());
    for (size_t index : path) {
        loop.push_back(edges_[index]);
    }
    return loop;
}

std::pair<std::vector<size_t>, bool> Topology::walk_loop(size_t start, std::uint32_t vertex, bool boundary) const {
    std::vector<size_t> path;
    std::set<size_t> visited{start};
    size_t edge = start;
    while (std::optional<size_t> next = next_in_loop(edge, vertex, boundary)) {
        if (*next == start) {
            return {std::move(path), true};
        }
        if (!visited.insert(*next).second) {
            break;
        }
        path.push_back(*next);
        vertex = edges_[*next].other(vertex);
        edge = *next;
    }
    return {std::move(path), false};
}

std::optional<size_t> Topology::next_in_loop(size_t edge, std::uint32_t vertex, bool boundary) const {
    std::span<const size_t> around = vertex_edges(vertex);
    std::vector<size_t> candidates;
    if (boundary) {
        for (size_t other : around) {
            if (other != edge && is_boundary(other)) {
                candidates.push_back(other);
            }
        }
    } else {
        if (around.size() != 4 || std::ranges::any_of(around, [&](size_t other) { return edge_faces_[other].size() != 2; })) {
            return std::nullopt;
        }
        const std::vector<size_t>& faces = edge_faces_[edge];
        for (size_t other : around) {
            if (other != edge && std::ranges::all_of(edge_faces_[other], [&](size_t face) { return !std::ranges::contains(faces, face); })) {
                candidates.push_back(other);
            }
        }
    }
    if (candidates.size() != 1) {
        return std::nullopt;
    }
    return candidates[0];
}

std::vector<Edge> Topology::edge_ring(Edge edge) const {
    Ring around = ring(edge);
    std::vector<Edge> edges;
    edges.reserve(around.edges.size());
    for (size_t index : around.edges) {
        edges.push_back(edges_[index]);
    }
    return edges;
}

std::vector<size_t> Topology::face_loop(Edge edge) const {
    return ring(edge).faces;
}

Ring Topology::ring(Edge edge) const {
    std::optional<size_t> found = edge_index(edge);
    if (!found) {
        return Ring{};
    }
    size_t start = *found;
    const std::vector<size_t>& faces = edge_faces_[start];
    if (faces.size() > 2) {
        return Ring{.edges = {start}, .faces = {}, .closed = false};
    }
    auto [forward, forward_faces, closed] = walk_ring(start, faces[0]);
    if (closed) {
        std::vector<size_t> edges{start};
        edges.insert(edges.end(), forward.begin(), forward.end());
        return Ring{.edges = std::move(edges), .faces = std::move(forward_faces), .closed = closed};
    }
    std::vector<size_t> backward;
    std::vector<size_t> backward_faces;
    if (faces.size() > 1) {
        std::tie(backward, backward_faces, std::ignore) = walk_ring(start, faces[1]);
    }
    std::vector<size_t> edges(backward.rbegin(), backward.rend());
    edges.push_back(start);
    edges.insert(edges.end(), forward.begin(), forward.end());
    std::vector<size_t> crossed(backward_faces.rbegin(), backward_faces.rend());
    crossed.insert(crossed.end(), forward_faces.begin(), forward_faces.end());
    return Ring{.edges = std::move(edges), .faces = std::move(crossed), .closed = false};
}

std::tuple<std::vector<size_t>, std::vector<size_t>, bool> Topology::walk_ring(size_t start, size_t face) const {
    std::vector<size_t> edges;
    std::vector<size_t> faces;
    std::set<size_t> visited{start};
    size_t edge = start;
    while (true) {
        const std::vector<size_t>& sides = face_edges_[face];
        if (sides.size() != 4) {
            break;
        }
        std::optional<size_t> position = position_of(sides, edge);
        if (!position) {
            break;
        }
        size_t opposite = sides[(*position + 2) % 4];
        if (opposite == start) {
            faces.push_back(face);
            return {std::move(edges), std::move(faces), true};
        }
        if (!visited.insert(opposite).second) {
            break;
        }
        faces.push_back(face);
        edges.push_back(opposite);
        const std::vector<size_t>& next_faces = edge_faces_[opposite];
        if (next_faces.size() != 2) {
            break;
        }
        face = next_faces[0] == face ? next_faces[1] : next_faces[0];
        edge = opposite;
    }
    return {std::move(edges), std::move(faces), false};
}

} // namespace polygon_mesh
