#include "PolygonMesh/Mesh.h"

#include "Base/Text.h"
#include "PolygonMesh/Support.h"
#include "PolygonMesh/Topology.h"

#include <algorithm>
#include <cmath>
#include <format>

namespace polygon_mesh {

// Not math::scale, which Mesh's own scale hides in its member functions: it's
// written out where it's used.
using math::add;
using math::cross;
using math::length;
using math::normalize;
using math::subtract;

std::string describe_value(Edge edge) {
    return "Edge(" + describe_numbers(edge.vertices()) + ")";
}

bool Face::has_side(std::uint32_t a, std::uint32_t b) const {
    return std::ranges::any_of(sides(), [&](std::array<std::uint32_t, 2> side) { return side == std::array{a, b}; });
}

void Face::flip() {
    std::reverse(vertices.begin() + 1, vertices.end());
}

std::string describe_value(const Face& face) {
    return std::format("Face {{ vertices: {}, material: {} }}", describe_numbers(face.vertices), face.material);
}

Topology Mesh::topology() const {
    return Topology(*this);
}

std::set<Edge> Mesh::edges() const {
    // Sorted, and then made into a set in their order, which is how Rust's
    // collect makes a BTreeSet, and faster for a large mesh than inserting
    // them one at a time. Every topology starts here.
    std::vector<Edge> sides;
    for (const Face& face : faces) {
        for (Edge edge : face.edges()) {
            sides.push_back(edge);
        }
    }
    std::ranges::stable_sort(sides);
    return std::set<Edge>(sides.begin(), sides.end());
}

float Mesh::crease(Edge edge) const {
    auto found = creases.find(edge);
    return found == creases.end() ? 0.0f : found->second;
}

void Mesh::set_crease(Edge edge, float sharpness) {
    sharpness = std::isnan(sharpness) ? 0.0f : std::clamp(sharpness, 0.0f, InfinitelySharp);
    if (sharpness > 0.0f) {
        creases.insert_or_assign(edge, sharpness);
    } else {
        creases.erase(edge);
    }
}

std::uint32_t Mesh::material_count() const {
    if (faces.empty()) {
        return 1;
    }
    std::uint32_t count = 0;
    for (const Face& face : faces) {
        count = std::max(count, face.material + 1);
    }
    return count;
}

math::Vector3 Mesh::face_center(size_t face) const {
    const std::vector<std::uint32_t>& vertices = faces[face].vertices;
    math::Vector3 sum{};
    for (std::uint32_t vertex : vertices) {
        sum = add(sum, position(vertex));
    }
    return math::scale(sum, 1.0f / static_cast<float>(vertices.size()));
}

math::Vector3 Mesh::face_area_vector(size_t face) const {
    math::Vector3 center = face_center(face);
    math::Vector3 sum{};
    for (auto [a, b] : faces[face].sides()) {
        sum = add(sum, cross(subtract(position(a), center), subtract(position(b), center)));
    }
    return math::scale(sum, 0.5f);
}

math::Vector3 Mesh::face_normal(size_t face) const {
    return normalize(face_area_vector(face));
}

float Mesh::face_area(size_t face) const {
    return length(face_area_vector(face));
}

std::expected<void, std::string> Mesh::validate() const {
    size_t count = positions.size();
    for (size_t vertex = 0; vertex < count; vertex++) {
        if (std::ranges::any_of(positions[vertex], [](float p) { return !std::isfinite(p); })) {
            return std::unexpected(std::format("Vertex {}'s position isn't finite.", vertex));
        }
    }
    for (size_t index = 0; index < faces.size(); index++) {
        const Face& face = faces[index];
        if (face.vertices.size() < 3) {
            return std::unexpected(std::format("Face {} has {} vertices, but a face needs at least 3.", index, face.vertices.size()));
        }
        auto beyond = std::ranges::find_if(face.vertices, [&](std::uint32_t vertex) { return vertex >= count; });
        if (beyond != face.vertices.end()) {
            return std::unexpected(std::format("Face {} uses vertex {}, but there are only {} vertices.", index, *beyond, count));
        }
        std::set<std::uint32_t> unique(face.vertices.begin(), face.vertices.end());
        if (unique.size() != face.vertices.size()) {
            return std::unexpected(std::format("Face {} uses a vertex more than once.", index));
        }
    }
    for (const auto& [edge, sharpness] : creases) {
        auto [a, b] = edge.vertices();
        if (a == b || b >= count) {
            return std::unexpected(std::format("The crease of {} to {} isn't between two vertices of the mesh.", a, b));
        }
        if (!(sharpness > 0.0f && sharpness <= InfinitelySharp)) {
            return std::unexpected(
                std::format("The crease of {} to {} has sharpness {}, but it must be more than 0 and at most {}.", a, b, base::display(sharpness), base::display(InfinitelySharp)));
        }
    }
    return {};
}

void Mesh::prune_creases() {
    if (creases.empty()) {
        return;
    }
    std::set<Edge> existing = edges();
    std::erase_if(creases, [&](const auto& crease) { return !existing.contains(crease.first); });
}

std::string describe_value(const Mesh& mesh) {
    std::string positions = describe_list(mesh.positions, describe_vector);
    std::string faces = describe_list(mesh.faces, [](const Face& face) { return describe_value(face); });
    std::string creases = describe_list(mesh.creases, [](const auto& crease) { return describe_value(crease.first) + ": " + base::debug(crease.second); }, "{}");
    return std::format("Mesh {{ positions: {}, faces: {}, creases: {} }}", positions, faces, creases);
}

} // namespace polygon_mesh
