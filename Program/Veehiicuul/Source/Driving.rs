// How a car moves: on the flat ground, along X and Z, with no mass, no grip
// and no steering wheel. The player pushes it toward where they want it to
// go, and it accelerates that way, at its vehicle's rates: forward, in
// reverse, and to either side. It always faces the way that it moves. The
// brake slows it down along a straight line, so it doesn't turn while it
// brakes, and it never moves it backward.

// How fast a vehicle speeds up each way, in units per second per second,
// in its own directions. Reverse is also how fast the brake slows it down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Acceleration {
    pub forward: f64,
    pub reverse: f64,
    pub left: f64,
    pub right: f64,
}

// Where a car is, which way it faces, and how fast it moves.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Car {
    // On the ground, along X and Z.
    pub position: [f64; 2],
    // Which way it faces, in radians from +Z toward +X.
    pub heading: f64,
    // In units per second, along X and Z.
    pub velocity: [f64; 2],
}

impl Car {
    pub fn speed(&self) -> f64 {
        self.velocity[0].hypot(self.velocity[1])
    }
}

// A dead zone along one axis of a stick: a value inside `inner` is 0, one
// past `outer` is 1, and one between them is scaled to cover 0 to 1. Both are
// from 0 to 1, `inner` below `outer`. Sticks don't read exactly 0 at rest,
// and a thumb can't hold one exactly still near the middle, so the inner dead
// zone keeps the car from creeping. Sticks reach the edge at different
// places, so the outer one lets each reach full acceleration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeadZone {
    pub inner: f64,
    pub outer: f64,
}

impl DeadZone {
    pub fn apply(&self, value: f64) -> f64 {
        let magnitude = value.abs();
        if magnitude < self.inner {
            return 0.0;
        }
        let scaled = if magnitude > self.outer {
            1.0
        } else {
            (magnitude - self.inner) / (self.outer - self.inner)
        };
        scaled.copysign(value)
    }
}

// The dead zones of the push, along the car's own directions: sideways, and
// forward and back. Pushing exactly ahead is harder than pushing exactly to
// the side, so the sideways one is wider, and pushing ahead doesn't turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeadZones {
    pub sideways: DeadZone,
    pub forward_and_back: DeadZone,
}

impl Default for DeadZones {
    fn default() -> Self {
        DeadZones {
            sideways: DeadZone {
                inner: 0.10,
                outer: 0.95,
            },
            forward_and_back: DeadZone {
                inner: 0.05,
                outer: 0.95,
            },
        }
    }
}

// Below this speed, in units per second, the brake stops the car at once,
// instead of slowing it by less and less.
const STOPPED: f64 = 0.01;

// Moves the car `seconds` on. `push` is where the player pushes it, on the
// ground along X and Z, about 1 long at most, and `brake` how hard they
// brake, from 0 to 1. While they brake, the push does nothing.
pub fn drive(
    car: &mut Car,
    push: [f64; 2],
    brake: f64,
    acceleration: &Acceleration,
    dead_zones: &DeadZones,
    seconds: f64,
) {
    let (sin, cos) = car.heading.sin_cos();
    if brake == 0.0 {
        // The push along the car's own directions: +X to its left and +Z
        // ahead. The dead zones apply to each, so a push that's nearly ahead
        // is exactly ahead.
        let [x, z] = push;
        let mut left = dead_zones.sideways.apply(x * cos - z * sin);
        let mut ahead = dead_zones.forward_and_back.apply(x * sin + z * cos);
        let length = left.hypot(ahead);
        if length > 1.0 {
            left /= length;
            ahead /= length;
        }
        left *= if left < 0.0 { acceleration.right } else { acceleration.left };
        ahead *= if ahead < 0.0 { acceleration.reverse } else { acceleration.forward };
        // Back along X and Z.
        car.velocity[0] += seconds * (left * cos + ahead * sin);
        car.velocity[1] += seconds * (ahead * cos - left * sin);
    } else {
        let speed = car.speed();
        let slowing = acceleration.reverse * brake * seconds;
        car.velocity = if speed < STOPPED || slowing >= speed {
            [0.0, 0.0]
        } else {
            car.velocity.map(|component| component * (speed - slowing) / speed)
        };
    }
    if car.velocity != [0.0, 0.0] {
        car.heading = car.velocity[0].atan2(car.velocity[1]);
    }
    car.position[0] += car.velocity[0] * seconds;
    car.position[1] += car.velocity[1] * seconds;
}
