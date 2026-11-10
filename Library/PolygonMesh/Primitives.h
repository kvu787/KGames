// Meshes to start modeling from, like Blender's Add > Mesh menu: a cube, a
// plane, a cylinder, a cone, a UV sphere and a torus. Each is centered on the
// origin with Y up, has material 0 and no creases, and is wound
// counterclockwise when seen from outside, so its normals point out. They're
// quads where they can be, with caps as single polygons and triangles only at
// the poles of the sphere and the tip of a cone, the way subdivision
// surfaces like them.

#pragma once

#include "PolygonMesh/Mesh.h"

#include <array>
#include <cstdint>

namespace polygon_mesh::primitives {

// A cube with edges of `size`, with 8 vertices and 6 quads.
Mesh cube(float size);

// A plane on Y = 0, facing +Y, `size` along X and Z, of `segments` quads
// along each.
Mesh plane(std::array<float, 2> size, std::array<std::uint32_t, 2> segments);

// A cylinder around the Y axis, `height` tall, with `sides` around and
// `segments` quads from its bottom to its top, and caps of `sides` vertices.
Mesh cylinder(float radius, float height, std::uint32_t sides, std::uint32_t segments);

// A cone around the Y axis, `height` tall, from a base of `bottom_radius` to
// a top of `top_radius`, with `sides` around. Where a radius is 0, the sides
// meet at a tip, in triangles, and elsewhere they end in a cap.
Mesh cone(float bottom_radius, float top_radius, float height, std::uint32_t sides);

// A sphere with its poles on the Y axis, `segments` vertices around each
// ring, and `rings` bands from pole to pole: quads, with triangles around the
// poles.
Mesh uv_sphere(float radius, std::uint32_t segments, std::uint32_t rings);

// A ring around the Y axis, `major_radius` from the axis to the middle of
// its tube, which has `minor_radius`: `major_segments` around the axis and
// `minor_segments` around the tube, all quads.
Mesh torus(float major_radius, float minor_radius, std::uint32_t major_segments, std::uint32_t minor_segments);

} // namespace polygon_mesh::primitives
