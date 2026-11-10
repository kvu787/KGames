// Whether a car touches the track's edges, on the ground: the car is a
// rectangle seen from above, its footprint, and an edge is one of the
// straight segments of the road's outline and its islands'. A car touches an
// edge when a side of its rectangle meets one, even at a point, or along
// it.
//
// Only where the car is in each frame counts, so a car that's fast enough,
// or a frame that's slow enough, can carry it past an edge between frames.

#pragma once

#include <array>
#include <cstddef>
#include <span>
#include <vector>

namespace veehiicuul::collision {

// The rectangle around a car seen from above, along its own directions: X
// to its left, and Z ahead. Its vehicle says how far it goes each way from
// the car's middle.
struct Footprint {
    std::array<double, 2> min_{};
    std::array<double, 2> max_{};

    // How far it goes to the car's left, right, front and back.
    static Footprint from_sides(double left, double right, double front, double back);

    // The rectangle around a model seen from above.
    static Footprint of_model(std::span<const std::array<float, 3>> positions);

    // Its corners on the ground, for a car at `position` facing `heading`,
    // in order around it.
    std::array<std::array<double, 2>, 4> corners(std::array<double, 2> position, double heading) const;

    bool operator==(const Footprint&) const = default;
};

// The track's edges: every segment of each outline, which closes from its
// last point back to its first.
class Edges {
public:
    explicit Edges(std::span<const std::span<const std::array<double, 2>>> outlines);
    Edges(const Edges&) = delete;
    Edges& operator=(const Edges&) = delete;
    Edges(Edges&&) = default;
    Edges& operator=(Edges&&) = default;

    size_t len() const;

    bool is_empty() const;

    // Whether a car with this footprint, at `position` facing `heading`,
    // touches an edge.
    bool touch(const Footprint& footprint, std::array<double, 2> position, double heading) const;

private:
    struct Segment {
        std::array<std::array<double, 2>, 2> ends;
        // The box around it, to skip it quickly.
        std::array<double, 2> min_;
        std::array<double, 2> max_;
    };

    std::vector<Segment> segments;
};

// Whether two segments meet: cross, touch at a point, or overlap along a
// line.
bool meet(const std::array<std::array<double, 2>, 2>& first, const std::array<std::array<double, 2>, 2>& second);

} // namespace veehiicuul::collision
