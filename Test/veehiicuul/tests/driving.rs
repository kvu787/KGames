// How a car drives: the dead zones, accelerating each way at its vehicle's
// rates, facing the way it moves, and braking.

use std::f64::consts::{FRAC_PI_2, PI};

use veehiicuul::driving::{Acceleration, Car, DeadZone, DeadZones, drive};

const RATES: Acceleration = Acceleration {
    forward: 10.0,
    reverse: 20.0,
    left: 4.0,
    right: 8.0,
};

fn near(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < 1e-9
}

fn near_point(actual: [f64; 2], expected: [f64; 2]) -> bool {
    near(actual[0], expected[0]) && near(actual[1], expected[1])
}

// A car at rest at the origin, facing +Z.
fn at_rest() -> Car {
    Car::default()
}

#[test]
fn a_dead_zone_zeroes_the_inside_fills_the_rest_and_keeps_the_sign() {
    let zone = DeadZone {
        inner: 0.1,
        outer: 0.9,
    };
    for (value, expected) in [
        (0.0, 0.0),
        (0.05, 0.0),
        (-0.0999, 0.0),
        (0.1, 0.0),
        (0.5, 0.5),
        (-0.5, -0.5),
        (0.9, 1.0),
        (0.95, 1.0),
        (-1.0, -1.0),
    ] {
        assert!(near(zone.apply(value), expected), "{value} gave {}", zone.apply(value));
    }
}

// Pushing ahead speeds the car up at its forward rate, and it keeps facing
// that way.
#[test]
fn pushing_ahead_speeds_up_at_the_forward_rate() {
    let mut car = at_rest();
    for _ in 0..10 {
        drive(&mut car, [0.0, 1.0], 0.0, &RATES, &DeadZones::default(), 0.1);
    }
    assert!(near_point(car.velocity, [0.0, 10.0]), "{:?}", car.velocity);
    assert!(near(car.heading, 0.0));
    // Each step moves it by its speed after the step: 0.1 + 0.2 + ... + 1.0.
    assert!(near_point(car.position, [0.0, 5.5]), "{:?}", car.position);
}

// A push to the car's side speeds it up that way, at the side's rate, and
// it turns to face the way it moves. +X is the left of a car facing +Z.
#[test]
fn pushing_to_a_side_speeds_up_at_that_side_s_rate() {
    for (push, velocity, heading) in [([1.0, 0.0], [0.4, 0.0], FRAC_PI_2), ([-1.0, 0.0], [-0.8, 0.0], -FRAC_PI_2)] {
        let mut car = at_rest();
        drive(&mut car, push, 0.0, &RATES, &DeadZones::default(), 0.1);
        assert!(near_point(car.velocity, velocity), "{push:?} gave {:?}", car.velocity);
        assert!(near(car.heading, heading));
    }
}

// Pulling back from rest speeds it up backward at its reverse rate, and it
// turns around, as it faces the way it moves.
#[test]
fn pulling_back_reverses_and_turns_the_car_around() {
    let mut car = at_rest();
    drive(&mut car, [0.0, -1.0], 0.0, &RATES, &DeadZones::default(), 0.1);
    assert!(near_point(car.velocity, [0.0, -2.0]));
    assert!(near(car.heading.abs(), PI));
}

// The dead zones apply along the car's own directions, so a push that's
// nearly ahead is exactly ahead, and a push past the outer dead zone is
// full.
#[test]
fn a_push_that_is_nearly_ahead_is_exactly_ahead() {
    let mut car = at_rest();
    drive(&mut car, [0.08, 0.96], 0.0, &RATES, &DeadZones::default(), 0.1);
    assert!(near_point(car.velocity, [0.0, 1.0]), "{:?}", car.velocity);
    // A diagonal that's past both outer dead zones is no longer than 1.
    let mut car = at_rest();
    drive(&mut car, [1.0, 1.0], 0.0, &RATES, &DeadZones::default(), 1.0);
    let half = std::f64::consts::FRAC_1_SQRT_2;
    assert!(near_point(car.velocity, [4.0 * half, 10.0 * half]), "{:?}", car.velocity);
}

// The brake slows the car along a straight line, at its reverse rate, while
// the push does nothing, and stops it without moving it backward.
#[test]
fn the_brake_slows_the_car_without_turning_it_and_stops_it() {
    let mut car = Car {
        velocity: [3.0, 4.0],
        heading: 3.0_f64.atan2(4.0),
        ..Car::default()
    };
    drive(&mut car, [-1.0, 0.0], 1.0, &RATES, &DeadZones::default(), 0.1);
    assert!(near_point(car.velocity, [1.8, 2.4]), "{:?}", car.velocity);
    assert!(near(car.heading, 3.0_f64.atan2(4.0)));
    // Half the brake slows it half as fast.
    drive(&mut car, [0.0, 0.0], 0.5, &RATES, &DeadZones::default(), 0.1);
    assert!(near(car.speed(), 2.0));
    drive(&mut car, [0.0, 0.0], 1.0, &RATES, &DeadZones::default(), 1.0);
    assert_eq!(car.velocity, [0.0, 0.0]);
    let heading = car.heading;
    let position = car.position;
    drive(&mut car, [0.0, 0.0], 1.0, &RATES, &DeadZones::default(), 1.0);
    assert_eq!((car.velocity, car.heading, car.position), ([0.0, 0.0], heading, position));
}

// Without a push, a moving car keeps going.
#[test]
fn a_car_keeps_going_without_a_push() {
    let mut car = Car {
        velocity: [2.0, 0.0],
        heading: FRAC_PI_2,
        ..Car::default()
    };
    drive(&mut car, [0.0, 0.0], 0.0, &RATES, &DeadZones::default(), 0.5);
    assert_eq!(car.velocity, [2.0, 0.0]);
    assert!(near_point(car.position, [1.0, 0.0]));
}
