// A tooltip: a box of text by the cursor, over everything else, about what
// the cursor is over. A program draws one while a widget's response is
// hovered, which is from the frame's own cursor, so it shows at once, and
// goes as soon as the cursor leaves.
//
// It goes under the cursor, or above it if there isn't room, or else beside
// what it's about, so that it covers neither that nor the cursor, and always
// inside the window.

use canvas::theme::{DIM, TEXT};
use canvas::{Color, Rect, wrap};

use crate::Frame;
use crate::interface::Layer;
use crate::panels::{MARGIN, PADDING, raised_box};

// A tooltip's text wraps at this many characters.
pub const TOOLTIP_COLUMNS: usize = 72;

impl Frame<'_> {
    // A tooltip about `anchor`, the area of what it's about, made of pieces
    // of text, each at its place from the top left of the box's contents,
    // in its color. The box is as big as the pieces need. `name` says what
    // it's about, for scripts. Returns the box's area.
    pub fn tooltip<S: AsRef<str>>(&mut self, name: &str, anchor: Rect, pieces: &[([f32; 2], S, Color)]) -> Rect {
        let canvas = &self.tooltips;
        let line_height = canvas.line_height();
        let char_width = canvas.char_width();
        let (mut right, mut bottom) = (0.0f32, 0.0f32);
        for ([x, y], text, _) in pieces {
            right = right.max(x + canvas.text_width(text.as_ref()));
            bottom = bottom.max(y + line_height);
        }
        let (box_width, box_height) = (right + PADDING * 2.0, bottom + PADDING * 2.0);
        let (width, height) = (self.width, self.height);
        let cursor = self.cursor().unwrap_or([anchor.x, anchor.bottom()]);
        let below = cursor[1] + line_height * 1.4;
        let above = cursor[1] - line_height * 0.6 - box_height;
        let across = (cursor[0] + char_width).min(width - MARGIN - box_width).max(MARGIN);
        let (left, top) = if below + box_height <= height - MARGIN {
            (across, below)
        } else if above >= MARGIN {
            (across, above)
        } else {
            let right_of = anchor.right() + char_width;
            let left_of = anchor.x - char_width - box_width;
            let left = if right_of + box_width <= width - MARGIN {
                right_of
            } else {
                left_of.max(MARGIN)
            };
            let top = (cursor[1] - box_height * 0.5).min(height - MARGIN - box_height).max(MARGIN);
            (left, top)
        };
        let area = Rect::new(left, top, box_width, box_height);
        let canvas = self.layer(Layer::Tooltip);
        raised_box(canvas, area, DIM);
        for ([x, y], text, color) in pieces {
            canvas.text(area.x + PADDING + x, area.y + PADDING + y, text.as_ref(), *color);
        }
        self.state.tooltip = Some((name.to_string(), area));
        area
    }

    // A tooltip of a paragraph of text, wrapped at TOOLTIP_COLUMNS.
    pub fn tooltip_text(&mut self, name: &str, anchor: Rect, text: &str) -> Rect {
        let line_height = self.line_height();
        let pieces: Vec<([f32; 2], String, Color)> = wrap(text, TOOLTIP_COLUMNS, 0)
            .into_iter()
            .enumerate()
            .map(|(index, line)| ([0.0, line_height * index as f32], line, TEXT))
            .collect();
        self.tooltip(name, anchor, &pieces)
    }
}
