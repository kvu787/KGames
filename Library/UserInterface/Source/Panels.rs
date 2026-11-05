// The spacing that KGames' programs share, and the boxes that hold their
// contents: panels, and boxes of text over a 3D view. Programs only support
// fixed window sizes (see Library/Application), so everything is in fixed
// pixels, and nothing adapts to the window's size.

use canvas::theme::{BORDER, DIM, PANEL, TITLE};
use canvas::{Canvas, Color, Rect};

use crate::Frame;

// The em height of the font, in pixels. Programs only support 100% scaling,
// so this never changes.
pub const FONT_SIZE: i32 = 15;
// Between panels, and between them and the window's edges.
pub const MARGIN: f32 = 16.0;
// Around the header's and footer's text.
pub const EDGE: f32 = 12.0;
// Inside a panel, around its contents.
pub const PADDING: f32 = 12.0;
// The header has a title, a status line, and room for this many lines of
// notices, so the panels start at the same place whether there are any.
pub const NOTICE_LINES: usize = 2;
// Where panels start, under the header.
pub const PANELS_TOP: f32 = 96.0;
// Under popups and tooltips, so that they stand out from the panels under
// them.
pub const SHADOW: Color = Color(0x2600_0000);

// How tall a control with a line of text is, like a button in a row of
// them, a select box, or a row of a list.
pub fn control_height(canvas: &Canvas) -> f32 {
    (canvas.line_height() * 1.3).round()
}

// Draws a panel's background and title, and returns the area under the
// title.
pub fn panel(canvas: &mut Canvas, area: Rect, title: &str, subtitle: &str) -> Rect {
    canvas.rect(area.x, area.y, area.width, area.height, PANEL);
    canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
    let x = canvas.text(area.x + PADDING, area.y + PADDING, title, TITLE);
    if !subtitle.is_empty() {
        canvas.text_clipped(
            x,
            area.y + PADDING,
            &format!("  {subtitle}"),
            area.right() - PADDING - x,
            DIM,
        );
    }
    let top = area.y + PADDING + canvas.line_height() * 1.4;
    Rect::new(
        area.x + PADDING,
        top,
        area.width - PADDING * 2.0,
        area.bottom() - PADDING - top,
    )
}

// A box of text over a 3D view, at (x, y), as big as its lines need. Each
// line is pieces of text in their colors. Returns the box's area.
pub fn text_box(canvas: &mut Canvas, x: f32, y: f32, lines: &[Vec<(String, Color)>]) -> Rect {
    let line_height = canvas.line_height();
    let width = lines
        .iter()
        .map(|line| line.iter().map(|(text, _)| canvas.text_width(text)).sum::<f32>())
        .fold(0.0, f32::max);
    let area = Rect::new(
        x,
        y,
        width + PADDING * 2.0,
        line_height * lines.len() as f32 + PADDING * 2.0,
    );
    canvas.rect(area.x, area.y, area.width, area.height, PANEL);
    canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
    for (index, line) in lines.iter().enumerate() {
        let mut end = area.x + PADDING;
        for (text, color) in line {
            end = canvas.text(end, area.y + PADDING + line_height * index as f32, text, *color);
        }
    }
    area
}

// Pieces of text in their colors, one after another from (x, y), cut short
// with "..." where they'd pass `right`. Returns the x past the last.
pub fn pieces(canvas: &mut Canvas, x: f32, y: f32, pieces: &[(&str, Color)], right: f32) -> f32 {
    let mut end = x;
    for (text, color) in pieces {
        end = canvas.text_clipped(end, y, text, (right - end).max(0.0), *color);
    }
    end
}

// A box of a popup or a tooltip: a shadow, a background and an outline.
pub(crate) fn raised_box(canvas: &mut Canvas, area: Rect, outline: Color) {
    canvas.rect(area.x + 3.0, area.y + 3.0, area.width, area.height, SHADOW);
    canvas.rect(area.x, area.y, area.width, area.height, PANEL);
    canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, outline);
}

// Lays out areas down a rectangle, one after another, `gap` apart.
pub struct Stack {
    pub area: Rect,
    pub gap: f32,
    // Where the next area starts.
    pub y: f32,
}

impl Stack {
    pub fn new(area: Rect, gap: f32) -> Self {
        Self { area, gap, y: area.y }
    }

    // The next area, `height` tall and as wide as the stack.
    pub fn next(&mut self, height: f32) -> Rect {
        let area = Rect::new(self.area.x, self.y, self.area.width, height);
        self.y += height + self.gap;
        area
    }

    // Leaves `height` empty.
    pub fn skip(&mut self, height: f32) {
        self.y += height;
    }

    // What's left of the stack's area.
    pub fn rest(&self) -> Rect {
        Rect::new(self.area.x, self.y, self.area.width, self.area.bottom() - self.y)
    }
}

impl Frame<'_> {
    // Draws a panel, and returns the area under its title.
    pub fn panel(&mut self, area: Rect, title: &str, subtitle: &str) -> Rect {
        panel(&mut self.canvas, area, title, subtitle)
    }

    // A box of text over a 3D view. See text_box.
    pub fn text_box(&mut self, x: f32, y: f32, lines: &[Vec<(String, Color)>]) -> Rect {
        text_box(&mut self.canvas, x, y, lines)
    }

    pub fn control_height(&self) -> f32 {
        control_height(&self.canvas)
    }
}
