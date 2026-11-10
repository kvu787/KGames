// Triangles for SimplePaint to draw: each face cut into triangles by ear
// clipping in its plane, so that concave faces work, with normals flat per
// face, or smooth across the edges where faces meet at up to an angle, like
// Blender's Auto Smooth. What comes out passes simple_paint::validate_mesh for
// any mesh with positions SimplePaint accepts: triangles without area are left
// out, and a triangle whose smooth normals fail SimplePaint's check that they
// share a cone gets its own flat normal instead.
//
// Mesh.h declares Mesh's triangulate and face_triangles, which
// Triangulation.cpp defines.

#pragma once

#include "Base/Sum.h"
#include "PolygonMesh/Mesh.h"
#include "PolygonMesh/Picking.h"
#include "SimplePaint/Geometry.h"

#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace polygon_mesh {

struct Shading {
    // Each face's own normal.
    struct Flat {
        bool operator==(const Flat&) const = default;
    };
    // Normals smooth across each edge between two faces that meet at up to
    // `angle` radians, and split at the others, at creased edges, of any
    // sharpness, and at boundary and non-manifold edges. An angle of pi
    // smooths every edge that isn't creased.
    struct Smooth {
        float angle;
        bool operator==(const Smooth&) const = default;
    };
    std::variant<Flat, Smooth> value;

    template <base::AlternativeOf<Shading> T> Shading(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Shading&) const = default;
};

std::string describe_value(const Shading& shading);

// An indexed triangle list for simple_paint::Vertex, wound counterclockwise
// when seen from the front, like the faces, and the face that each triangle
// comes from, for picking and for drawing what's selected.
struct Triangles {
    std::vector<simple_paint::Vertex> vertices;
    std::vector<std::uint32_t> indices;
    std::vector<size_t> faces;

    // The nearest triangle that the ray hits, as a hit on the face it's of:
    // what Mesh::ray_cast gives on the mesh these are of, without
    // triangulating every face again, for a modeler that asks every frame
    // what's under the pointer, with the triangles it draws. Picking.cpp
    // defines it.
    std::optional<Hit> ray_cast(math::Vector3 origin, math::Vector3 direction) const;

    bool operator==(const Triangles&) const = default;
};

std::string describe_value(const Triangles& triangles);

} // namespace polygon_mesh
