// The offset shadow: a copy of the box in black, moved by the offset, with
// hard edges, as flat designs have it. Shaders/Offset.hlsl.

use user_interface::Rect;

use crate::approaches::{Setting, box_distance, coverage};

pub const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/OffsetVertex.cso"));
pub const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/OffsetPixel.cso"));

pub const DESCRIPTION: &str = "A copy of the box in black, moved by the offset, with hard edges, as flat designs have it.";

pub const SETTINGS: &[Setting] = &[
    Setting::offset("OffsetX", "Offset across", -48.0, 48.0, 12.0),
    Setting::offset("OffsetY", "Offset down", -48.0, 48.0, 12.0),
    Setting::opacity("Opacity", "Opacity", 0.3),
];

pub fn params(values: &[f32]) -> [f32; 8] {
    [values[0], values[1], values[2], 0.0, 0.0, 0.0, 0.0, 0.0]
}

pub fn area(values: &[f32], rect: Rect) -> Rect {
    rect.union(rect.offset(values[0], values[1])).expand(1.0)
}

pub fn opacity(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    values[2] * coverage(box_distance(rect.offset(values[0], values[1]), x, y))
}
