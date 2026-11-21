// Layered Gaussians, like several of CSS's box-shadows with no offset over
// each other: the widest layer is the box blurred by the blur, and each one
// after it by half the one before, each at the opacity, over each other. The
// narrow layers make a dark core near the box, and the wide ones a long soft
// tail. Shaders/Layered.hlsl.

use user_interface::Rect;

use crate::approaches::Setting;
use crate::gaussian::{blurred_box, sigma_of};

pub const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/LayeredVertex.cso"));
pub const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/LayeredPixel.cso"));

pub const DESCRIPTION: &str = "Gaussians over each other, each with half the blur of the one before: a dark core near the box, and a long soft tail.";

pub const SETTINGS: &[Setting] = &[
    Setting {
        name: "Layers",
        label: "Layers",
        minimum: 1.0,
        maximum: 5.0,
        step: 1.0,
        default: 3.0,
        decimals: 0,
        suffix: "",
    },
    Setting::pixels("Blur", "Widest blur", 64.0, 56.0),
    Setting::opacity("Opacity", "Opacity of each", 0.2),
];

pub fn params(values: &[f32]) -> [f32; 8] {
    [values[0], values[1] / 2.0, values[2], 0.0, 0.0, 0.0, 0.0, 0.0]
}

// Past 4 deviations of the widest layer, the shadow is less than a
// ten-thousandth of its opacity.
pub fn area(values: &[f32], rect: Rect) -> Rect {
    rect.expand(4.0 * sigma_of(values[1]) + 1.0)
}

pub fn opacity(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    let [layers, blur, opacity] = [values[0], values[1], values[2]];
    let mut through = 1.0;
    let mut sigma = blur / 2.0;
    for layer in 0..5 {
        if (layer as f32) < layers {
            through *= 1.0 - opacity * blurred_box(rect, sigma.max(0.25), x, y);
        }
        sigma *= 0.5;
    }
    1.0 - through
}
