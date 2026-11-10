// How a car drives: the dead zones, accelerating each way at its vehicle's
// rates, facing the way it moves, and braking.

#include "Veehiicuul/Driving.h"
#include "Base/Text.h"
#include "Testing/Testing.h"

#include <array>
#include <cmath>
#include <numbers>
#include <tuple>
#include <utility>

using veehiicuul::driving::Acceleration;
using veehiicuul::driving::Car;
using veehiicuul::driving::DeadZone;
using veehiicuul::driving::DeadZones;
using veehiicuul::driving::drive;

namespace {

constexpr double FracPi2 = std::numbers::pi / 2.0;
constexpr double Pi = std::numbers::pi;

constexpr Acceleration Rates{
    .forward = 10.0,
    .reverse = 20.0,
    .left = 4.0,
    .right = 8.0,
};

bool near_(double actual, double expected) {
    return std::abs(actual - expected) < 1e-9;
}

bool near_point(std::array<double, 2> actual, std::array<double, 2> expected) {
    return near_(actual[0], expected[0]) && near_(actual[1], expected[1]);
}

// A car at rest at the origin, facing +Z.
Car at_rest() {
    return Car();
}

} // namespace

TEST(a_dead_zone_zeroes_the_inside_fills_the_rest_and_keeps_the_sign) {
    DeadZone zone{
        .inner = 0.1,
        .outer = 0.9,
    };
    for (auto [value, expected] : {
             std::pair{0.0, 0.0},
             std::pair{0.05, 0.0},
             std::pair{-0.0999, 0.0},
             std::pair{0.1, 0.0},
             std::pair{0.5, 0.5},
             std::pair{-0.5, -0.5},
             std::pair{0.9, 1.0},
             std::pair{0.95, 1.0},
             std::pair{-1.0, -1.0},
         }) {
        CHECK(near_(zone.apply(value), expected), "{} gave {}", base::display(value), base::display(zone.apply(value)));
    }
}

// Pushing ahead speeds the car up at its forward rate, and it keeps facing
// that way.
TEST(pushing_ahead_speeds_up_at_the_forward_rate) {
    Car car = at_rest();
    for (int step = 0; step < 10; step++) {
        drive(car, {0.0, 1.0}, 0.0, Rates, DeadZones(), 0.1);
    }
    CHECK(near_point(car.velocity, {0.0, 10.0}), "{}", testing::describe(car.velocity));
    CHECK(near_(car.heading, 0.0));
    // Each step moves it by its speed after the step: 0.1 + 0.2 + ... + 1.0.
    CHECK(near_point(car.position, {0.0, 5.5}), "{}", testing::describe(car.position));
}

// A push to the car's side speeds it up that way, at the side's rate, and
// it turns to face the way it moves. +X is the left of a car facing +Z.
TEST(pushing_to_a_side_speeds_up_at_that_side_s_rate) {
    struct Case {
        std::array<double, 2> push;
        std::array<double, 2> velocity;
        double heading;
    };
    for (const Case& expected : {Case{{1.0, 0.0}, {0.4, 0.0}, FracPi2}, Case{{-1.0, 0.0}, {-0.8, 0.0}, -FracPi2}}) {
        Car car = at_rest();
        drive(car, expected.push, 0.0, Rates, DeadZones(), 0.1);
        CHECK(near_point(car.velocity, expected.velocity), "{} gave {}", testing::describe(expected.push), testing::describe(car.velocity));
        CHECK(near_(car.heading, expected.heading));
    }
}

// Pulling back from rest speeds it up backward at its reverse rate, and it
// turns around, as it faces the way it moves.
TEST(pulling_back_reverses_and_turns_the_car_around) {
    Car car = at_rest();
    drive(car, {0.0, -1.0}, 0.0, Rates, DeadZones(), 0.1);
    CHECK(near_point(car.velocity, {0.0, -2.0}));
    CHECK(near_(std::abs(car.heading), Pi));
}

// The dead zones apply along the car's own directions, so a push that's
// nearly ahead is exactly ahead, and a push past the outer dead zone is
// full.
TEST(a_push_that_is_nearly_ahead_is_exactly_ahead) {
    Car car = at_rest();
    drive(car, {0.08, 0.96}, 0.0, Rates, DeadZones(), 0.1);
    CHECK(near_point(car.velocity, {0.0, 1.0}), "{}", testing::describe(car.velocity));
    // A diagonal that's past both outer dead zones is no longer than 1.
    Car diagonal = at_rest();
    drive(diagonal, {1.0, 1.0}, 0.0, Rates, DeadZones(), 1.0);
    double half = std::numbers::sqrt2 / 2.0;
    CHECK(near_point(diagonal.velocity, {4.0 * half, 10.0 * half}), "{}", testing::describe(diagonal.velocity));
}

// The brake slows the car along a straight line, at its reverse rate, while
// the push does nothing, and stops it without moving it backward.
TEST(the_brake_slows_the_car_without_turning_it_and_stops_it) {
    Car car{
        .heading = std::atan2(3.0, 4.0),
        .velocity = {3.0, 4.0},
    };
    drive(car, {-1.0, 0.0}, 1.0, Rates, DeadZones(), 0.1);
    CHECK(near_point(car.velocity, {1.8, 2.4}), "{}", testing::describe(car.velocity));
    CHECK(near_(car.heading, std::atan2(3.0, 4.0)));
    // Half the brake slows it half as fast.
    drive(car, {0.0, 0.0}, 0.5, Rates, DeadZones(), 0.1);
    CHECK(near_(car.speed(), 2.0));
    drive(car, {0.0, 0.0}, 1.0, Rates, DeadZones(), 1.0);
    CHECK_EQ(car.velocity, (std::array{0.0, 0.0}));
    double heading = car.heading;
    std::array<double, 2> position = car.position;
    drive(car, {0.0, 0.0}, 1.0, Rates, DeadZones(), 1.0);
    CHECK_EQ(std::tuple(car.velocity, car.heading, car.position), std::tuple(std::array{0.0, 0.0}, heading, position));
}

// Without a push, a moving car keeps going.
TEST(a_car_keeps_going_without_a_push) {
    Car car{
        .heading = FracPi2,
        .velocity = {2.0, 0.0},
    };
    drive(car, {0.0, 0.0}, 0.0, Rates, DeadZones(), 0.5);
    CHECK_EQ(car.velocity, (std::array{2.0, 0.0}));
    CHECK(near_point(car.position, {1.0, 0.0}));
}
