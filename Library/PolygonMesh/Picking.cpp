#include "PolygonMesh/Picking.h"

#include "PolygonMesh/Subdivision.h"
#include "PolygonMesh/Support.h"
#include "PolygonMesh/Triangulation.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <tuple>

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

// How far past a triangle's edges, as a fraction of the triangle, a ray still
// hits it.
constexpr float EdgeTolerance = 1.0e-5f;

// How far along a ray of unit `direction` it hits the triangle, from either
// side, by Moller and Trumbore's method. A little past the triangle's edges
// counts, so that a ray along the edge between two triangles hits one of them
// whatever the rounding.
std::optional<float> ray_triangle(math::Vector3 origin, math::Vector3 direction, const std::array<math::Vector3, 3>& corners) {
    const auto& [a, b, c] = corners;
    math::Vector3 first = subtract(b, a);
    math::Vector3 second = subtract(c, a);
    math::Vector3 p = cross(direction, second);
    float determinant = dot(first, p);
    // Along the triangle's plane, or a triangle without area.
    if (std::abs(determinant) <= 1.0e-9f * length(first) * length(second)) {
        return std::nullopt;
    }
    float inverse = 1.0f / determinant;
    math::Vector3 from_a = subtract(origin, a);
    float u = dot(from_a, p) * inverse;
    if (!(u >= -EdgeTolerance && u <= 1.0f + EdgeTolerance)) {
        return std::nullopt;
    }
    math::Vector3 q = cross(from_a, first);
    float v = dot(direction, q) * inverse;
    if (v < -EdgeTolerance || u + v > 1.0f + EdgeTolerance) {
        return std::nullopt;
    }
    float distance = dot(second, q) * inverse;
    if (!(distance >= 0.0f)) {
        return std::nullopt;
    }
    return distance;
}

// How near the ray from `origin` along unit `direction` comes to the segment
// from a to b, and how far along the ray, from the nearest points of their
// lines, kept on the ray and the segment.
std::pair<float, float> ray_segment_nearest(math::Vector3 origin, math::Vector3 direction, math::Vector3 a, math::Vector3 b) {
    math::Vector3 along = subtract(b, a);
    float length_squared = dot(along, along);
    auto on_segment = [&](math::Vector3 point) {
        return length_squared > 0.0f ? std::clamp(dot(subtract(point, a), along) / length_squared, 0.0f, 1.0f) : 0.0f;
    };
    // std::fmax, which gives the other number for a NaN, as f32::max does.
    auto on_ray = [&](math::Vector3 point) {
        return std::fmax(dot(subtract(point, origin), direction), 0.0f);
    };
    math::Vector3 from_a = subtract(origin, a);
    float d = dot(direction, along);
    float e = dot(along, from_a);
    float denominator = length_squared - d * d;
    // Parallel lines are nearest everywhere, so from a.
    float s = denominator > 1.0e-12f * length_squared ? std::clamp((e - d * dot(direction, from_a)) / denominator, 0.0f, 1.0f) : 0.0f;
    // Kept on the ray, then back on the segment, then on the ray.
    float t = on_ray(add(a, math::scale(along, s)));
    s = on_segment(add(origin, math::scale(direction, t)));
    t = on_ray(add(a, math::scale(along, s)));
    return {length(subtract(add(origin, math::scale(direction, t)), add(a, math::scale(along, s)))), t};
}

} // namespace

std::string describe_value(const Hit& hit) {
    return std::format("Hit {{ face: {}, distance: {}, point: {} }}", hit.face, base::debug(hit.distance), describe_vector(hit.point));
}

std::optional<Hit> Mesh::ray_cast(math::Vector3 origin, math::Vector3 direction) const {
    direction = normalize(direction);
    if (direction == math::Vector3{}) {
        return std::nullopt;
    }
    std::optional<Hit> nearest;
    for (size_t index = 0; index < faces.size(); index++) {
        const Face& face = faces[index];
        for (std::array<size_t, 3> triangle : face_triangles(index)) {
            std::array<math::Vector3, 3> corners{position(face.vertices[triangle[0]]), position(face.vertices[triangle[1]]), position(face.vertices[triangle[2]])};
            std::optional<float> distance = ray_triangle(origin, direction, corners);
            if (!distance) {
                continue;
            }
            if (!nearest || *distance < nearest->distance) {
                nearest = Hit{.face = index, .distance = *distance, .point = add(origin, math::scale(direction, *distance))};
            }
        }
    }
    return nearest;
}

std::optional<std::uint32_t> Mesh::nearest_vertex(math::Vector3 origin, math::Vector3 direction, float radius) const {
    direction = normalize(direction);
    std::optional<std::tuple<float, float, std::uint32_t>> nearest;
    for (size_t vertex = 0; vertex < positions.size(); vertex++) {
        math::Vector3 position = positions[vertex];
        float along = dot(subtract(position, origin), direction);
        if (along < 0.0f) {
            continue;
        }
        float away = length(subtract(position, add(origin, math::scale(direction, along))));
        if (away <= radius && (!nearest || std::pair(away, along) < std::pair(std::get<0>(*nearest), std::get<1>(*nearest)))) {
            nearest = std::tuple(away, along, static_cast<std::uint32_t>(vertex));
        }
    }
    if (!nearest) {
        return std::nullopt;
    }
    return std::get<2>(*nearest);
}

std::optional<Edge> Mesh::nearest_edge(math::Vector3 origin, math::Vector3 direction, float radius) const {
    direction = normalize(direction);
    std::optional<std::tuple<float, float, Edge>> nearest;
    for (Edge edge : edges()) {
        math::Vector3 a = position(edge.vertices()[0]);
        math::Vector3 b = position(edge.vertices()[1]);
        auto [away, along] = ray_segment_nearest(origin, direction, a, b);
        if (away <= radius && (!nearest || std::pair(away, along) < std::pair(std::get<0>(*nearest), std::get<1>(*nearest)))) {
            nearest = std::tuple(away, along, edge);
        }
    }
    if (!nearest) {
        return std::nullopt;
    }
    return std::get<2>(*nearest);
}

std::optional<std::array<math::Vector3, 2>> Mesh::bounds() const {
    if (positions.empty()) {
        return std::nullopt;
    }
    math::Vector3 low = positions[0];
    math::Vector3 high = positions[0];
    for (math::Vector3 position : positions) {
        // std::fmin and std::fmax, which give the other number for a NaN,
        // as f32::min and f32::max do.
        for (size_t axis = 0; axis < 3; axis++) {
            low[axis] = std::fmin(low[axis], position[axis]);
            high[axis] = std::fmax(high[axis], position[axis]);
        }
    }
    return std::array{low, high};
}

std::optional<Hit> Triangles::ray_cast(math::Vector3 origin, math::Vector3 direction) const {
    direction = normalize(direction);
    if (direction == math::Vector3{}) {
        return std::nullopt;
    }
    std::optional<Hit> nearest;
    for (size_t index = 0; index < indices.size() / 3; index++) {
        std::array<math::Vector3, 3> corners{vertices[indices[index * 3]].position, vertices[indices[index * 3 + 1]].position, vertices[indices[index * 3 + 2]].position};
        std::optional<float> distance = ray_triangle(origin, direction, corners);
        if (!distance) {
            continue;
        }
        if (!nearest || *distance < nearest->distance) {
            nearest = Hit{.face = faces[index], .distance = *distance, .point = add(origin, math::scale(direction, *distance))};
        }
    }
    return nearest;
}

std::optional<Hit> SubdivisionSurface::ray_cast(math::Vector3 origin, math::Vector3 direction) const {
    std::optional<Hit> hit = mesh.ray_cast(origin, direction);
    if (hit) {
        hit->face = base_faces[hit->face];
    }
    return hit;
}

} // namespace polygon_mesh
