// Points, sizes, rectangles and colors, in the client area's pixels, which
// are logical and physical pixels alike at the one supported scale.

use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(self, other: Point) -> f32 {
        (self - other).length()
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn lerp(self, other: Point, t: f32) -> Point {
        Point::new(self.x + (other.x - self.x) * t, self.y + (other.y - self.y) * t)
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, scale: f32) -> Point {
        Point::new(self.x * scale, self.y * scale)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

// A rectangle from its top left corner, which is inside it, to its bottom
// right, which isn't.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_corners(a: Point, b: Point) -> Self {
        let (left, right) = (a.x.min(b.x), a.x.max(b.x));
        let (top, bottom) = (a.y.min(b.y), a.y.max(b.y));
        Self::new(left, top, right - left, bottom - top)
    }

    pub fn from_center(center: Point, size: Size) -> Self {
        Self::new(center.x - size.width / 2.0, center.y - size.height / 2.0, size.width, size.height)
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn origin(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.x < self.right() && point.y >= self.y && point.y < self.bottom()
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn intersect(&self, other: Rect) -> Rect {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        Rect::new(left, top, (right - left).max(0.0), (bottom - top).max(0.0))
    }

    pub fn union(&self, other: Rect) -> Rect {
        let left = self.x.min(other.x);
        let top = self.y.min(other.y);
        Rect::from_corners(
            Point::new(left, top),
            Point::new(self.right().max(other.right()), self.bottom().max(other.bottom())),
        )
    }

    // Smaller by `amount` on every side, or larger for a negative amount.
    pub fn inset(&self, amount: f32) -> Rect {
        self.inset_by(amount, amount, amount, amount)
    }

    pub fn inset_by(&self, left: f32, top: f32, right: f32, bottom: f32) -> Rect {
        Rect::new(
            self.x + left,
            self.y + top,
            (self.width - left - right).max(0.0),
            (self.height - top - bottom).max(0.0),
        )
    }

    pub fn expand(&self, amount: f32) -> Rect {
        Rect::new(self.x - amount, self.y - amount, self.width + amount * 2.0, self.height + amount * 2.0)
    }

    pub fn offset(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    // A rectangle of the size, centered in this one.
    pub fn centered(&self, size: Size) -> Rect {
        Rect::from_center(self.center(), size)
    }

    // The strip of the height along the top, and likewise for the others.
    pub fn top_strip(&self, height: f32) -> Rect {
        Rect::new(self.x, self.y, self.width, height.min(self.height))
    }

    pub fn bottom_strip(&self, height: f32) -> Rect {
        let height = height.min(self.height);
        Rect::new(self.x, self.bottom() - height, self.width, height)
    }

    pub fn left_strip(&self, width: f32) -> Rect {
        Rect::new(self.x, self.y, width.min(self.width), self.height)
    }

    pub fn right_strip(&self, width: f32) -> Rect {
        let width = width.min(self.width);
        Rect::new(self.right() - width, self.y, width, self.height)
    }

    // Cuts the strip of the height off the top, and the gap after it, and
    // returns the strip. Like laying out a column from the top down.
    pub fn cut_top(&mut self, height: f32, gap: f32) -> Rect {
        let strip = self.top_strip(height);
        let taken = (height + gap).min(self.height);
        self.y += taken;
        self.height -= taken;
        strip
    }

    pub fn cut_bottom(&mut self, height: f32, gap: f32) -> Rect {
        let strip = self.bottom_strip(height);
        self.height = (self.height - height - gap).max(0.0);
        strip
    }

    pub fn cut_left(&mut self, width: f32, gap: f32) -> Rect {
        let strip = self.left_strip(width);
        let taken = (width + gap).min(self.width);
        self.x += taken;
        self.width -= taken;
        strip
    }

    pub fn cut_right(&mut self, width: f32, gap: f32) -> Rect {
        let strip = self.right_strip(width);
        self.width = (self.width - width - gap).max(0.0);
        strip
    }

    // N columns of equal width, with gaps between them.
    pub fn columns<const N: usize>(&self, gap: f32) -> [Rect; N] {
        let width = (self.width - gap * (N as f32 - 1.0)) / N as f32;
        std::array::from_fn(|index| Rect::new(self.x + index as f32 * (width + gap), self.y, width, self.height))
    }

    // N rows of equal height, with gaps between them.
    pub fn rows<const N: usize>(&self, gap: f32) -> [Rect; N] {
        let height = (self.height - gap * (N as f32 - 1.0)) / N as f32;
        std::array::from_fn(|index| Rect::new(self.x, self.y + index as f32 * (height + gap), self.width, height))
    }

    // Snaps the edges to whole pixels.
    pub fn round(&self) -> Rect {
        let left = self.x.round();
        let top = self.y.round();
        Rect::new(left, top, self.right().round() - left, self.bottom().round() - top)
    }
}

// A color in sRGB, with straight alpha, each from 0 to 1. Fluent's colors are
// sRGB, and the interface blends in sRGB, like Windows' own controls.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Color = Color::rgba(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    // From 0xRRGGBB, opaque.
    pub const fn hex(rgb: u32) -> Self {
        Self::rgba(
            ((rgb >> 16) & 0xFF) as f32 / 255.0,
            ((rgb >> 8) & 0xFF) as f32 / 255.0,
            (rgb & 0xFF) as f32 / 255.0,
            1.0,
        )
    }

    // From 0xAARRGGBB, the way Fluent's tokens write colors.
    pub const fn argb(argb: u32) -> Self {
        let rgb = Self::hex(argb & 0xFF_FFFF);
        Self::rgba(rgb.r, rgb.g, rgb.b, ((argb >> 24) & 0xFF) as f32 / 255.0)
    }

    // From "#RRGGBB" or "#AARRGGBB", without regard to case.
    pub fn parse(text: &str) -> Option<Color> {
        let digits = text.strip_prefix('#')?;
        if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let value = u32::from_str_radix(digits, 16).ok()?;
        match digits.len() {
            6 => Some(Color::hex(value)),
            8 => Some(Color::argb(value)),
            _ => None,
        }
    }

    // As "#RRGGBB", which leaves out alpha.
    pub fn to_hex(&self) -> String {
        let [r, g, b, _] = self.to_bytes();
        format!("#{r:02X}{g:02X}{b:02X}")
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        [byte(self.r), byte(self.g), byte(self.b), byte(self.a)]
    }

    // As the GPU reads it: premultiplied by alpha, in a u32 of RGBA bytes,
    // with red in the lowest byte.
    pub fn pack_premultiplied(&self) -> u32 {
        let a = self.a.clamp(0.0, 1.0);
        let byte = |value: f32| (value.clamp(0.0, 1.0) * a * 255.0).round() as u32;
        byte(self.r) | (byte(self.g) << 8) | (byte(self.b) << 16) | (((a * 255.0).round() as u32) << 24)
    }

    pub fn with_alpha(&self, a: f32) -> Color {
        Color::rgba(self.r, self.g, self.b, a)
    }

    // With its alpha scaled, like for fading it out.
    pub fn fade(&self, opacity: f32) -> Color {
        Color::rgba(self.r, self.g, self.b, self.a * opacity)
    }

    pub fn lerp(&self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::rgba(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }

    // This color drawn over another, as the result looks.
    pub fn over(&self, below: Color) -> Color {
        let a = self.a + below.a * (1.0 - self.a);
        if a <= 0.0 {
            return Color::TRANSPARENT;
        }
        let mix = |top: f32, bottom: f32| (top * self.a + bottom * below.a * (1.0 - self.a)) / a;
        Color::rgba(mix(self.r, below.r), mix(self.g, below.g), mix(self.b, below.b), a)
    }

    // From a linear color, like SimplePaint's, encoded to sRGB.
    pub fn from_linear(r: f32, g: f32, b: f32) -> Color {
        Color::rgba(encode_srgb(r), encode_srgb(g), encode_srgb(b), 1.0)
    }

    pub fn to_linear(&self) -> [f32; 3] {
        [decode_srgb(self.r), decode_srgb(self.g), decode_srgb(self.b)]
    }

    // Relative luminance, from 0 for black to 1 for white.
    pub fn luminance(&self) -> f32 {
        let [r, g, b] = self.to_linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}

pub fn encode_srgb(linear: f32) -> f32 {
    let linear = linear.clamp(0.0, 1.0);
    if linear <= 0.003_130_8 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

pub fn decode_srgb(encoded: f32) -> f32 {
    let encoded = encoded.clamp(0.0, 1.0);
    if encoded <= 0.040_45 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}
