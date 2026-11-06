// WinUI's ListView: rows that scroll, of which one may be selected. Only the
// rows in sight are shown, so a list may be long. Each row is a target named
// "<name>/<index>", which the program draws in, given the row's content
// area inside ListViewItem's padding, and its state.
//
// A row's backplate is inset from it by 4 and 2, with a subtle fill under the
// pointer, and the selected row has the accent's pill at its left. With
// focus, the arrows, Home, End, Page Up and Page Down move the selection,
// and Enter activates the selected row, as a double-click does.

use crate::core::ui::{Response, Sense, Ui};
use crate::design::controls::{
    LIST_ITEM_BACKPLATE_MARGIN, LIST_ITEM_PADDING, LIST_ITEM_RADIUS, SCROLL_BAR_SIZE, SELECTION_PILL_LENGTH, SELECTION_PILL_PRESSED_LENGTH,
    SELECTION_PILL_THICKNESS,
};
use crate::design::tokens::{CURVE_FAST_OUT_SLOW_IN, DURATION_FAST, DURATION_FASTER};
use crate::drawing::geometry::Rect;
use crate::input::devices::Key;
use crate::widgets::scrolling::ScrollBar;
use crate::widgets::widget::Widget;

// Rows that a notch of the wheel scrolls.
pub const WHEEL_ROWS: f32 = 3.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct RowState {
    pub hovered: bool,
    pub pressed: bool,
    pub selected: bool,
    // The list has keyboard focus, and this is its selected row.
    pub focused: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ListResponse {
    // The list's own, which has its focus.
    pub response: Response,
    // The selection changed.
    pub changed: bool,
    // A row was clicked, or double-clicked or activated with Enter.
    pub clicked: Option<usize>,
    pub activated: Option<usize>,
    // The row under the pointer, and the row that was right-clicked.
    pub hovered: Option<usize>,
    pub context: Option<usize>,
}

pub struct ListView<'v, F: FnMut(&mut Ui, usize, Rect, RowState)> {
    count: usize,
    row_height: f32,
    selected: Option<&'v mut Option<usize>>,
    draw: F,
}

impl<'v, F: FnMut(&mut Ui, usize, Rect, RowState)> ListView<'v, F> {
    // `count` rows, `row_height` tall, each drawn by `draw` with its index,
    // its content area and its state.
    pub fn new(count: usize, row_height: f32, draw: F) -> Self {
        Self {
            count,
            row_height,
            selected: None,
            draw,
        }
    }

    // Rows can be selected, one at a time.
    pub fn selection(mut self, selected: &'v mut Option<usize>) -> Self {
        self.selected = Some(selected);
        self
    }
}

impl<F: FnMut(&mut Ui, usize, Rect, RowState)> Widget for ListView<'_, F> {
    type Response = ListResponse;

    fn show(mut self, ui: &mut Ui, name: &str, rect: Rect) -> ListResponse {
        let selectable = self.selected.is_some();
        let sense = if selectable { Sense::NAVIGATE } else { Sense::NONE };
        let response = ui.interact(name, rect, sense, "ListView");
        let mut result = ListResponse {
            response,
            ..Default::default()
        };
        let row_height = self.row_height.max(1.0);
        let content = self.count as f32 * row_height;
        let range = (content - rect.height).max(0.0);
        let mut offset = *ui.memory::<f32>(response.id);
        let mut selected = self.selected.as_deref().copied().flatten();
        let before = selected;

        if response.focused && selectable && self.count > 0 {
            let page = ((rect.height / row_height).floor() as usize).max(1);
            let input = ui.input();
            let last = self.count - 1;
            let current = selected.unwrap_or(0);
            let mut moved = None;
            if input.key_pressed(Key::DOWN) {
                moved = Some(if selected.is_none() { 0 } else { (current + 1).min(last) });
            }
            if input.key_pressed(Key::UP) {
                moved = Some(current.saturating_sub(1));
            }
            if input.key_pressed(Key::PAGE_DOWN) {
                moved = Some((current + page).min(last));
            }
            if input.key_pressed(Key::PAGE_UP) {
                moved = Some(current.saturating_sub(page));
            }
            if input.key_pressed(Key::HOME) {
                moved = Some(0);
            }
            if input.key_pressed(Key::END) {
                moved = Some(last);
            }
            if input.key_pressed_once(Key::ENTER) && let Some(row) = selected {
                result.activated = Some(row);
            }
            if let Some(row) = moved {
                selected = Some(row);
                // The selected row scrolls into sight.
                let top = row as f32 * row_height;
                if top < offset {
                    offset = top;
                } else if top + row_height > offset + rect.height {
                    offset = top + row_height - rect.height;
                }
            }
        }
        if ui.pointer_in(rect) && ui.input().wheel.y != 0.0 && !ui.hovered_takes_wheel() {
            offset -= ui.input().wheel.y * WHEEL_ROWS * row_height;
        }
        offset = offset.clamp(0.0, range);

        let bar_width = if range > 0.0 { SCROLL_BAR_SIZE } else { 0.0 };
        let rows_area = Rect::new(rect.x, rect.y, rect.width - bar_width, rect.height);
        ui.draw.push_clip(rect, false);
        let first = (offset / row_height).floor() as usize;
        let last = (((offset + rect.height) / row_height).ceil() as usize).min(self.count);
        for index in first..last {
            let row = Rect::new(rows_area.x, rect.y + index as f32 * row_height - offset, rows_area.width, row_height);
            let row_name = format!("{name}/{index}");
            let row_response = ui.interact(&row_name, row, Sense::CLICK, "ListViewItem");
            if row_response.hovered {
                result.hovered = Some(index);
            }
            if row_response.pressed && selectable {
                selected = Some(index);
                if ui.focusable() {
                    ui.focus(name);
                    ui.set_focus_visible(false);
                }
            }
            if row_response.clicked {
                result.clicked = Some(index);
            }
            if row_response.double_clicked {
                result.activated = Some(index);
            }
            if row_response.secondary_pressed {
                result.context = Some(index);
            }
            let is_selected = selected == Some(index);
            let pressed = row_response.held && row_response.hovered;
            let theme = ui.theme();
            // ListViewItemBackground.
            let fill = match (is_selected, pressed, row_response.hovered) {
                (false, true, _) => theme.subtle_fill_tertiary,
                (false, false, true) => theme.subtle_fill_secondary,
                (false, false, false) => theme.subtle_fill_transparent,
                (true, true, _) => theme.subtle_fill_secondary,
                (true, false, true) => theme.subtle_fill_tertiary,
                (true, false, false) => theme.subtle_fill_secondary,
            };
            let indicator = if ui.enabled() { theme.accent } else { theme.accent_disabled };
            let fill = ui.animate_color(row_response.id, 0, fill, DURATION_FASTER);
            let pill = ui.animate(row_response.id, 1, if pressed { SELECTION_PILL_PRESSED_LENGTH } else { SELECTION_PILL_LENGTH }, DURATION_FAST, CURVE_FAST_OUT_SLOW_IN);
            let plate = row.pad(LIST_ITEM_BACKPLATE_MARGIN);
            {
                let mut painter = ui.painter();
                painter.rounded(plate, LIST_ITEM_RADIUS, fill);
                if is_selected {
                    let bar = Rect::new(plate.x, plate.center().y - pill / 2.0, SELECTION_PILL_THICKNESS, pill);
                    painter.rounded(bar, SELECTION_PILL_THICKNESS / 2.0, indicator);
                }
            }
            let state = RowState {
                hovered: row_response.hovered,
                pressed,
                selected: is_selected,
                focused: is_selected && response.focused,
            };
            (self.draw)(ui, index, row.pad(LIST_ITEM_PADDING), state);
            if state.focused {
                ui.focus_visual_around(true, plate, LIST_ITEM_RADIUS, crate::drawing::geometry::Insets::all(-1.0));
            }
        }
        ui.draw.pop_clip();
        if range > 0.0 {
            let bar = rect.right_strip(SCROLL_BAR_SIZE);
            ui.add(&format!("{name}/ScrollBar"), bar, ScrollBar::new(&mut offset, content, rect.height));
        }
        *ui.memory::<f32>(response.id) = offset;
        if let Some(target) = self.selected.as_deref_mut() {
            *target = selected;
        }
        result.changed = selected != before;
        result.response.changed = result.changed;
        result
    }
}
