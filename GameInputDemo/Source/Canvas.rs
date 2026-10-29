// Builds one frame's 2D geometry: rectangles, antialiased lines and circles,
// and monospaced text, all as one triangle list for a single draw call.
// Coordinates are physical pixels with the origin at the window's top left.

use crate::font::Font;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    // Read by the GPU as R8G8B8A8_UNORM, so red is the lowest byte.
    pub color: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub u32);

impl Color {
    pub const fn rgb(hex: u32) -> Self {
        let r = (hex >> 16) & 0xFF;
        let g = (hex >> 8) & 0xFF;
        let b = hex & 0xFF;
        Self(0xFF00_0000 | (b << 16) | (g << 8) | r)
    }

    pub fn alpha(self, alpha: f32) -> Self {
        let current = (self.0 >> 24) as f32 / 255.0;
        let a = (current * alpha.clamp(0.0, 1.0) * 255.0).round() as u32;
        Self((self.0 & 0x00FF_FFFF) | (a << 24))
    }
}

pub struct Canvas<'a> {
    font: &'a Font,
    solid_uv: [f32; 2],
    pub vertices: Vec<Vertex>,
}

impl<'a> Canvas<'a> {
    pub fn new(font: &'a Font) -> Self {
        Self {
            font,
            solid_uv: font.solid_uv(),
            vertices: Vec::with_capacity(64 * 1024),
        }
    }

    pub fn char_width(&self) -> f32 {
        self.font.cell_width as f32
    }

    pub fn line_height(&self) -> f32 {
        self.font.cell_height as f32
    }

    pub fn text_width(&self, text: &str) -> f32 {
        text.chars().count() as f32 * self.char_width()
    }

    fn vertex(&mut self, position: [f32; 2], color: Color) {
        self.vertices.push(Vertex {
            position,
            uv: self.solid_uv,
            color: color.0,
        });
    }

    // Two triangles. The corners go around the quad in order.
    fn quad(&mut self, corners: [[f32; 2]; 4], colors: [Color; 4]) {
        for index in [0, 1, 2, 0, 2, 3] {
            self.vertex(corners[index], colors[index]);
        }
    }

    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.quad(
            [
                [x, y],
                [x + width, y],
                [x + width, y + height],
                [x, y + height],
            ],
            [color; 4],
        );
    }

    pub fn rect_outline(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        thickness: f32,
        color: Color,
    ) {
        self.rect(x, y, width, thickness, color);
        self.rect(x, y + height - thickness, width, thickness, color);
        self.rect(x, y + thickness, thickness, height - 2.0 * thickness, color);
        self.rect(
            x + width - thickness,
            y + thickness,
            thickness,
            height - 2.0 * thickness,
            color,
        );
    }

    // Antialiased: a solid core with a one-pixel fringe on each side that
    // fades to transparent. Lines thinner than a pixel get fainter instead.
    pub fn line(&mut self, from: [f32; 2], to: [f32; 2], width: f32, color: Color) {
        let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
        let length = (dx * dx + dy * dy).sqrt();
        if length < 0.001 {
            return;
        }
        let normal = [-dy / length, dx / length];
        let inner = (width * 0.5 - 0.5).max(0.0);
        let outer = width * 0.5 + 0.5;
        let solid = color.alpha(width.min(1.0));
        let clear = color.alpha(0.0);
        let offset = |point: [f32; 2], distance: f32| {
            [
                point[0] + normal[0] * distance,
                point[1] + normal[1] * distance,
            ]
        };
        for (near, far, near_color, far_color) in [
            (-outer, -inner, clear, solid),
            (-inner, inner, solid, solid),
            (inner, outer, solid, clear),
        ] {
            self.quad(
                [
                    offset(from, near),
                    offset(to, near),
                    offset(to, far),
                    offset(from, far),
                ],
                [near_color, near_color, far_color, far_color],
            );
        }
    }

    pub fn circle(&mut self, center: [f32; 2], radius: f32, color: Color) {
        let inner = (radius - 0.5).max(0.0);
        let clear = color.alpha(0.0);
        let segments = segment_count(radius);
        for segment in 0..segments {
            let (a, b) = (angle(segment, segments), angle(segment + 1, segments));
            self.vertex(center, color);
            self.vertex(on_circle(center, inner, a), color);
            self.vertex(on_circle(center, inner, b), color);
            self.quad(
                [
                    on_circle(center, inner, a),
                    on_circle(center, radius + 0.5, a),
                    on_circle(center, radius + 0.5, b),
                    on_circle(center, inner, b),
                ],
                [color, clear, clear, color],
            );
        }
    }

    pub fn ring(&mut self, center: [f32; 2], radius: f32, width: f32, color: Color) {
        let inner = (width * 0.5 - 0.5).max(0.0);
        let outer = width * 0.5 + 0.5;
        let solid = color.alpha(width.min(1.0));
        let clear = color.alpha(0.0);
        let segments = segment_count(radius);
        for segment in 0..segments {
            let (a, b) = (angle(segment, segments), angle(segment + 1, segments));
            for (near, far, near_color, far_color) in [
                (-outer, -inner, clear, solid),
                (-inner, inner, solid, solid),
                (inner, outer, solid, clear),
            ] {
                self.quad(
                    [
                        on_circle(center, radius + near, a),
                        on_circle(center, radius + near, b),
                        on_circle(center, radius + far, b),
                        on_circle(center, radius + far, a),
                    ],
                    [near_color, near_color, far_color, far_color],
                );
            }
        }
    }

    // Draws text from the top left, snapped to whole pixels so the atlas maps
    // one texel to one pixel. Returns the x just past the last character.
    pub fn text(&mut self, x: f32, y: f32, text: &str, color: Color) -> f32 {
        let (mut x, y) = (x.round(), y.round());
        let (width, height) = (self.font.cell_width as f32, self.font.cell_height as f32);
        for character in text.chars() {
            if character != ' ' {
                let [u0, v0, u1, v1] = self.font.glyph_uv(character);
                let corners = [
                    [x, y],
                    [x + width, y],
                    [x + width, y + height],
                    [x, y + height],
                ];
                let uvs = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];
                for index in [0, 1, 2, 0, 2, 3] {
                    self.vertices.push(Vertex {
                        position: corners[index],
                        uv: uvs[index],
                        color: color.0,
                    });
                }
            }
            x += width;
        }
        x
    }

    // Like text, but cuts the text short with "..." if it's wider than max_width.
    pub fn text_clipped(
        &mut self,
        x: f32,
        y: f32,
        text: &str,
        max_width: f32,
        color: Color,
    ) -> f32 {
        let fit = (max_width / self.char_width()).floor().max(0.0) as usize;
        if text.chars().count() <= fit {
            return self.text(x, y, text, color);
        }
        let mut shortened: String = text.chars().take(fit.saturating_sub(3)).collect();
        shortened.push_str("...");
        let shortened: String = shortened.chars().take(fit).collect();
        self.text(x, y, &shortened, color)
    }
}

// Enough segments that a circle's edge looks round at its size.
fn segment_count(radius: f32) -> u32 {
    (radius * 0.8).clamp(16.0, 128.0) as u32
}

fn angle(segment: u32, segments: u32) -> f32 {
    segment as f32 / segments as f32 * std::f32::consts::TAU
}

fn on_circle(center: [f32; 2], radius: f32, angle: f32) -> [f32; 2] {
    [
        center[0] + radius * angle.cos(),
        center[1] + radius * angle.sin(),
    ]
}
