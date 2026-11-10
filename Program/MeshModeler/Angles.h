// Degrees and radians, as Rust's f32::to_radians and f32::to_degrees give
// them, to the bit, which the C++ standard library has no functions for: one
// multiplication by a constant that's rounded once.

#pragma once

#include <numbers>

namespace mesh_modeler {

constexpr float to_radians(float degrees) {
    constexpr float RadiansPerDegree = std::numbers::pi_v<float> / 180.0f;
    return degrees * RadiansPerDegree;
}

constexpr float to_degrees(float radians) {
    // A literal, as Rust has it, since dividing 180 by a pi that's rounded
    // already would round again.
    constexpr float DegreesPerRadian = 57.2957795130823208767981548141051703f;
    return radians * DegreesPerRadian;
}

} // namespace mesh_modeler
