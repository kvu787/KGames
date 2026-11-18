// Points, sizes, rectangles, insets and colors, in the window's pixels, with
// +y down. Cards are laid out in fixed pixels by cutting rectangles apart:
//
//     let mut area = card.content;
//     let first = area.cut_top(32.0, SPACE_S);
//     let [left, right] = area.cut_top(32.0, 0.0).columns(SPACE_S);

use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Point = Point::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }

    pub fn distance(self, other: Point) -> f32 {
        (self - other).length()
    }

    pub fn lerp(self, other: Point, t: f32) -> Point {
        self + (other - self) * t
    }

    pub fn round(self) -> Point {
        Point::new(self.x.round(), self.y.round())
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Point) {
        *self = *self + other;
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, other: Point) {
        *self = *self - other;
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, factor: f32) -> Point {
        Point::new(self.x * factor, self.y * factor)
    }
}

impl Neg for Point {
    type Output = Point;
    fn neg(self) -> Point {
        Point::new(-self.x, -self.y)
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

    pub const fn square(side: f32) -> Self {
        Self::new(side, side)
    }
}

// Space on each side, like WinUI's Thickness: a padding, a margin, or a
// border's width on each side.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Insets {
    pub const ZERO: Insets = Insets::all(0.0);

    pub const fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self { left, top, right, bottom }
    }

    pub const fn all(amount: f32) -> Self {
        Self::new(amount, amount, amount, amount)
    }

    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self::new(horizontal, vertical, horizontal, vertical)
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    pub fn is_zero(&self) -> bool {
        *self == Insets::ZERO
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const ZERO: Rect = Rect::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_origin(origin: Point, size: Size) -> Self {
        Self::new(origin.x, origin.y, size.width, size.height)
    }

    // The rectangle between two corners, in either order.
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

    pub fn origin(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn top_left(&self) -> Point {
        self.origin()
    }

    pub fn bottom_right(&self) -> Point {
        Point::new(self.right(), self.bottom())
    }

    // Whether the point is in it, counting its left and top edges but not
    // its right and bottom ones, so that rectangles side by side never both
    // have a point.
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.y >= self.y && point.x < self.right() && point.y < self.bottom()
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn intersects(&self, other: Rect) -> bool {
        !self.intersect(other).is_empty()
    }

    // The part that's in both, which is empty if they don't overlap.
    pub fn intersect(&self, other: Rect) -> Rect {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        Rect::new(left, top, (right - left).max(0.0), (bottom - top).max(0.0))
    }

    // The smallest rectangle around both.
    pub fn union(&self, other: Rect) -> Rect {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return *self;
        }
        let left = self.x.min(other.x);
        let top = self.y.min(other.y);
        Rect::new(left, top, self.right().max(other.right()) - left, self.bottom().max(other.bottom()) - top)
    }

    pub fn inset(&self, amount: f32) -> Rect {
        self.pad(Insets::all(amount))
    }

    // Smaller by the insets, like a control's content inside its padding.
    pub fn pad(&self, insets: Insets) -> Rect {
        Rect::new(
            self.x + insets.left,
            self.y + insets.top,
            (self.width - insets.horizontal()).max(0.0),
            (self.height - insets.vertical()).max(0.0),
        )
    }

    pub fn expand(&self, amount: f32) -> Rect {
        Rect::new(self.x - amount, self.y - amount, self.width + amount * 2.0, self.height + amount * 2.0)
    }

    pub fn offset(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    pub fn translate(&self, by: Point) -> Rect {
        self.offset(by.x, by.y)
    }

    // A rectangle of the size, centered in this one.
    pub fn centered(&self, size: Size) -> Rect {
        Rect::from_center(self.center(), size)
    }

    // Strips along an edge, of the full width or height.
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

    // Cuts a strip off an edge and returns it, leaving the rest, less the
    // gap after the strip. This is how cards lay out their controls.
    pub fn cut_top(&mut self, height: f32, gap: f32) -> Rect {
        let strip = self.top_strip(height);
        let taken = (height + gap).min(self.height);
        self.y += taken;
        self.height -= taken;
        strip
    }

    pub fn cut_bottom(&mut self, height: f32, gap: f32) -> Rect {
        let strip = self.bottom_strip(height);
        self.height -= (height + gap).min(self.height);
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
        self.width -= (width + gap).min(self.width);
        strip
    }

    // N columns of equal width, with gaps between them, in whole pixels.
    pub fn columns<const N: usize>(&self, gap: f32) -> [Rect; N] {
        let widths = self.split_widths(N, gap);
        let mut x = self.x;
        std::array::from_fn(|index| {
            let column = Rect::new(x, self.y, widths[index], self.height);
            x += widths[index] + gap;
            column
        })
    }

    // N rows of equal height, with gaps between them, in whole pixels.
    pub fn rows<const N: usize>(&self, gap: f32) -> [Rect; N] {
        let flipped = Rect::new(self.y, self.x, self.height, self.width);
        flipped.columns::<N>(gap).map(|row| Rect::new(row.y, row.x, row.height, row.width))
    }

    // Columns of the widths that the weights give, out of the width less the
    // gaps, in whole pixels.
    pub fn columns_weighted(&self, weights: &[f32], gap: f32) -> Vec<Rect> {
        let total: f32 = weights.iter().sum();
        let available = self.width - gap * weights.len().saturating_sub(1) as f32;
        let mut x = self.x;
        let mut consumed = 0.0;
        weights
            .iter()
            .enumerate()
            .map(|(index, weight)| {
                consumed += weight;
                let end = if index + 1 == weights.len() {
                    self.right()
                } else {
                    (self.x + available * consumed / total + gap * index as f32).round()
                };
                let column = Rect::new(x, self.y, end - x, self.height);
                x = end + gap;
                column
            })
            .collect()
    }

    fn split_widths(&self, count: usize, gap: f32) -> Vec<f32> {
        let available = (self.width - gap * count.saturating_sub(1) as f32).max(0.0);
        let mut widths = Vec::with_capacity(count);
        let mut used = 0.0;
        for index in 0..count {
            let end = (available * (index + 1) as f32 / count as f32).round();
            widths.push(end - used);
            used = end;
        }
        widths
    }

    // On whole pixels, which keeps 1-pixel strokes sharp.
    pub fn round(&self) -> Rect {
        let left = self.x.round();
        let top = self.y.round();
        Rect::new(left, top, (self.right()).round() - left, (self.bottom()).round() - top)
    }

    pub fn lerp(&self, other: Rect, t: f32) -> Rect {
        Rect::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.width + (other.width - self.width) * t,
            self.height + (other.height - self.height) * t,
        )
    }

    // Where the point is in it, as fractions of its width and height.
    pub fn fraction_of(&self, point: Point) -> Point {
        Point::new(
            if self.width > 0.0 { (point.x - self.x) / self.width } else { 0.0 },
            if self.height > 0.0 { (point.y - self.y) / self.height } else { 0.0 },
        )
    }

    // The point at fractions of its width and height.
    pub fn at_fraction(&self, fraction: Point) -> Point {
        Point::new(self.x + self.width * fraction.x, self.y + self.height * fraction.y)
    }
}

// A color in sRGB, from 0 to 1, with straight alpha. The theme's colors and
// every control's are sRGB, and the interface blends in sRGB, as Windows does.
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

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    // From 0xRRGGBB, opaque.
    pub const fn hex(rgb: u32) -> Self {
        Self::argb(0xFF00_0000 | rgb)
    }

    // From 0xAARRGGBB, the way XAML writes colors, like #E4000000.
    pub const fn argb(argb: u32) -> Self {
        Self::rgba(
            ((argb >> 16) & 0xFF) as f32 / 255.0,
            ((argb >> 8) & 0xFF) as f32 / 255.0,
            (argb & 0xFF) as f32 / 255.0,
            ((argb >> 24) & 0xFF) as f32 / 255.0,
        )
    }

    // From bytes, red, green, blue and alpha.
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self::rgba(bytes[0] as f32 / 255.0, bytes[1] as f32 / 255.0, bytes[2] as f32 / 255.0, bytes[3] as f32 / 255.0)
    }

    // From "#RRGGBB" or "#AARRGGBB", with or without the #.
    pub fn parse(text: &str) -> Option<Color> {
        let digits = text.trim().trim_start_matches('#');
        if !digits.chars().all(|character| character.is_ascii_hexdigit()) {
            return None;
        }
        let value = u32::from_str_radix(digits, 16).ok()?;
        match digits.len() {
            6 => Some(Color::hex(value)),
            8 => Some(Color::argb(value)),
            _ => None,
        }
    }

    // Like "#1E90FF", or "#801E90FF" if it isn't opaque.
    pub fn to_hex(&self) -> String {
        let [r, g, b, a] = self.to_bytes();
        if a == 255 { format!("#{r:02X}{g:02X}{b:02X}") } else { format!("#{a:02X}{r:02X}{g:02X}{b:02X}") }
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        [byte(self.r), byte(self.g), byte(self.b), byte(self.a)]
    }

    // Premultiplied, as the shader takes it: red in the low byte.
    pub fn pack_premultiplied(&self) -> u32 {
        let a = self.a.clamp(0.0, 1.0);
        let byte = |value: f32| (value.clamp(0.0, 1.0) * a * 255.0).round() as u32;
        byte(self.r) | (byte(self.g) << 8) | (byte(self.b) << 16) | (((a * 255.0).round() as u32) << 24)
    }

    pub fn with_alpha(&self, a: f32) -> Color {
        Color::rgba(self.r, self.g, self.b, a)
    }

    // More transparent, by the opacity, like WinUI's Opacity on an element.
    pub fn fade(&self, opacity: f32) -> Color {
        self.with_alpha(self.a * opacity)
    }

    // Between this color and the other, with straight alpha.
    pub fn lerp(&self, other: Color, t: f32) -> Color {
        let mix = |a: f32, b: f32| a + (b - a) * t;
        Color::rgba(mix(self.r, other.r), mix(self.g, other.g), mix(self.b, other.b), mix(self.a, other.a))
    }

    // This color drawn over another, as it looks.
    pub fn over(&self, below: Color) -> Color {
        let a = self.a + below.a * (1.0 - self.a);
        if a <= 0.0 {
            return Color::TRANSPARENT;
        }
        let mix = |top: f32, bottom: f32| (top * self.a + bottom * below.a * (1.0 - self.a)) / a;
        Color::rgba(mix(self.r, below.r), mix(self.g, below.g), mix(self.b, below.b), a)
    }

    // From linear light, like a shader's output.
    pub fn from_linear(r: f32, g: f32, b: f32) -> Color {
        Color::rgb(encode_srgb(r), encode_srgb(g), encode_srgb(b))
    }

    pub fn to_linear(&self) -> [f32; 3] {
        [decode_srgb(self.r), decode_srgb(self.g), decode_srgb(self.b)]
    }

    // Relative luminance, as WCAG defines it.
    pub fn luminance(&self) -> f32 {
        let [r, g, b] = self.to_linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    // Hue in degrees from 0 to 360, saturation and value from 0 to 1.
    pub fn to_hsv(&self) -> [f32; 3] {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;
        let hue = if delta <= 0.0 {
            0.0
        } else if max == self.r {
            60.0 * ((self.g - self.b) / delta).rem_euclid(6.0)
        } else if max == self.g {
            60.0 * ((self.b - self.r) / delta + 2.0)
        } else {
            60.0 * ((self.r - self.g) / delta + 4.0)
        };
        let saturation = if max <= 0.0 { 0.0 } else { delta / max };
        [hue, saturation, max]
    }

    pub fn from_hsv(hue: f32, saturation: f32, value: f32) -> Color {
        let hue = hue.rem_euclid(360.0) / 60.0;
        let chroma = value * saturation;
        let x = chroma * (1.0 - (hue % 2.0 - 1.0).abs());
        let (r, g, b) = match hue as u32 {
            0 => (chroma, x, 0.0),
            1 => (x, chroma, 0.0),
            2 => (0.0, chroma, x),
            3 => (0.0, x, chroma),
            4 => (x, 0.0, chroma),
            _ => (chroma, 0.0, x),
        };
        let m = value - chroma;
        Color::rgb(r + m, g + m, b + m)
    }
}

pub fn encode_srgb(linear: f32) -> f32 {
    let linear = linear.clamp(0.0, 1.0);
    if linear <= 0.003_130_8 { linear * 12.92 } else { 1.055 * linear.powf(1.0 / 2.4) - 0.055 }
}

pub fn decode_srgb(encoded: f32) -> f32 {
    let encoded = encoded.clamp(0.0, 1.0);
    if encoded <= 0.040_45 { encoded / 12.92 } else { ((encoded + 0.055) / 1.055).powf(2.4) }
}
