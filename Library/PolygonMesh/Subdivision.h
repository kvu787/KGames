// Catmull-Clark subdivision, the Catmark scheme as OpenSubdiv defines it,
// with its boundary interpolation VTX_BOUNDARY_EDGE_AND_CORNER and its
// uniform creasing: face points, edge points, and vertex points by the
// smooth, dart, crease and corner rules, with semi-sharp creases that lose 1
// of sharpness each level, and vertices whose rule changes between levels
// blending the two by the fractional weight. README.md has the formulas.
//
// Mesh.h declares Mesh's catmull_clark, which Subdivision.cpp defines.

#pragma once

#include "PolygonMesh/Mesh.h"
#include "PolygonMesh/Picking.h"

#include <cstddef>
#include <optional>
#include <string>
#include <vector>

namespace polygon_mesh {

// A mesh subdivided some levels, and the face of the base mesh that each of
// its faces comes from, for picking and drawing what's selected.
struct SubdivisionSurface {
    Mesh mesh;
    std::vector<size_t> base_faces;

    // Where the ray hits the subdivided mesh, with `face` the base face that
    // the face it hits comes from. Picking.cpp defines it.
    std::optional<Hit> ray_cast(math::Vector3 origin, math::Vector3 direction) const;

    bool operator==(const SubdivisionSurface&) const = default;
};

std::string describe_value(const SubdivisionSurface& surface);

} // namespace polygon_mesh
