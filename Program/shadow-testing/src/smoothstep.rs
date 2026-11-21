// The smoothstep shadow: pure black around the box, as far as its length,
// round around the box's corners, fading to nothing along an S curve,
// smoothstep, by the distance from the box. Shaders/Smoothstep.hlsl.

use user_interface::Rect;

use crate::approaches::{Setting, box_distance, coverage};

pub const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SmoothstepVertex.cso"));
pub const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SmoothstepPixel.cso"));

pub const DESCRIPTION: &str = "Black around the box, as far as its length, fading to nothing along an S curve: soft where it leaves the box and where it ends.";

pub const SETTINGS: &[Setting] = &[Setting::pixels("Length", "Length", 128.0, 20.0), Setting::opacity("Opacity", "Opacity", 0.25)];

pub fn params(values: &[f32]) -> [f32; 8] {
    [values[0], values[1], 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
}

pub fn area(values: &[f32], rect: Rect) -> Rect {
    rect.expand(values[0] + 1.0)
}

pub fn opacity(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    let [length, opacity] = [values[0], values[1]];
    if length <= 0.0 {
        return 0.0;
    }
    let distance = box_distance(rect, x, y);
    let t = (distance / length).clamp(0.0, 1.0);
    opacity * (1.0 - t * t * (3.0 - 2.0 * t)) * coverage(distance - length)
}
