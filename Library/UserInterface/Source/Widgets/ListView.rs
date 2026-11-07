// WinUI's ListView: rows that scroll, of which one may be selected. Only the
// rows in sight are shown, so a list may be long. The list owns its rows as
// data: an icon, a line of text, and a trailing caption, or a painter of the
// program's own over that data. Each row is a target named "<name>/<index>",
// labeled with its text.
//
// A row's backplate is inset from it by 4 and 2, with a subtle fill under the
// pointer, and the selected row has the accent's pill at its left. With
// focus, the arrows, Home, End, Page Up and Page Down move the selection,
// and Enter activates the selected row, as a double-click does. Selecting
// emits Selected, and opening a row emits Activated.

use crate::core::control::{Control, NodeContext, UiEventKind, adopted};
use crate::core::ui::{Response, Sense, Ui};
use crate::design::controls::{
    LIST_ITEM_BACKPLATE_MARGIN, LIST_ITEM_PADDING, LIST_ITEM_RADIUS, SCROLL_BAR_SIZE, SELECTION_PILL_LENGTH, SELECTION_PILL_PRESSED_LENGTH,
    SELECTION_PILL_THICKNESS,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, CAPTION, CURVE_FAST_OUT_SLOW_IN, DURATION_FAST, DURATION_FASTER, ICON, SPACE_M};
use crate::drawing::geometry::Rect;
use crate::drawing::painter::Align;
use crate::input::devices::Key;
use crate::widgets::scrolling::{scroll_bar_input, scroll_bar_paint};

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

// What a row shows: an icon, its text, and a trailing caption, like "#12",
// with emphasis hints for a painter of the program's own.
#[derive(Clone, Debug, Default)]
pub struct ListRow {
    pub icon: Option<Icon>,
    pub text: String,
    pub trailing: String,
    // Emphasized, like the device a card listens to.
    pub accent: bool,
    // De-emphasized, like a device that's ignored.
    pub faint: bool,
}

impl ListRow {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            ..Self::default()
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn trailing(mut self, trailing: &str) -> Self {
        self.trailing = trailing.to_string();
        self
    }

    pub fn accent(mut self, accent: bool) -> Self {
        self.accent = accent;
        self
    }

    pub fn faint(mut self, faint: bool) -> Self {
        self.faint = faint;
        self
    }
}

pub struct ListView {
    rows: Vec<ListRow>,
    row_height: f32,
    selectable: bool,
    selected: Option<usize>,
    offset: f32,
    // A painter of the program's own for a row's content, over the default.
    painter: Option<fn(&mut Ui, usize, Rect, RowState, &ListRow)>,
    // What this frame's dispatch saw, for the program and for draw.
    hovered: Option<usize>,
    context: Option<usize>,
    shown: Vec<(usize, Response)>,
    bar: Option<Response>,
    // The rows changed after dispatch, so draw re-registers them, and they
    // are targets the same frame.
    stale: bool,
}

impl ListView {
    pub fn new(rows: Vec<ListRow>, row_height: f32) -> Self {
        Self {
            rows,
            row_height: row_height.max(1.0),
            selectable: false,
            selected: None,
            offset: 0.0,
            painter: None,
            hovered: None,
            context: None,
            shown: Vec::new(),
            bar: None,
            stale: false,
        }
    }

    // Rows can be selected, one at a time.
    pub fn selectable(mut self) -> Self {
        self.selectable = true;
        self
    }

    // A painter of the program's own for a row's content, given its index,
    // its content area inside ListViewItem's padding, its state and its row.
    pub fn painter(mut self, painter: fn(&mut Ui, usize, Rect, RowState, &ListRow)) -> Self {
        self.painter = Some(painter);
        self
    }

    pub fn rows(&self) -> &[ListRow] {
        &self.rows
    }

    // Swaps the rows, keeping the selection if it still fits.
    pub fn set_rows(&mut self, rows: Vec<ListRow>) {
        self.rows = rows;
        if self.selected.is_some_and(|selected| selected >= self.rows.len()) {
            self.selected = None;
        }
        let range = (self.rows.len() as f32 * self.row_height).max(0.0);
        self.offset = self.offset.min(range);
        self.stale = true;
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn set_selected(&mut self, selected: Option<usize>) {
        self.selected = selected.filter(|selected| *selected < self.rows.len());
    }

    // The row under the pointer this frame, and the row that was
    // right-clicked.
    pub fn hovered_row(&self) -> Option<usize> {
        self.hovered
    }

    pub fn context_row(&self) -> Option<usize> {
        self.context
    }
}

impl Control for ListView {
    fn kind(&self) -> &'static str {
        "ListView"
    }

    fn sense(&self) -> Sense {
        Sense::NAVIGATE
    }

    fn adopt(&mut self, previous: Box<dyn Control>) {
        if let Some(previous) = adopted::<ListView>(previous) {
            self.selected = previous.selected.filter(|selected| *selected < self.rows.len());
            self.offset = previous.offset;
        }
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let (rect, response) = (ctx.rect, &ctx.response);
        let count = self.rows.len();
        let row_height = self.row_height;
        let content = count as f32 * row_height;
        let range = (content - rect.height).max(0.0);
        let before = self.selected;
        self.hovered = None;
        self.context = None;

        if response.focused && self.selectable && count > 0 {
            let page = ((rect.height / row_height).floor() as usize).max(1);
            let input = ui.input();
            let last = count - 1;
            let current = self.selected.unwrap_or(0);
            let mut moved = None;
            if input.key_pressed(Key::DOWN) {
                moved = Some(if self.selected.is_none() { 0 } else { (current + 1).min(last) });
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
            if input.key_pressed_once(Key::ENTER) && let Some(row) = self.selected {
                ctx.emit(ui, UiEventKind::Activated(row));
            }
            if let Some(row) = moved {
                self.selected = Some(row);
                // The selected row scrolls into sight.
                let top = row as f32 * row_height;
                if top < self.offset {
                    self.offset = top;
                } else if top + row_height > self.offset + rect.height {
                    self.offset = top + row_height - rect.height;
                }
            }
        }
        if ui.wheel_over(rect) {
            self.offset -= ui.input().wheel.y * WHEEL_ROWS * row_height;
        }
        self.offset = self.offset.clamp(0.0, range);

        let bar_width = if range > 0.0 { SCROLL_BAR_SIZE } else { 0.0 };
        let rows_area = Rect::new(rect.x, rect.y, rect.width - bar_width, rect.height);
        let first = (self.offset / row_height).floor() as usize;
        let last = (((self.offset + rect.height) / row_height).ceil() as usize).min(count);
        self.shown.clear();
        for index in first..last {
            let row = Rect::new(rows_area.x, rect.y + index as f32 * row_height - self.offset, rows_area.width, row_height);
            let row_name = format!("{}/{index}", ctx.name);
            let row_response = ui.interact(&row_name, row.intersect(rect), Sense::CLICK, "ListViewItem");
            ui.set_label(&row_name, &self.rows[index].text);
            if row_response.hovered {
                self.hovered = Some(index);
            }
            if row_response.pressed && self.selectable {
                self.selected = Some(index);
                if ui.focusable() {
                    ui.focus(&ctx.name);
                    ui.set_focus_visible(false);
                }
            }
            if row_response.double_clicked {
                ctx.emit(ui, UiEventKind::Activated(index));
            }
            if row_response.secondary_pressed {
                self.context = Some(index);
            }
            self.shown.push((index, row_response));
        }
        self.bar = None;
        if range > 0.0 {
            let bar = rect.right_strip(SCROLL_BAR_SIZE);
            let (bar_response, _) = scroll_bar_input(ui, &format!("{}/ScrollBar", ctx.name), bar, &mut self.offset, content, rect.height);
            self.bar = Some(bar_response);
        }
        self.stale = false;
        if self.selected != before {
            if let Some(selected) = self.selected {
                ctx.emit(ui, UiEventKind::Selected(selected));
            }
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let (rect, response) = (ctx.rect, &ctx.response);
        let count = self.rows.len();
        let row_height = self.row_height;
        let content = count as f32 * row_height;
        let range = (content - rect.height).max(0.0);
        let bar_width = if range > 0.0 { SCROLL_BAR_SIZE } else { 0.0 };
        let rows_area = Rect::new(rect.x, rect.y, rect.width - bar_width, rect.height);
        // Rows that update swapped in missed dispatch, so they register here,
        // and are targets the same frame. No input landed on them yet.
        if self.stale {
            self.stale = false;
            self.offset = self.offset.clamp(0.0, range);
            ui.remove_targets(&format!("{}/", ctx.name), "ListViewItem");
            let first = (self.offset / row_height).floor() as usize;
            let last = (((self.offset + rect.height) / row_height).ceil() as usize).min(count);
            self.shown.clear();
            for index in first..last {
                let row = Rect::new(rows_area.x, rect.y + index as f32 * row_height - self.offset, rows_area.width, row_height);
                let row_name = format!("{}/{index}", ctx.name);
                let row_response = ui.interact(&row_name, row.intersect(rect), Sense::CLICK, "ListViewItem");
                ui.set_label(&row_name, &self.rows[index].text);
                self.shown.push((index, row_response));
            }
            if range > 0.0 && self.bar.is_none() {
                let bar = rect.right_strip(SCROLL_BAR_SIZE);
                let (bar_response, _) = scroll_bar_input(ui, &format!("{}/ScrollBar", ctx.name), bar, &mut self.offset, content, rect.height);
                self.bar = Some(bar_response);
            }
        }
        ui.draw.push_clip(rect, false);
        for (index, row_response) in self.shown.clone() {
            let index: usize = index;
            if index >= count {
                continue;
            }
            let row = Rect::new(rows_area.x, rect.y + index as f32 * row_height - self.offset, rows_area.width, row_height);
            let is_selected = self.selected == Some(index);
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
            let content_rect = row.pad(LIST_ITEM_PADDING);
            let data = self.rows[index].clone();
            match self.painter {
                Some(painter) => painter(ui, index, content_rect, state, &data),
                None => default_row(ui, content_rect, &data),
            }
            if state.focused {
                ui.focus_visual_around(true, plate, LIST_ITEM_RADIUS, crate::drawing::geometry::Insets::all(-1.0));
            }
        }
        ui.draw.pop_clip();
        if let Some(bar_response) = self.bar {
            let bar = rect.right_strip(SCROLL_BAR_SIZE);
            scroll_bar_paint(ui, &bar_response, bar, self.offset, content, rect.height);
        }
    }
}

// A row's default look: its icon, its text, and its trailing caption.
fn default_row(ui: &mut Ui, rect: Rect, row: &ListRow) {
    let theme = ui.theme();
    let (primary, tertiary) = (theme.text_primary, theme.text_tertiary);
    let mut x = rect.x;
    if let Some(icon) = row.icon {
        ui.icon(Rect::new(rect.x, rect.y, ICON, rect.height), icon, ICON, primary);
        x += ICON + SPACE_M;
    }
    let trailing_width = if row.trailing.is_empty() { 0.0 } else { ui.measure(&row.trailing, CAPTION).width + SPACE_M };
    ui.text(Rect::new(x, rect.y, (rect.right() - x - trailing_width).max(0.0), rect.height), &row.text, BODY, primary);
    if !row.trailing.is_empty() {
        ui.text_aligned(rect, &row.trailing, CAPTION, tertiary, Align::End);
    }
}
