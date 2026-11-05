// What a frame draws: shapes and text, as instances that the GPU draws in one
// call, each a quad whose pixel shader computes its shape's coverage from a
// signed distance, so that every edge is antialiased and every corner round.
//
// Programs draw their own graphics, like graphs and previews, with a Painter
// from Ui::painter, in the theme's colors.

use crate::geometry::{Color, Point, Rect, Size};
use crate::icons::Icon;
use crate::text::{Fonts, GlyphQuad, TextStyle};
use crate::theme::ThemeKind;
use crate::tokens::Elevation;

// Kinds of instance, as the shader numbers them.
const KIND_SHAPE: u32 = 0;
const KIND_GRADIENT: u32 = 1;
const KIND_SHADOW: u32 = 2;
const KIND_GLYPH: u32 = 3;
const KIND_LINE: u32 = 4;
const KIND_ARC: u32 = 5;
const KIND_POLYGON: u32 = 6;

// One quad for the GPU. Its layout matches Interface.hlsl's.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Instance {
    pub bounds: [f32; 4],
    pub data: [f32; 4],
    pub color: u32,
    pub color2: u32,
    pub color3: u32,
    // The kind, and the index of its clip rectangle in the top 16 bits.
    pub kind: u32,
    pub params: [f32; 4],
}

// Layers draw in order, so that a popup covers the cards under it, and a
// tooltip covers both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Layer {
    #[default]
    Base,
    Popup,
    Tooltip,
}

impl Layer {
    pub const ALL: [Layer; 3] = [Layer::Base, Layer::Popup, Layer::Tooltip];
}

// The radius of each corner.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Corners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Corners {
    pub const fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    pub const fn top(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: 0.0,
            bottom_left: 0.0,
        }
    }

    pub const fn bottom(radius: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: 0.0,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    pub const fn left(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: radius,
        }
    }

    pub const fn right(radius: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: radius,
            bottom_right: radius,
            bottom_left: 0.0,
        }
    }

    fn array(&self) -> [f32; 4] {
        [self.top_left, self.top_right, self.bottom_right, self.bottom_left]
    }
}

impl From<f32> for Corners {
    fn from(radius: f32) -> Self {
        Corners::all(radius)
    }
}

// A shape's outline, inside its edge. With `bottom`, its bottom few pixels
// shade into that color, the way Fluent's controls look raised.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
    pub bottom: Option<Color>,
}

impl Stroke {
    pub const fn new(color: Color, width: f32) -> Self {
        Self {
            color,
            width,
            bottom: None,
        }
    }

    // Fluent's elevation border: `color`, with `bottom` along the bottom edge.
    pub const fn elevated(color: Color, bottom: Color) -> Self {
        Self {
            color,
            width: 1.0,
            bottom: Some(bottom),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

// Everything that a frame draws.
#[derive(Default)]
pub struct DrawList {
    layers: [Vec<Instance>; 3],
    pub clips: Vec<[f32; 4]>,
    pub points: Vec<[f32; 2]>,
    clip_stack: Vec<u16>,
    layer: Layer,
    // The window's size, which is the first clip rectangle.
    size: Size,
}

impl DrawList {
    pub fn clear(&mut self, size: Size) {
        for layer in &mut self.layers {
            layer.clear();
        }
        self.clips.clear();
        self.points.clear();
        self.clip_stack.clear();
        self.size = size;
        self.clips.push([0.0, 0.0, size.width, size.height]);
        self.layer = Layer::Base;
    }

    pub fn layer(&self, layer: Layer) -> &[Instance] {
        &self.layers[layer as usize]
    }

    pub fn instance_count(&self) -> usize {
        self.layers.iter().map(Vec::len).sum()
    }

    // Every instance, in the order they draw.
    pub fn instances(&self) -> impl Iterator<Item = &Instance> {
        self.layers.iter().flatten()
    }

    pub fn current_layer(&self) -> Layer {
        self.layer
    }

    fn clip_index(&self) -> u32 {
        self.clip_stack.last().copied().unwrap_or(0) as u32
    }

    pub fn current_clip(&self) -> Rect {
        let [left, top, right, bottom] = self.clips[self.clip_index() as usize];
        Rect::new(left, top, right - left, bottom - top)
    }

    fn push(&mut self, kind: u32, mut instance: Instance) {
        instance.kind = kind | (self.clip_index() << 16);
        self.layers[self.layer as usize].push(instance);
    }
}

pub struct Painter<'a> {
    pub(crate) list: &'a mut DrawList,
    pub(crate) fonts: &'a mut Fonts,
    pub(crate) theme_kind: ThemeKind,
}

impl Painter<'_> {
    pub fn set_layer(&mut self, layer: Layer) {
        self.list.layer = layer;
    }

    pub fn layer(&self) -> Layer {
        self.list.layer
    }

    // Draws only inside the rectangle, and inside the clip it's in, until
    // pop_clip.
    pub fn push_clip(&mut self, rect: Rect) {
        let rect = rect.intersect(self.list.current_clip());
        self.list.clips.push([rect.left(), rect.top(), rect.right(), rect.bottom()]);
        let index = (self.list.clips.len() - 1).min(u16::MAX as usize) as u16;
        self.list.clip_stack.push(index);
    }

    // Like push_clip, but without regard to the clip it's in, for things
    // that float over everything, like tooltips.
    pub fn push_clip_absolute(&mut self, rect: Rect) {
        self.list.clips.push([rect.left(), rect.top(), rect.right(), rect.bottom()]);
        let index = (self.list.clips.len() - 1).min(u16::MAX as usize) as u16;
        self.list.clip_stack.push(index);
    }

    pub fn pop_clip(&mut self) {
        self.list.clip_stack.pop();
    }

    pub fn clip(&self) -> Rect {
        self.list.current_clip()
    }

    pub fn fill(&mut self, rect: Rect, color: Color) {
        self.shape(rect, Corners::default(), color, None);
    }

    pub fn rounded(&mut self, rect: Rect, radius: f32, color: Color) {
        self.shape(rect, Corners::all(radius), color, None);
    }

    // A rounded rectangle, filled, with an optional outline inside its edge.
    pub fn shape(&mut self, rect: Rect, corners: Corners, fill: Color, stroke: Option<Stroke>) {
        if rect.is_empty() || (fill.a <= 0.0 && stroke.is_none_or(|stroke| stroke.color.a <= 0.0)) {
            return;
        }
        let half = rect.width.min(rect.height) / 2.0;
        let radii = corners.array().map(|radius| radius.clamp(0.0, half));
        let (stroke_color, width, bottom) = match stroke {
            Some(stroke) => (stroke.color, stroke.width, stroke.bottom),
            None => (Color::TRANSPARENT, 0.0, None),
        };
        self.list.push(
            KIND_SHAPE,
            Instance {
                bounds: [rect.left(), rect.top(), rect.right(), rect.bottom()],
                data: radii,
                color: fill.pack_premultiplied(),
                color2: stroke_color.pack_premultiplied(),
                color3: bottom.unwrap_or(stroke_color).pack_premultiplied(),
                params: [width, if bottom.is_some() { 3.0 } else { 0.0 }, 0.0, 0.0],
                ..Default::default()
            },
        );
    }

    // A rectangle that shades from one color to another, left to right, or
    // top to bottom.
    pub fn gradient(&mut self, rect: Rect, radius: f32, from: Color, to: Color, vertical: bool) {
        if rect.is_empty() {
            return;
        }
        let half = rect.width.min(rect.height) / 2.0;
        self.list.push(
            KIND_GRADIENT,
            Instance {
                bounds: [rect.left(), rect.top(), rect.right(), rect.bottom()],
                data: [radius.min(half); 4],
                color: from.pack_premultiplied(),
                color2: to.pack_premultiplied(),
                params: [if vertical { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0],
                ..Default::default()
            },
        );
    }

    // The shadows of a surface at an elevation, outside the surface only, so
    // that they never darken a translucent surface from below.
    pub fn shadow(&mut self, rect: Rect, radius: f32, elevation: Elevation) {
        for shadow in [elevation.ambient, elevation.key] {
            let opacity = match self.theme_kind {
                ThemeKind::Light => shadow.opacity_light,
                ThemeKind::Dark => shadow.opacity_dark,
            };
            self.list.push(
                KIND_SHADOW,
                Instance {
                    bounds: [rect.left(), rect.top(), rect.right(), rect.bottom()],
                    data: [radius, (shadow.blur / 2.0).max(0.5), shadow.offset_y, 0.0],
                    color: Color::BLACK.with_alpha(opacity).pack_premultiplied(),
                    ..Default::default()
                },
            );
        }
    }

    pub fn circle(&mut self, center: Point, radius: f32, color: Color) {
        let rect = Rect::from_center(center, Size::new(radius * 2.0, radius * 2.0));
        self.shape(rect, Corners::all(radius), color, None);
    }

    // A circle's outline, `width` wide, inside the radius.
    pub fn ring(&mut self, center: Point, radius: f32, width: f32, color: Color) {
        let rect = Rect::from_center(center, Size::new(radius * 2.0, radius * 2.0));
        self.shape(rect, Corners::all(radius), Color::TRANSPARENT, Some(Stroke::new(color, width)));
    }

    // Part of a ring, centered on the radius, from `start` clockwise by
    // `sweep`, in radians from straight up, with round ends.
    pub fn arc(&mut self, center: Point, radius: f32, width: f32, start: f32, sweep: f32, color: Color) {
        if sweep.abs() <= 0.0 {
            return;
        }
        let (start, sweep) = if sweep < 0.0 { (start + sweep, -sweep) } else { (start, sweep) };
        self.list.push(
            KIND_ARC,
            Instance {
                bounds: [center.x, center.y, radius, width],
                data: [start.rem_euclid(std::f32::consts::TAU), sweep.min(std::f32::consts::TAU), 0.0, 0.0],
                color: color.pack_premultiplied(),
                ..Default::default()
            },
        );
    }

    // A line `width` wide, with round ends.
    pub fn line(&mut self, from: Point, to: Point, width: f32, color: Color) {
        self.segment(from, to, width, color, true);
    }

    // A line `width` wide, with square ends that stop at its points.
    pub fn line_flat(&mut self, from: Point, to: Point, width: f32, color: Color) {
        self.segment(from, to, width, color, false);
    }

    fn segment(&mut self, from: Point, to: Point, width: f32, color: Color, round: bool) {
        self.list.push(
            KIND_LINE,
            Instance {
                bounds: [from.x, from.y, to.x, to.y],
                data: [width, 0.0, 0.0, 0.0],
                color: color.pack_premultiplied(),
                params: [if round { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0],
                ..Default::default()
            },
        );
    }

    pub fn polyline(&mut self, points: &[Point], width: f32, color: Color) {
        for pair in points.windows(2) {
            self.line(pair[0], pair[1], width, color);
        }
    }

    // A convex polygon, like a hexagon or a triangle, its corners in either
    // order around it.
    pub fn polygon(&mut self, points: &[Point], color: Color) {
        if points.len() < 3 {
            return;
        }
        let first = self.list.points.len();
        // The shader expects the corners clockwise on screen, where y is down.
        let area: f32 = (0..points.len())
            .map(|index| {
                let (a, b) = (points[index], points[(index + 1) % points.len()]);
                a.x * b.y - b.x * a.y
            })
            .sum();
        if area >= 0.0 {
            self.list.points.extend(points.iter().map(|point| [point.x, point.y]));
        } else {
            self.list.points.extend(points.iter().rev().map(|point| [point.x, point.y]));
        }
        let mut bounds = Rect::from_corners(points[0], points[0]);
        for point in points {
            bounds = bounds.union(Rect::from_corners(*point, *point));
        }
        self.list.push(
            KIND_POLYGON,
            Instance {
                bounds: [bounds.left(), bounds.top(), bounds.right(), bounds.bottom()],
                data: [first as f32, points.len() as f32, 0.0, 0.0],
                color: color.pack_premultiplied(),
                ..Default::default()
            },
        );
    }

    pub fn measure(&mut self, text: &str, style: TextStyle) -> Size {
        self.fonts.measure(text, style)
    }

    // The height of the text wrapped at a width.
    pub fn measure_wrapped(&mut self, text: &str, style: TextStyle, width: f32) -> Size {
        self.fonts.layout(text, style, Some(width)).size()
    }

    pub fn fonts(&mut self) -> &mut Fonts {
        self.fonts
    }

    // One line of text, with its line box's top left at `origin`. Returns
    // its size.
    pub fn text(&mut self, origin: Point, text: &str, style: TextStyle, color: Color) -> Size {
        self.glyphs(origin, text, style, None, color)
    }

    // Text wrapped to the rectangle's width, from its top left.
    pub fn text_wrapped(&mut self, rect: Rect, text: &str, style: TextStyle, color: Color) -> Size {
        self.glyphs(rect.origin(), text, style, Some(rect.width), color)
    }

    // One line of text, centered vertically in the rectangle and aligned
    // horizontally. Returns where its line box went.
    pub fn text_in(&mut self, rect: Rect, text: &str, style: TextStyle, color: Color, align: Align) -> Rect {
        let size = self.measure(text, style);
        let x = match align {
            Align::Start => rect.x,
            Align::Center => rect.x + (rect.width - size.width) / 2.0,
            Align::End => rect.right() - size.width,
        };
        let y = (rect.y + (rect.height - size.height) / 2.0).round();
        self.glyphs(Point::new(x, y), text, style, None, color);
        Rect::new(x, y, size.width, size.height)
    }

    // Like text_in, but text too wide for the rectangle ends with an
    // ellipsis where it's cut.
    pub fn text_fit(&mut self, rect: Rect, text: &str, style: TextStyle, color: Color, align: Align) -> Rect {
        if self.measure(text, style).width <= rect.width {
            return self.text_in(rect, text, style, color, align);
        }
        let fitted = self.fit(text, style, rect.width);
        self.text_in(rect, &fitted, style, color, align)
    }

    // The text, cut short with an ellipsis if it's wider than `width`.
    pub fn fit(&mut self, text: &str, style: TextStyle, width: f32) -> String {
        if self.measure(text, style).width <= width {
            return text.to_string();
        }
        let boundaries: Vec<usize> = text.char_indices().map(|(index, _)| index).collect();
        let (mut low, mut high) = (0usize, boundaries.len());
        while low < high {
            let middle = (low + high).div_ceil(2);
            let candidate = format!("{}…", text[..boundaries[middle.min(boundaries.len() - 1)]].trim_end());
            if middle < boundaries.len() && self.measure(&candidate, style).width <= width {
                low = middle;
            } else {
                high = middle - 1;
            }
        }
        format!("{}…", text[..boundaries.get(low).copied().unwrap_or(0)].trim_end())
    }

    fn glyphs(&mut self, origin: Point, text: &str, style: TextStyle, wrap: Option<f32>, color: Color) -> Size {
        if text.is_empty() {
            return Size::new(0.0, style.line_height);
        }
        let color = color.pack_premultiplied();
        let list = &mut *self.list;
        let mut emit = |quad: GlyphQuad| {
            if quad.width <= 0.0 {
                return;
            }
            list.push(
                KIND_GLYPH,
                Instance {
                    bounds: [quad.x, quad.y, quad.x + quad.width, quad.y + quad.height],
                    data: [quad.atlas_x, quad.atlas_y, 0.0, 0.0],
                    color,
                    ..Default::default()
                },
            );
        };
        self.fonts.quads(text, style, wrap, origin, &mut emit)
    }

    // An icon of `size` pixels, centered on the point.
    pub fn icon(&mut self, center: Point, icon: Icon, size: f32, color: Color) {
        let mut buffer = [0u8; 4];
        let text = icon.glyph().encode_utf8(&mut buffer);
        let style = TextStyle::icons(size);
        let measured = self.measure(text, style);
        let origin = Point::new((center.x - measured.width / 2.0).round(), (center.y - size / 2.0).round());
        self.glyphs(origin, text, style, None, color);
    }

    // A rounded rectangle's outline, outside it, like the ring that shows
    // keyboard focus.
    pub fn outline(&mut self, rect: Rect, radius: f32, width: f32, color: Color) {
        let outer = rect.expand(width);
        self.shape(outer, Corners::all(radius + width), Color::TRANSPARENT, Some(Stroke::new(color, width)));
    }
}
