// Picking: the face that a ray from the camera hits, the vertex and edge
// nearest it, and where a mesh or some of its vertices are, to frame them or
// to rotate and scale about.
//
// Mesh.h declares Mesh's ray_cast, nearest_vertex, nearest_edge, bounds and
// center_of, Triangulation.h the ray_cast of Triangles, and Subdivision.h
// that of a SubdivisionSurface. Picking.cpp defines them.

#pragma once

#include "PolygonMesh/Mesh.h"

#include <cstddef>
#include <cstdint>
#include <optional>
#include <ranges>
#include <string>

namespace polygon_mesh {

// Where a ray hits a face: `distance` along the ray, in the mesh's units, to
// `point`.
struct Hit {
    size_t face;
    float distance;
    math::Vector3 point;

    bool operator==(const Hit&) const = default;
};

std::string describe_value(const Hit& hit);

template <RangeOf<std::uint32_t> Vertices> std::optional<math::Vector3> Mesh::center_of(Vertices&& vertices) const {
    return average(vertex_set(vertices) | std::views::transform([&](std::uint32_t vertex) { return position(vertex); }));
}

} // namespace polygon_mesh
