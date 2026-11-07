// WinUI's ScrollBar, and a region whose content scrolls behind it, like a
// ScrollViewer. The bar's thumb is a 2-pixel line while the pointer is away,
// and grows to 6 pixels, over an acrylic track, once the pointer has rested
// on the bar; the wheel scrolls the region under the pointer.

use crate::core::interaction::{Id, Response, Sense, id};
use crate::core::ui::Ui;
use crate::design::controls::{
    SCROLL_BAR_CONTRACT_DELAY, SCROLL_BAR_EXPAND_DELAY, SCROLL_BAR_MIN_THUMB, SCROLL_BAR_RADIUS, SCROLL_BAR_SIZE, SCROLL_BAR_THUMB_COLLAPSED,
    SCROLL_BAR_THUMB_EXPANDED, SCROLL_BAR_THUMB_OFFSET, SCROLL_VIEWER_BAR_MARGIN,
};
use crate::design::tokens::{CURVE_FAST_OUT_SLOW_IN, DURATION_FAST};
use crate::drawing::geometry::Rect;
use crate::widgets::widget::Widget;

// Pixels that a notch of the wheel scrolls: three lines of 48.
pub const WHEEL_SCROLL: f32 = 144.0;

// A vertical scroll bar for content `content` tall, seen through a
// viewport `viewport` tall, scrolled down by `offset`.
pub struct ScrollBar<'v> {
    offset: &'v mut f32,
    content: f32,
    viewport: f32,
}

impl<'v> ScrollBar<'v> {
    pub fn new(offset: &'v mut f32, content: f32, viewport: f32) -> Self {
        Self { offset, content, viewport }
    }
}

// When the pointer came onto the bar or left it, for its delays.
#[derive(Default)]
struct BarState {
    over_since: Option<f64>,
    away_since: Option<f64>,
    expanded: bool,
    // Where in the thumb a drag took it.
    grab: f32,
}

impl Widget for ScrollBar<'_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let mut response = ui.interact(name, rect, Sense::CLICK, "ScrollBar");
        let range = (self.content - self.viewport).max(0.0);
        let before = *self.offset;
        let track = rect.inset(SCROLL_VIEWER_BAR_MARGIN);
        let thumb_length = if self.content > 0.0 { (track.height * self.viewport / self.content).clamp(SCROLL_BAR_MIN_THUMB.min(track.height), track.height) } else { track.height };
        let travel = (track.height - thumb_length).max(0.0);
        let thumb_y = |offset: f32| track.y + if range > 0.0 { travel * offset / range } else { 0.0 };
        let now = ui.time();
        let mut state = std::mem::take(ui.memory::<BarState>(response.id));

        if response.pressed && let Some(pointer) = response.pointer {
            let top = thumb_y(*self.offset);
            if pointer.y >= top && pointer.y < top + thumb_length {
                state.grab = pointer.y - top;
            } else {
                // A press on the track pages toward it.
                let page = self.viewport.max(1.0);
                *self.offset = if pointer.y < top { *self.offset - page } else { *self.offset + page };
                state.grab = thumb_length / 2.0;
            }
        } else if response.held && let Some(pointer) = response.pointer {
            let top = pointer.y - state.grab;
            *self.offset = if travel > 0.0 { (top - track.y) / travel * range } else { 0.0 };
        }
        *self.offset = self.offset.clamp(0.0, range);
        response.changed = *self.offset != before;

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
        let expansion = ui.animate(response.id, 0, if state.expanded { 1.0 } else { 0.0 }, DURATION_FAST, CURVE_FAST_OUT_SLOW_IN);
        *ui.memory::<BarState>(response.id) = state;
        if range <= 0.0 {
            return response;
        }
        let width = SCROLL_BAR_THUMB_COLLAPSED + (SCROLL_BAR_THUMB_EXPANDED - SCROLL_BAR_THUMB_COLLAPSED) * expansion;
        // A collapsed thumb sits toward the outer edge.
        let center_x = track.center().x + SCROLL_BAR_THUMB_OFFSET * (1.0 - expansion);
        let mut painter = ui.painter();
        let theme = painter.theme();
        if expansion > 0.0 {
            painter.rounded(track, SCROLL_BAR_RADIUS * 2.0, theme.acrylic.fade(expansion));
        }
        let thumb = Rect::new(center_x - width / 2.0, thumb_y(*self.offset), width, thumb_length);
        let color = if response.enabled { theme.control_strong_fill } else { theme.control_strong_fill_disabled };
        painter.rounded(thumb, SCROLL_BAR_RADIUS.min(width / 2.0), color);
        response
    }
}

// A region whose content scrolls: begin_scroll gives where the content goes,
// moved up by how far it's scrolled, and clips what's drawn to the region;
// end_scroll draws its bar.
pub struct ScrollArea {
    id: Id,
    name: String,
    pub viewport: Rect,
    // Where the content goes: the viewport's width less the bar's, as tall
    // as the content, moved up by the offset.
    pub content: Rect,
    pub offset: f32,
    content_height: f32,
}

impl Ui {
    pub fn begin_scroll(&mut self, name: &str, viewport: Rect, content_height: f32) -> ScrollArea {
        let scroll_id = id(name);
        let range = (content_height - viewport.height).max(0.0);
        let mut offset = *self.memory::<f32>(scroll_id);
        // The wheel scrolls it while the pointer is over it, unless a widget
        // under the pointer takes the wheel itself, like a slider, or a menu
        // or a dialog is over it.
        if self.wheel_over(viewport) {
            offset -= self.input.wheel.y * WHEEL_SCROLL;
        }
        offset = offset.clamp(0.0, range);
        *self.memory::<f32>(scroll_id) = offset;
        self.cover(viewport);
        self.draw.push_clip(viewport, false);
        let bar = if range > 0.0 { SCROLL_BAR_SIZE } else { 0.0 };
        ScrollArea {
            id: scroll_id,
            name: name.to_string(),
            viewport,
            content: Rect::new(viewport.x, viewport.y - offset, viewport.width - bar, content_height),
            offset,
            content_height,
        }
    }

    pub fn end_scroll(&mut self, area: ScrollArea) {
        self.draw.pop_clip();
        let range = (area.content_height - area.viewport.height).max(0.0);
        if range <= 0.0 {
            return;
        }
        let mut offset = area.offset;
        let bar = area.viewport.right_strip(SCROLL_BAR_SIZE);
        self.add(&format!("{}/ScrollBar", area.name), bar, ScrollBar::new(&mut offset, area.content_height, area.viewport.height));
        *self.memory::<f32>(area.id) = offset;
    }
}
