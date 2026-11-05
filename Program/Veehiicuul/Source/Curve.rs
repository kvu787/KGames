// The closed curves that make a track's edges: uniform cubic B-splines on the
// ground, like Blender's cyclic NURBS curves with every weight 1. A curve
// is smooth everywhere, and passes near its control points, not through
// them, so that moving one bends the curve around it without kinks. Each
// span, between the curve's parameters i and i + 1, is pulled by control
// points i - 1 to i + 2.

// How many points each span is sampled at, for the track's outlines.
pub const SAMPLES_PER_SPAN: usize = 8;
// The fewest control points a curve has.
pub const FEWEST: usize = 3;

// The curve's point at parameter `u`, which goes around once from 0 to the
// number of control points.
pub fn point(controls: &[[f64; 2]], u: f64) -> [f64; 2] {
    let count = controls.len();
    let span = u.floor();
    let t = u - span;
    let span = (span as i64).rem_euclid(count as i64) as usize;
    let weights = [
        (1.0 - t).powi(3) / 6.0,
        (3.0 * t.powi(3) - 6.0 * t * t + 4.0) / 6.0,
        (-3.0 * t.powi(3) + 3.0 * t * t + 3.0 * t + 1.0) / 6.0,
        t.powi(3) / 6.0,
    ];
    let mut result = [0.0; 2];
    for (offset, weight) in weights.iter().enumerate() {
        let control = controls[(span + count + offset - 1) % count];
        result[0] += weight * control[0];
        result[1] += weight * control[1];
    }
    result
}

// The curve as an outline: SAMPLES_PER_SPAN points a span, all the way
// around.
pub fn sample(controls: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let count = controls.len() * SAMPLES_PER_SPAN;
    (0..count)
        .map(|index| point(controls, index as f64 / SAMPLES_PER_SPAN as f64))
        .collect()
}

// Where to put a new control point so that it bends the curve near `near`:
// after the control point that starts the span nearest it. Returns the
// index for the new point.
pub fn insertion(controls: &[[f64; 2]], near: [f64; 2]) -> usize {
    let outline = sample(controls);
    let nearest = (0..outline.len())
        .min_by(|&a, &b| distance(outline[a], near).total_cmp(&distance(outline[b], near)))
        .unwrap_or(0);
    nearest / SAMPLES_PER_SPAN + 1
}

pub fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

// A circle of `count` control points around `center`, whose curve is nearly
// a circle of `radius`. A cubic B-spline's curve is inside its control
// points, so they're a little farther out.
pub fn circle(center: [f64; 2], radius: f64, count: usize) -> Vec<[f64; 2]> {
    let step = std::f64::consts::TAU / count as f64;
    // A control polygon of radius R makes a curve of about R * (2 + cos(step)) / 3.
    let outer = radius * 3.0 / (2.0 + step.cos());
    (0..count)
        .map(|index| {
            let angle = index as f64 * step;
            [center[0] + outer * angle.cos(), center[1] + outer * angle.sin()]
        })
        .collect()
}
