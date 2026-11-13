// WinUI's ListView: rows that scroll, of which one may be selected. Only the
// rows in sight are shown, so a list may be long. Each row is a target named
// "<name>/<index>", with its label, and what's in it is the list's children,
// which the program lays out in the rows that `rows` gives, inside
// ListViewItem's padding:
//
//     let list = ListView::new(count, LIST_ITEM_HEIGHT).selection(self.selected).on_select(Message::Select);
//     let rows = list.rows(ui, "Devices", rect);
//     tree.container(Some("Devices"), rect, list, |list| {
//         for index in rows.indices() {
//             list.draw(rows.content(index), Text::label(&names[index]));
//         }
//     });
//
// A row's backplate is inset from it by 4 and 2, with a subtle fill under the
// pointer, and the selected row has the accent's pill at its left.

use std::ops::Range;

use crate::core::interaction::{ControlState, Interaction, Part, Sense, id};
use crate::core::interface::Ui;
use crate::core::tree::{Dispatch, Draw, Parts, Tooltip, Widget};
use crate::design::controls::{
    LIST_ITEM_BACKPLATE_MARGIN, LIST_ITEM_PADDING, LIST_ITEM_RADIUS, SCROLL_BAR_SIZE, SELECTION_PILL_LENGTH, SELECTION_PILL_PRESSED_LENGTH,
    SELECTION_PILL_THICKNESS,
};
use crate::drawing::geometry::Rect;
use crate::widgets::scrolling::{Scrolled, bar_draw, bar_input, bar_part, scroll_bar_part};
use crate::widgets::widget::{Reaction, react};

// Rows that a notch of the wheel scrolls.
pub const WHEEL_ROWS: f32 = 3.0;

pub struct ListView<M> {
    count: usize,
    row_height: f32,
    selectable: bool,
    selected: Option<usize>,
    on_select: Option<Reaction<usize, M>>,
    on_activate: Option<Reaction<usize, M>>,
    on_context: Option<Reaction<usize, M>>,
    label_of: Option<Box<dyn Fn(usize) -> String>>,
    tooltip_of: Option<Box<dyn Fn(usize) -> Option<Tooltip>>>,
}

// The rows of a list in sight, and where each is.
#[derive(Clone, Copy, Debug)]
pub struct ListRows {
    pub first: usize,
    // One past the last.
    pub last: usize,
    pub offset: f32,
    // The rows' area: the list's, less its bar.
    pub area: Rect,
    list: Rect,
    row_height: f32,
}

impl ListRows {
    pub fn indices(&self) -> Range<usize> {
        self.first..self.last
    }

    // A row's whole area, as scrolled.
    pub fn row(&self, index: usize) -> Rect {
        Rect::new(self.area.x, self.list.y + index as f32 * self.row_height - self.offset, self.area.width, self.row_height)
    }

    // A row's area for what's in it, inside ListViewItem's padding.
    pub fn content(&self, index: usize) -> Rect {
        self.row(index).pad(LIST_ITEM_PADDING)
    }
}

impl<M: Clone + 'static> ListView<M> {
    // `count` rows, `row_height` tall.
    pub fn new(count: usize, row_height: f32) -> Self {
        Self {
            count,
            row_height,
            selectable: false,
            selected: None,
            on_select: None,
            on_activate: None,
            on_context: None,
            label_of: None,
            tooltip_of: None,
        }
    }

    // Rows can be selected, one at a time, and this one is. A selection
    // past the rows, like one that a program keeps of a list that got
    // shorter, is none: the list never draws it, moves from it, or gives it
    // to what the program set with on_activate, labels or tooltips.
    pub fn selection(mut self, selected: Option<usize>) -> Self {
        self.selectable = true;
        self.selected = selected.filter(|index| *index < self.count);
        self
    }

    // What selecting a row sends, with its index.
    pub fn on_select(mut self, reaction: impl Fn(usize) -> M + 'static) -> Self {
        self.on_select = Some(react(reaction));
        self
    }

    // What a double-click on a row sends.
    pub fn on_activate(mut self, reaction: impl Fn(usize) -> M + 'static) -> Self {
        self.on_activate = Some(react(reaction));
        self
    }

    // What a right-click on a row sends.
    pub fn on_context(mut self, reaction: impl Fn(usize) -> M + 'static) -> Self {
        self.on_context = Some(react(reaction));
        self
    }

    // Each row's label, for scripts and tests.
    pub fn labels(mut self, label_of: impl Fn(usize) -> String + 'static) -> Self {
        self.label_of = Some(Box::new(label_of));
        self
    }

    // Each row's tooltip, if it has one.
    pub fn tooltips(mut self, tooltip_of: impl Fn(usize) -> Option<Tooltip> + 'static) -> Self {
        self.tooltip_of = Some(Box::new(tooltip_of));
        self
    }

    // The rows in sight of the list with the name in the rectangle, as it's
    // scrolled, for laying out what's in them.
    pub fn rows(&self, ui: &mut Ui, name: &str, rect: Rect) -> ListRows {
        let offset = ui.memory::<Scrolled>(name).offset;
        self.geometry(rect, offset)
    }

    fn content_height(&self) -> f32 {
        self.count as f32 * self.row_height.max(1.0)
    }

    fn geometry(&self, rect: Rect, offset: f32) -> ListRows {
        let row_height = self.row_height.max(1.0);
        let range = (self.content_height() - rect.height).max(0.0);
        let offset = offset.clamp(0.0, range);
        let bar_width = if range > 0.0 { SCROLL_BAR_SIZE } else { 0.0 };
        let area = Rect::new(rect.x, rect.y, rect.width - bar_width, rect.height);
        let first = ((offset / row_height).floor() as usize).min(self.count);
        let last = (((offset + rect.height) / row_height).ceil() as usize).min(self.count).max(first);
        ListRows {
            first,
            last,
            offset,
            area,
            list: rect,
            row_height,
        }
    }
}

impl<M: Clone + 'static> Widget<M> for ListView<M> {
    fn kind(&self) -> &'static str {
        "ListView"
    }

    fn label(&self) -> Option<String> {
        self.selected.and_then(|index| self.label_of.as_ref().map(|label_of| label_of(index)))
    }

    fn sense(&self) -> Option<Sense> {
        Some(if self.selectable { Sense::CLICK } else { Sense::NONE })
    }

    fn parts(&self, ctx: &mut Parts) {
        let rect = ctx.rect();
        let offset = ctx.memory::<Scrolled>().offset;
        let rows = self.geometry(rect, offset);
        for index in rows.indices() {
            let label = self.label_of.as_ref().map(|label_of| label_of(index));
            ctx.part(Part::Index(index), &index.to_string(), rows.row(index).intersect(rect), Sense::CLICK, "ListViewItem", label);
        }
        scroll_bar_part(ctx, rect, self.content_height());
    }

    fn part_tooltip(&self, part: &Part) -> Option<Tooltip> {
        let index = part.index()?;
        self.tooltip_of.as_ref().and_then(|tooltip_of| tooltip_of(index))
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let rect = ctx.rect();
        let mut offset = ctx.memory::<Scrolled>().offset;
        let rows = self.geometry(rect, offset);
        offset = rows.offset;
        let mut selected = self.selected;
        let before = selected;
        let mut activated = None;
        let mut context = None;
        offset -= interaction.wheel_over.y * WHEEL_ROWS * rows.row_height;
        for (part, hit) in &interaction.hits {
            let Part::Index(index) = part else { continue };
            if hit.pressed && self.selectable {
                selected = Some(*index);
            }
            if hit.double_clicked {
                activated = Some(*index);
            }
            if hit.secondary_pressed {
                context = Some(*index);
            }
        }
        if let Some(hit) = interaction.hit(&bar_part()) {
            let state_id = id(&format!("{}/ScrollBar", ctx.name()));
            offset = bar_input(ctx, state_id, hit, interaction.pointer, rect.right_strip(SCROLL_BAR_SIZE), self.content_height(), rect.height, offset);
        }
        let range = (self.content_height() - rect.height).max(0.0);
        ctx.memory::<Scrolled>().offset = offset.clamp(0.0, range);

        if selected != before
            && let Some(index) = selected
            && let Some(reaction) = &self.on_select
        {
            ctx.send(reaction(index));
        }
        if let Some(index) = activated
            && let Some(reaction) = &self.on_activate
        {
            ctx.send(reaction(index));
        }
        if let Some(index) = context
            && let Some(reaction) = &self.on_context
        {
            ctx.send(reaction(index));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let look = ctx.look().clone();
        let offset = ctx.memory::<Scrolled>().offset;
        let rows = self.geometry(rect, offset);
        let enabled = ctx.enabled();
        {
            let mut painter = ctx.painter();
            let theme = painter.theme();
            painter.push_clip(rect);
            for index in rows.indices() {
                let part = Part::Index(index);
                let state = look.part_state(&part);
                let is_selected = self.selected == Some(index);
                let pressed = state == ControlState::Pressed;
                let hovered = state == ControlState::PointerOver || pressed;
                // ListViewItemBackground.
                let fill = match (is_selected, pressed, hovered) {
                    (false, true, _) => theme.subtle_fill_tertiary,
                    (false, false, true) => theme.subtle_fill_secondary,
                    (false, false, false) => theme.subtle_fill_transparent,
                    (true, true, _) => theme.subtle_fill_secondary,
                    (true, false, true) => theme.subtle_fill_tertiary,
                    (true, false, false) => theme.subtle_fill_secondary,
                };
                let indicator = if enabled { theme.accent } else { theme.accent_disabled };
                let pill = if pressed { SELECTION_PILL_PRESSED_LENGTH } else { SELECTION_PILL_LENGTH };
                let plate = rows.row(index).pad(LIST_ITEM_BACKPLATE_MARGIN);
                painter.rounded(plate, LIST_ITEM_RADIUS, fill);
                if is_selected {
                    let bar = Rect::new(plate.x, plate.center().y - pill / 2.0, SELECTION_PILL_THICKNESS, pill);
                    painter.rounded(bar, SELECTION_PILL_THICKNESS / 2.0, indicator);
                }
            }
            painter.pop_clip();
        }
        let part = bar_part();
        let over = look.hovered(&part) || look.held(&part);
        let state_id = id(&format!("{}/ScrollBar", ctx.name()));
        bar_draw(ctx, state_id, over, rect.right_strip(SCROLL_BAR_SIZE), self.content_height(), rect.height, offset);
    }

    fn takes_free_wheel(&self) -> bool {
        true
    }

    fn children_clip(&self, ctx: &mut Parts) -> Option<Rect> {
        Some(ctx.rect())
    }
}
