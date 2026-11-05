// A list of rows that scrolls, of which one can be selected, as WinUI's
// ListView. The program draws each row's content; the list draws its
// background, its selection, and its scroll bar, and only shows the rows
// that are in view.
//
// The list is a target named after it, and each row in view is one named
// after the list and the row's index, like "Items/12".

use crate::geometry::{Color, Rect};
use crate::input::Key;
use crate::painter::Corners;
use crate::tokens::{DURATION_FASTER, RADIUS_CONTROL, SELECTION_PILL_LENGTH, SELECTION_PILL_THICKNESS, SPACE_XS};
use crate::ui::{Response, Sense, Ui};
use crate::widgets::common::{Visual, control_focus_ring};

#[derive(Clone, Copy, Debug, Default)]
pub struct RowState {
    pub selected: bool,
    pub hovered: bool,
    pub pressed: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ListResponse {
    pub response: Response,
    // The row a press ended on this frame.
    pub clicked: Option<usize>,
    // The row that was double-clicked, or had Enter pressed on it.
    pub activated: Option<usize>,
    // Whether the selection changed.
    pub changed: bool,
}

#[derive(Default)]
struct ScrollState {
    offset: f32,
    // Where on the thumb its drag began, while it's dragged.
    grab: Option<f32>,
}

// How many rows a notch of the wheel scrolls.
pub const WHEEL_ROWS: f32 = 3.0;
const BAR_THIN: f32 = 2.0;
const BAR_WIDE: f32 = 6.0;

impl Ui {
    pub fn list_view(
        &mut self,
        name: &str,
        rect: Rect,
        count: usize,
        row_height: f32,
        selected: &mut Option<usize>,
        mut row: impl FnMut(&mut Ui, usize, Rect, RowState),
    ) -> ListResponse {
        let response = self.interact(name, rect, Sense::NAVIGATE, "ListView");
        let before = *selected;
        let content = count as f32 * row_height;
        let overflow = (content - rect.height).max(0.0);
        let mut state = std::mem::take(self.memory::<ScrollState>(response.id));
        let enabled = self.enabled();

        // The wheel scrolls anywhere over the list.
        if enabled && self.pointer_in(rect) && !self.any_popup() {
            state.offset -= self.input().wheel.y * WHEEL_ROWS * row_height;
        }
        // The keys move the selection, and scroll it into view.
        let mut activated = None;
        if response.focused && enabled && count > 0 {
            let input = self.input();
            let page = ((rect.height / row_height).floor() as usize).max(1);
            let current = selected.unwrap_or(0);
            let mut target = *selected;
            if input.key_pressed(Key::DOWN) {
                target = Some(selected.map_or(0, |index| (index + 1).min(count - 1)));
            }
            if input.key_pressed(Key::UP) {
                target = Some(current.saturating_sub(1));
            }
            if input.key_pressed(Key::PAGE_DOWN) {
                target = Some((current + page).min(count - 1));
            }
            if input.key_pressed(Key::PAGE_UP) {
                target = Some(current.saturating_sub(page));
            }
            if input.key_pressed(Key::HOME) {
                target = Some(0);
            }
            if input.key_pressed(Key::END) {
                target = Some(count - 1);
            }
            if input.key_pressed_once(Key::ENTER) {
                activated = *selected;
            }
            if target != *selected {
                *selected = target;
                self.set_focus_visible(true);
                if let Some(index) = target {
                    let top = index as f32 * row_height;
                    if top < state.offset {
                        state.offset = top;
                    } else if top + row_height > state.offset + rect.height {
                        state.offset = top + row_height - rect.height;
                    }
                }
            }
        }

        // The scroll bar, which widens while the pointer is near it.
        let bar_area = Rect::new(rect.right() - BAR_WIDE - SPACE_XS, rect.y + SPACE_XS, BAR_WIDE, rect.height - SPACE_XS * 2.0);
        let mut bar_response = None;
        if overflow > 0.0 {
            let thumb_length = (bar_area.height * rect.height / content).max(24.0);
            let travel = bar_area.height - thumb_length;
            let thumb_top = bar_area.y + travel * (state.offset / overflow).clamp(0.0, 1.0);
            let bar = self.interact(&format!("{name}/ScrollBar"), bar_area.expand(2.0), Sense::CLICK, "ScrollBar");
            if bar.pressed
                && let Some(pointer) = self.pointer()
            {
                let on_thumb = pointer.y >= thumb_top && pointer.y < thumb_top + thumb_length;
                state.grab = Some(if on_thumb { pointer.y - thumb_top } else { thumb_length / 2.0 });
            }
            if bar.held {
                if let (Some(grab), Some(pointer)) = (state.grab, self.pointer()) {
                    let t = if travel > 0.0 { (pointer.y - grab - bar_area.y) / travel } else { 0.0 };
                    state.offset = t.clamp(0.0, 1.0) * overflow;
                }
            } else {
                state.grab = None;
            }
            bar_response = Some((bar, thumb_length, travel));
        }
        state.offset = state.offset.clamp(0.0, overflow).round();

        // The rows in view.
        let theme = self.theme().clone();
        let mut clicked = None;
        let mut painter = self.painter();
        painter.push_clip(rect);
        let first = (state.offset / row_height).floor() as usize;
        let last = (((state.offset + rect.height) / row_height).ceil() as usize).min(count);
        for index in first..last {
            let row_rect = Rect::new(rect.x, rect.y + index as f32 * row_height - state.offset, rect.width, row_height);
            let row_name = format!("{name}/{index}");
            let row_response = self.interact(&row_name, row_rect, Sense::CLICK, "ListViewItem");
            if row_response.pressed {
                *selected = Some(index);
                self.focus(name);
            }
            if row_response.double_clicked {
                activated = Some(index);
            }
            if row_response.clicked {
                clicked = Some(index);
            }
            let is_selected = *selected == Some(index);
            let visual = Visual::of(self, &row_response);
            let fill = match (visual, is_selected) {
                (Visual::Pressed, _) => theme.subtle_fill_tertiary,
                (Visual::Hover, true) => theme.subtle_fill_tertiary,
                (Visual::Hover, false) | (_, true) => theme.subtle_fill_secondary,
                _ => Color::TRANSPARENT,
            };
            let fill = self.animate_color(row_response.id, 0, fill, DURATION_FASTER);
            let background = row_rect.inset_by(SPACE_XS, 2.0, SPACE_XS, 2.0);
            let mut painter = self.painter();
            painter.shape(background, Corners::all(RADIUS_CONTROL), fill, None);
            if is_selected {
                let pill_color = if enabled { theme.accent } else { theme.accent_disabled };
                let pill = Rect::new(
                    background.x,
                    (background.center().y - SELECTION_PILL_LENGTH / 2.0).round(),
                    SELECTION_PILL_THICKNESS,
                    SELECTION_PILL_LENGTH,
                );
                painter.rounded(pill, SELECTION_PILL_THICKNESS / 2.0, pill_color);
            }
            row(
                self,
                index,
                row_rect,
                RowState {
                    selected: is_selected,
                    hovered: row_response.hovered,
                    pressed: visual == Visual::Pressed,
                },
            );
        }
        let mut painter = self.painter();
        painter.pop_clip();

        if let Some((bar, thumb_length, travel)) = bar_response {
            let wide = bar.hovered || bar.held;
            let width = self.animate(bar.id, 0, if wide { BAR_WIDE } else { BAR_THIN }, DURATION_FASTER, crate::tokens::CURVE_DECELERATE);
            let thumb_top = bar_area.y + travel * (state.offset / overflow).clamp(0.0, 1.0);
            let thumb = Rect::new(bar_area.right() - width, thumb_top, width, thumb_length);
            let mut painter = self.painter();
            if wide {
                painter.rounded(bar_area, BAR_WIDE / 2.0, theme.control_alt_fill_secondary);
            }
            painter.rounded(thumb, width / 2.0, theme.control_strong_fill);
        }
        control_focus_ring(self, &response);
        *self.memory::<ScrollState>(response.id) = state;
        ListResponse {
            response,
            clicked,
            activated,
            changed: *selected != before,
        }
    }

    // How far a list has scrolled, in pixels.
    pub fn list_offset(&mut self, name: &str) -> f32 {
        self.memory::<ScrollState>(crate::ui::id(name)).offset
    }
}
