// WinUI's ScrollBar, and a region whose content scrolls behind it, like a
// ScrollViewer. The bar's thumb is a 2-pixel line while the pointer is away,
// and grows to 6 pixels, over an acrylic track, once the pointer has rested
// on the bar; the wheel scrolls the region under the pointer.
//
// The input and paint halves are split, so controls that hold a bar, like a
// list, run them in the dispatch and draw phases.

use crate::core::ui::{Response, Sense, Ui};
use crate::design::controls::{
    SCROLL_BAR_CONTRACT_DELAY, SCROLL_BAR_EXPAND_DELAY, SCROLL_BAR_MIN_THUMB, SCROLL_BAR_RADIUS, SCROLL_BAR_THUMB_COLLAPSED,
    SCROLL_BAR_THUMB_EXPANDED, SCROLL_BAR_THUMB_OFFSET, SCROLL_VIEWER_BAR_MARGIN,
};
use crate::design::tokens::{CURVE_FAST_OUT_SLOW_IN, DURATION_FAST};
use crate::drawing::geometry::Rect;

// When the pointer came onto the bar or left it, for its delays.
#[derive(Default)]
struct BarState {
    over_since: Option<f64>,
    away_since: Option<f64>,
    expanded: bool,
    // Where in the thumb a drag took it.
    grab: f32,
}

fn thumb_metrics(rect: Rect, content: f32, viewport: f32) -> (Rect, f32, f32) {
    let track = rect.inset(SCROLL_VIEWER_BAR_MARGIN);
    let thumb_length = if content > 0.0 { (track.height * viewport / content).clamp(SCROLL_BAR_MIN_THUMB.min(track.height), track.height) } else { track.height };
    let travel = (track.height - thumb_length).max(0.0);
    (track, thumb_length, travel)
}

// The input half of a vertical scroll bar for content `content` tall, seen
// through a viewport `viewport` tall, scrolled down by `offset`. Returns its
// response, and whether the offset moved.
pub(crate) fn scroll_bar_input(ui: &mut Ui, name: &str, rect: Rect, offset: &mut f32, content: f32, viewport: f32) -> (Response, bool) {
    let response = ui.interact(name, rect, Sense::CLICK, "ScrollBar");
    let range = (content - viewport).max(0.0);
    let before = *offset;
    let (track, thumb_length, travel) = thumb_metrics(rect, content, viewport);
    let thumb_y = |offset: f32| track.y + if range > 0.0 { travel * offset / range } else { 0.0 };
    let now = ui.time();
    let mut state = std::mem::take(ui.memory::<BarState>(response.id));

    if response.pressed && let Some(pointer) = response.pointer {
        let top = thumb_y(*offset);
        if pointer.y >= top && pointer.y < top + thumb_length {
            state.grab = pointer.y - top;
        } else {
            // A press on the track pages toward it.
            let page = viewport.max(1.0);
            *offset = if pointer.y < top { *offset - page } else { *offset + page };
            state.grab = thumb_length / 2.0;
        }
    } else if response.held && let Some(pointer) = response.pointer {
        let top = pointer.y - state.grab;
        *offset = if travel > 0.0 { (top - track.y) / travel * range } else { 0.0 };
    }
    *offset = offset.clamp(0.0, range);

    // It expands after the pointer rests on it, and collapses a while
    // after the pointer leaves.
    let over = response.hovered || response.held;
    if over {
        state.away_since = None;
        let since = *state.over_since.get_or_insert(now);
        if now - since >= SCROLL_BAR_EXPAND_DELAY || response.held {
            state.expanded = true;
        }
    } else {
        state.over_since = None;
        let since = *state.away_since.get_or_insert(now);
        if now - since >= SCROLL_BAR_CONTRACT_DELAY {
            state.expanded = false;
        }
    }
    *ui.memory::<BarState>(response.id) = state;
    (response, *offset != before)
}

// The paint half, from the input half's response.
pub(crate) fn scroll_bar_paint(ui: &mut Ui, response: &Response, rect: Rect, offset: f32, content: f32, viewport: f32) {
    let range = (content - viewport).max(0.0);
    if range <= 0.0 {
        return;
    }
    let (track, thumb_length, travel) = thumb_metrics(rect, content, viewport);
    let thumb_top = track.y + if range > 0.0 { travel * offset / range } else { 0.0 };
    let expanded = ui.memory::<BarState>(response.id).expanded;
    let expansion = ui.animate(response.id, 0, if expanded { 1.0 } else { 0.0 }, DURATION_FAST, CURVE_FAST_OUT_SLOW_IN);
    let width = SCROLL_BAR_THUMB_COLLAPSED + (SCROLL_BAR_THUMB_EXPANDED - SCROLL_BAR_THUMB_COLLAPSED) * expansion;
    // A collapsed thumb sits toward the outer edge.
    let center_x = track.center().x + SCROLL_BAR_THUMB_OFFSET * (1.0 - expansion);
    let mut painter = ui.painter();
    let theme = painter.theme();
    if expansion > 0.0 {
        painter.rounded(track, SCROLL_BAR_RADIUS * 2.0, theme.acrylic.fade(expansion));
    }
    let thumb = Rect::new(center_x - width / 2.0, thumb_top, width, thumb_length);
    let color = if response.enabled { theme.control_strong_fill } else { theme.control_strong_fill_disabled };
    painter.rounded(thumb, SCROLL_BAR_RADIUS.min(width / 2.0), color);
}
