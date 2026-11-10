// Meshes for SimplePaint: a UV sphere, a box and a plane, made in code, and
// the car, made in Blender. Each made in code has one material, smooth
// normals on the sphere and flat ones on the others. All have triangles
// wound counterclockwise when seen from outside, the right-handed
// convention.

#pragma once

#include "SimplePaint/Geometry.h"

#include <array>
#include <cstdint>
#include <span>
#include <vector>

namespace meshes {

// The car, with Y up and its front toward +Z. It's 3 units wide, from X =
// -1.5 to 1.5, and about 6 long, from Z = -3 to 3.16, with its wheels on Y =
// 0. Library/Meshes/Generator makes it from
// Program/Veehiicuul/Models/SlopeCar.glb, the model that Blender exported, as
// Meshes builds: Vertices, Indices, and MaterialCount materials, one for each
// part, Axles, Body, Cabin, Headlights and Wheels.
namespace car {

// Each part's material index, which is its place among the generator's
// parts. What the generator makes doesn't compile if these say otherwise.
inline constexpr std::uint32_t Axles = 0;
inline constexpr std::uint32_t Body = 1;
inline constexpr std::uint32_t Cabin = 2;
inline constexpr std::uint32_t Headlights = 3;
inline constexpr std::uint32_t Wheels = 4;
inline constexpr std::uint32_t MaterialCount = 5;

// The arrays that the generator made, whose sizes only Meshes.cpp knows.
extern const std::span<const simple_paint::Vertex> Vertices;
extern const std::span<const std::uint32_t> Indices;

} // namespace car

struct Mesh {
    std::vector<simple_paint::Vertex> vertices;
    std::vector<std::uint32_t> indices;

    Mesh() = default;
    Mesh(std::vector<simple_paint::Vertex> vertices, std::vector<std::uint32_t> indices);
    Mesh(const Mesh&) = delete;
    Mesh& operator=(const Mesh&) = delete;
    Mesh(Mesh&&) = default;
    Mesh& operator=(Mesh&&) = default;
};

// A sphere of radius 1 around the origin, with its poles on the Y axis:
// `longitudes` vertices around each ring, and `latitudes` bands from pole to
// pole. With 3 longitudes, which are 120 degrees apart, the normals of a
// triangle near the equator are too far apart for SimplePaint, so 4 is the
// fewest.
Mesh uv_sphere(std::uint32_t longitudes, std::uint32_t latitudes, std::uint32_t material_index);

// A box centered on the origin, `size` along X, Y and Z.
Mesh cuboid(std::array<float, 3> size, std::uint32_t material_index);

// A plane through the origin, facing +Y, `size` along X and Z.
Mesh plane(std::array<float, 2> size, std::uint32_t material_index);

} // namespace meshes
