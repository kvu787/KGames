// A tooltip: a small card of text by the cursor, over everything else, about
// what the cursor is over, like Fluent's. A program draws one while a
// widget's response is hovered, which is from the frame's own cursor, so it
// shows at once, and goes as soon as the cursor leaves.
//
// It goes under the cursor, or above it if there isn't room, or else beside
// what it's about, so that it covers neither that nor the cursor, and always
// inside the window.

use canvas::{Color, Rect, TextStyle};

use crate::Frame;
use crate::interface::Layer;
use crate::surfaces::shadow;
use crate::tokens::*;

// A tooltip's paragraph wraps at this width.
pub const TOOLTIP_WIDTH: f32 = 400.0;
const PADDING_ACROSS: f32 = 10.0;
const PADDING_DOWN: f32 = 7.0;

// A piece of a tooltip's text: where it goes from the top left of the
// tooltip's contents, the text, its style and its color.
pub type TooltipText<S> = ([f32; 2], S, TextStyle, Color);

impl Frame<'_> {
    // A tooltip about `anchor`, the area of what it's about, made of pieces
    // of text. The box is as big as the pieces need. `name` says what it's
    // about, for scripts. Returns the box's area.
    pub fn tooltip<S: AsRef<str>>(&mut self, name: &str, anchor: Rect, pieces: &[TooltipText<S>]) -> Rect {
        let theme = self.theme();
        let canvas = &self.overlay;
        let (mut right, mut bottom) = (0.0f32, 0.0f32);
        for ([x, y], text, style, _) in pieces {
            right = right.max(x + canvas.measure(text.as_ref(), *style));
            bottom = bottom.max(y + canvas.line_height(*style));
        }
        let box_width = (right + PADDING_ACROSS * 2.0).ceil();
        let box_height = (bottom + PADDING_DOWN * 2.0).ceil();
        let (width, height) = (self.width, self.height);
        let cursor = self.cursor().unwrap_or([anchor.x, anchor.bottom()]);
        // Under the cursor's arrow, which is about 20 pixels tall.
        let below = cursor[1] + 24.0;
        let above = cursor[1] - 8.0 - box_height;
        let across = (cursor[0] - 8.0).min(width - MARGIN - box_width).max(MARGIN);
        let (left, top) = if below + box_height <= height - MARGIN {
            (across, below)
        } else if above >= MARGIN {
            (across, above)
        } else {
            let right_of = anchor.right() + SPACE_S;
            let left_of = anchor.x - SPACE_S - box_width;
            let left = if right_of + box_width <= width - MARGIN { right_of } else { left_of.max(MARGIN) };
            let top = (cursor[1] - box_height * 0.5).min(height - MARGIN - box_height).max(MARGIN);
            (left, top)
        };
        let area = Rect::new(left.round(), top.round(), box_width, box_height);
        let canvas = self.layer(Layer::Overlay);
        shadow(canvas, theme, area, RADIUS_CONTROL, TOOLTIP_ELEVATION);
        canvas.fill_stroke(area, RADIUS_CONTROL, theme.flyout, 1.0, theme.flyout_stroke);
        for ([x, y], text, style, color) in pieces {
            canvas.text([area.x + PADDING_ACROSS + x, area.y + PADDING_DOWN + y], text.as_ref(), *style, *color);
        }
        self.state.tooltip = Some((name.to_string(), area));
        area
    }

    // A tooltip of a paragraph of text, wrapped at TOOLTIP_WIDTH.
    pub fn tooltip_text(&mut self, name: &str, anchor: Rect, text: &str) -> Rect {
        let style = TextStyle::Body;
        let line_height = self.line_height(style);
        let color = self.theme().text;
        let pieces: Vec<TooltipText<String>> = self
            .fonts()
            .wrap(text, style, TOOLTIP_WIDTH, 0.0)
            .into_iter()
            .enumerate()
            .map(|(index, line)| ([0.0, line_height * index as f32], line, style, color))
            .collect();
        self.tooltip(name, anchor, &pieces)
    }
}
