#include "PolygonMesh/Cutting.h"

#include "PolygonMesh/Support.h"
#include "PolygonMesh/Topology.h"

#include <algorithm>
#include <cmath>
#include <format>

namespace polygon_mesh {

namespace {

// The face's vertices with each cut edge's new vertex between its ends.
std::vector<std::uint32_t> with_cuts(const Face& face, const std::map<Edge, std::uint32_t>& cuts) {
    std::vector<std::uint32_t> vertices;
    vertices.reserve(face.vertices.size());
    for (auto [a, b] : face.sides()) {
        vertices.push_back(a);
        if (auto cut = cuts.find(Edge(a, b)); cut != cuts.end()) {
            vertices.push_back(cut->second);
        }
    }
    return vertices;
}

// The side of a quad across from the side from `from` to `to`, from its end
// beside `from`.
std::array<std::uint32_t, 2> across(const Face& face, std::array<std::uint32_t, 2> side) {
    auto [from, to] = side;
    const std::vector<std::uint32_t>& quad = face.vertices;
    size_t place = position_of(quad, from).value();
    if (quad[(place + 1) % 4] == to) {
        return {quad[(place + 3) % 4], quad[(place + 2) % 4]};
    }
    return {quad[(place + 1) % 4], quad[(place + 2) % 4]};
}

} // namespace

Selection Mesh::subdivide_faces_impl(const std::set<size_t>& selected) {
    std::set<Edge> edges;
    for (size_t face : selected) {
        for (Edge edge : faces[face].edges()) {
            edges.insert(edge);
        }
    }
    std::map<Edge, std::uint32_t> midpoints;
    for (Edge edge : edges) {
        math::Vector3 a = position(edge.vertices()[0]);
        math::Vector3 b = position(edge.vertices()[1]);
        midpoints.insert_or_assign(edge, add_vertex(lerp(a, b, 0.5f)));
    }
    split_creases(midpoints);
    std::vector<size_t> pieces;
    std::vector<Face> extra;
    for (size_t index = 0; index < faces.size(); index++) {
        if (!selected.contains(index)) {
            faces[index].vertices = with_cuts(faces[index], midpoints);
            continue;
        }
        std::vector<std::uint32_t> vertices = faces[index].vertices;
        std::uint32_t material = faces[index].material;
        size_t count = vertices.size();
        auto middle = [&](size_t place) {
            return midpoints.at(Edge(vertices[place % count], vertices[(place + 1) % count]));
        };
        std::vector<std::vector<std::uint32_t>> split;
        if (count == 3) {
            std::uint32_t ab = middle(0);
            std::uint32_t bc = middle(1);
            std::uint32_t ca = middle(2);
            split.reserve(4);
            split.push_back({vertices[0], ab, ca});
            split.push_back({ab, vertices[1], bc});
            split.push_back({ca, bc, vertices[2]});
            split.push_back({ab, bc, ca});
        } else {
            std::uint32_t center = add_vertex(face_center(index));
            split.reserve(count);
            for (size_t place = 0; place < count; place++) {
                split.push_back({vertices[place], middle(place), center, middle(place + count - 1)});
            }
        }
        faces[index].vertices = std::move(split[0]);
        pieces.push_back(index);
        for (size_t piece = 1; piece < split.size(); piece++) {
            extra.emplace_back(std::move(split[piece]), material);
        }
    }
    for (Face& face : extra) {
        pieces.push_back(faces.size());
        faces.push_back(std::move(face));
    }
    prune_creases();
    return Selection::from_faces(*this, pieces);
}

std::expected<Selection, std::string> Mesh::loop_cut(Edge edge, float fraction) {
    Topology topology = this->topology();
    auto [a, b] = edge.vertices();
    if (!topology.edge_index(edge)) {
        return std::unexpected(std::format("The mesh has no edge from vertex {} to {}.", a, b));
    }
    Ring ring = topology.ring(edge);
    if (ring.faces.empty()) {
        return std::unexpected("A loop cut goes across quads, and this edge has none next to it.");
    }
    if (std::set<size_t>(ring.faces.begin(), ring.faces.end()).size() != ring.faces.size()) {
        return std::unexpected("This edge's ring crosses a quad twice, so it can't be cut.");
    }
    if (std::ranges::any_of(ring.edges, [&](size_t index) { return topology.edge_faces(index).size() > 2; })) {
        return std::unexpected("The loop runs through an edge with more than two faces, so it can't be cut.");
    }
    fraction = std::isnan(fraction) ? 0.5f : std::clamp(fraction, 0.0f, 1.0f);
    // Each edge of the ring from its end beside the start of the one
    // before, from `edge` both ways.
    size_t count = ring.edges.size();
    size_t start = position_of(ring.edges, edge, [&](size_t index) { return topology.edge(index); }).value();
    std::vector<std::array<std::uint32_t, 2>> directions(count, {0, 0});
    directions[start] = {a, b};
    size_t forward = ring.closed ? count - 1 : count - 1 - start;
    for (size_t step = 0; step < forward; step++) {
        size_t from = (start + step) % count;
        size_t to = (start + step + 1) % count;
        directions[to] = across(faces[ring.faces[from]], directions[from]);
    }
    if (!ring.closed) {
        for (size_t to = start; to-- > 0;) {
            directions[to] = across(faces[ring.faces[to]], directions[to + 1]);
        }
    }
    std::map<Edge, std::uint32_t> cuts;
    for (size_t place = 0; place < count; place++) {
        auto [from, to] = directions[place];
        math::Vector3 position = lerp(this->position(from), this->position(to), fraction);
        cuts.insert_or_assign(topology.edge(ring.edges[place]), add_vertex(position));
    }
    split_creases(cuts);
    std::set<size_t> crossed(ring.faces.begin(), ring.faces.end());
    for (size_t index = 0; index < faces.size(); index++) {
        if (!crossed.contains(index)) {
            faces[index].vertices = with_cuts(faces[index], cuts);
        }
    }
    std::vector<Edge> loop_edges;
    loop_edges.reserve(ring.faces.size());
    for (size_t face : ring.faces) {
        std::vector<std::uint32_t> quad = faces[face].vertices;
        std::uint32_t material = faces[face].material;
        auto cut = [&](size_t place) -> std::optional<std::uint32_t> {
            auto found = cuts.find(Edge(quad[place % 4], quad[(place + 1) % 4]));
            if (found == cuts.end()) {
                return std::nullopt;
            }
            return found->second;
        };
        std::optional<size_t> cut_place;
        for (size_t candidate = 0; candidate < 2; candidate++) {
            if (cut(candidate) && cut(candidate + 2)) {
                cut_place = candidate;
                break;
            }
        }
        size_t place = cut_place.value();
        std::uint32_t first = cut(place).value();
        std::uint32_t second = cut(place + 2).value();
        auto corner = [&](size_t offset) {
            return quad[(place + offset) % 4];
        };
        faces[face].vertices = {corner(0), first, second, corner(3)};
        add_face({first, corner(1), corner(2), second}, material);
        loop_edges.push_back(Edge(first, second));
    }
    prune_creases();
    return Selection::from_edges(*this, loop_edges);
}

void Mesh::split_creases(const std::map<Edge, std::uint32_t>& cuts) {
    for (const auto& [edge, middle] : cuts) {
        float sharpness = crease(edge);
        if (sharpness > 0.0f) {
            auto [a, b] = edge.vertices();
            set_crease(Edge(a, middle), sharpness);
            set_crease(Edge(middle, b), sharpness);
        }
    }
}

} // namespace polygon_mesh
