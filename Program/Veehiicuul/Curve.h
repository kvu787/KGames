// The closed curves that make a track's edges: uniform cubic B-splines on the
// ground, like Blender's cyclic NURBS curves with every weight 1. A curve
// is smooth everywhere, and passes near its control points, not through
// them, so that moving one bends the curve around it without kinks. Each
// span, between the curve's parameters i and i + 1, is pulled by control
// points i - 1 to i + 2.

#pragma once

#include <array>
#include <cstddef>
#include <span>
#include <vector>

namespace veehiicuul::curve {

// How many points each span is sampled at, for the track's outlines.
inline constexpr size_t SamplesPerSpan = 8;
// The fewest control points a curve has.
inline constexpr size_t Fewest = 3;

// The curve's point at parameter `u`, which goes around once from 0 to the
// number of control points.
std::array<double, 2> point(std::span<const std::array<double, 2>> controls, double u);

// The curve as an outline: SamplesPerSpan points a span, all the way
// around.
std::vector<std::array<double, 2>> sample(std::span<const std::array<double, 2>> controls);

// Where to put a new control point so that it bends the curve near `near_`:
// after the control point that starts the span nearest it. Returns the
// index for the new point.
size_t insertion(std::span<const std::array<double, 2>> controls, std::array<double, 2> near_);

double distance(std::array<double, 2> a, std::array<double, 2> b);

// A circle of `count` control points around `center`, whose curve is nearly
// a circle of `radius`. A cubic B-spline's curve is inside its control
// points, so they're a little farther out.
std::vector<std::array<double, 2>> circle(std::array<double, 2> center, double radius, size_t count);

} // namespace veehiicuul::curve
