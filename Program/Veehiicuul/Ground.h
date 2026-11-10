// What's flat on the ground, as meshes that are drawn in order, each part
// over the ones before, without a depth buffer: the terrain's hexels, then
// the road, with its islands cut out of it, so that the terrain shows in
// them, then the barriers beside every edge, away from the road, and the
// start line across the road. The scene's background shows where nothing
// else is. Each part is a flat color, which Ground.hlsl draws.

#pragma once

#include "Meshes/Meshes.h"
#include "Veehiicuul/Terrain.h"
#include "Veehiicuul/Track.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <span>
#include <vector>

namespace veehiicuul::ground {

// The meshes' materials, which are Ground.hlsl's colors: the terrain's
// palette, then the track's.
inline constexpr std::uint32_t Road = 64;
// The barriers alternate between these.
inline constexpr std::array<std::uint32_t, 2> Barriers{65, 66};
// So do the start line's squares.
inline constexpr std::array<std::uint32_t, 2> StartLine{67, 68};
// As Ground.hlsl's MaterialCount says.
inline constexpr size_t ColorCount = 72;

// Every material's color, as sRGB.
std::array<std::array<float, 3>, ColorCount> colors(const terrain::Terrain& terrain, const track::Track& track);

// The terrain's painted hexels, each in its palette color.
meshes::Mesh terrain_mesh(const terrain::Terrain& terrain);

// The track: the road, the barriers and the start line.
meshes::Mesh track_mesh(const track::Track& track, const track::Shape& shape);

// Twice an outline's signed area: below 0 if it goes counterclockwise seen
// from above, where X is right and -Z up.
double area(std::span<const std::array<double, 2>> outline);

// One outline that goes around `outline` and every hole in it, joined to it
// by a cut from each hole's rightmost point to a point of the outline that
// it can see, there and back, so that ear clipping fills all but the holes.
std::vector<std::array<double, 2>> with_holes(std::span<const std::array<double, 2>> outline, std::span<const std::vector<std::array<double, 2>>> holes);

// Splits a simple polygon, one whose edges don't cross, into triangles, by
// cutting off its ears one at a time: corners that turn toward its inside,
// whose triangle has none of its other corners in it. Each triangle is its
// corners' indices, wound counterclockwise seen from above. Corners in the
// same place, like the two sides of a cut to a hole, don't count as in a
// triangle.
std::vector<std::array<size_t, 3>> triangulate(std::span<const std::array<double, 2>> outline);

} // namespace veehiicuul::ground
