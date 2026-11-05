// What a car touches: segments that meet, a car's footprint, and a car
// against a track's edges.

use std::f64::consts::FRAC_PI_2;

use veehiicuul::collision::{Edges, Footprint, meet};

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

// A footprint goes from the car's middle to each side, and fits a model's
// rectangle from above, along X, to the model's left, and Z, ahead.
#[test]
fn a_footprint_is_its_sides_or_a_model_s_rectangle() {
    let footprint = Footprint::from_sides(1.5, 1.25, 3.0, 2.0);
    assert_eq!((footprint.min, footprint.max), ([-1.25, -2.0], [1.5, 3.0]));
    let model = Footprint::of_model(&[[-1.0, 5.0, -3.0], [2.0, 0.0, 4.0], [0.0, -1.0, 0.0]]);
    assert_eq!((model.min, model.max), ([-1.0, -3.0], [2.0, 4.0]));
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
