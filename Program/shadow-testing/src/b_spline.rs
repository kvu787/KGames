// The B-spline shadow: like the Gaussian, with its settings, a copy of the
// box grown by the spread, but blurred by a cubic B-spline, four box blurs
// over each other, of the same standard deviation. It looks nearly the same,
// but ends exactly two knots, about 3.46 deviations, out from the grown box.
// Shaders/BSpline.hlsl.

use user_interface::Rect;

use crate::gaussian::{grown, sigma};

pub const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/BSplineVertex.cso"));
pub const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/BSplinePixel.cso"));

pub const DESCRIPTION: &str = "Like Gaussian, but blurred by a cubic B-spline, four box blurs: nearly the same, but it ends exactly.";

// The spacing of the B-spline's knots for the blur: a cubic B-spline's
// variance is a third of its knot spacing squared.
pub fn knot(values: &[f32]) -> f32 {
    sigma(values) * 3.0f32.sqrt()
}

// The B-spline reaches two knots either side, and no further.
pub fn area(values: &[f32], rect: Rect) -> Rect {
    let shadow = grown(rect, values[1]);
    if shadow.is_empty() {
        return rect.expand(1.0);
    }
    rect.union(shadow.expand(2.0 * knot(values))).expand(1.0)
}

pub fn opacity(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    let shadow = grown(rect, values[1]);
    if shadow.is_empty() {
        return 0.0;
    }
    let knot = knot(values);
    let across = spline_below((x - shadow.left()) / knot) - spline_below((x - shadow.right()) / knot);
    let down = spline_below((y - shadow.top()) / knot) - spline_below((y - shadow.bottom()) / knot);
    values[2] * across * down
}

// How much of a cubic B-spline whose knots are 1 apart lies left of t: its
// cumulative distribution, from 0 at -2 to 1 at 2, as the shader has it.
pub fn spline_below(t: f32) -> f32 {
    let a = t.abs();
    let between = if a < 1.0 {
        (4.0 * a - 2.0 * a * a * a + 0.75 * a * a * a * a) / 6.0
    } else if a < 2.0 {
        let rest = 2.0 - a;
        0.5 - rest * rest * rest * rest / 24.0
    } else {
        0.5
    };
    if t < 0.0 { 0.5 - between } else { 0.5 + between }
}
