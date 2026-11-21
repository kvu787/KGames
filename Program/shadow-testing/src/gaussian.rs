// The Gaussian shadow, like CSS's box-shadow with no offset: a copy of the
// box, grown by the spread on every side, blurred by a Gaussian whose
// standard deviation is half the blur, as CSS's is. A rectangle with square
// corners blurs one axis at a time, so its coverage is a product of the error
// function's differences in x and in y. Shaders/Gaussian.hlsl.

use user_interface::Rect;

use crate::approaches::Setting;

pub const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/GaussianVertex.cso"));
pub const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/GaussianPixel.cso"));

pub const DESCRIPTION: &str = "A blurred copy of the box, grown by the spread on every side, like CSS's box-shadow with no offset.";

pub const SETTINGS: &[Setting] = &[
    Setting::pixels("Blur", "Blur", 48.0, 28.0),
    Setting::signed_pixels("Spread", "Spread", -12.0, 12.0, 0.0),
    // Twice the others' start, since the blurred edge is half as dark.
    Setting::opacity("Opacity", "Opacity", 0.5),
];

// The Gaussian's standard deviation: half the blur, and never quite 0.
pub fn sigma(values: &[f32]) -> f32 {
    (values[0] / 2.0).max(0.25)
}

// The box, grown by the spread: what's blurred.
fn shadow_rect(values: &[f32], rect: Rect) -> Rect {
    let spread = values[1];
    Rect::new(rect.x - spread, rect.y - spread, rect.width + spread * 2.0, rect.height + spread * 2.0)
}

pub fn params(values: &[f32]) -> [f32; 8] {
    [sigma(values), values[1], values[2], 0.0, 0.0, 0.0, 0.0, 0.0]
}

// Past 4 deviations from the blurred rectangle, the shadow is less than a
// ten-thousandth of its opacity.
pub fn area(values: &[f32], rect: Rect) -> Rect {
    let shadow = shadow_rect(values, rect);
    if shadow.is_empty() {
        return rect.expand(1.0);
    }
    rect.union(shadow.expand(4.0 * sigma(values))).expand(1.0)
}

pub fn opacity(values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    let shadow = shadow_rect(values, rect);
    if shadow.is_empty() {
        return 0.0;
    }
    let scale = 1.0 / (sigma(values) * std::f32::consts::SQRT_2);
    let across = 0.5 * (erf((x - shadow.left()) * scale) - erf((x - shadow.right()) * scale));
    let down = 0.5 * (erf((y - shadow.top()) * scale) - erf((y - shadow.bottom()) * scale));
    values[2] * across * down
}

// Abramowitz and Stegun's 7.1.26, within 1.5e-7 of erf everywhere, as the
// shader has it.
pub fn erf(x: f32) -> f32 {
    let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
    let polynomial = ((((1.061_405_4 * t - 1.453_152) * t + 1.421_413_7) * t - 0.284_496_74) * t + 0.254_829_6) * t;
    let y = 1.0 - polynomial * (-x * x).exp();
    if x < 0.0 { -y } else { y }
}
