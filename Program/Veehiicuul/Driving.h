// How a car moves: on the flat ground, along X and Z, with no mass, no grip
// and no steering wheel. The player pushes it toward where they want it to
// go, and it accelerates that way, at its vehicle's rates: forward, in
// reverse, and to either side. It always faces the way that it moves. The
// brake slows it down along a straight line, so it doesn't turn while it
// brakes, and it never moves it backward.

#pragma once

#include <array>
#include <string>

namespace veehiicuul::driving {

// How fast a vehicle speeds up each way, in units per second per second,
// in its own directions. Reverse is also how fast the brake slows it down.
struct Acceleration {
    double forward = 0.0;
    double reverse = 0.0;
    double left = 0.0;
    double right = 0.0;

    bool operator==(const Acceleration&) const = default;
};

// Where a car is, which way it faces, and how fast it moves.
struct Car {
    // On the ground, along X and Z.
    std::array<double, 2> position{};
    // Which way it faces, in radians from +Z toward +X.
    double heading = 0.0;
    // In units per second, along X and Z.
    std::array<double, 2> velocity{};

    double speed() const;

    bool operator==(const Car&) const = default;
};

// A dead zone along one axis of a stick: a value inside `inner` is 0, one
// past `outer` is 1, and one between them is scaled to cover 0 to 1. Both are
// from 0 to 1, `inner` below `outer`. Sticks don't read exactly 0 at rest,
// and a thumb can't hold one exactly still near the middle, so the inner dead
// zone keeps the car from creeping. Sticks reach the edge at different
// places, so the outer one lets each reach full acceleration.
struct DeadZone {
    double inner = 0.0;
    double outer = 0.0;

    double apply(double value) const;

    bool operator==(const DeadZone&) const = default;
};

// The dead zones of the push, along the car's own directions: sideways, and
// forward and back. Pushing exactly ahead is harder than pushing exactly to
// the side, so the sideways one is wider, and pushing ahead doesn't turn.
struct DeadZones {
    DeadZone sideways{
        .inner = 0.10,
        .outer = 0.95,
    };
    DeadZone forward_and_back{
        .inner = 0.05,
        .outer = 0.95,
    };

    bool operator==(const DeadZones&) const = default;
};

// How a failed check in a test shows them.
std::string describe_value(const Acceleration& acceleration);
std::string describe_value(const Car& car);
std::string describe_value(const DeadZone& dead_zone);
std::string describe_value(const DeadZones& dead_zones);

// Moves the car `seconds` on. `push` is where the player pushes it, on the
// ground along X and Z, about 1 long at most, and `brake` how hard they
// brake, from 0 to 1. While they brake, the push does nothing.
void drive(Car& car, std::array<double, 2> push, double brake, const Acceleration& acceleration, const DeadZones& dead_zones, double seconds);

} // namespace veehiicuul::driving
