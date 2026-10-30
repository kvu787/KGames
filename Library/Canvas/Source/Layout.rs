// Helpers for laying out panels in fixed pixels, and the spacing that KGames'
// programs share. Programs only support fixed window sizes (see
// Library/Application), so nothing here adapts to the window's size.

use crate::{Canvas, Color};
use crate::theme::{BORDER, DIM, ERROR, PANEL, TEXT, TITLE, WARNING};

// Between panels, and between them and the window's edges.
pub const MARGIN: f32 = 16.0;
// Around the header's and footer's text.
pub const EDGE: f32 = 12.0;
// Inside a panel, around its contents.
pub const PADDING: f32 = 12.0;

#[derive(Clone, Copy)]
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
        }
    }

    // A label in one color followed by a value in another.
    pub fn pair(&mut self, canvas: &mut Canvas, label: &str, value: &str, color: Color) {
        if self.has_room() {
            let end = canvas.text(self.x, self.y, label, DIM);
            canvas.text_clipped(end, self.y, value, self.right() - end, color);
            self.y += self.line_height;
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

// Draws a panel's background and title, and returns the area under the title.
pub fn panel(canvas: &mut Canvas, area: Rect, title: &str, subtitle: &str) -> Rect {
    let padding = PADDING;
    canvas.rect(area.x, area.y, area.width, area.height, PANEL);
    canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
    let x = canvas.text(area.x + padding, area.y + padding, title, TITLE);
    if !subtitle.is_empty() {
        canvas.text_clipped(
            x,
            area.y + padding,
            &format!("  {subtitle}"),
            area.right() - padding - x,
            DIM,
        );
    }
    let top = area.y + padding + canvas.line_height() * 1.4;
    Rect::new(
        area.x + padding,
        top,
        area.width - padding * 2.0,
        area.bottom() - padding - top,
    )
}

// Instead of the program's panels, when the window isn't in a supported
// setup: `supported` says which setups are, and `problems` what's wrong.
pub fn draw_window_problems(canvas: &mut Canvas, supported: &str, problems: &[String], width: f32) {
    let line_height = canvas.line_height();
    let area = Rect::new(MARGIN, MARGIN, width - MARGIN * 2.0, f32::INFINITY);
    let mut lines = Lines::new(area, line_height);
    lines.line(canvas, "Unsupported window", ERROR);
    lines.gap(0.5);
    lines.wrapped(canvas, supported, "", TEXT);
    lines.gap(0.5);
    for problem in problems {
        lines.wrapped(canvas, &format!("- {problem}"), "  ", WARNING);
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
