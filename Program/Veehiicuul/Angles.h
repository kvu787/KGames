// Degrees and radians, as Rust's f64::to_radians and f64::to_degrees give
// them, to the bit, which the C++ standard library has no functions for: one
// multiplication by a constant that's rounded once.

#pragma once

#include <numbers>

namespace veehiicuul {

constexpr double to_radians(double degrees) {
    constexpr double RadiansPerDegree = std::numbers::pi / 180.0;
    return degrees * RadiansPerDegree;
}

constexpr double to_degrees(double radians) {
    constexpr double DegreesPerRadian = 180.0 / std::numbers::pi;
    return radians * DegreesPerRadian;
}

} // namespace veehiicuul
