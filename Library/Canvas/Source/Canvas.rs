// Builds one frame's 2D drawing as a list of instances, one for each shape or
// glyph, which the pipeline draws with one draw call. Coordinates are pixels,
// with the origin at the window's top left.
//
// Shapes aren't triangles: each is a quad around it, and the pixel shader
// works out how much of each pixel the shape covers from its outline, so
// edges and corners are antialiased exactly at any size:
//
// - A box is a rectangle with rounded corners, filled, outlined on its
//   inside, or both, or a soft shadow, blurred by a Gaussian. Circles and
//   rings are boxes that are all corner.
// - A segment is a line with round ends.
// - A glyph is a quad of coverage from the fonts' atlas, tinted.
//
// Each instance has the clip rectangle that was current when it was added,
// outside of which it doesn't draw, for parts of a frame that scroll.

use crate::fonts::{ELLIPSIS, Fonts, Icon, TextStyle};
use crate::layout::{Align, Rect};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Color(pub u32);

impl Color {
    // Read by the GPU as R8G8B8A8_UNORM, so red is the lowest byte.
    pub const fn rgb(hex: u32) -> Self {
        let r = (hex >> 16) & 0xFF;
        let g = (hex >> 8) & 0xFF;
        let b = hex & 0xFF;
        Self(0xFF00_0000 | (b << 16) | (g << 8) | r)
    }

    pub const fn rgba(hex: u32, alpha: f32) -> Self {
        let a = (alpha * 255.0 + 0.5) as u32;
        Self((Self::rgb(hex).0 & 0x00FF_FFFF) | (a << 24))
    }

    pub const TRANSPARENT: Color = Color(0);

    fn channels(self) -> [f32; 4] {
        [0, 8, 16, 24].map(|shift| ((self.0 >> shift) & 0xFF) as f32 / 255.0)
    }

    fn from_channels(channels: [f32; 4]) -> Self {
        let byte = |index: usize, shift: u32| ((channels[index].clamp(0.0, 1.0) * 255.0).round() as u32) << shift;
        Self(byte(0, 0) | byte(1, 8) | byte(2, 16) | byte(3, 24))
    }

    pub fn to_floats(self) -> [f32; 4] {
        self.channels()
    }

    // The same color, with its opacity multiplied by `alpha`.
    pub fn alpha(self, alpha: f32) -> Self {
        let mut channels = self.channels();
        channels[3] *= alpha.clamp(0.0, 1.0);
        Self::from_channels(channels)
    }

    pub fn opacity(self) -> f32 {
        self.channels()[3]
    }

    // From self at 0 to `other` at 1, in sRGB, as the canvas blends.
    pub fn mix(self, other: Color, amount: f32) -> Self {
        let t = amount.clamp(0.0, 1.0);
        let (a, b) = (self.channels(), other.channels());
        Self::from_channels([0, 1, 2, 3].map(|index| a[index] + (b[index] - a[index]) * t))
    }

    // `self` drawn over `under`, as one color.
    pub fn over(self, under: Color) -> Self {
        let (top, bottom) = (self.channels(), under.channels());
        let alpha = top[3] + bottom[3] * (1.0 - top[3]);
        if alpha <= 0.0 {
            return Color::TRANSPARENT;
        }
        let blend = |index: usize| (top[index] * top[3] + bottom[index] * bottom[3] * (1.0 - top[3])) / alpha;
        Self::from_channels([blend(0), blend(1), blend(2), alpha])
    }
}

// What the pixel shader draws for an instance.
pub const BOX: u32 = 0;
pub const SEGMENT: u32 = 1;
pub const GLYPH: u32 = 2;

// One shape or glyph, as the GPU reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Instance {
    // A box's or a glyph's left, top, right and bottom, or a segment's ends.
    pub bounds: [f32; 4],
    // Left, top, right and bottom, outside of which nothing draws.
    pub clip: [f32; 4],
    // A box's corner radius, outline width and blur, in pixels. A glyph's
    // texels in the atlas: left, top, right and bottom. A segment's width.
    pub data: [f32; 4],
    pub fill: Color,
    // A box's outline.
    pub stroke: Color,
    pub kind: u32,
    pub padding: u32,
}

const NO_CLIP: [f32; 4] = [-1.0e6, -1.0e6, 1.0e6, 1.0e6];

pub struct Canvas<'a> {
    pub fonts: &'a Fonts,
    pub instances: &'a mut Vec<Instance>,
    clips: Vec<[f32; 4]>,
    // How many lines Lines left out, as they didn't fit in its area, so that
    // tests can check that a panel's text fits.
    pub dropped_lines: usize,
}

impl<'a> Canvas<'a> {
    // Takes the instances to fill, so that one Vec can be reused every frame.
    pub fn new(fonts: &'a Fonts, instances: &'a mut Vec<Instance>) -> Self {
        instances.clear();
        Self {
            fonts,
            instances,
            clips: Vec::new(),
            dropped_lines: 0,
        }
    }

    fn clip(&self) -> [f32; 4] {
        self.clips.last().copied().unwrap_or(NO_CLIP)
    }

    // Draws only inside `area`, and inside the clip before, until pop_clip.
    pub fn push_clip(&mut self, area: Rect) {
        let [left, top, right, bottom] = self.clip();
        self.clips.push([
            area.x.max(left),
            area.y.max(top),
            area.right().min(right),
            area.bottom().min(bottom),
        ]);
    }

    pub fn pop_clip(&mut self) {
        self.clips.pop();
    }

    // Whether anything in `area` would show inside the clip.
    pub fn visible(&self, area: Rect) -> bool {
        let [left, top, right, bottom] = self.clip();
        area.right() > left && area.x < right && area.bottom() > top && area.y < bottom
    }

    fn push(&mut self, kind: u32, bounds: [f32; 4], data: [f32; 4], fill: Color, stroke: Color) {
        let clip = self.clip();
        self.instances.push(Instance {
            bounds,
            clip,
            data,
            fill,
            stroke,
            kind,
            padding: 0,
        });
    }

    // A filled rectangle with square corners.
    pub fn rect(&mut self, area: Rect, color: Color) {
        self.fill(area, 0.0, color);
    }

    // A filled rectangle with corners of `radius`.
    pub fn fill(&mut self, area: Rect, radius: f32, color: Color) {
        if color.0 >> 24 == 0 || area.width <= 0.0 || area.height <= 0.0 {
            return;
        }
        self.push(BOX, bounds(area), [radius, 0.0, 0.0, 0.0], color, Color::TRANSPARENT);
    }

    // An outline `width` wide inside the edge of a rectangle with corners of
    // `radius`.
    pub fn stroke(&mut self, area: Rect, radius: f32, width: f32, color: Color) {
        self.fill_stroke(area, radius, Color::TRANSPARENT, width, color);
    }

    // A rectangle with corners of `radius`, filled, with an outline `width`
    // wide inside its edge.
    pub fn fill_stroke(&mut self, area: Rect, radius: f32, fill: Color, width: f32, stroke: Color) {
        if area.width <= 0.0 || area.height <= 0.0 {
            return;
        }
        self.push(BOX, bounds(area), [radius, width, 0.0, 0.0], fill, stroke);
    }

    // The soft shadow of a rectangle with corners of `radius`, blurred by a
    // Gaussian of `blur` pixels' standard deviation.
    pub fn shadow(&mut self, area: Rect, radius: f32, blur: f32, color: Color) {
        self.push(BOX, bounds(area), [radius, 0.0, blur.max(0.01), 0.0], color, Color::TRANSPARENT);
    }

    pub fn circle(&mut self, [x, y]: [f32; 2], radius: f32, color: Color) {
        let area = Rect::new(x - radius, y - radius, radius * 2.0, radius * 2.0);
        self.fill(area, radius, color);
    }

    // A ring `width` wide, centered on the circle of `radius`.
    pub fn ring(&mut self, [x, y]: [f32; 2], radius: f32, width: f32, color: Color) {
        let outer = radius + width * 0.5;
        let area = Rect::new(x - outer, y - outer, outer * 2.0, outer * 2.0);
        self.stroke(area, outer, width, color);
    }

    // A line with round ends. Lines thinner than a pixel are fainter
    // instead.
    pub fn line(&mut self, from: [f32; 2], to: [f32; 2], width: f32, color: Color) {
        let color = color.alpha(width.min(1.0));
        self.push(
            SEGMENT,
            [from[0], from[1], to[0], to[1]],
            [width.max(1.0), 0.0, 0.0, 0.0],
            color,
            Color::TRANSPARENT,
        );
    }

    // Lines through each point in turn.
    pub fn polyline(&mut self, points: &[[f32; 2]], width: f32, color: Color) {
        for pair in points.windows(2) {
            self.line(pair[0], pair[1], width, color);
        }
    }

    pub fn line_height(&self, style: TextStyle) -> f32 {
        self.fonts.line_height(style)
    }

    pub fn measure(&self, text: &str, style: TextStyle) -> f32 {
        self.fonts.measure(text, style)
    }

    // Draws text on a line whose top is at `y`, from `x`, and returns the x
    // past its end. Baselines are on whole pixels.
    pub fn text(&mut self, [x, y]: [f32; 2], text: &str, style: TextStyle, color: Color) -> f32 {
        let baseline = y.round() + self.fonts.style(style).baseline;
        self.text_on_baseline([x, baseline], text, style, color)
    }

    // Draws text whose baseline is at `y`, for text of different sizes on a
    // line.
    pub fn text_on_baseline(&mut self, [x, y]: [f32; 2], text: &str, style: TextStyle, color: Color) -> f32 {
        let fonts = self.fonts;
        let metrics = fonts.style(style);
        let baseline = y.round();
        let mut pen = x;
        for character in text.chars() {
            if character != ' ' && character != '\u{A0}' {
                let (glyph, left) = fonts.glyph(style, character, pen);
                let [u0, v0, u1, v1] = glyph.texels;
                if u1 > u0 {
                    let top = baseline + glyph.offset[1];
                    self.push(
                        GLYPH,
                        [left, top, left + (u1 - u0), top + (v1 - v0)],
                        glyph.texels,
                        color,
                        Color::TRANSPARENT,
                    );
                }
            }
            pen += metrics.advance(character);
        }
        pen
    }

    // Like text, but cuts the text short with an ellipsis if it's wider than
    // `width`.
    pub fn text_clipped(&mut self, [x, y]: [f32; 2], text: &str, width: f32, style: TextStyle, color: Color) -> f32 {
        let fitted = self.fit(text, style, width);
        self.text([x, y], &fitted, style, color)
    }

    // As much of `text` as fits in `width`, with an ellipsis if it's cut.
    pub fn fit(&self, text: &str, style: TextStyle, width: f32) -> String {
        if self.measure(text, style) <= width + 0.01 {
            return text.to_string();
        }
        let room = (width - self.measure(ELLIPSIS, style)).max(0.0);
        let end = self.fonts.fit(text, style, room);
        if room <= 0.0 {
            return String::new();
        }
        format!("{}{ELLIPSIS}", text[..end].trim_end())
    }

    // Text in `area`, centered down it, and across it as `align` says, cut
    // short if it doesn't fit. Returns the x past its end.
    pub fn text_in(&mut self, area: Rect, text: &str, style: TextStyle, color: Color, align: Align) -> f32 {
        let fitted = self.fit(text, style, area.width);
        let width = self.measure(&fitted, style);
        let x = match align {
            Align::Start => area.x,
            Align::Center => (area.x + (area.width - width) * 0.5).round(),
            Align::End => area.right() - width,
        };
        let y = area.y + ((area.height - self.line_height(style)) * 0.5).round();
        self.text([x, y], &fitted, style, color)
    }

    // An icon of `size`, one of ICON_SIZES, centered on a point.
    pub fn icon(&mut self, [x, y]: [f32; 2], icon: Icon, size: f32, color: Color) {
        let Some(glyph) = self.fonts.icon(icon, size) else {
            return;
        };
        let [u0, v0, u1, v1] = glyph.texels;
        if u1 <= u0 {
            return;
        }
        let left = x.round() + glyph.offset[0];
        let top = y.round() + glyph.offset[1];
        self.push(
            GLYPH,
            [left, top, left + (u1 - u0), top + (v1 - v0)],
            glyph.texels,
            color,
            Color::TRANSPARENT,
        );
    }
}

fn bounds(area: Rect) -> [f32; 4] {
    [area.x, area.y, area.right(), area.bottom()]
}

// The instances that a part of a frame was drawn with, to add again in later
// frames instead of drawing it again, while what it shows stays the same:
// while its key, made of everything it depends on, is the same. They have
// the fonts' atlas coordinates, so one Cached is for canvases with one Fonts.
pub struct Cached<K> {
    key: Option<K>,
    instances: Vec<Instance>,
}

impl<K> Default for Cached<K> {
    fn default() -> Self {
        Self {
            key: None,
            instances: Vec::new(),
        }
    }
}

impl<K: PartialEq> Cached<K> {
    // Adds the instances kept from the last call if it had the same key, or
    // else draws with `draw`, and keeps what it added.
    pub fn draw(&mut self, canvas: &mut Canvas, key: K, draw: impl FnOnce(&mut Canvas)) {
        if self.key.as_ref() == Some(&key) {
            canvas.instances.extend_from_slice(&self.instances);
            return;
        }
        let start = canvas.instances.len();
        draw(canvas);
        self.instances.clear();
        self.instances.extend_from_slice(&canvas.instances[start..]);
        self.key = Some(key);
    }
}
