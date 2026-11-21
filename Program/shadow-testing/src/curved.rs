// The curved shadow: like the linear one, pure black around the box, as far
// as its length, round around the box's corners, but fading to nothing along
// a curve: its opacity times what's left of the length, as a fraction, to the
// power. Shaders/Curved.hlsl.

use user_interface::Rect;

use crate::approaches::{Setting, box_distance, coverage};

pub const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/CurvedVertex.cso"));
pub const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/CurvedPixel.cso"));

pub const DESCRIPTION: &str = "Like linear, but fading to nothing along a curve: the opacity times the fraction of the length left, to the power.";

pub const SETTINGS: &[Setting] = &[
    Setting::pixels("Length", "Length", 128.0, 32.0),
    Setting::opacity("Opacity", "Opacity", 0.25),
    Setting {
        name: "Power",
        label: "Power",
        minimum: 0.25,
        maximum: 4.0,
        step: 0.05,
        default: 2.0,
        decimals: 2,
        suffix: "",
    },
];

pub fn params(values: &[f32]) -> [f32; 8] {
    [values[0], values[1], values[2], 0.0, 0.0, 0.0, 0.0, 0.0]
}

pub fn area(values: &[f32], rect: Rect) -> Rect {
    rect.expand(values[0] + 1.0)
}

pub fn opacity(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    let [length, opacity, power] = [values[0], values[1], values[2]];
    if length <= 0.0 {
        return 0.0;
    }
    let distance = box_distance(rect, x, y);
    opacity * (1.0 - (distance / length).clamp(0.0, 1.0)).powf(power) * coverage(distance - length)
}
