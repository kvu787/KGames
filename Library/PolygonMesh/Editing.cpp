#include "PolygonMesh/Editing.h"

#include "Base/Numbers.h"
#include "Base/VecDeque.h"
#include "PolygonMesh/Support.h"
#include "PolygonMesh/Topology.h"

#include <algorithm>
#include <cmath>
#include <iterator>
#include <numeric>
#include <unordered_map>
#include <unordered_set>

namespace polygon_mesh {

// Not math::scale, which Mesh's own scale hides in its member functions: it's
// written out where it's used.
using math::add;
using math::cross;
using math::dot;
using math::length;
using math::normalize;
using math::subtract;

namespace {

// A polygon's vertices without the sides that collapsed to a point, split
// into simple polygons where it touches itself, without the ones of fewer
// than 3 vertices.
std::vector<std::vector<std::uint32_t>> simple_polygons(std::vector<std::uint32_t> vertices) {
    vertices.erase(std::unique(vertices.begin(), vertices.end()), vertices.end());
    while (vertices.size() > 1 && vertices.front() == vertices.back()) {
        vertices.pop_back();
    }
    if (vertices.size() < 3) {
        return {};
    }
    std::span<const std::uint32_t> all = vertices;
    for (size_t later = 1; later < vertices.size(); later++) {
        if (std::optional<size_t> earlier = position_of(all.first(later), vertices[later])) {
            std::span<const std::uint32_t> after = all.subspan(later);
            std::span<const std::uint32_t> before = all.first(*earlier);
            std::span<const std::uint32_t> between = all.subspan(*earlier, later - *earlier);
            std::vector<std::uint32_t> rest(after.begin(), after.end());
            rest.insert(rest.end(), before.begin(), before.end());
            std::vector<std::vector<std::uint32_t>> pieces = simple_polygons(std::vector<std::uint32_t>(between.begin(), between.end()));
            for (std::vector<std::uint32_t>& piece : simple_polygons(std::move(rest))) {
                pieces.push_back(std::move(piece));
            }
            return pieces;
        }
    }
    std::vector<std::vector<std::uint32_t>> pieces;
    pieces.push_back(std::move(vertices));
    return pieces;
}

// Every vertex as itself: 0, 1, 2 and so on.
std::vector<std::uint32_t> each_vertex(size_t count) {
    std::vector<std::uint32_t> vertices(count);
    std::iota(vertices.begin(), vertices.end(), std::uint32_t{0});
    return vertices;
}

} // namespace

void Mesh::translate_impl(const std::set<std::uint32_t>& vertices, math::Vector3 offset) {
    for (std::uint32_t vertex : vertices) {
        math::Vector3& position = positions[vertex];
        position = add(position, offset);
    }
}

void Mesh::rotate_impl(const std::set<std::uint32_t>& vertices, math::Vector3 pivot, math::Vector3 axis, float angle) {
    axis = normalize(axis);
    if (axis == math::Vector3{}) {
        return;
    }
    float sin = std::sin(angle);
    float cos = std::cos(angle);
    for (std::uint32_t vertex : vertices) {
        math::Vector3& position = positions[vertex];
        // Rodrigues' rotation formula.
        math::Vector3 p = subtract(position, pivot);
        math::Vector3 rotated = add(add(math::scale(p, cos), math::scale(cross(axis, p), sin)), math::scale(axis, dot(axis, p) * (1.0f - cos)));
        position = add(pivot, rotated);
    }
}

void Mesh::scale_impl(const std::set<std::uint32_t>& vertices, math::Vector3 pivot, math::Vector3 factors) {
    for (std::uint32_t vertex : vertices) {
        math::Vector3& position = positions[vertex];
        math::Vector3 p = subtract(position, pivot);
        position = add(pivot, {p[0] * factors[0], p[1] * factors[1], p[2] * factors[2]});
    }
}

void Mesh::transform_impl(const std::set<std::uint32_t>& vertices, math::Vector3 pivot, const math::Matrix& matrix) {
    for (std::uint32_t vertex : vertices) {
        math::Vector3& position = positions[vertex];
        position = add(pivot, matrix.transform_point(subtract(position, pivot)));
    }
}

Selection Mesh::delete_vertices_impl(const std::set<std::uint32_t>& removed) {
    std::set<size_t> using_one;
    for (size_t face = 0; face < faces.size(); face++) {
        if (std::ranges::any_of(faces[face].vertices, [&](std::uint32_t vertex) { return removed.contains(vertex); })) {
            using_one.insert(face);
        }
    }
    remove_faces(using_one);
    remove_vertices(removed);
    return Selection();
}

Selection Mesh::delete_edges_impl(const std::set<Edge>& edges) {
    std::set<size_t> having_one;
    for (size_t face = 0; face < faces.size(); face++) {
        if (std::ranges::any_of(faces[face].edges(), [&](Edge edge) { return edges.contains(edge); })) {
            having_one.insert(face);
        }
    }
    std::set<std::uint32_t> candidates;
    for (Edge edge : edges) {
        for (std::uint32_t vertex : edge.vertices()) {
            candidates.insert(vertex);
        }
    }
    remove_faces(having_one);
    remove_unused(candidates);
    return Selection();
}

Selection Mesh::delete_faces_impl(const std::set<size_t>& selected) {
    std::set<std::uint32_t> candidates;
    for (size_t face : selected) {
        candidates.insert(faces[face].vertices.begin(), faces[face].vertices.end());
    }
    remove_faces(selected);
    remove_unused(candidates);
    return Selection();
}

Selection Mesh::delete_only_faces_impl(const std::set<size_t>& selected) {
    remove_faces(selected);
    return Selection();
}

std::vector<std::optional<std::uint32_t>> Mesh::remove_unused_vertices() {
    std::set<std::uint32_t> used;
    for (const Face& face : faces) {
        used.insert(face.vertices.begin(), face.vertices.end());
    }
    std::set<std::uint32_t> unused;
    for (std::uint32_t vertex = 0; vertex < positions.size(); vertex++) {
        if (!used.contains(vertex)) {
            unused.insert(vertex);
        }
    }
    return remove_vertices(unused);
}

std::expected<Selection, std::string> Mesh::make_face_impl(const std::set<std::uint32_t>& vertices) {
    if (vertices.size() < 3) {
        return std::unexpected("Making a face takes at least 3 vertices.");
    }
    Topology topology = this->topology();
    std::optional<std::vector<std::uint32_t>> cycle = boundary_cycle(topology, vertices);
    std::vector<std::uint32_t> order = cycle ? std::move(*cycle) : around_middle(topology, vertices);
    if (std::ranges::any_of(faces, [&](const Face& face) { return std::set<std::uint32_t>(face.vertices.begin(), face.vertices.end()) == vertices; })) {
        return std::unexpected("There's already a face with these vertices.");
    }
    std::uint32_t material = neighbor_material(topology, order);
    size_t face = add_face(std::move(order), material);
    return Selection::from_faces(*this, {face});
}

std::optional<std::vector<std::uint32_t>> Mesh::boundary_cycle(const Topology& topology, const std::set<std::uint32_t>& vertices) const {
    std::map<std::uint32_t, std::uint32_t> next;
    for (Edge edge : topology.boundary_edges()) {
        auto [a, b] = edge.vertices();
        if (!vertices.contains(a) || !vertices.contains(b)) {
            continue;
        }
        auto [from, to] = filling_side(topology, edge);
        if (!next.emplace(from, to).second) {
            return std::nullopt;
        }
    }
    if (vertices.empty()) {
        return std::nullopt;
    }
    std::uint32_t start = *vertices.begin();
    std::vector<std::uint32_t> cycle{start};
    auto after = next.find(start);
    if (after == next.end()) {
        return std::nullopt;
    }
    std::uint32_t vertex = after->second;
    while (vertex != start) {
        if (cycle.size() > vertices.size()) {
            return std::nullopt;
        }
        cycle.push_back(vertex);
        after = next.find(vertex);
        if (after == next.end()) {
            return std::nullopt;
        }
        vertex = after->second;
    }
    if (cycle.size() != vertices.size()) {
        return std::nullopt;
    }
    return cycle;
}

std::array<std::uint32_t, 2> Mesh::filling_side(const Topology& topology, Edge edge) const {
    auto [a, b] = edge.vertices();
    size_t face = topology.faces_of_edge(edge)[0];
    return faces[face].has_side(a, b) ? std::array{b, a} : std::array{a, b};
}

std::vector<std::uint32_t> Mesh::around_middle(const Topology& topology, const std::set<std::uint32_t>& vertices) const {
    math::Vector3 center = average(vertices | std::views::transform([&](std::uint32_t vertex) { return position(vertex); })).value();
    auto offset = [&](std::uint32_t vertex) {
        return subtract(position(vertex), center);
    };
    // The plane through the farthest vertex and the one farthest from the
    // line through it.
    auto far_ = [&](auto&& measure) {
        return max_by(vertices, [&](std::uint32_t a, std::uint32_t b) { return total_cmp(measure(a), measure(b)); });
    };
    std::uint32_t first = far_([&](std::uint32_t vertex) { return length(offset(vertex)); });
    math::Vector3 u = normalize(offset(first));
    std::uint32_t second = far_([&](std::uint32_t vertex) { return length(cross(u, offset(vertex))); });
    math::Vector3 normal = normalize(cross(offset(first), offset(second)));
    math::Vector3 w = cross(normal, u);
    auto angle = [&](std::uint32_t vertex) {
        return std::atan2(dot(offset(vertex), w), dot(offset(vertex), u));
    };
    std::vector<std::uint32_t> order(vertices.begin(), vertices.end());
    std::ranges::stable_sort(order, [&](std::uint32_t a, std::uint32_t b) { return total_cmp(angle(a), angle(b)) < 0; });
    // Counterclockwise around `normal` now. Each face that shares a side
    // with it should go along that side the other way.
    int agree = 0;
    int disagree = 0;
    size_t count = order.size();
    for (size_t index = 0; index < count; index++) {
        std::uint32_t a = order[index];
        std::uint32_t b = order[(index + 1) % count];
        for (size_t face : topology.faces_of_edge(Edge(a, b))) {
            if (faces[face].has_side(a, b)) {
                disagree += 1;
            } else {
                agree += 1;
            }
        }
    }
    std::optional<math::Vector3> middle = average(positions);
    float away = middle ? dot(normal, subtract(center, *middle)) : 0.0f;
    if (disagree > agree || (agree == disagree && away < 0.0f)) {
        std::reverse(order.begin() + 1, order.end());
    }
    return order;
}

std::uint32_t Mesh::neighbor_material(const Topology& topology, std::span<const std::uint32_t> order) const {
    size_t count = order.size();
    for (size_t index = 0; index < count; index++) {
        std::span<const size_t> beside = topology.faces_of_edge(Edge(order[index], order[(index + 1) % count]));
        if (!beside.empty()) {
            return faces[beside[0]].material;
        }
    }
    for (std::uint32_t vertex : order) {
        std::span<const size_t> around = topology.vertex_faces(vertex);
        if (!around.empty()) {
            return faces[around[0]].material;
        }
    }
    return 0;
}

std::expected<Selection, std::string> Mesh::fill_impl(const std::set<Edge>& edges) {
    Topology topology = this->topology();
    std::map<std::uint32_t, std::vector<std::uint32_t>> next;
    std::vector<std::array<std::uint32_t, 2>> sides;
    for (Edge edge : edges) {
        if (topology.faces_of_edge(edge).size() == 1) {
            auto [from, to] = filling_side(topology, edge);
            next[from].push_back(to);
            sides.push_back({from, to});
        }
    }
    std::set<std::array<std::uint32_t, 2>> used;
    std::vector<size_t> filled;
    for (auto [from, to] : sides) {
        if (!used.insert({from, to}).second) {
            continue;
        }
        std::vector<std::uint32_t> cycle{from};
        std::uint32_t vertex = to;
        std::vector<std::vector<std::uint32_t>> loops;
        bool closed = false;
        while (true) {
            if (vertex == from) {
                closed = true;
                break;
            }
            // Where two holes touch at a vertex, the walk can come back
            // to a vertex on its way, around the second hole: that loop
            // is a face of its own, and the walk goes on from the vertex.
            if (std::optional<size_t> at = position_of(cycle, vertex)) {
                std::vector<std::uint32_t> around(cycle.begin() + static_cast<std::ptrdiff_t>(*at), cycle.end());
                cycle.resize(*at);
                if (around.size() >= 3) {
                    loops.push_back(std::move(around));
                }
            }
            std::optional<std::uint32_t> after;
            if (auto ends = next.find(vertex); ends != next.end()) {
                for (std::uint32_t end : ends->second) {
                    if (!used.contains({vertex, end})) {
                        after = end;
                        break;
                    }
                }
            }
            if (!after) {
                break;
            }
            used.insert({vertex, *after});
            cycle.push_back(vertex);
            vertex = *after;
        }
        if (closed && cycle.size() >= 3) {
            loops.push_back(std::move(cycle));
        }
        for (std::vector<std::uint32_t>& loop : loops) {
            std::uint32_t material = neighbor_material(topology, loop);
            filled.push_back(add_face(std::move(loop), material));
        }
    }
    if (filled.empty()) {
        return std::unexpected("The selection has no loop of boundary edges around a hole to fill.");
    }
    return Selection::from_faces(*this, filled);
}

Selection Mesh::merge_at_center_impl(const std::set<std::uint32_t>& vertices) {
    if (vertices.empty()) {
        return Selection();
    }
    std::uint32_t target = *vertices.begin();
    positions[target] = average(vertices | std::views::transform([&](std::uint32_t vertex) { return position(vertex); })).value();
    std::vector<std::uint32_t> representative = each_vertex(positions.size());
    for (std::uint32_t vertex : vertices) {
        representative[vertex] = target;
    }
    std::vector<std::optional<std::uint32_t>> renumbered = weld(representative);
    if (std::optional<std::uint32_t> merged = renumbered[target]) {
        return Selection::from_vertices(*this, {*merged});
    }
    return Selection::from_vertices(*this, {});
}

Selection Mesh::merge_by_distance_impl(const std::set<std::uint32_t>& vertices, float distance) {
    distance = distance > 0.0f ? distance : 0.0f;
    // Vertices by the cube of `distance` that they're in, so each looks
    // only at its own and the 26 around it.
    double cell_size = static_cast<double>(std::max(distance, 1.0e-6f));
    auto cell = [&](math::Vector3 position) {
        std::array<std::int64_t, 3> numbers{};
        for (size_t axis = 0; axis < 3; axis++) {
            numbers[axis] = base::saturating_cast<std::int64_t>(std::floor(static_cast<double>(position[axis]) / cell_size));
        }
        return numbers;
    };
    std::unordered_map<std::array<std::int64_t, 3>, std::vector<std::uint32_t>, Hash> cells;
    for (std::uint32_t vertex : vertices) {
        cells[cell(position(vertex))].push_back(vertex);
    }
    std::vector<std::uint32_t> representative = each_vertex(positions.size());
    for (std::uint32_t vertex : vertices) {
        if (representative[vertex] != vertex) {
            continue;
        }
        math::Vector3 position = this->position(vertex);
        auto [x, y, z] = cell(position);
        std::vector<std::uint32_t> near_;
        for (std::int64_t dx = -1; dx <= 1; dx++) {
            for (std::int64_t dy = -1; dy <= 1; dy++) {
                for (std::int64_t dz = -1; dz <= 1; dz++) {
                    if (auto found = cells.find({x + dx, y + dy, z + dz}); found != cells.end()) {
                        near_.insert(near_.end(), found->second.begin(), found->second.end());
                    }
                }
            }
        }
        for (std::uint32_t other : near_) {
            if (other > vertex && representative[other] == other && length(subtract(this->position(other), position)) <= distance) {
                representative[other] = vertex;
            }
        }
    }
    std::vector<std::uint32_t> survivors;
    for (std::uint32_t vertex : vertices) {
        if (representative[vertex] == vertex) {
            survivors.push_back(vertex);
        }
    }
    std::vector<std::optional<std::uint32_t>> renumbered = weld(representative);
    return Selection::from_vertices(*this, survivors | std::views::filter([&](std::uint32_t vertex) { return renumbered[vertex].has_value(); }) |
                                               std::views::transform([&](std::uint32_t vertex) { return *renumbered[vertex]; }));
}

std::vector<std::optional<std::uint32_t>> Mesh::weld(std::span<const std::uint32_t> representative) {
    std::unordered_set<std::vector<std::uint32_t>, Hash> seen;
    std::vector<Face> welded;
    welded.reserve(faces.size());
    for (Face& face : std::exchange(faces, {})) {
        std::vector<std::uint32_t> vertices;
        vertices.reserve(face.vertices.size());
        for (std::uint32_t vertex : face.vertices) {
            vertices.push_back(representative[vertex]);
        }
        for (std::vector<std::uint32_t>& piece : simple_polygons(std::move(vertices))) {
            std::vector<std::uint32_t> key = piece;
            std::ranges::sort(key);
            if (seen.insert(std::move(key)).second) {
                welded.emplace_back(std::move(piece), face.material);
            }
        }
    }
    faces = std::move(welded);
    std::map<Edge, float> merged_creases;
    for (const auto& [edge, sharpness] : std::exchange(creases, {})) {
        std::uint32_t a = representative[edge.vertices()[0]];
        std::uint32_t b = representative[edge.vertices()[1]];
        if (a != b) {
            float& merged = merged_creases[Edge(a, b)];
            merged = std::max(merged, sharpness);
        }
    }
    creases = std::move(merged_creases);
    std::set<std::uint32_t> replaced;
    for (std::uint32_t vertex = 0; vertex < positions.size(); vertex++) {
        if (representative[vertex] != vertex) {
            replaced.insert(vertex);
        }
    }
    std::vector<std::optional<std::uint32_t>> renumbered = remove_vertices(replaced);
    prune_creases();
    return renumbered;
}

void Mesh::remove_faces(const std::set<size_t>& removed) {
    if (removed.empty()) {
        return;
    }
    size_t kept = 0;
    for (size_t index = 0; index < faces.size(); index++) {
        if (!removed.contains(index)) {
            if (kept != index) {
                faces[kept] = std::move(faces[index]);
            }
            kept += 1;
        }
    }
    faces.erase(faces.begin() + static_cast<std::ptrdiff_t>(kept), faces.end());
    prune_creases();
}

void Mesh::remove_unused(const std::set<std::uint32_t>& candidates) {
    std::set<std::uint32_t> used;
    for (const Face& face : faces) {
        used.insert(face.vertices.begin(), face.vertices.end());
    }
    std::set<std::uint32_t> unused;
    std::ranges::set_difference(candidates, used, std::inserter(unused, unused.end()));
    remove_vertices(unused);
}

std::vector<std::optional<std::uint32_t>> Mesh::remove_vertices(const std::set<std::uint32_t>& removed) {
    std::vector<std::optional<std::uint32_t>> renumbered;
    renumbered.reserve(positions.size());
    size_t kept = 0;
    for (size_t vertex = 0; vertex < positions.size(); vertex++) {
        if (removed.contains(static_cast<std::uint32_t>(vertex))) {
            renumbered.push_back(std::nullopt);
        } else {
            positions[kept] = positions[vertex];
            renumbered.push_back(static_cast<std::uint32_t>(kept));
            kept += 1;
        }
    }
    if (removed.empty()) {
        return renumbered;
    }
    positions.resize(kept);
    auto new_ = [&](std::uint32_t vertex) {
        return renumbered[vertex];
    };
    std::erase_if(faces, [&](const Face& face) { return !std::ranges::all_of(face.vertices, [&](std::uint32_t vertex) { return new_(vertex).has_value(); }); });
    for (Face& face : faces) {
        for (std::uint32_t& vertex : face.vertices) {
            vertex = new_(vertex).value();
        }
    }
    std::map<Edge, float> renumbered_creases;
    for (const auto& [edge, sharpness] : std::exchange(creases, {})) {
        auto [a, b] = edge.vertices();
        std::optional<std::uint32_t> new_a = new_(a);
        std::optional<std::uint32_t> new_b = new_(b);
        if (new_a && new_b) {
            renumbered_creases.insert_or_assign(Edge(*new_a, *new_b), sharpness);
        }
    }
    creases = std::move(renumbered_creases);
    return renumbered;
}

void Mesh::flip_faces_impl(const std::set<size_t>& selected) {
    for (size_t face : selected) {
        faces[face].flip();
    }
}

void Mesh::recalculate_normals_outside_impl(const std::set<size_t>& selected) {
    Topology topology = this->topology();
    std::set<size_t> visited;
    for (size_t start : selected) {
        if (!visited.insert(start).second) {
            continue;
        }
        std::vector<size_t> surface{start};
        base::VecDeque<size_t> queue{start};
        while (std::optional<size_t> next = queue.pop_front()) {
            size_t face = *next;
            for (size_t edge : topology.face_edges(face)) {
                std::span<const size_t> neighbors = topology.edge_faces(edge);
                if (neighbors.size() != 2) {
                    continue;
                }
                size_t other = neighbors[0] == face ? neighbors[1] : neighbors[0];
                if (!selected.contains(other) || !visited.insert(other).second) {
                    continue;
                }
                auto [a, b] = topology.edge(edge).vertices();
                std::array<std::uint32_t, 2> along = faces[face].has_side(a, b) ? std::array{a, b} : std::array{b, a};
                if (faces[other].has_side(along[0], along[1])) {
                    faces[other].flip();
                }
                surface.push_back(other);
                queue.push_back(other);
            }
        }
        if (faces_in(surface)) {
            for (size_t face : surface) {
                faces[face].flip();
            }
        }
    }
}

bool Mesh::faces_in(std::span<const size_t> surface) const {
    std::set<std::uint32_t> vertices;
    for (size_t face : surface) {
        vertices.insert(faces[face].vertices.begin(), faces[face].vertices.end());
    }
    math::Vector3 middle = average(vertices | std::views::transform([&](std::uint32_t vertex) { return position(vertex); })).value();
    double volume = 0.0;
    float size = 0.0f;
    for (size_t face : surface) {
        std::vector<math::Vector3> corners;
        corners.reserve(faces[face].vertices.size());
        for (std::uint32_t vertex : faces[face].vertices) {
            corners.push_back(subtract(position(vertex), middle));
        }
        for (size_t index = 1; index + 1 < corners.size(); index++) {
            volume += static_cast<double>(dot(corners[0], cross(corners[index], corners[index + 1]))) / 6.0;
        }
        for (math::Vector3 corner : corners) {
            size = std::max(size, length(corner));
        }
    }
    // The size cubed, multiplied out as powi(3) is.
    double wide = static_cast<double>(size);
    if (std::abs(volume) > 1.0e-4 * (wide * wide * wide)) {
        return volume < 0.0;
    }
    size_t farthest = max_by(surface, [&](size_t a, size_t b) {
        auto distance = [&](size_t face) {
            return length(subtract(face_center(face), middle));
        };
        return total_cmp(distance(a), distance(b));
    });
    return dot(face_normal(farthest), subtract(face_center(farthest), middle)) < 0.0f;
}

void Mesh::crease_edges_impl(const std::set<Edge>& edges, float sharpness) {
    for (Edge edge : edges) {
        set_crease(edge, sharpness);
    }
}

void Mesh::assign_material_impl(const std::set<size_t>& selected, std::uint32_t material) {
    for (size_t face : selected) {
        faces[face].material = material;
    }
}

} // namespace polygon_mesh
