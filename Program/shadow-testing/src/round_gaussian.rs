// The round Gaussian shadow: the Gaussian's edge, by the distance from the
// box grown by the spread, as many UI renderers draw it. Straight out from a
// side it's the Gaussian's, with the same settings, but around the corners
// it's round, and as dark diagonally out from a corner as straight out from a
// side. Shaders/RoundGaussian.hlsl.

use user_interface::Rect;

use crate::approaches::box_distance;
use crate::gaussian::{erf, sigma};

pub const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/RoundGaussianVertex.cso"));
pub const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/RoundGaussianPixel.cso"));

pub const DESCRIPTION: &str = "Gaussian's edge, by the distance from the box, as many UI renderers draw it: the same on the sides, round at the corners.";

// Past 4 deviations out from the grown box, the shadow is less than a
// ten-thousandth of its opacity.
pub fn area(values: &[f32], rect: Rect) -> Rect {
    rect.expand((values[1] + 4.0 * sigma(values)).max(0.0) + 1.0)
}

pub fn opacity(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    let distance = box_distance(rect, x, y) - values[1];
    values[2] * 0.5 * (1.0 - erf(distance / (sigma(values) * std::f32::consts::SQRT_2)))
}
