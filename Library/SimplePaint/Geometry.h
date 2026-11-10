// Host-independent mesh validation, before GPU upload. A conservative
// common-cone test guarantees that every barycentrically interpolated normal
// stays nonzero. Bad meshes must be corrected by their author, not in the
// pixel shader.

#pragma once

#include "SimplePaint/Error.h"

#include <array>
#include <cmath>
#include <concepts>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <ranges>
#include <span>

namespace simple_paint {

// What validate_mesh reads from a host's vertex.
template <class V>
concept PaintVertex = requires(const V& vertex) {
    { vertex.position } -> std::convertible_to<std::array<float, 3>>;
    { vertex.normal } -> std::convertible_to<std::array<float, 3>>;
    { vertex.material_index } -> std::convertible_to<std::uint32_t>;
};

// The vertex that SimplePaint.hlsl's adapter reads: POSITION at byte 0,
// NORMAL at 12, and MATERIAL at 24, 28 bytes in all.
struct Vertex {
    std::array<float, 3> position{};
    std::array<float, 3> normal{};
    std::uint32_t material_index = 0;

    bool operator==(const Vertex&) const = default;
};

static_assert(sizeof(Vertex) == 28);
static_assert(offsetof(Vertex, normal) == 12 && offsetof(Vertex, material_index) == 24);

// Checks a nonempty indexed triangle list: finite positions with components
// of magnitude at most a million, normal lengths in [0.5, 2], valid material
// indices, one material per triangle, and normals that share a cone per
// triangle. See Specification.md.
template <PaintVertex V> std::expected<void, Error> validate_mesh(std::span<const V> vertices, std::span<const std::uint32_t> indices, std::uint32_t material_count) {
    auto error = [](const char* message) {
        return std::unexpected(Error{message});
    };
    if (vertices.empty() || indices.empty() || indices.size() % 3 != 0 || material_count == 0) {
        return error("SimplePaint requires a nonempty indexed triangle mesh and materials.");
    }
    for (const V& vertex : vertices) {
        std::array<float, 3> position = vertex.position;
        std::array<float, 3> normal = vertex.normal;
        double length_squared = 0.0;
        for (float n : normal) {
            length_squared += static_cast<double>(n) * static_cast<double>(n);
        }
        bool bounded = true;
        for (float p : position) {
            bounded = bounded && std::isfinite(p) && std::abs(p) <= 1.0e6f;
        }
        if (!bounded || !std::isfinite(length_squared) || !(length_squared >= 0.25 && length_squared <= 4.0) || vertex.material_index >= material_count) {
            return error("SimplePaint vertex requires finite bounded position, normal length [0.5,2], and valid material index.");
        }
    }
    for (size_t first = 0; first + 3 <= indices.size(); first += 3) {
        std::span<const std::uint32_t> triangle = indices.subspan(first, 3);
        for (std::uint32_t index : triangle) {
            if (index >= vertices.size()) {
                return error("SimplePaint mesh index is out of bounds.");
            }
        }
        std::array<const V*, 3> corners{&vertices[triangle[0]], &vertices[triangle[1]], &vertices[triangle[2]]};
        std::uint32_t material = corners[0]->material_index;
        for (const V* corner : corners) {
            if (corner->material_index != material) {
                return error("SimplePaint triangles must have one material.");
            }
        }
        std::array<double, 3> sum{};
        for (size_t axis = 0; axis < 3; axis++) {
            for (const V* corner : corners) {
                sum[axis] += static_cast<double>(std::array<float, 3>(corner->normal)[axis]);
            }
        }
        double length = std::sqrt(sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]);
        for (const V* corner : corners) {
            std::array<float, 3> normal = corner->normal;
            double projection = 0.0;
            for (size_t axis = 0; axis < 3; axis++) {
                projection += static_cast<double>(normal[axis]) * sum[axis];
            }
            if (length == 0.0 || projection < 0.125 * length) {
                return error("SimplePaint triangle normals must share a cone with projection at least 0.125.");
            }
        }
    }
    return {};
}

// For a std::vector or a std::array of vertices and indices.
template <class Vertices, class Indices>
    requires PaintVertex<std::ranges::range_value_t<Vertices>>
std::expected<void, Error> validate_mesh(const Vertices& vertices, const Indices& indices, std::uint32_t material_count) {
    return validate_mesh(std::span<const std::ranges::range_value_t<Vertices>>(vertices), std::span<const std::uint32_t>(indices), material_count);
}

} // namespace simple_paint
