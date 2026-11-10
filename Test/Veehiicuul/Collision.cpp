// What a car touches: segments that meet, a car's footprint, and a car
// against a track's edges.

#include "Veehiicuul/Collision.h"
#include "Testing/Testing.h"

#include <array>
#include <numbers>
#include <span>
#include <utility>

using veehiicuul::collision::Edges;
using veehiicuul::collision::Footprint;
using veehiicuul::collision::meet;

namespace {

constexpr double FracPi2 = std::numbers::pi / 2.0;

using Segment = std::array<std::array<double, 2>, 2>;

// A square track 20 units across, around the origin.
Edges square() {
    std::array<std::array<double, 2>, 4> outline{{{-10.0, -10.0}, {10.0, -10.0}, {10.0, 10.0}, {-10.0, 10.0}}};
    std::array<std::span<const std::array<double, 2>>, 1> outlines{outline};
    return Edges(outlines);
}

constexpr Footprint CarFootprint{
    .min_ = {-1.0, -2.0},
    .max_ = {1.0, 2.0},
};

} // namespace

TEST(segments_meet_when_they_cross_touch_or_overlap) {
    Segment across{{{0.0, 0.0}, {2.0, 2.0}}};
    struct Case {
        Segment other;
        bool meets;
    };
    for (const Case& expected : {
             Case{{{{0.0, 2.0}, {2.0, 0.0}}}, true},
             // At an end of each.
             Case{{{{2.0, 2.0}, {3.0, 0.0}}}, true},
             // An end on the other's middle.
             Case{{{{1.0, 1.0}, {3.0, 0.0}}}, true},
             // Along the same line, overlapping, and not.
             Case{{{{1.0, 1.0}, {3.0, 3.0}}}, true},
             Case{{{{2.5, 2.5}, {3.0, 3.0}}}, false},
             // Parallel, apart.
             Case{{{{1.0, 0.0}, {3.0, 2.0}}}, false},
             // Would cross if it were longer.
             Case{{{{0.0, 2.0}, {0.9, 1.1}}}, false},
         }) {
        CHECK_EQ(meet(across, expected.other), expected.meets, "{}", testing::describe(expected.other));
        CHECK_EQ(meet(expected.other, across), expected.meets, "{} the other way", testing::describe(expected.other));
    }
}

// A footprint goes from the car's middle to each side, and fits a model's
// rectangle from above, along X, to the model's left, and Z, ahead.
TEST(a_footprint_is_its_sides_or_a_model_s_rectangle) {
    Footprint footprint = Footprint::from_sides(1.5, 1.25, 3.0, 2.0);
    CHECK_EQ(std::pair(footprint.min_, footprint.max_), std::pair(std::array{-1.25, -2.0}, std::array{1.5, 3.0}));
    std::array<std::array<float, 3>, 3> positions{{{-1.0f, 5.0f, -3.0f}, {2.0f, 0.0f, 4.0f}, {0.0f, -1.0f, 0.0f}}};
    Footprint model = Footprint::of_model(positions);
    CHECK_EQ(std::pair(model.min_, model.max_), std::pair(std::array{-1.0, -3.0}, std::array{2.0, 4.0}));
}

TEST(a_car_touches_an_edge_that_a_side_meets) {
    Edges edges = square();
    CHECK_EQ(edges.len(), 4);
    CHECK(!edges.touch(CarFootprint, {0.0, 0.0}, 0.0));
    // Facing +Z, its front is 2 ahead: it touches the edge at Z = 10 from 8.
    CHECK(!edges.touch(CarFootprint, {0.0, 7.99}, 0.0));
    CHECK(edges.touch(CarFootprint, {0.0, 8.0}, 0.0));
    // Turned to face +X, it's 2 long along Z, so it fits there.
    CHECK(!edges.touch(CarFootprint, {0.0, 8.5}, FracPi2));
    CHECK(edges.touch(CarFootprint, {8.5, 0.0}, FracPi2));
    // Across a corner, at 45 degrees.
    CHECK(edges.touch(CarFootprint, {9.5, 9.5}, 0.785));
    // All the way past an edge, it touches none, as only where it is counts.
    CHECK(!edges.touch(CarFootprint, {0.0, 20.0}, 0.0));
}
