// A list: rows of the same height, of which one can be selected, and which
// scroll by pixels, like Fluent's list view. The wheel scrolls it 3 rows a
// notch, and so does dragging its scroll bar's thumb. A press on the scroll
// bar's track moves the thumb there. A press on a row selects it, and with
// keyboard focus, Up, Down, Home, End, Page Up and Page Down move the
// selection, scrolling to keep it in view.
//
// The list draws the selected row and the row under the cursor, and its
// scroll bar, which is thin until the cursor is over the list. The program
// draws what's in each row that shows, with a function that `list` calls for
// each, while the rows are clipped to the list.

use application::Event;
use canvas::{Canvas, Color, Rect};
use windows_api::*;

use crate::interface::{Id, Layer, Sense};
use crate::surfaces::focus_ring;
use crate::tokens::*;
use crate::{Frame, Response};

const KEYS: &[i32] = &[VK_UP, VK_DOWN, VK_HOME, VK_END, VK_PRIOR, VK_NEXT];
const ROWS_PER_NOTCH: f32 = 3.0;
// Where presses and the wheel go to the scroll bar, at the list's right.
pub const SCROLL_BAR_WIDTH: f32 = 12.0;
// The thumb is never shorter than this, so that it's easy to grab.
const MINIMUM_THUMB: f32 = 24.0;

// A row that shows, for the program to draw what's in it.
#[derive(Clone, Copy, Debug)]
pub struct ListRow {
    pub index: usize,
    // The row's area. Rows at the list's top and bottom can be partly
    // scrolled out of it, and are clipped.
    pub area: Rect,
    pub selected: bool,
    pub hovered: bool,
}

// What a list shows this frame.
#[derive(Debug, Default)]
pub struct ListView {
    pub response: Response,
    // The row that the cursor is over, if any, and its area.
    pub hovered: Option<(usize, Rect)>,
    // How far it's scrolled, in pixels.
    pub scroll: f32,
    // How many rows fit, whole.
    pub visible: usize,
}

impl Frame<'_> {
    // A list of `count` rows, `row_height` tall, in `area`, of which
    // `selected` is selected. `draw` draws what's in each row that shows.
    // The scroll bar is called "<name>/Scroll bar", and each row that shows
    // "<name>/<index>".
    pub fn list(
        &mut self,
        name: &str,
        area: Rect,
        count: usize,
        row_height: f32,
        selected: &mut Option<usize>,
        mut draw: impl FnMut(&mut Canvas, ListRow),
    ) -> ListView {
        let id = Id::new(name);
        let content = row_height * count as f32;
        let most = (content - area.height).max(0.0);
        let mut scroll = self.state.scroll.get(&id).copied().unwrap_or(0.0).clamp(0.0, most);
        let visible = ((area.height / row_height).floor() as usize).max(1);
        let sense = Sense {
            click: true,
            wheel: true,
            focus: true,
            keys: KEYS,
            ..Sense::HOVER
        };
        let mut response = self.interact_on(name, "List", "", area, sense, Layer::Base, None);
        let before = *selected;
        let last = count.saturating_sub(1);
        let mut keyed = false;
        for event in self.events_for(id) {
            match event {
                // Turning away from the user scrolls up, to earlier rows.
                Event::Wheel(_, notches) => scroll = (scroll - notches * ROWS_PER_NOTCH * row_height).clamp(0.0, most),
                Event::Key { key, .. } if count > 0 => {
                    keyed = true;
                    let page = visible.saturating_sub(1).max(1);
                    *selected = Some(match (key as i32, *selected) {
                        (VK_UP, Some(row)) => row.saturating_sub(1),
                        (VK_DOWN, Some(row)) => (row + 1).min(last),
                        (VK_PRIOR, Some(row)) => row.saturating_sub(page),
                        (VK_NEXT, Some(row)) => (row + page).min(last),
                        (VK_HOME, _) => 0,
                        (VK_END, _) => last,
                        (_, Some(row)) => row,
                        (_, None) => 0,
                    });
                }
                _ => {}
            }
        }

        // The scroll bar, if the rows don't all fit.
        let scrollable = most > 0.0;
        let bar = Rect::new(area.right() - SCROLL_BAR_WIDTH, area.y, SCROLL_BAR_WIDTH, area.height);
        let track = bar.inset_by(0.0, 4.0);
        let thumb_height = (track.height * area.height / content.max(1.0)).max(MINIMUM_THUMB).min(track.height);
        let thumb_y = |scroll: f32| track.y + (track.height - thumb_height) * if most > 0.0 { scroll / most } else { 0.0 };
        let bar_name = format!("{name}/Scroll bar");
        let bar_id = Id::new(&bar_name);
        if scrollable {
            // The scroll for the thumb's top `grab` above `y`.
            let dragged = |y: f32, grab: f32| {
                let travel = (track.height - thumb_height).max(1.0);
                ((y - grab - track.y) / travel).clamp(0.0, 1.0) * most
            };
            for event in self.events_for(bar_id) {
                match event {
                    Event::Press([_, y]) => {
                        let top = thumb_y(scroll);
                        self.state.grab = if (top..top + thumb_height).contains(&y) {
                            y - top
                        } else {
                            thumb_height * 0.5
                        };
                        scroll = dragged(y, self.state.grab);
                    }
                    Event::Wheel(_, notches) => {
                        scroll = (scroll - notches * ROWS_PER_NOTCH * row_height).clamp(0.0, most)
                    }
                    _ => {}
                }
            }
            if self.state.held == Some(bar_id)
                && let Some([_, y]) = self.cursor()
            {
                scroll = dragged(y, self.state.grab);
            }
        }

        // A press on a row selects it, even one that the wheel scrolled away
        // in the same frame.
        for (index, event) in self.part_events(id) {
            if matches!(event, Event::Press(_)) && index < count {
                *selected = Some(index);
            }
        }
        // Keys scroll the selection into view.
        if keyed && let Some(row) = *selected {
            let top = row_height * row as f32;
            if top < scroll {
                scroll = top;
            } else if top + row_height > scroll + area.height {
                scroll = top + row_height - area.height;
            }
        }
        self.state.scroll.insert(id, scroll);
        response.changed = *selected != before;

        let enabled = self.enabled();
        let theme = self.theme();
        let rows_right = area.right() - if scrollable { SCROLL_BAR_WIDTH } else { 0.0 };
        let mut view = ListView {
            response,
            hovered: None,
            scroll,
            visible,
        };
        let first = (scroll / row_height).floor() as usize;
        let shown = ((scroll + area.height) / row_height).ceil() as usize;
        self.canvas.push_clip(area);
        for index in first..count.min(shown) {
            let top = area.y + row_height * index as f32 - scroll;
            let row = Rect::from_edges(area.x, top, rows_right, top + row_height);
            // Presses only land on what of a row shows.
            let target = Rect::from_edges(row.x, row.y.max(area.y), row.right(), row.bottom().min(area.bottom()));
            let part = format!("{name}/{index}");
            let row_response = self.interact_on(&part, "Row", "", target, Sense::CLICK, Layer::Base, Some(id));
            let (hover, press) = self.hover_and_press(Id::new(&part), &row_response);
            let chosen = Some(index) == *selected;
            let canvas = &mut self.canvas;
            let face = row.inset_by(SPACE_XS, 2.0);
            let base = if chosen { theme.subtle_selected } else { Color::TRANSPARENT };
            canvas.fill(face, RADIUS_CONTROL, base.mix(theme.subtle_hover, hover).mix(theme.subtle_pressed, press));
            if chosen {
                let [pill_width, pill_height] = PILL_SIZE;
                let height = pill_height.min(face.height - 4.0);
                let pill = Rect::new(face.x, face.center()[1] - height * 0.5, pill_width, height);
                canvas.fill(pill, pill_width * 0.5, if enabled { theme.accent } else { theme.accent_disabled });
            }
            if row_response.hovered {
                view.hovered = Some((index, row));
                view.response.hovered = true;
            }
            // A press on a row is a press on the list.
            view.response.pressed |= row_response.pressed;
            view.response.double_clicked |= row_response.double_clicked;
            draw(
                canvas,
                ListRow {
                    index,
                    area: row,
                    selected: chosen,
                    hovered: row_response.hovered,
                },
            );
        }
        self.canvas.pop_clip();

        if scrollable {
            let bar_response = self.interact_on(&bar_name, "ScrollBar", "", bar, Sense { wheel: true, ..Sense::CLICK }, Layer::Base, Some(id));
            // Thin until the cursor is over the list, and wider still while
            // it's over the bar.
            let over_list = view.response.hovered || bar_response.hovered || bar_response.held;
            let wide = self.ease(bar_id.part(3), if over_list { 1.0 } else { 0.0 });
            let width = 2.0 + 4.0 * wide;
            let canvas = &mut self.canvas;
            if wide > 0.0 {
                canvas.fill(Rect::new(bar.center()[0] - 3.0, track.y, 6.0, track.height), 3.0, theme.subtle_hover.alpha(wide));
            }
            let thumb = Rect::new(bar.center()[0] - width * 0.5, thumb_y(scroll), width, thumb_height);
            let color = if bar_response.held { theme.text_secondary } else { theme.text_tertiary };
            canvas.fill(thumb, width * 0.5, color);
        }
        if self.shows_focus(&view.response) {
            focus_ring(&mut self.canvas, theme, area, RADIUS_CONTROL);
        }
        view
    }
}
