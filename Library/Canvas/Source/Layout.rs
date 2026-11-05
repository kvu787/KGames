// Rectangles, and lines of text, in fixed pixels. Programs only support
// fixed window sizes (see Library/Application), so nothing here adapts to
// the window's size. Library/UserInterface lays out panels and widgets with
// these.

use crate::theme::DIM;
use crate::{Canvas, Color};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width: width.max(0.0),
            height: height.max(0.0),
        }
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

    // Smaller by `amount` on every side.
    pub fn inset(&self, amount: f32) -> Rect {
        Rect::new(
            self.x + amount,
            self.y + amount,
            self.width - amount * 2.0,
            self.height - amount * 2.0,
        )
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

// Writes lines of text downward, and stops once they no longer fit.
pub struct Lines {
    pub x: f32,
    pub y: f32,
    pub bottom: f32,
    pub width: f32,
    pub line_height: f32,
}

impl Lines {
    pub fn new(area: Rect, line_height: f32) -> Self {
        Self {
            x: area.x,
            y: area.y,
            bottom: area.bottom(),
            width: area.width,
            line_height,
        }
    }

    pub fn has_room(&self) -> bool {
        self.y + self.line_height <= self.bottom + 0.5
    }

    pub fn line(&mut self, canvas: &mut Canvas, text: &str, color: Color) {
        if self.has_room() {
            canvas.text_clipped(self.x, self.y, text, self.width, color);
            self.y += self.line_height;
        } else {
            canvas.dropped_lines += 1;
        }
    }

    // A label in one color followed by a value in another.
    pub fn pair(&mut self, canvas: &mut Canvas, label: &str, value: &str, color: Color) {
        if self.has_room() {
            let end = canvas.text(self.x, self.y, label, DIM);
            canvas.text_clipped(end, self.y, value, self.right() - end, color);
            self.y += self.line_height;
        } else {
            canvas.dropped_lines += 1;
        }
    }

    pub fn wrapped(&mut self, canvas: &mut Canvas, text: &str, indent: &str, color: Color) {
        let columns = (self.width / canvas.char_width()).floor().max(8.0) as usize;
        for (index, line) in wrap(text, columns, indent.chars().count())
            .iter()
            .enumerate()
        {
            let prefix = if index == 0 { "" } else { indent };
            self.line(canvas, &format!("{prefix}{line}"), color);
        }
    }

    pub fn gap(&mut self, fraction: f32) {
        self.y += self.line_height * fraction;
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }
}

// Splits text into lines of at most `columns` characters at spaces. Lines
// after the first are `indent` characters narrower, for a hanging indent.
pub fn wrap(text: &str, columns: usize, indent: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        let limit = if lines.is_empty() {
            columns
        } else {
            columns.saturating_sub(indent).max(1)
        };
        let length = line.chars().count();
        if length > 0 && length + 1 + word.chars().count() > limit {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    lines.push(line);
    lines
}
