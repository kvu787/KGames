// The bell shadow: pure black around the box, round around its corners,
// whose opacity falls along a bell curve, a Gaussian of the distance from the
// box, whose standard deviation is the width. Its tail never quite ends, so
// its quad ends where the tail is fainter than half of a pixel's step.
// Shaders/Bell.hlsl.

use user_interface::Rect;

use crate::approaches::{FAINTEST, Setting, box_distance};

pub const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/BellVertex.cso"));
pub const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/BellPixel.cso"));

pub const DESCRIPTION: &str = "Black around the box, falling along a bell curve of the distance: flat where it leaves the box, then faster and faster.";

pub const SETTINGS: &[Setting] = &[Setting::pixels("Width", "Width", 28.0, 8.0), Setting::opacity("Opacity", "Opacity", 0.25)];

pub fn params(values: &[f32]) -> [f32; 8] {
    [values[0].max(0.5), values[1], 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
}

// How far out the shadow is still at least the faintest that shows.
pub fn reach(values: &[f32]) -> f32 {
    let [width, opacity] = [values[0].max(0.5), values[1]];
    if opacity <= FAINTEST { 0.0 } else { width * (2.0 * (opacity / FAINTEST).ln()).sqrt() }
}

pub fn area(values: &[f32], rect: Rect) -> Rect {
    rect.expand(reach(values).ceil() + 1.0)
}

pub fn opacity(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    let [width, opacity] = [values[0].max(0.5), values[1]];
    let distance = box_distance(rect, x, y).max(0.0);
    opacity * (-distance * distance / (2.0 * width * width)).exp()
}
