#include "Veehiicuul/Collision.h"

#include <algorithm>
#include <cmath>
#include <limits>

namespace veehiicuul::collision {

namespace {

// Twice the signed area of the triangle a, b, c: positive if they turn one
// way, negative the other, and 0 if they're on a line.
double turn(std::array<double, 2> a, std::array<double, 2> b, std::array<double, 2> c) {
    return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
}

// Whether p, on the line through a and b, is between them.
bool within(const std::array<std::array<double, 2>, 2>& segment, std::array<double, 2> p) {
    const std::array<double, 2>& a = segment[0];
    const std::array<double, 2>& b = segment[1];
    for (size_t axis = 0; axis < 2; axis++) {
        if (!(std::min(a[axis], b[axis]) <= p[axis] && p[axis] <= std::max(a[axis], b[axis]))) {
            return false;
        }
    }
    return true;
}

} // namespace

Footprint Footprint::from_sides(double left, double right, double front, double back) {
    return Footprint{
        .min_ = {-right, -back},
        .max_ = {left, front},
    };
}

Footprint Footprint::of_model(std::span<const std::array<float, 3>> positions) {
    std::array<double, 2> smallest{std::numeric_limits<double>::infinity(), std::numeric_limits<double>::infinity()};
    std::array<double, 2> largest{-std::numeric_limits<double>::infinity(), -std::numeric_limits<double>::infinity()};
    for (const std::array<float, 3>& position : positions) {
        std::array<float, 2> across{position[0], position[2]};
        for (size_t axis = 0; axis < 2; axis++) {
            smallest[axis] = std::min(smallest[axis], static_cast<double>(across[axis]));
            largest[axis] = std::max(largest[axis], static_cast<double>(across[axis]));
        }
    }
    return Footprint{.min_ = smallest, .max_ = largest};
}

std::array<std::array<double, 2>, 4> Footprint::corners(std::array<double, 2> position, double heading) const {
    double sin = std::sin(heading);
    double cos = std::cos(heading);
    auto place = [&](std::array<double, 2> corner) -> std::array<double, 2> {
        double left = corner[0];
        double ahead = corner[1];
        return {
            position[0] + left * cos + ahead * sin,
            position[1] + ahead * cos - left * sin,
        };
    };
    return {
        place(min_),
        place({max_[0], min_[1]}),
        place(max_),
        place({min_[0], max_[1]}),
    };
}

Edges::Edges(std::span<const std::span<const std::array<double, 2>>> outlines) {
    for (std::span<const std::array<double, 2>> outline : outlines) {
        for (size_t index = 0; index < outline.size(); index++) {
            std::array<double, 2> start = outline[index];
            std::array<double, 2> end = outline[(index + 1) % outline.size()];
            segments.push_back(Segment{
                .ends = {start, end},
                .min_ = {std::min(start[0], end[0]), std::min(start[1], end[1])},
                .max_ = {std::max(start[0], end[0]), std::max(start[1], end[1])},
            });
        }
    }
}

size_t Edges::len() const {
    return segments.size();
}

bool Edges::is_empty() const {
    return segments.empty();
}

bool Edges::touch(const Footprint& footprint, std::array<double, 2> position, double heading) const {
    std::array<std::array<double, 2>, 4> corners = footprint.corners(position, heading);
    std::array<double, 2> smallest = corners[0];
    std::array<double, 2> largest = corners[0];
    for (size_t corner = 1; corner < corners.size(); corner++) {
        for (size_t axis = 0; axis < 2; axis++) {
            smallest[axis] = std::min(smallest[axis], corners[corner][axis]);
            largest[axis] = std::max(largest[axis], corners[corner][axis]);
        }
    }
    for (const Segment& segment : segments) {
        if (!(segment.max_[0] >= smallest[0] && segment.min_[0] <= largest[0] && segment.max_[1] >= smallest[1] && segment.min_[1] <= largest[1])) {
            continue;
        }
        for (size_t side = 0; side < 4; side++) {
            if (meet(segment.ends, {corners[side], corners[(side + 1) % 4]})) {
                return true;
            }
        }
    }
    return false;
}

bool meet(const std::array<std::array<double, 2>, 2>& first, const std::array<std::array<double, 2>, 2>& second) {
    std::array<double, 2> a = first[0];
    std::array<double, 2> b = first[1];
    std::array<double, 2> c = second[0];
    std::array<double, 2> d = second[1];
    std::array<double, 4> turns{turn(c, d, a), turn(c, d, b), turn(a, b, c), turn(a, b, d)};
    auto opposite = [](double x, double y) {
        return (x > 0.0 && y < 0.0) || (x < 0.0 && y > 0.0);
    };
    if (opposite(turns[0], turns[1]) && opposite(turns[2], turns[3])) {
        return true;
    }
    return (turns[0] == 0.0 && within(second, a)) || (turns[1] == 0.0 && within(second, b)) || (turns[2] == 0.0 && within(first, c)) || (turns[3] == 0.0 && within(first, d));
}

} // namespace veehiicuul::collision
