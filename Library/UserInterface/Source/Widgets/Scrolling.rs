// WinUI's ScrollBar, and a region whose content scrolls behind it, like a
// ScrollViewer. The bar's thumb is a 2-pixel line while the pointer is away,
// and is 6 pixels, over an acrylic track, once the pointer has rested on the
// bar; the wheel scrolls the region under the pointer.
//
// How far something is scrolled is its own, kept between frames, which the
// program reads with Ui::memory to lay out what's in it:
//
//     let content = ScrollArea::content(ui, "Log", viewport, height);
//     tree.container(Some("Log"), viewport, ScrollArea::new(height), |area| {
//         area.draw(Rect::new(content.x, content.y + 40.0, content.width, 20.0), Text::label("A line"));
//     });

use crate::core::interaction::{Hit, Id, Interaction, Part, Sense, id};
use crate::core::interface::Ui;
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{
    SCROLL_BAR_CONTRACT_DELAY, SCROLL_BAR_EXPAND_DELAY, SCROLL_BAR_MIN_THUMB, SCROLL_BAR_RADIUS, SCROLL_BAR_SIZE, SCROLL_BAR_THUMB_COLLAPSED,
    SCROLL_BAR_THUMB_EXPANDED, SCROLL_BAR_THUMB_OFFSET, SCROLL_VIEWER_BAR_MARGIN,
};
use crate::drawing::geometry::{Point, Rect};
use crate::widgets::widget::{Reaction, react};

// Pixels that a notch of the wheel scrolls: three lines of 48.
pub const WHEEL_SCROLL: f32 = 144.0;

// How far something that scrolls is scrolled, which it keeps between frames
// under its name.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Scrolled {
    pub offset: f32,
}

// The part that a widget's bar is, named "<widget>/ScrollBar".
const BAR: &str = "ScrollBar";

pub(crate) fn bar_part() -> Part {
    Part::Named(BAR.to_string())
}

fn bar_id(widget: &str) -> Id {
    id(&format!("{widget}/{BAR}"))
}

// When the pointer came onto the bar or left it, for its delays, and where
// in the thumb a drag took it.
#[derive(Default)]
struct BarState {
    over_since: Option<f64>,
    away_since: Option<f64>,
    expanded: bool,
    grab: f32,
}

// A bar's track and thumb, for content `content` tall seen through
// `viewport`.
struct BarGeometry {
    track: Rect,
    thumb_length: f32,
    travel: f32,
    range: f32,
}

impl BarGeometry {
    fn new(bar: Rect, content: f32, viewport: f32) -> Self {
        let track = bar.inset(SCROLL_VIEWER_BAR_MARGIN);
        let thumb_length = if content > 0.0 { (track.height * viewport / content).clamp(SCROLL_BAR_MIN_THUMB.min(track.height), track.height) } else { track.height };
        Self {
            track,
            thumb_length,
            travel: (track.height - thumb_length).max(0.0),
            range: (content - viewport).max(0.0),
        }
    }

    fn thumb_y(&self, offset: f32) -> f32 {
        self.track.y + if self.range > 0.0 { self.travel * offset / self.range } else { 0.0 }
    }
}

// Registers a widget's bar along its viewport's right edge, if its content
// is taller than the viewport.
pub(crate) fn scroll_bar_part(ctx: &mut Parts, viewport: Rect, content: f32) {
    if content > viewport.height {
        ctx.part(bar_part(), BAR, viewport.right_strip(SCROLL_BAR_SIZE), Sense::CLICK, "ScrollBar", None);
    }
}

// What the input on a bar does: a press on the thumb drags it, and a press
// on the track pages toward it. Returns the offset as the input leaves it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn bar_input<M>(ctx: &mut Dispatch<M>, state_id: Id, hit: &Hit, pointer: Option<Point>, bar: Rect, content: f32, viewport: f32, offset: f32) -> f32 {
    let geometry = BarGeometry::new(bar, content, viewport);
    let mut offset = offset;
    let state = ctx.memory_of::<BarState>(state_id);
    if hit.pressed && let Some(pointer) = pointer {
        let top = geometry.thumb_y(offset);
        if pointer.y >= top && pointer.y < top + geometry.thumb_length {
            state.grab = pointer.y - top;
        } else {
            let page = viewport.max(1.0);
            offset = if pointer.y < top { offset - page } else { offset + page };
            state.grab = geometry.thumb_length / 2.0;
        }
    } else if hit.held && let Some(pointer) = pointer {
        let top = pointer.y - state.grab;
        offset = if geometry.travel > 0.0 { (top - geometry.track.y) / geometry.travel * geometry.range } else { 0.0 };
    }
    offset.clamp(0.0, geometry.range)
}

// Draws a bar: expanded after the pointer has rested on it, and collapsed
// a while after the pointer left. `over` is whether the pointer is on it or
// holds it this frame.
pub(crate) fn bar_draw(ctx: &mut Draw, state_id: Id, over: bool, bar: Rect, content: f32, viewport: f32, offset: f32) {
    let geometry = BarGeometry::new(bar, content, viewport);
    if geometry.range <= 0.0 {
        return;
    }
    let now = ctx.time();
    let enabled = ctx.enabled();
    let state = ctx.memory_of::<BarState>(state_id);
    if over {
        state.away_since = None;
        let since = *state.over_since.get_or_insert(now);
        if now - since >= SCROLL_BAR_EXPAND_DELAY {
            state.expanded = true;
        }
    } else {
        state.over_since = None;
        let since = *state.away_since.get_or_insert(now);
        if now - since >= SCROLL_BAR_CONTRACT_DELAY {
            state.expanded = false;
        }
    }
    let expanded = state.expanded;
    // A collapsed thumb sits toward the outer edge.
    let (width, center_x) = if expanded {
        (SCROLL_BAR_THUMB_EXPANDED, geometry.track.center().x)
    } else {
        (SCROLL_BAR_THUMB_COLLAPSED, geometry.track.center().x + SCROLL_BAR_THUMB_OFFSET)
    };
    let mut painter = ctx.painter();
    let theme = painter.theme();
    if expanded {
        painter.rounded(geometry.track, SCROLL_BAR_RADIUS * 2.0, theme.acrylic);
    }
    let thumb = Rect::new(center_x - width / 2.0, geometry.thumb_y(offset.clamp(0.0, geometry.range)), width, geometry.thumb_length);
    let color = if enabled { theme.control_strong_fill } else { theme.control_strong_fill_disabled };
    painter.rounded(thumb, SCROLL_BAR_RADIUS.min(width / 2.0), color);
}

// A vertical scroll bar on its own, for content `content` tall, seen
// through a viewport `viewport` tall, scrolled down by `offset`.
pub struct ScrollBar<M> {
    offset: f32,
    content: f32,
    viewport: f32,
    on_scroll: Option<Reaction<f32, M>>,
}

impl<M: Clone + 'static> ScrollBar<M> {
    pub fn new(offset: f32, content: f32, viewport: f32) -> Self {
        Self {
            offset,
            content,
            viewport,
            on_scroll: None,
        }
    }

    // What moving it sends, with the offset.
    pub fn on_scroll(mut self, reaction: impl Fn(f32) -> M + 'static) -> Self {
        self.on_scroll = Some(react(reaction));
        self
    }
}

impl<M: Clone + 'static> Widget<M> for ScrollBar<M> {
    fn kind(&self) -> &'static str {
        "ScrollBar"
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::CLICK)
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let hit = interaction.whole();
        let rect = ctx.rect();
        let id = ctx.id();
        let offset = bar_input(ctx, id, &hit, interaction.pointer, rect, self.content, self.viewport, self.offset);
        if offset != self.offset
            && let Some(reaction) = &self.on_scroll
        {
            ctx.send(reaction(offset));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let id = ctx.id();
        let over = ctx.look().hit.hovered || ctx.look().hit.held;
        bar_draw(ctx, id, over, rect, self.content, self.viewport, self.offset);
    }
}

// A region whose content scrolls: its children are clipped to it and go
// where `content` says, moved up by how far it's scrolled, and its bar is
// along its right edge, named "<name>/ScrollBar". The wheel over it scrolls
// it, unless a widget under the pointer takes the wheel itself, like a
// slider, or a menu or a dialog is over it.
pub struct ScrollArea {
    content_height: f32,
}

impl ScrollArea {
    pub fn new(content_height: f32) -> Self {
        Self { content_height }
    }

    // Where the content of the scroll area with the name goes: the
    // viewport's width less the bar's, as tall as the content, moved up by
    // how far it's scrolled.
    pub fn content(ui: &mut Ui, name: &str, viewport: Rect, content_height: f32) -> Rect {
        let range = (content_height - viewport.height).max(0.0);
        let offset = ui.memory::<Scrolled>(name).offset.clamp(0.0, range);
        let bar = if range > 0.0 { SCROLL_BAR_SIZE } else { 0.0 };
        Rect::new(viewport.x, viewport.y - offset, viewport.width - bar, content_height)
    }
}

impl<M: 'static> Widget<M> for ScrollArea {
    fn kind(&self) -> &'static str {
        "ScrollViewer"
    }

    fn covers(&self) -> bool {
        true
    }

    fn parts(&self, ctx: &mut Parts) {
        let rect = ctx.rect();
        scroll_bar_part(ctx, rect, self.content_height);
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let rect = ctx.rect();
        let range = (self.content_height - rect.height).max(0.0);
        let mut offset = ctx.memory::<Scrolled>().offset;
        offset -= interaction.wheel_over.y * WHEEL_SCROLL;
        if let Some(hit) = interaction.hit(&bar_part()) {
            let state_id = bar_id(ctx.name());
            offset = bar_input(ctx, state_id, hit, interaction.pointer, rect.right_strip(SCROLL_BAR_SIZE), self.content_height, rect.height, offset);
        }
        ctx.memory::<Scrolled>().offset = offset.clamp(0.0, range);
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let offset = ctx.memory::<Scrolled>().offset;
        let part = bar_part();
        let over = ctx.look().hovered(&part) || ctx.look().held(&part);
        let state_id = bar_id(ctx.name());
        bar_draw(ctx, state_id, over, rect.right_strip(SCROLL_BAR_SIZE), self.content_height, rect.height, offset);
    }

    fn takes_free_wheel(&self) -> bool {
        true
    }

    fn children_clip(&self, ctx: &mut Parts) -> Option<Rect> {
        Some(ctx.rect())
    }
}
