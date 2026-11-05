// What a car touches: segments that meet, the car's footprint, and a car
// against a track's edges.

use std::f64::consts::FRAC_PI_2;

use meshes::car;
use veehiicuul::collision::{Edges, FRONT_TRIM, Footprint, meet};

#[test]
fn segments_meet_when_they_cross_touch_or_overlap() {
    let across = [[0.0, 0.0], [2.0, 2.0]];
    for (other, meets) in [
        ([[0.0, 2.0], [2.0, 0.0]], true),
        // At an end of each.
        ([[2.0, 2.0], [3.0, 0.0]], true),
        // An end on the other's middle.
        ([[1.0, 1.0], [3.0, 0.0]], true),
        // Along the same line, overlapping, and not.
        ([[1.0, 1.0], [3.0, 3.0]], true),
        ([[2.5, 2.5], [3.0, 3.0]], false),
        // Parallel, apart.
        ([[1.0, 0.0], [3.0, 2.0]], false),
        // Would cross if it were longer.
        ([[0.0, 2.0], [0.9, 1.1]], false),
    ] {
        assert_eq!(meet(across, other), meets, "{other:?}");
        assert_eq!(meet(other, across), meets, "{other:?} the other way");
    }
}

// The car is 3 units wide and about 6 long, facing +Z, and its footprint
// is shorter at the front.
#[test]
fn the_footprint_is_the_car_mesh_from_above_shorter_at_the_front() {
    let footprint = Footprint::of_mesh(&car::VERTICES);
    let near = |actual: f64, expected: f64| (actual - expected).abs() < 1e-6;
    assert!(near(footprint.min[0], -1.5) && near(footprint.max[0], 1.5), "{footprint:?}");
    assert!(near(footprint.min[1], -3.0), "{footprint:?}");
    assert!(near(footprint.max[1], 3.1575217 - FRONT_TRIM), "{footprint:?}");
}

// A square track 20 units across, around the origin.
fn square() -> Edges {
    Edges::new(&[&[[-10.0, -10.0], [10.0, -10.0], [10.0, 10.0], [-10.0, 10.0]]])
}

const FOOTPRINT: Footprint = Footprint {
    min: [-1.0, -2.0],
    max: [1.0, 2.0],
};

#[test]
fn a_car_touches_an_edge_that_a_side_meets() {
    let edges = square();
    assert_eq!(edges.len(), 4);
    assert!(!edges.touch(&FOOTPRINT, [0.0, 0.0], 0.0));
    // Facing +Z, its front is 2 ahead: it touches the edge at Z = 10 from 8.
    assert!(!edges.touch(&FOOTPRINT, [0.0, 7.99], 0.0));
    assert!(edges.touch(&FOOTPRINT, [0.0, 8.0], 0.0));
    // Turned to face +X, it's 2 long along Z, so it fits there.
    assert!(!edges.touch(&FOOTPRINT, [0.0, 8.5], FRAC_PI_2));
    assert!(edges.touch(&FOOTPRINT, [8.5, 0.0], FRAC_PI_2));
    // Across a corner, at 45 degrees.
    assert!(edges.touch(&FOOTPRINT, [9.5, 9.5], 0.785));
    // All the way past an edge, it touches none, as only where it is counts.
    assert!(!edges.touch(&FOOTPRINT, [0.0, 20.0], 0.0));
}
