// The exponential shadow: pure black around the box, round around its
// corners, whose opacity falls by a factor of e every decay length out from
// the box. Its tail never quite ends, so its quad ends where the tail is
// fainter than half of a pixel's step. Shaders/Exponential.hlsl.

use user_interface::Rect;

use crate::approaches::{FAINTEST, Setting, box_distance};

pub const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ExponentialVertex.cso"));
pub const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ExponentialPixel.cso"));

pub const DESCRIPTION: &str = "Black around the box, falling by a factor of e every decay length out: steeply at first, with a long faint tail.";

pub const SETTINGS: &[Setting] = &[
    Setting {
        name: "Decay",
        label: "Decay length",
        minimum: 1.0,
        maximum: 16.0,
        step: 0.5,
        default: 14.0,
        decimals: 1,
        suffix: " px",
    },
    Setting::opacity("Opacity", "Opacity", 0.25),
];

pub fn params(values: &[f32]) -> [f32; 8] {
    [values[0], values[1], 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
}

// How far out the shadow is still at least the faintest that shows.
pub fn reach(values: &[f32]) -> f32 {
    let [decay, opacity] = [values[0], values[1]];
    if opacity <= FAINTEST { 0.0 } else { decay * (opacity / FAINTEST).ln() }
}

pub fn area(values: &[f32], rect: Rect) -> Rect {
    rect.expand(reach(values).ceil() + 1.0)
}

pub fn opacity(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    let [decay, opacity] = [values[0], values[1]];
    let distance = box_distance(rect, x, y).max(0.0);
    opacity * (-distance / decay).exp()
}
