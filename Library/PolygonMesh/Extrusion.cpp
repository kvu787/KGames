#include "PolygonMesh/Extrusion.h"

#include "PolygonMesh/Support.h"
#include "PolygonMesh/Topology.h"

#include <algorithm>
#include <numeric>
#include <unordered_map>
#include <utility>

namespace polygon_mesh {

// Not math::scale, which Mesh's own scale hides in its member functions: it's
// written out where it's used.
using math::add;
using math::cross;
using math::dot;
using math::normalize;
using math::subtract;

namespace {

// A corner of a face: the face, and a place in its list of vertices.
using Corner = std::pair<size_t, size_t>;

// How many of the faces have each of their edges: 1 on the boundary of the
// region that they make.
std::map<Edge, size_t> edge_uses(const Mesh& mesh, const std::set<size_t>& faces) {
    std::map<Edge, size_t> uses;
    for (size_t face : faces) {
        for (Edge edge : mesh.faces[face].edges()) {
            uses[edge] += 1;
        }
    }
    return uses;
}

// Where a vertex moves to be `thickness` inside each of the edges that it's
// on, from the directions into the face away from each edge. Along the
// bisector of two of them, it goes farther the more they turn, up to 4 times
// as far, for a nearly folded-back corner.
math::Vector3 inset_offset(std::span<const math::Vector3> inward, float thickness) {
    math::Vector3 sum{};
    for (math::Vector3 direction : inward) {
        sum = add(sum, direction);
    }
    math::Vector3 direction = normalize(sum);
    if (direction == math::Vector3{}) {
        return math::scale(inward[0], thickness);
    }
    if (inward.size() == 2) {
        return math::scale(direction, thickness / std::max(dot(direction, inward[0]), 0.25f));
    }
    return math::scale(direction, thickness);
}

} // namespace

Selection Mesh::extrude_faces_impl(const std::set<size_t>& region) {
    if (region.empty()) {
        return Selection();
    }
    Topology topology = this->topology();
    std::map<Edge, size_t> uses = edge_uses(*this, region);
    auto outside = [&](size_t face) {
        return !region.contains(face);
    };
    bool leaves_copy = !std::ranges::any_of(uses, [&](const auto& use) { return std::ranges::any_of(topology.faces_of_edge(use.first), outside); });
    std::set<std::uint32_t> on_boundary;
    for (const auto& [edge, count] : uses) {
        if (count == 1) {
            for (std::uint32_t vertex : edge.vertices()) {
                on_boundary.insert(vertex);
            }
        }
    }
    std::set<std::uint32_t> vertices;
    for (size_t face : region) {
        vertices.insert(faces[face].vertices.begin(), faces[face].vertices.end());
    }
    std::map<std::uint32_t, std::uint32_t> copies;
    for (std::uint32_t vertex : vertices) {
        if (leaves_copy || on_boundary.contains(vertex) || std::ranges::any_of(topology.vertex_faces(vertex), outside)) {
            copies.emplace(vertex, add_vertex(position(vertex)));
        }
    }
    auto moved = [&](std::uint32_t vertex) {
        auto copy = copies.find(vertex);
        return copy == copies.end() ? vertex : copy->second;
    };
    // Creases go with the region's edges, and stay on the boundary's old
    // edges too, which the walls start from.
    for (const auto& use : uses) {
        Edge edge = use.first;
        auto [a, b] = edge.vertices();
        if (crease(edge) > 0.0f) {
            set_crease(Edge(moved(a), moved(b)), crease(edge));
        }
    }
    std::vector<Face> originals;
    originals.reserve(region.size());
    for (size_t face : region) {
        originals.push_back(faces[face]);
    }
    for (size_t face : region) {
        for (std::uint32_t& vertex : faces[face].vertices) {
            vertex = moved(vertex);
        }
    }
    for (const Face& face : originals) {
        for (auto [a, b] : face.sides()) {
            if (uses.at(Edge(a, b)) == 1) {
                add_face({a, b, moved(b), moved(a)}, face.material);
            }
        }
    }
    if (leaves_copy) {
        for (Face& face : originals) {
            face.flip();
            faces.push_back(std::move(face));
        }
    }
    prune_creases();
    return Selection::from_faces(*this, region);
}

Selection Mesh::extrude_edges_impl(const std::set<Edge>& edges) {
    Topology topology = this->topology();
    std::map<std::uint32_t, std::uint32_t> copies;
    for (Edge edge : edges) {
        for (std::uint32_t vertex : edge.vertices()) {
            if (!copies.contains(vertex)) {
                copies.emplace(vertex, add_vertex(position(vertex)));
            }
        }
    }
    std::vector<Edge> extruded;
    extruded.reserve(edges.size());
    for (Edge edge : edges) {
        size_t face = topology.faces_of_edge(edge)[0];
        auto [a, b] = edge.vertices();
        if (!faces[face].has_side(a, b)) {
            std::swap(a, b);
        }
        std::uint32_t material = faces[face].material;
        std::uint32_t copy_a = copies.at(a);
        std::uint32_t copy_b = copies.at(b);
        add_face({b, a, copy_a, copy_b}, material);
        set_crease(Edge(copy_a, copy_b), crease(edge));
        extruded.push_back(Edge(copy_a, copy_b));
    }
    return Selection::from_edges(*this, extruded);
}

Selection Mesh::inset_region_impl(const std::set<size_t>& region, float thickness) {
    if (region.empty()) {
        return Selection();
    }
    std::map<Edge, size_t> uses = edge_uses(*this, region);
    std::map<size_t, math::Vector3> normals;
    for (size_t face : region) {
        normals.emplace(face, face_normal(face));
    }
    // Each vertex's corners in the region's faces, as a face and a place
    // in it.
    std::map<std::uint32_t, std::vector<Corner>> corners;
    for (size_t face : region) {
        const std::vector<std::uint32_t>& vertices = faces[face].vertices;
        for (size_t place = 0; place < vertices.size(); place++) {
            corners[vertices[place]].push_back({face, place});
        }
    }
    // Each copy to make, with the corners that move onto it.
    std::vector<std::pair<math::Vector3, std::vector<Corner>>> moves;
    for (const auto& [vertex, at_vertex] : corners) {
        // Corners that share an edge inside the region move together. A
        // vertex where the region touches itself only at the vertex gets
        // a copy for each part.
        auto sides = [&](Corner corner) {
            const std::vector<std::uint32_t>& vertices = faces[corner.first].vertices;
            size_t count = vertices.size();
            return std::array{vertices[(corner.second + count - 1) % count], vertices[(corner.second + 1) % count]};
        };
        std::vector<size_t> group(at_vertex.size());
        std::iota(group.begin(), group.end(), size_t{0});
        for (size_t first = 0; first < at_vertex.size(); first++) {
            for (size_t second = first + 1; second < at_vertex.size(); second++) {
                bool shared = std::ranges::any_of(sides(at_vertex[first]),
                                                  [&](std::uint32_t end) { return std::ranges::contains(sides(at_vertex[second]), end) && uses.at(Edge(vertex, end)) >= 2; });
                if (shared) {
                    size_t old = group[second];
                    size_t new_ = group[first];
                    for (size_t& member : group) {
                        if (member == old) {
                            member = new_;
                        }
                    }
                }
            }
        }
        for (size_t root : std::set<size_t>(group.begin(), group.end())) {
            std::vector<Corner> members;
            for (size_t corner = 0; corner < at_vertex.size(); corner++) {
                if (group[corner] == root) {
                    members.push_back(at_vertex[corner]);
                }
            }
            math::Vector3 position = this->position(vertex);
            std::vector<math::Vector3> inward;
            for (Corner corner : members) {
                auto [previous, next] = sides(corner);
                math::Vector3 normal = normals.at(corner.first);
                if (uses.at(Edge(previous, vertex)) == 1) {
                    inward.push_back(normalize(cross(normal, subtract(position, this->position(previous)))));
                }
                if (uses.at(Edge(vertex, next)) == 1) {
                    inward.push_back(normalize(cross(normal, subtract(this->position(next), position))));
                }
            }
            if (inward.empty()) {
                continue;
            }
            moves.push_back({add(position, inset_offset(inward, thickness)), std::move(members)});
        }
    }
    std::unordered_map<Corner, std::uint32_t, Hash> copies;
    for (const auto& [position, members] : moves) {
        std::uint32_t copy = add_vertex(position);
        for (Corner corner : members) {
            copies.insert_or_assign(corner, copy);
        }
    }
    std::vector<std::pair<size_t, Face>> originals;
    originals.reserve(region.size());
    for (size_t face : region) {
        originals.emplace_back(face, faces[face]);
    }
    for (const auto& [face, original] : originals) {
        size_t count = original.vertices.size();
        auto inner = [&](size_t place) {
            auto copy = copies.find({face, place % count});
            return copy == copies.end() ? original.vertices[place % count] : copy->second;
        };
        for (size_t place = 0; place < count; place++) {
            std::uint32_t a = original.vertices[place];
            std::uint32_t b = original.vertices[(place + 1) % count];
            Edge edge(a, b);
            if (uses.at(edge) == 1) {
                add_face({a, b, inner(place + 1), inner(place)}, original.material);
            } else if (crease(edge) > 0.0f) {
                set_crease(Edge(inner(place), inner(place + 1)), crease(edge));
            }
        }
        std::vector<std::uint32_t> shrunk;
        shrunk.reserve(count);
        for (size_t place = 0; place < count; place++) {
            shrunk.push_back(inner(place));
        }
        faces[face].vertices = std::move(shrunk);
    }
    prune_creases();
    return Selection::from_faces(*this, region);
}

Selection Mesh::inset_individual_impl(const std::set<size_t>& selected, float thickness) {
    for (size_t face : selected) {
        math::Vector3 normal = face_normal(face);
        if (normal == math::Vector3{}) {
            continue;
        }
        std::vector<std::uint32_t> vertices = faces[face].vertices;
        std::uint32_t material = faces[face].material;
        size_t count = vertices.size();
        std::vector<std::uint32_t> inner;
        inner.reserve(count);
        for (size_t place = 0; place < count; place++) {
            math::Vector3 previous = position(vertices[(place + count - 1) % count]);
            math::Vector3 vertex = position(vertices[place]);
            math::Vector3 next = position(vertices[(place + 1) % count]);
            std::array<math::Vector3, 2> inward{
                normalize(cross(normal, subtract(vertex, previous))),
                normalize(cross(normal, subtract(next, vertex))),
            };
            inner.push_back(add_vertex(add(vertex, inset_offset(inward, thickness))));
        }
        for (size_t place = 0; place < count; place++) {
            size_t next = (place + 1) % count;
            add_face({vertices[place], vertices[next], inner[next], inner[place]}, material);
        }
        faces[face].vertices = std::move(inner);
    }
    return Selection::from_faces(*this, selected);
}

} // namespace polygon_mesh
