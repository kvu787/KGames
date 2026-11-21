// The linear shadow: pure black all around the box, as far as its length
// from the box's edge, whose opacity goes in a straight line from the start's
// at the edge to the end's at the length, by the distance from the box, so
// that it's round around the box's square corners. Shaders/Linear.hlsl.

use user_interface::Rect;

use crate::approaches::{Setting, box_distance, coverage};

pub const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/LinearVertex.cso"));
pub const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/LinearPixel.cso"));

pub const DESCRIPTION: &str = "Black around the box, as far as its length. Its opacity goes in a straight line from the start at the edge to the end.";

pub const SETTINGS: &[Setting] = &[
    Setting::pixels("Length", "Length", 100.0, 20.0),
    Setting::opacity("StartOpacity", "Start opacity", 0.25),
    Setting::opacity("EndOpacity", "End opacity", 0.0),
];

pub fn params(values: &[f32]) -> [f32; 8] {
    [values[0], values[1], values[2], 0.0, 0.0, 0.0, 0.0, 0.0]
}

pub fn area(values: &[f32], rect: Rect) -> Rect {
    rect.expand(values[0] + 1.0)
}

pub fn opacity(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    let [length, start, end] = [values[0], values[1], values[2]];
    if length <= 0.0 {
        return 0.0;
    }
    let distance = box_distance(rect, x, y);
    let along = start + (end - start) * (distance / length).clamp(0.0, 1.0);
    along * coverage(distance - length)
}
