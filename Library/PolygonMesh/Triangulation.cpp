#include "PolygonMesh/Triangulation.h"

#include "Base/Numbers.h"
#include "PolygonMesh/Support.h"
#include "PolygonMesh/Topology.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <numeric>
#include <unordered_map>

namespace polygon_mesh {

using math::cross;
using math::dot;
using math::normalize;
using math::subtract;
using simple_paint::Vertex;

namespace {

// What validate_mesh needs of each normal: projecting onto the normalized sum
// of the three at least 0.125. This asks a little more, so that rounding
// can't fail what passes here.
constexpr double ConeProjection = 0.13;

// The fan that the corner is in, by the number of the corner that every
// corner of the fan leads to.
size_t root(std::span<size_t> fans, size_t corner) {
    while (fans[corner] != corner) {
        fans[corner] = fans[fans[corner]];
        corner = fans[corner];
    }
    return corner;
}

// Whether SimplePaint's interpolation between the normals stays away from
// zero, as validate_mesh checks it, in the same arithmetic.
bool share_cone(const std::array<math::Vector3, 3>& normals) {
    std::array<double, 3> sum{};
    for (size_t axis = 0; axis < 3; axis++) {
        double total = 0.0;
        for (const math::Vector3& normal : normals) {
            total += static_cast<double>(normal[axis]);
        }
        sum[axis] = total;
    }
    double squares = 0.0;
    for (double s : sum) {
        squares += s * s;
    }
    double length = std::sqrt(squares);
    if (!(length > 0.0)) {
        return false;
    }
    return std::ranges::all_of(normals, [&](const math::Vector3& normal) {
        double projection = 0.0;
        for (size_t axis = 0; axis < 3; axis++) {
            projection += static_cast<double>(normal[axis]) * sum[axis];
        }
        return projection >= ConeProjection * length;
    });
}

// Twice the signed area of the triangle a, b, c: positive when it's
// counterclockwise.
double turn(std::array<double, 2> a, std::array<double, 2> b, std::array<double, 2> c) {
    return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
}

// The triangles of a counterclockwise polygon, as indices into its points.
// Each step cuts off an ear: a corner that turns left, with no other point
// in or on its triangle. A polygon with no ear, which only one that's
// degenerate or crosses itself can be, loses the corner that turns left most.
std::vector<std::array<size_t, 3>> ear_clip(std::span<const std::array<double, 2>> points) {
    std::vector<size_t> remaining(points.size());
    std::iota(remaining.begin(), remaining.end(), size_t{0});
    std::vector<std::array<size_t, 3>> triangles;
    triangles.reserve(base::saturating_sub(points.size(), size_t{2}));
    while (remaining.size() > 3) {
        size_t count = remaining.size();
        auto corner = [&](size_t place) {
            return std::array{remaining[(place + count - 1) % count], remaining[place], remaining[(place + 1) % count]};
        };
        auto turn_at = [&](size_t place) {
            std::array<size_t, 3> indices = corner(place);
            return turn(points[indices[0]], points[indices[1]], points[indices[2]]);
        };
        auto is_ear = [&](size_t place) {
            std::array<size_t, 3> indices = corner(place);
            std::array<double, 2> a = points[indices[0]];
            std::array<double, 2> b = points[indices[1]];
            std::array<double, 2> c = points[indices[2]];
            return turn(a, b, c) > 0.0 && std::ranges::all_of(remaining, [&](size_t other) {
                       std::array<double, 2> p = points[other];
                       return p == a || p == b || p == c || turn(a, b, p) < 0.0 || turn(b, c, p) < 0.0 || turn(c, a, p) < 0.0;
                   });
        };
        std::optional<size_t> ear;
        for (size_t place = 0; place < count; place++) {
            if (is_ear(place)) {
                ear = place;
                break;
            }
        }
        size_t place = ear ? *ear : max_by(std::views::iota(size_t{0}, count), [&](size_t first, size_t second) { return total_cmp(turn_at(first), turn_at(second)); });
        triangles.push_back(corner(place));
        remaining.erase(remaining.begin() + static_cast<std::ptrdiff_t>(place));
    }
    if (remaining.size() == 3) {
        triangles.push_back({remaining[0], remaining[1], remaining[2]});
    }
    return triangles;
}

std::string describe_vertex(const Vertex& vertex) {
    return std::format("Vertex {{ position: {}, normal: {}, material_index: {} }}", describe_vector(vertex.position), describe_vector(vertex.normal), vertex.material_index);
}

} // namespace

std::string describe_value(const Shading& shading) {
    return base::match(
        shading, [](const Shading::Flat&) { return std::string("Flat"); },
        [](const Shading::Smooth& smooth) { return std::format("Smooth {{ angle: {} }}", base::debug(smooth.angle)); });
}

std::string describe_value(const Triangles& triangles) {
    return std::format("Triangles {{ vertices: {}, indices: {}, faces: {} }}", describe_list(triangles.vertices, describe_vertex), describe_numbers(triangles.indices),
                       describe_numbers(triangles.faces));
}

Triangles Mesh::triangulate(Shading shading) const {
    std::vector<math::Vector3> normals;
    normals.reserve(faces.size());
    for (size_t face = 0; face < faces.size(); face++) {
        normals.push_back(face_normal(face));
    }
    std::vector<size_t> first_corner;
    first_corner.reserve(faces.size());
    size_t corners = 0;
    for (const Face& face : faces) {
        first_corner.push_back(corners);
        corners += face.vertices.size();
    }
    // Each corner's normal, and the fan of corners that it shares a
    // vertex with: just itself when flat.
    std::vector<math::Vector3> corner_normals;
    std::vector<size_t> fans;
    base::match(
        shading,
        [&](const Shading::Flat&) {
            for (size_t face = 0; face < faces.size(); face++) {
                corner_normals.insert(corner_normals.end(), faces[face].vertices.size(), normals[face]);
            }
            fans.resize(corners);
            std::iota(fans.begin(), fans.end(), size_t{0});
        },
        [&](const Shading::Smooth& smooth) {
            fans = smooth_fans(normals, first_corner, corners, smooth.angle);
            std::vector<std::array<double, 3>> sums(corners);
            for (size_t index = 0; index < faces.size(); index++) {
                const Face& face = faces[index];
                for (size_t place = 0; place < face.vertices.size(); place++) {
                    double weight = static_cast<double>(corner_angle(index, place));
                    std::array<double, 3>& sum = sums[fans[first_corner[index] + place]];
                    for (size_t axis = 0; axis < 3; axis++) {
                        sum[axis] += static_cast<double>(normals[index][axis]) * weight;
                    }
                }
            }
            corner_normals.reserve(fans.size());
            for (size_t fan : fans) {
                const std::array<double, 3>& sum = sums[fan];
                corner_normals.push_back(normalize({static_cast<float>(sum[0]), static_cast<float>(sum[1]), static_cast<float>(sum[2])}));
            }
        });
    // Vertices are made as triangles first use them, so that every one
    // is used, which validate_mesh needs of them, and shared by a fan's
    // corners of the same material.
    std::unordered_map<std::pair<size_t, std::uint32_t>, std::uint32_t, Hash> shared;
    Triangles triangles;
    for (size_t index = 0; index < faces.size(); index++) {
        const Face& face = faces[index];
        for (std::array<size_t, 3> triangle : face_triangles(index)) {
            std::array<math::Vector3, 3> points{position(face.vertices[triangle[0]]), position(face.vertices[triangle[1]]), position(face.vertices[triangle[2]])};
            const auto& [a, b, c] = points;
            math::Vector3 flat = normalize(cross(subtract(b, a), subtract(c, a)));
            if (flat == math::Vector3{}) {
                continue;
            }
            std::array<size_t, 3> at{first_corner[index] + triangle[0], first_corner[index] + triangle[1], first_corner[index] + triangle[2]};
            if (share_cone({corner_normals[at[0]], corner_normals[at[1]], corner_normals[at[2]]})) {
                for (size_t place = 0; place < 3; place++) {
                    size_t corner = at[place];
                    auto [entry, added] = shared.try_emplace(std::pair(fans[corner], face.material), std::uint32_t{0});
                    if (added) {
                        math::Vector3 normal = corner_normals[corner];
                        triangles.vertices.push_back(Vertex{.position = points[place], .normal = normal, .material_index = face.material});
                        entry->second = static_cast<std::uint32_t>(triangles.vertices.size()) - 1;
                    }
                    triangles.indices.push_back(entry->second);
                }
            } else {
                // A face without area of its own, or smooth normals too far
                // apart: the triangle's own flat normal.
                for (math::Vector3 point : points) {
                    triangles.indices.push_back(static_cast<std::uint32_t>(triangles.vertices.size()));
                    triangles.vertices.push_back(Vertex{.position = point, .normal = flat, .material_index = face.material});
                }
            }
            triangles.faces.push_back(index);
        }
    }
    return triangles;
}

std::vector<size_t> Mesh::smooth_fans(std::span<const math::Vector3> normals, std::span<const size_t> first_corner, size_t corners, float angle) const {
    std::vector<size_t> fans(corners);
    std::iota(fans.begin(), fans.end(), size_t{0});
    float limit = std::cos(angle) - 1.0e-6f;
    Topology topology = this->topology();
    for (size_t edge = 0; edge < topology.edges().size(); edge++) {
        std::span<const size_t> pair = topology.edge_faces(edge);
        if (pair.size() != 2) {
            continue;
        }
        size_t first = pair[0];
        size_t second = pair[1];
        bool smooth = !topology.is_non_manifold(edge) && crease(topology.edge(edge)) == 0.0f && normals[first] != math::Vector3{} && normals[second] != math::Vector3{} &&
                      dot(normals[first], normals[second]) >= limit;
        if (!smooth) {
            continue;
        }
        for (std::uint32_t vertex : topology.edge(edge).vertices()) {
            auto corner = [&](size_t face) {
                return first_corner[face] + position_of(faces[face].vertices, vertex).value();
            };
            size_t a = root(fans, corner(first));
            size_t b = root(fans, corner(second));
            fans[a] = b;
        }
    }
    std::vector<size_t> roots;
    roots.reserve(corners);
    for (size_t corner = 0; corner < corners; corner++) {
        roots.push_back(root(fans, corner));
    }
    return roots;
}

float Mesh::corner_angle(size_t face, size_t place) const {
    const std::vector<std::uint32_t>& vertices = faces[face].vertices;
    size_t count = vertices.size();
    math::Vector3 position = this->position(vertices[place]);
    math::Vector3 previous = normalize(subtract(this->position(vertices[(place + count - 1) % count]), position));
    math::Vector3 next = normalize(subtract(this->position(vertices[(place + 1) % count]), position));
    return std::acos(std::clamp(dot(previous, next), -1.0f, 1.0f));
}

std::vector<std::array<size_t, 3>> Mesh::face_triangles(size_t face) const {
    const std::vector<std::uint32_t>& vertices = faces[face].vertices;
    math::Vector3 normal = face_area_vector(face);
    if (vertices.size() == 3) {
        return {std::array<size_t, 3>{0, 1, 2}};
    }
    // Axes in the plane whose cross product is the normal, so that the
    // face is counterclockwise in them.
    math::Vector3 u = normalize(std::abs(normal[0]) > std::abs(normal[1]) ? math::Vector3{-normal[2], 0.0f, normal[0]} : math::Vector3{0.0f, normal[2], -normal[1]});
    math::Vector3 v = normalize(cross(normal, u));
    std::vector<std::array<double, 2>> points;
    points.reserve(vertices.size());
    for (std::uint32_t vertex : vertices) {
        math::Vector3 position = this->position(vertex);
        points.push_back({static_cast<double>(dot(position, u)), static_cast<double>(dot(position, v))});
    }
    return ear_clip(points);
}

} // namespace polygon_mesh
