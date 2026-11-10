// An independent binary64 evaluation of Specification.md, and the cases that
// the GPU test compares it with. Shared by the tests, as reference::.

#pragma once

#include "SimplePaint/Material.h"

#include <array>
#include <cstdint>
#include <vector>

namespace reference {

using Normal = std::array<float, 3>;

struct Case {
    simple_paint::Parameters parameters;
    Normal normal{};
};

// SplitMix64, so the cases are the same on every run without a library.
class Random {
public:
    explicit Random(std::uint64_t seed) : state(seed) {}

    std::uint64_t next();

    // Uniform in [0, 1).
    double unit();

    // Uniform in [low, high).
    double range(double low, double high);

private:
    std::uint64_t state;
};

double linear(double srgb);

double srgb(double linear);

double curve(double color, double brightness, double tone);

// The rational facing of the normalized, rotated normal, the explicit tone
// interpolation, and the uncomposed Schlick color function. Only roundoff
// beyond the mathematical [0, 1] facing range is removed here.
std::array<double, 3> reference(const Case& case_);

std::vector<Case> cases();

} // namespace reference
