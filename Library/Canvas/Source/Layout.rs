// Rectangles, stacks, and lines of text, in fixed pixels. Programs only
// support fixed window sizes (see Library/Application), so nothing here
// adapts to the window's size. Library/UserInterface lays out cards and
// widgets with these.

use crate::{Canvas, Color, TextStyle};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

// Where something goes across, or down, the space it has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width: if width > 0.0 { width } else { 0.0 },
            height: if height > 0.0 { height } else { 0.0 },
        }
    }

    // From its left, top, right and bottom.
    pub fn from_edges(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Rect::new(left, top, right - left, bottom - top)
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn contains(&self, [x, y]: [f32; 2]) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    pub fn center(&self) -> [f32; 2] {
        [self.x + self.width * 0.5, self.y + self.height * 0.5]
    }

    // A point at fractions of the way across and down.
    pub fn at(&self, [across, down]: [f32; 2]) -> [f32; 2] {
        [self.x + self.width * across, self.y + self.height * down]
    }

    // Smaller by `amount` on every side.
    pub fn inset(&self, amount: f32) -> Rect {
        self.inset_by(amount, amount)
    }

    // Smaller by `across` on the left and right, and `down` on the top and
    // bottom.
    pub fn inset_by(&self, across: f32, down: f32) -> Rect {
        Rect::new(self.x + across, self.y + down, self.width - across * 2.0, self.height - down * 2.0)
    }

    // Bigger by `amount` on every side.
    pub fn outset(&self, amount: f32) -> Rect {
        Rect::new(
            self.x - amount,
            self.y - amount,
            self.width + amount * 2.0,
            self.height + amount * 2.0,
        )
    }

    pub fn translate(&self, [x, y]: [f32; 2]) -> Rect {
        Rect::new(self.x + x, self.y + y, self.width, self.height)
    }

    // A rectangle of `size` in this one, placed as `across` and `down` say.
    pub fn place(&self, [width, height]: [f32; 2], across: Align, down: Align) -> Rect {
        let offset = |align: Align, room: f32| match align {
            Align::Start => 0.0,
            Align::Center => (room * 0.5).round(),
            Align::End => room,
        };
        Rect::new(
            self.x + offset(across, self.width - width),
            self.y + offset(down, self.height - height),
            width,
            height,
        )
    }

    // A rectangle of `size` centered in this one, on whole pixels.
    pub fn centered(&self, size: [f32; 2]) -> Rect {
        self.place(size, Align::Center, Align::Center)
    }

    // The top `height` of the rectangle, and what's left under it after `gap`.
    pub fn split_top(&self, height: f32, gap: f32) -> (Rect, Rect) {
        let height = height.min(self.height);
        (
            Rect::new(self.x, self.y, self.width, height),
            Rect::new(self.x, self.y + height + gap, self.width, self.height - height - gap),
        )
    }

    // The bottom `height` of the rectangle, and what's left over it before
    // `gap`.
    pub fn split_bottom(&self, height: f32, gap: f32) -> (Rect, Rect) {
        let height = height.min(self.height);
        (
            Rect::new(self.x, self.bottom() - height, self.width, height),
            Rect::new(self.x, self.y, self.width, self.height - height - gap),
        )
    }

    // The left `width` of the rectangle, and what's left right of it after
    // `gap`.
    pub fn split_left(&self, width: f32, gap: f32) -> (Rect, Rect) {
        let width = width.min(self.width);
        (
            Rect::new(self.x, self.y, width, self.height),
            Rect::new(self.x + width + gap, self.y, self.width - width - gap, self.height),
        )
    }

    // The right `width` of the rectangle, and what's left left of it before
    // `gap`.
    pub fn split_right(&self, width: f32, gap: f32) -> (Rect, Rect) {
        let width = width.min(self.width);
        (
            Rect::new(self.right() - width, self.y, width, self.height),
            Rect::new(self.x, self.y, self.width - width - gap, self.height),
        )
    }

    // `count` columns of equal width, `gap` apart, whole pixels wide but
    // the last, which takes what's left.
    pub fn columns(&self, count: usize, gap: f32) -> Vec<Rect> {
        let count = count.max(1);
        let width = ((self.width - gap * (count - 1) as f32) / count as f32).floor();
        (0..count)
            .map(|index| {
                let x = self.x + (width + gap) * index as f32;
                let right = if index + 1 == count { self.right() } else { x + width };
                Rect::new(x, self.y, right - x, self.height)
            })
            .collect()
    }

    // `count` rows of `height`, `gap` apart, from the top.
    pub fn rows(&self, count: usize, height: f32, gap: f32) -> Vec<Rect> {
        (0..count)
            .map(|index| Rect::new(self.x, self.y + (height + gap) * index as f32, self.width, height))
            .collect()
    }
}

// Which way a stack lays out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Down,
    Across,
}

// Lays out areas one after another in a rectangle, `gap` apart, down it or
// across it, like a vertical or horizontal stack in SwiftUI, Compose or XAML.
#[derive(Clone, Copy, Debug)]
pub struct Stack {
    pub area: Rect,
    pub axis: Axis,
    pub gap: f32,
    // Where the next area starts, down or across.
    pub cursor: f32,
}

impl Stack {
    // A stack down `area`.
    pub fn down(area: Rect, gap: f32) -> Self {
        Self {
            area,
            axis: Axis::Down,
            gap,
            cursor: area.y,
        }
    }

    // A stack across `area`.
    pub fn across(area: Rect, gap: f32) -> Self {
        Self {
            area,
            axis: Axis::Across,
            gap,
            cursor: area.x,
        }
    }

    // The next area, `length` long along the stack, and as wide or tall as
    // the stack across it.
    pub fn next(&mut self, length: f32) -> Rect {
        let area = match self.axis {
            Axis::Down => Rect::new(self.area.x, self.cursor, self.area.width, length),
            Axis::Across => Rect::new(self.cursor, self.area.y, length, self.area.height),
        };
        self.cursor += length + self.gap;
        area
    }

    // The next area, of `size`, at the start of the stack's width or height.
    pub fn next_sized(&mut self, [width, height]: [f32; 2]) -> Rect {
        let area = match self.axis {
            Axis::Down => Rect::new(self.area.x, self.cursor, width, height),
            Axis::Across => Rect::new(self.cursor, self.area.y, width, height),
        };
        self.cursor += match self.axis {
            Axis::Down => height,
            Axis::Across => width,
        } + self.gap;
        area
    }

    // Leaves `length` empty, besides the gap.
    pub fn skip(&mut self, length: f32) {
        self.cursor += length;
    }

    // What's left of the stack's area.
    pub fn rest(&self) -> Rect {
        match self.axis {
            Axis::Down => Rect::from_edges(self.area.x, self.cursor, self.area.right(), self.area.bottom()),
            Axis::Across => Rect::from_edges(self.cursor, self.area.y, self.area.right(), self.area.bottom()),
        }
    }
}

// Writes lines of text down an area, each in its style, and stops once they
// no longer fit, counting what it left out in the canvas's dropped_lines.
pub struct Lines {
    pub x: f32,
    pub y: f32,
    pub bottom: f32,
    pub width: f32,
}

impl Lines {
    pub fn new(area: Rect) -> Self {
        Self {
            x: area.x,
            y: area.y,
            bottom: area.bottom(),
            width: area.width,
        }
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn has_room(&self, canvas: &Canvas, style: TextStyle) -> bool {
        self.y + canvas.line_height(style) <= self.bottom + 0.5
    }

    // The area of the next line, in a style, if it fits, and moves past it.
    pub fn next(&mut self, canvas: &mut Canvas, style: TextStyle) -> Option<Rect> {
        if !self.has_room(canvas, style) {
            canvas.dropped_lines += 1;
            return None;
        }
        let height = canvas.line_height(style);
        let area = Rect::new(self.x, self.y, self.width, height);
        self.y += height;
        Some(area)
    }

    pub fn line(&mut self, canvas: &mut Canvas, text: &str, style: TextStyle, color: Color) {
        if let Some(area) = self.next(canvas, style) {
            canvas.text_clipped([area.x, area.y], text, area.width, style, color);
        }
    }

    // Pieces of text in their colors, one after another on a line.
    pub fn pieces(&mut self, canvas: &mut Canvas, pieces: &[(&str, Color)], style: TextStyle) {
        if let Some(area) = self.next(canvas, style) {
            let mut x = area.x;
            for (text, color) in pieces {
                x = canvas.text_clipped([x, area.y], text, (area.right() - x).max(0.0), style, *color);
            }
        }
    }

    // A label in the secondary color, and a value after it, at `value_x`
    // from the left if it's past the label, so that values line up.
    pub fn pair(&mut self, canvas: &mut Canvas, label: &str, value: &str, value_x: f32, label_color: Color, value_color: Color) {
        if let Some(area) = self.next(canvas, TextStyle::Body) {
            let end = canvas.text([area.x, area.y], label, TextStyle::Body, label_color);
            let x = (area.x + value_x).max(end + 4.0);
            canvas.text_clipped([x, area.y], value, (area.right() - x).max(0.0), TextStyle::Body, value_color);
        }
    }

    // A paragraph, wrapped to the width, with lines after the first
    // `indent` in.
    pub fn wrapped(&mut self, canvas: &mut Canvas, text: &str, style: TextStyle, color: Color, indent: f32) {
        let lines = canvas.fonts.wrap(text, style, self.width, indent);
        for (index, line) in lines.iter().enumerate() {
            let shift = if index == 0 { 0.0 } else { indent };
            if let Some(area) = self.next(canvas, style) {
                canvas.text_clipped([area.x + shift, area.y], line, area.width - shift, style, color);
            }
        }
    }

    pub fn gap(&mut self, pixels: f32) {
        self.y += pixels;
    }
}
