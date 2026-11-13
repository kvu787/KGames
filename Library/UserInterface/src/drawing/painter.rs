// What a frame draws: shapes and text, as instances of one quad that the GPU
// draws in one call. Each instance's pixel shader computes its shape's
// coverage from a signed distance, so every edge is antialiased and every
// corner round, at any size.
//
// Widgets draw with a Painter, and so do programs, for their own graphics
// like graphs and previews, in the theme's colors. Brushes are WinUI's:
// solid colors, and linear gradients, which is how WinUI draws the elevation
// borders that make controls look raised.

use std::borrow::Cow;
use crate::design::icons::Icon;
use crate::design::theme::{Theme, ThemeKind};
use crate::design::tokens::Elevation;
use crate::drawing::geometry::{Color, Insets, Point, Rect, Size};
use crate::drawing::text::{Fonts, GlyphQuad, TextStyle};

// The kinds of instance, as the shader numbers them.
const KIND_SHAPE: u32 = 0;
const KIND_SHADOW: u32 = 1;
const KIND_GLYPH: u32 = 2;
const KIND_LINE: u32 = 3;
const KIND_ARC: u32 = 4;
const KIND_POLYGON: u32 = 5;

// One quad for the GPU, laid out as Interface.hlsl's Instance.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Instance {
    // A shape's left, top, right and bottom, or a line's ends, or an arc's
    // center, radius and width.
    pub bounds: [f32; 4],
    // Corner radii, top left first, clockwise; or what else the kind needs.
    pub radii: [f32; 4],
    // The border's width on each side: left, top, right, bottom.
    pub border: [f32; 4],
    // The fill's gradient, from (x, y) to (z, w), in the window's pixels.
    pub fill_line: [f32; 4],
    // The border's gradient, from (x, y) to (z, w).
    pub border_line: [f32; 4],
    // Premultiplied sRGB: the fill from and to, the border from and to.
    pub colors: [u32; 4],
    // The kind, and the index of its clip in the top 16 bits.
    pub kind: u32,
    pub padding: [u32; 3],
}

// Layers draw in order: a dialog covers the page, a popup like a combo box's
// list covers both, and a tooltip covers everything.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Layer {
    #[default]
    Base,
    Dialog,
    Popup,
    Tooltip,
}

impl Layer {
    pub const ALL: [Layer; 4] = [Layer::Base, Layer::Dialog, Layer::Popup, Layer::Tooltip];
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
    pub const ZERO: Corners = Corners::all(0.0);

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

// What fills a shape or its border: a color, or WinUI's LinearGradientBrush
// with two stops, between two points in the window. Past the points, the
// end colors go on, as XAML's do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Brush {
    Solid(Color),
    Linear { from: Point, to: Point, start: Color, end: Color },
}

impl Brush {
    // From one color at the top of the rectangle to another at its bottom.
    pub fn vertical(rect: Rect, top: Color, bottom: Color) -> Brush {
        Brush::Linear {
            from: Point::new(rect.x, rect.y),
            to: Point::new(rect.x, rect.bottom()),
            start: top,
            end: bottom,
        }
    }

    // From one color at the left of the rectangle to another at its right.
    pub fn horizontal(rect: Rect, left: Color, right: Color) -> Brush {
        Brush::Linear {
            from: Point::new(rect.x, rect.y),
            to: Point::new(rect.right(), rect.y),
            start: left,
            end: right,
        }
    }

    fn is_visible(&self) -> bool {
        match self {
            Brush::Solid(color) => color.a > 0.0,
            Brush::Linear { start, end, .. } => start.a > 0.0 || end.a > 0.0,
        }
    }

    // The colors and the line, as the shader takes them.
    fn encode(&self) -> ([u32; 2], [f32; 4]) {
        match self {
            Brush::Solid(color) => {
                let packed = color.pack_premultiplied();
                ([packed, packed], [0.0, 0.0, 0.0, 1.0])
            }
            Brush::Linear { from, to, start, end } => {
                ([start.pack_premultiplied(), end.pack_premultiplied()], [from.x, from.y, to.x, to.y])
            }
        }
    }
}

impl From<Color> for Brush {
    fn from(color: Color) -> Self {
        Brush::Solid(color)
    }
}

// A shape's border, inside its edge: a width on each side, like a XAML
// Border's BorderThickness, and its brush.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Border {
    pub widths: Insets,
    pub brush: Brush,
}

impl Border {
    pub fn new(brush: impl Into<Brush>, width: f32) -> Self {
        Self {
            widths: Insets::all(width),
            brush: brush.into(),
        }
    }

    pub fn sides(brush: impl Into<Brush>, widths: Insets) -> Self {
        Self { widths, brush: brush.into() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

// A run of text in its own style and color, for a line of several.
#[derive(Clone, Copy, Debug)]
pub struct Span<'a> {
    pub text: &'a str,
    pub style: TextStyle,
    pub color: Color,
}

impl<'a> Span<'a> {
    pub fn new(text: &'a str, style: TextStyle, color: Color) -> Self {
        Self { text, style, color }
    }
}

// Everything a frame draws, layer by layer.
#[derive(Default)]
pub struct DrawList {
    layers: [Vec<Instance>; 4],
    pub(crate) clips: Vec<[f32; 4]>,
    pub(crate) points: Vec<[f32; 2]>,
    clip_stack: Vec<u16>,
    layer: Layer,
}

impl DrawList {
    // Starts over for a frame of the size, whose bounds are the first clip.
    pub fn clear(&mut self, size: Size) {
        for layer in &mut self.layers {
            layer.clear();
        }
        self.clips.clear();
        self.points.clear();
        self.clip_stack.clear();
        self.clips.push([0.0, 0.0, size.width, size.height]);
        self.layer = Layer::Base;
    }

    pub fn layer_instances(&self, layer: Layer) -> &[Instance] {
        &self.layers[layer as usize]
    }

    pub fn instance_count(&self) -> usize {
        self.layers.iter().map(Vec::len).sum()
    }

    pub fn layer(&self) -> Layer {
        self.layer
    }

    // How many instances the current layer has, to mark a place in it.
    pub fn layer_len(&self) -> usize {
        self.layers[self.layer as usize].len()
    }

    // Moves the current layer's instances from `middle` on to before those
    // from `first`, so that what was drawn last draws under what was drawn
    // since `first`, like a card that's drawn after what's in it.
    pub fn move_to_back(&mut self, first: usize, middle: usize) {
        let layer = &mut self.layers[self.layer as usize];
        let moved = layer.len() - middle;
        layer[first..].rotate_right(moved);
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.layer = layer;
    }

    fn clip_index(&self) -> u32 {
        self.clip_stack.last().copied().unwrap_or(0) as u32
    }

    pub fn clip(&self) -> Rect {
        let [left, top, right, bottom] = self.clips[self.clip_index() as usize];
        Rect::new(left, top, right - left, bottom - top)
    }

    // Draws only inside the rectangle, and inside the clip it's in, until
    // pop_clip. With `absolute`, the clip it's in doesn't matter, for what
    // floats over everything, like a popup from a scrolled list.
    pub fn push_clip(&mut self, rect: Rect, absolute: bool) {
        let rect = if absolute { rect } else { rect.intersect(self.clip()) };
        self.clips.push([rect.left(), rect.top(), rect.right(), rect.bottom()]);
        let index = (self.clips.len() - 1).min(u16::MAX as usize) as u16;
        self.clip_stack.push(index);
    }

    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    fn push(&mut self, kind: u32, mut instance: Instance) {
        instance.kind = kind | (self.clip_index() << 16);
        self.layers[self.layer as usize].push(instance);
    }
}

pub struct Painter<'a> {
    pub(crate) list: &'a mut DrawList,
    pub(crate) fonts: &'a mut Fonts,
    pub(crate) theme: &'a Theme,
}

impl<'a> Painter<'a> {
    // The theme, for as long as the painter's borrow of the interface, so
    // that it can be read while the painter draws.
    pub fn theme(&self) -> &'a Theme {
        self.theme
    }

    pub fn layer(&self) -> Layer {
        self.list.layer()
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.list.set_layer(layer);
    }

    pub fn clip(&self) -> Rect {
        self.list.clip()
    }

    pub fn push_clip(&mut self, rect: Rect) {
        self.list.push_clip(rect, false);
    }

    pub fn pop_clip(&mut self) {
        self.list.pop_clip();
    }

    pub fn fill(&mut self, rect: Rect, color: Color) {
        self.shape(rect, Corners::ZERO, color, None);
    }

    pub fn rounded(&mut self, rect: Rect, corners: impl Into<Corners>, color: Color) {
        self.shape(rect, corners, color, None);
    }

    // A rounded rectangle with a fill and maybe a border inside its edge,
    // like a XAML Border.
    pub fn shape(&mut self, rect: Rect, corners: impl Into<Corners>, fill: impl Into<Brush>, border: Option<Border>) {
        let fill = fill.into();
        let border = border.filter(|border| border.brush.is_visible() && !border.widths.is_zero());
        if rect.is_empty() || (!fill.is_visible() && border.is_none()) {
            return;
        }
        let half = rect.width.min(rect.height) / 2.0;
        let radii = corners.into().array().map(|radius| radius.clamp(0.0, half));
        let (fill_colors, fill_line) = fill.encode();
        let (border_colors, border_line, widths) = match border {
            Some(border) => {
                let (colors, line) = border.brush.encode();
                let widths = border.widths;
                (colors, line, [widths.left, widths.top, widths.right, widths.bottom])
            }
            None => ([0, 0], [0.0, 0.0, 0.0, 1.0], [0.0; 4]),
        };
        self.list.push(
            KIND_SHAPE,
            Instance {
                bounds: [rect.left(), rect.top(), rect.right(), rect.bottom()],
                radii,
                border: widths,
                fill_line,
                border_line,
                colors: [fill_colors[0], fill_colors[1], border_colors[0], border_colors[1]],
                ..Default::default()
            },
        );
    }

    // An outline outside the rectangle.
    pub fn outline(&mut self, rect: Rect, radius: f32, width: f32, color: Color) {
        self.shape(rect.expand(width), radius + width, Color::TRANSPARENT, Some(Border::new(color, width)));
    }

    // The shadows of a surface at an elevation, only outside the surface, so
    // that they never darken a translucent surface from below.
    pub fn shadow(&mut self, rect: Rect, radius: f32, elevation: Elevation) {
        for shadow in elevation.shadows {
            let opacity = match self.theme.kind {
                ThemeKind::Light => shadow.opacity_light,
                ThemeKind::Dark => shadow.opacity_dark,
            };
            if opacity <= 0.0 {
                continue;
            }
            self.list.push(
                KIND_SHADOW,
                Instance {
                    bounds: [rect.left(), rect.top(), rect.right(), rect.bottom()],
                    radii: [radius, shadow.sigma(), shadow.offset_y, 0.0],
                    colors: [Color::BLACK.with_alpha(opacity).pack_premultiplied(), 0, 0, 0],
                    ..Default::default()
                },
            );
        }
    }

    pub fn circle(&mut self, center: Point, radius: f32, color: Color) {
        let rect = Rect::from_center(center, Size::square(radius * 2.0));
        self.shape(rect, radius, color, None);
    }

    // A circle's outline, `width` wide, inside the radius.
    pub fn ring(&mut self, center: Point, radius: f32, width: f32, color: Color) {
        let rect = Rect::from_center(center, Size::square(radius * 2.0));
        self.shape(rect, radius, Color::TRANSPARENT, Some(Border::new(color, width)));
    }

    // Part of a ring centered on the radius, from `start` clockwise by
    // `sweep`, in radians from straight up, with round ends, like a progress
    // ring's.
    pub fn arc(&mut self, center: Point, radius: f32, width: f32, start: f32, sweep: f32, color: Color) {
        if sweep == 0.0 {
            return;
        }
        let (start, sweep) = if sweep < 0.0 { (start + sweep, -sweep) } else { (start, sweep) };
        self.list.push(
            KIND_ARC,
            Instance {
                bounds: [center.x, center.y, radius, width],
                radii: [start.rem_euclid(std::f32::consts::TAU), sweep.min(std::f32::consts::TAU), 0.0, 0.0],
                colors: [color.pack_premultiplied(), 0, 0, 0],
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

    pub fn polyline(&mut self, points: &[Point], width: f32, color: Color) {
        for pair in points.windows(2) {
            self.line(pair[0], pair[1], width, color);
        }
    }

    fn segment(&mut self, from: Point, to: Point, width: f32, color: Color, round: bool) {
        // Only the part in the clip is drawn, so that a line that reaches far
        // past the window, like a grid's in a zoomed view, isn't a huge quad.
        let Some((from, to)) = clip_segment(from, to, self.clip().expand(width)) else {
            return;
        };
        self.list.push(
            KIND_LINE,
            Instance {
                bounds: [from.x, from.y, to.x, to.y],
                radii: [width, if round { 1.0 } else { 0.0 }, 0.0, 0.0],
                colors: [color.pack_premultiplied(), 0, 0, 0],
                ..Default::default()
            },
        );
    }

    // A convex polygon, like a hexagon or a triangle, with its corners in
    // either order around it.
    pub fn polygon(&mut self, points: &[Point], color: Color) {
        if points.len() < 3 || color.a <= 0.0 {
            return;
        }
        let first = self.list.points.len();
        // The shader takes the corners clockwise on screen, where y is down.
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
        let mut bounds = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for point in points {
            bounds = [bounds[0].min(point.x), bounds[1].min(point.y), bounds[2].max(point.x), bounds[3].max(point.y)];
        }
        self.list.push(
            KIND_POLYGON,
            Instance {
                bounds,
                radii: [first as f32, points.len() as f32, 0.0, 0.0],
                colors: [color.pack_premultiplied(), 0, 0, 0],
                ..Default::default()
            },
        );
    }

    pub fn fonts(&mut self) -> &mut Fonts {
        self.fonts
    }

    pub fn measure(&mut self, text: &str, style: TextStyle) -> Size {
        self.fonts.measure(text, style)
    }

    // The size of the text wrapped at a width.
    pub fn measure_wrapped(&mut self, text: &str, style: TextStyle, width: f32) -> Size {
        self.fonts.measure_wrapped(text, style, width)
    }

    // A line of text with its line box's top left at `origin`. Returns its
    // size.
    pub fn text(&mut self, origin: Point, text: &str, style: TextStyle, color: Color) -> Size {
        self.glyphs(origin, text, style, None, color)
    }

    // Text wrapped at the rectangle's width, from its top left. Returns its
    // size.
    pub fn text_wrapped(&mut self, rect: Rect, text: &str, style: TextStyle, color: Color) -> Size {
        self.glyphs(rect.origin(), text, style, Some(rect.width.max(1.0)), color)
    }

    // A line of text centered vertically in the rectangle and aligned across
    // it. Returns where its line box went.
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

    // Like text_in, but text too wide for the rectangle is cut short with an
    // ellipsis, as XAML's TextTrimming does. Returns where it went, and
    // whether it was cut.
    pub fn text_fit(&mut self, rect: Rect, text: &str, style: TextStyle, color: Color, align: Align) -> (Rect, bool) {
        let fitted = self.fit(text, style, rect.width);
        let cut = matches!(fitted, Cow::Owned(_));
        (self.text_in(rect, &fitted, style, color, align), cut)
    }

    // The text, cut short with an ellipsis if it's wider than `width`, or
    // the text itself if it fits, without a copy.
    pub fn fit<'t>(&mut self, text: &'t str, style: TextStyle, width: f32) -> Cow<'t, str> {
        fit_text(self.fonts, text, style, width)
    }

    // A line of runs in their own styles and colors, one after another,
    // sharing a baseline. Returns the width.
    pub fn spans(&mut self, origin: Point, spans: &[Span]) -> f32 {
        let line_height = spans.iter().map(|span| span.style.line_height).fold(0.0, f32::max);
        let mut x = origin.x;
        for span in spans {
            let y = origin.y + ((line_height - span.style.line_height) / 2.0).round();
            x += self.text(Point::new(x, y), span.text, span.style, span.color).width;
        }
        x - origin.x
    }

    fn glyphs(&mut self, origin: Point, text: &str, style: TextStyle, wrap: Option<f32>, color: Color) -> Size {
        if text.is_empty() {
            return Size::new(0.0, style.line_height);
        }
        let packed = color.pack_premultiplied();
        // Direct2D's grayscale text blending, as Windows Terminal's copy of
        // DirectWrite's has it: enhanced contrast, the default 1 for dark
        // text and none for light, and the text's intensity, for the alpha
        // correction that the shader applies with them.
        let lightness = 0.30 * color.r + 0.59 * color.g + 0.11 * color.b;
        let contrast = (4.0 * (0.75 - lightness)).clamp(0.0, 1.0);
        let intensity = 0.25 * color.r + 0.5 * color.g + 0.25 * color.b;
        let list = &mut *self.list;
        let mut emit = |quad: GlyphQuad| {
            list.push(
                KIND_GLYPH,
                Instance {
                    bounds: [quad.x, quad.y, quad.x + quad.width, quad.y + quad.height],
                    radii: [quad.atlas_x, quad.atlas_y, contrast, intensity],
                    colors: [packed, 0, 0, 0],
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
}

// The text, cut short with an ellipsis if it's wider than `width`, or the
// text itself if it fits, without a copy: most labels fit, every frame.
pub(crate) fn fit_text<'t>(fonts: &mut Fonts, text: &'t str, style: TextStyle, width: f32) -> Cow<'t, str> {
    if fonts.measure(text, style).width <= width {
        return Cow::Borrowed(text);
    }
    let boundaries: Vec<usize> = text.char_indices().map(|(index, _)| index).skip(1).collect();
    // The most characters whose text with an ellipsis fits.
    let (mut low, mut high) = (0usize, boundaries.len());
    while low < high {
        let middle = (low + high).div_ceil(2);
        let candidate = format!("{}…", text[..boundaries[middle - 1]].trim_end());
        if fonts.measure(&candidate, style).width <= width {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    Cow::Owned(if low == 0 { "…".to_string() } else { format!("{}…", text[..boundaries[low - 1]].trim_end()) })
}

// The part of the segment inside the rectangle, by Liang and Barsky's
// clipping, or None if none of it is.
fn clip_segment(from: Point, to: Point, rect: Rect) -> Option<(Point, Point)> {
    let delta = to - from;
    let (mut enter, mut leave) = (0.0f32, 1.0f32);
    for (p, q) in [
        (-delta.x, from.x - rect.left()),
        (delta.x, rect.right() - from.x),
        (-delta.y, from.y - rect.top()),
        (delta.y, rect.bottom() - from.y),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                enter = enter.max(t);
            } else {
                leave = leave.min(t);
            }
        }
    }
    (enter <= leave).then(|| (from + delta * enter, from + delta * leave))
}
