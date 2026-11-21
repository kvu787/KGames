// The long shadow, as flat designs have it: the box dragged along a
// direction as far as the length, whose opacity goes in a straight line from
// the start's to the end's by how far along the box had to go to cover a
// point. Its edges run along the direction, at any angle, so each pixel is
// the mean of 16 samples in a 4 x 4 grid. Shaders/Long.hlsl.

use user_interface::Rect;

use crate::approaches::Setting;

pub const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/LongVertex.cso"));
pub const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/LongPixel.cso"));

pub const DESCRIPTION: &str = "The box dragged along a direction as far as the length, from the start opacity to the end's. 0° is right, and 90° down.";

pub const SETTINGS: &[Setting] = &[
    Setting::pixels("Length", "Length", 128.0, 64.0),
    Setting {
        name: "Angle",
        label: "Angle",
        minimum: 0.0,
        maximum: 359.0,
        step: 1.0,
        default: 45.0,
        decimals: 0,
        suffix: "°",
    },
    Setting::opacity("StartOpacity", "Start opacity", 0.3),
    Setting::opacity("EndOpacity", "End opacity", 0.0),
];

// The direction that the box is dragged in, a unit vector, with y down.
pub fn direction(values: &[f32]) -> (f32, f32) {
    let angle = values[1].to_radians();
    (angle.cos(), angle.sin())
}

pub fn params(values: &[f32]) -> [f32; 8] {
    let (x, y) = direction(values);
    [values[0], x, y, values[2], values[3], 0.0, 0.0, 0.0]
}

pub fn area(values: &[f32], rect: Rect) -> Rect {
    let (x, y) = direction(values);
    rect.union(rect.offset(x * values[0], y * values[0])).expand(1.0)
}

// The opacity at a point: from the least distance along the direction, from
// 0 to the length, at which the box, moved that far, has the point in it.
fn sample(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    let [length, _, start, end] = [values[0], values[1], values[2], values[3]];
    let (dx, dy) = direction(values);
    let (mut enter, mut leave) = (0.0f32, length);
    for (position, direction, low, high) in [(x, dx, rect.left(), rect.right()), (y, dy, rect.top(), rect.bottom())] {
        if direction.abs() < 1e-6 {
            if position < low || position > high {
                return 0.0;
            }
        } else {
            let first = (position - high) / direction;
            let second = (position - low) / direction;
            enter = enter.max(first.min(second));
            leave = leave.min(first.max(second));
        }
    }
    if enter > leave {
        return 0.0;
    }
    start + (end - start) * (enter / length)
}

pub fn opacity(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    if values[0] <= 0.0 {
        return 0.0;
    }
    let mut sum = 0.0;
    for row in 0..4 {
        for column in 0..4 {
            sum += sample(values, rect, x + (column as f32 + 0.5) / 4.0 - 0.5, y + (row as f32 + 0.5) / 4.0 - 0.5);
        }
    }
    sum / 16.0
}
