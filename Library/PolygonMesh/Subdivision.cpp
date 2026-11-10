#include "PolygonMesh/Subdivision.h"

#include "PolygonMesh/Support.h"
#include "PolygonMesh/Topology.h"

#include <algorithm>
#include <format>
#include <numeric>

namespace polygon_mesh {

using math::add;

namespace {

enum class Rule {
    Smooth,
    Dart,
    Crease,
    Corner,
};

// An edge's sharpness after a level: 1 less, infinite staying infinite.
float child_sharpness(float sharpness) {
    if (sharpness >= InfinitelySharp) {
        return InfinitelySharp;
    }
    if (sharpness > 1.0f) {
        return sharpness - 1.0f;
    }
    return 0.0f;
}

Rule rule_of(bool corner, size_t sharp_edges) {
    if (corner) {
        return Rule::Corner;
    }
    switch (sharp_edges) {
    case 0:
        return Rule::Smooth;
    case 1:
        return Rule::Dart;
    case 2:
        return Rule::Crease;
    default:
        return Rule::Corner;
    }
}

// The smooth rule: the average of the vertex's face points, twice the average
// of its edges' midpoints, and n - 3 times the vertex, over its valence n.
math::Vector3 smooth_point(const Mesh& mesh, const Topology& topology, std::span<const math::Vector3> face_points, std::uint32_t vertex) {
    math::Vector3 position = mesh.position(vertex);
    std::span<const size_t> around = topology.vertex_edges(vertex);
    std::span<const size_t> faces = topology.vertex_faces(vertex);
    float valence = static_cast<float>(around.size());
    math::Vector3 faces_sum{};
    for (size_t face : faces) {
        faces_sum = add(faces_sum, face_points[face]);
    }
    math::Vector3 faces_average = math::scale(faces_sum, 1.0f / static_cast<float>(faces.size()));
    math::Vector3 neighbors_sum{};
    for (size_t edge : around) {
        neighbors_sum = add(neighbors_sum, mesh.position(topology.edge(edge).other(vertex)));
    }
    math::Vector3 neighbors_average = math::scale(neighbors_sum, 1.0f / valence);
    // Q + 2R + (n - 3) S, where 2R is S plus the neighbors' average.
    return math::scale(add(add(faces_average, neighbors_average), math::scale(position, valence - 2.0f)), 1.0f / valence);
}

// The crease rule: along the curve of the two sharp edges, 6/8 of the vertex
// and 1/8 of each edge's other end.
math::Vector3 crease_point(const Mesh& mesh, const Topology& topology, std::uint32_t vertex, std::span<const size_t> sharp) {
    math::Vector3 a = mesh.position(topology.edge(sharp[0]).other(vertex));
    math::Vector3 b = mesh.position(topology.edge(sharp[1]).other(vertex));
    return math::scale(add(add(a, b), math::scale(mesh.position(vertex), 6.0f)), 0.125f);
}

// Whether the vertex is infinitely sharp: on the boundary with only one face,
// a corner by VTX_BOUNDARY_EDGE_AND_CORNER, or where the surface isn't a
// manifold: an edge of it with more than two faces or two that disagree, more
// than one stretch of boundary through it, or faces around it that don't make
// one fan.
bool is_corner(const Topology& topology, std::uint32_t vertex) {
    std::span<const size_t> around = topology.vertex_edges(vertex);
    std::span<const size_t> faces = topology.vertex_faces(vertex);
    size_t boundary = static_cast<size_t>(std::ranges::count_if(around, [&](size_t edge) { return topology.is_boundary(edge); }));
    if ((boundary > 0 && faces.size() == 1) || std::ranges::any_of(around, [&](size_t edge) { return topology.is_non_manifold(edge); })) {
        return true;
    }
    if (boundary != 0 && boundary != 2) {
        return true;
    }
    // One fan: the faces joined through the edges between them are one group.
    std::vector<size_t> group(faces.size());
    std::iota(group.begin(), group.end(), size_t{0});
    for (size_t edge : around) {
        std::span<const size_t> pair = topology.edge_faces(edge);
        if (pair.size() == 2) {
            auto place = [&](size_t face) {
                return position_of(faces, face).value();
            };
            size_t old = group[place(pair[1])];
            size_t new_ = group[place(pair[0])];
            for (size_t& member : group) {
                if (member == old) {
                    member = new_;
                }
            }
        }
    }
    return std::ranges::any_of(group, [&](size_t member) { return member != group[0]; });
}

// Where a vertex moves: by its rule this level, blended with its rule at the
// next where that's different.
math::Vector3 vertex_point(const Mesh& mesh, const Topology& topology, std::span<const math::Vector3> face_points, std::span<const float> sharpness, std::span<const float> child,
                           std::uint32_t vertex) {
    math::Vector3 position = mesh.position(vertex);
    if (topology.vertex_faces(vertex).empty()) {
        return position;
    }
    bool corner = is_corner(topology, vertex);
    std::span<const size_t> around = topology.vertex_edges(vertex);
    std::vector<size_t> sharp;
    std::vector<size_t> sharp_after;
    for (size_t edge : around) {
        if (sharpness[edge] > 0.0f) {
            sharp.push_back(edge);
        }
    }
    for (size_t edge : around) {
        if (child[edge] > 0.0f) {
            sharp_after.push_back(edge);
        }
    }
    Rule rule = rule_of(corner, sharp.size());
    Rule rule_after = rule_of(corner, sharp_after.size());
    auto point_by = [&](Rule by, std::span<const size_t> sharp_edges) {
        switch (by) {
        case Rule::Smooth:
        case Rule::Dart:
            return smooth_point(mesh, topology, face_points, vertex);
        case Rule::Crease:
            return crease_point(mesh, topology, vertex, sharp_edges);
        case Rule::Corner:
            break;
        }
        return position;
    };
    math::Vector3 point = point_by(rule, sharp);
    if (rule == Rule::Smooth || rule == Rule::Dart || rule_after == rule) {
        return point;
    }
    // The fractional weight: the average sharpness of the edges that stop
    // being sharp after this level.
    std::vector<float> fading;
    for (size_t edge : sharp) {
        if (child[edge] == 0.0f) {
            fading.push_back(sharpness[edge]);
        }
    }
    float weight = 0.0f;
    if (!fading.empty()) {
        float sum = 0.0f;
        for (float faded : fading) {
            sum += faded;
        }
        weight = std::min(sum / static_cast<float>(fading.size()), 1.0f);
    }
    return lerp(point_by(rule_after, sharp_after), point, weight);
}

// One level, and the face that each new face comes from.
std::pair<Mesh, std::vector<size_t>> subdivide(const Mesh& mesh) {
    Topology topology = mesh.topology();
    std::span<const Edge> edges = topology.edges();
    size_t vertex_count = mesh.positions.size();
    size_t edge_count = edges.size();
    // Boundary and non-manifold edges are infinitely sharp, whatever their
    // creases say.
    std::vector<float> sharpness;
    sharpness.reserve(edge_count);
    for (size_t edge = 0; edge < edge_count; edge++) {
        if (topology.edge_faces(edge).size() != 2 || topology.is_non_manifold(edge)) {
            sharpness.push_back(InfinitelySharp);
        } else {
            sharpness.push_back(mesh.crease(edges[edge]));
        }
    }
    std::vector<float> child;
    child.reserve(edge_count);
    for (float sharp : sharpness) {
        child.push_back(child_sharpness(sharp));
    }
    std::vector<math::Vector3> face_points;
    face_points.reserve(mesh.faces.size());
    for (size_t face = 0; face < mesh.faces.size(); face++) {
        face_points.push_back(mesh.face_center(face));
    }

    std::vector<math::Vector3> positions;
    positions.reserve(vertex_count + edge_count + face_points.size());
    for (std::uint32_t vertex = 0; vertex < vertex_count; vertex++) {
        positions.push_back(vertex_point(mesh, topology, face_points, sharpness, child, vertex));
    }
    for (size_t index = 0; index < edge_count; index++) {
        math::Vector3 a = mesh.position(edges[index].vertices()[0]);
        math::Vector3 b = mesh.position(edges[index].vertices()[1]);
        math::Vector3 sharp = lerp(a, b, 0.5f);
        auto smooth = [&] {
            std::span<const size_t> faces = topology.edge_faces(index);
            return math::scale(add(add(a, b), add(face_points[faces[0]], face_points[faces[1]])), 0.25f);
        };
        // A crease that's gone after this level blends toward smooth by how
        // much of it was left.
        math::Vector3 point;
        if (!(sharpness[index] > 0.0f)) {
            point = smooth();
        } else if (child[index] > 0.0f) {
            point = sharp;
        } else {
            point = lerp(smooth(), sharp, std::min(sharpness[index], 1.0f));
        }
        positions.push_back(point);
    }
    positions.insert(positions.end(), face_points.begin(), face_points.end());

    auto edge_point = [&](size_t edge) {
        return static_cast<std::uint32_t>(vertex_count + edge);
    };
    std::vector<Face> faces;
    faces.reserve(mesh.faces.size() * 4);
    std::vector<size_t> parents;
    parents.reserve(mesh.faces.size() * 4);
    for (size_t index = 0; index < mesh.faces.size(); index++) {
        const Face& face = mesh.faces[index];
        std::span<const size_t> sides = topology.face_edges(index);
        size_t count = sides.size();
        std::uint32_t center = static_cast<std::uint32_t>(vertex_count + edge_count + index);
        for (size_t corner = 0; corner < count; corner++) {
            size_t before = sides[(corner + count - 1) % count];
            faces.emplace_back(std::vector<std::uint32_t>{face.vertices[corner], edge_point(sides[corner]), center, edge_point(before)}, face.material);
            parents.push_back(index);
        }
    }
    Mesh child_mesh{.positions = std::move(positions), .faces = std::move(faces)};
    for (const auto& [edge, crease] : mesh.creases) {
        std::optional<size_t> index = topology.edge_index(edge);
        if (!index || !(child_sharpness(crease) > 0.0f)) {
            continue;
        }
        auto [a, b] = edge.vertices();
        child_mesh.set_crease(Edge(a, edge_point(*index)), child_sharpness(crease));
        child_mesh.set_crease(Edge(edge_point(*index), b), child_sharpness(crease));
    }
    return {std::move(child_mesh), std::move(parents)};
}

} // namespace

std::string describe_value(const SubdivisionSurface& surface) {
    return std::format("SubdivisionSurface {{ mesh: {}, base_faces: {} }}", describe_value(surface.mesh), describe_numbers(surface.base_faces));
}

SubdivisionSurface Mesh::catmull_clark(std::uint32_t levels) const {
    SubdivisionSurface surface{.mesh = *this, .base_faces = std::vector<size_t>(faces.size())};
    std::iota(surface.base_faces.begin(), surface.base_faces.end(), size_t{0});
    for (std::uint32_t level = 0; level < levels; level++) {
        auto [mesh, parents] = subdivide(surface.mesh);
        for (size_t& parent : parents) {
            parent = surface.base_faces[parent];
        }
        surface.base_faces = std::move(parents);
        surface.mesh = std::move(mesh);
    }
    return surface;
}

} // namespace polygon_mesh
