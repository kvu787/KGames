#include "Veehiicuul/Curve.h"

#include "Base/Numbers.h"

#include <cmath>
#include <compare>
#include <cstdint>
#include <numbers>

namespace veehiicuul::curve {

namespace {

// The value cubed, multiplied out as powi(3) is.
double cubed(double value) {
    return value * (value * value);
}

} // namespace

std::array<double, 2> point(std::span<const std::array<double, 2>> controls, double u) {
    size_t count = controls.size();
    double whole = std::floor(u);
    double t = u - whole;
    size_t span = static_cast<size_t>(base::rem_euclid(base::saturating_cast<std::int64_t>(whole), static_cast<std::int64_t>(count)));
    std::array<double, 4> weights{
        cubed(1.0 - t) / 6.0,
        (3.0 * cubed(t) - 6.0 * t * t + 4.0) / 6.0,
        (-3.0 * cubed(t) + 3.0 * t * t + 3.0 * t + 1.0) / 6.0,
        cubed(t) / 6.0,
    };
    std::array<double, 2> result{0.0, 0.0};
    for (size_t offset = 0; offset < weights.size(); offset++) {
        double weight = weights[offset];
        std::array<double, 2> control = controls[(span + count + offset - 1) % count];
        result[0] += weight * control[0];
        result[1] += weight * control[1];
    }
    return result;
}

std::vector<std::array<double, 2>> sample(std::span<const std::array<double, 2>> controls) {
    size_t count = controls.size() * SamplesPerSpan;
    std::vector<std::array<double, 2>> outline;
    outline.reserve(count);
    for (size_t index = 0; index < count; index++) {
        outline.push_back(point(controls, static_cast<double>(index) / static_cast<double>(SamplesPerSpan)));
    }
    return outline;
}

size_t insertion(std::span<const std::array<double, 2>> controls, std::array<double, 2> near_) {
    std::vector<std::array<double, 2>> outline = sample(controls);
    // The first of the nearest, in the order of every double, as min_by with
    // total_cmp gives.
    size_t nearest = 0;
    for (size_t index = 1; index < outline.size(); index++) {
        if (std::strong_order(distance(outline[nearest], near_), distance(outline[index], near_)) > 0) {
            nearest = index;
        }
    }
    return nearest / SamplesPerSpan + 1;
}

double distance(std::array<double, 2> a, std::array<double, 2> b) {
    return std::hypot(a[0] - b[0], a[1] - b[1]);
}

std::vector<std::array<double, 2>> circle(std::array<double, 2> center, double radius, size_t count) {
    double step = 2.0 * std::numbers::pi / static_cast<double>(count);
    // A control polygon of radius R makes a curve of about R * (2 + cos(step)) / 3.
    double outer = radius * 3.0 / (2.0 + std::cos(step));
    std::vector<std::array<double, 2>> controls;
    controls.reserve(count);
    for (size_t index = 0; index < count; index++) {
        double angle = static_cast<double>(index) * step;
        controls.push_back({center[0] + outer * std::cos(angle), center[1] + outer * std::sin(angle)});
    }
    return controls;
}

} // namespace veehiicuul::curve
