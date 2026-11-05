// A list: rows of the same height, of which one can be selected, and as
// many as fit show at a time. The wheel scrolls it, 3 rows a notch, and so
// does dragging its scroll bar's thumb. A press on the scroll bar's track
// moves the thumb there. A press on a row selects it, and with keyboard
// focus, Up, Down, Home and End move the selection, scrolling to keep it in
// view.
//
// The list draws its background, the selected row and the row under the
// cursor, and its scroll bar. The program draws what's in each row that
// shows, from the rows that `list` returns. A list scrolls by whole rows, so
// that every row it shows is whole.

use application::Event;
use canvas::Rect;
use canvas::theme::{ACCENT, BORDER, DIM, FAINT, INSET, PANEL};
use windows_api::*;

use crate::controls::focus_ring;
use crate::interface::{Id, Layer, Sense};
use crate::{Frame, Response};

const KEYS: &[i32] = &[VK_UP, VK_DOWN, VK_HOME, VK_END];
const ROWS_PER_NOTCH: f32 = 3.0;
pub const SCROLL_BAR_WIDTH: f32 = 10.0;
// The thumb is never shorter than this, so that it's easy to grab.
const MINIMUM_THUMB: f32 = 24.0;

// What a list shows this frame.
#[derive(Debug, Default)]
pub struct ListView {
    pub response: Response,
    // The rows that show, by index, with their areas, top to bottom.
    pub rows: Vec<(usize, Rect)>,
    // The row that the cursor is over, if any.
    pub hovered: Option<usize>,
    // The first row that shows.
    pub first: usize,
    // How many rows fit.
    pub visible: usize,
}

impl Frame<'_> {
    // A list of `count` rows, `row_height` tall, in `area`. `selected` is
    // the selected row. The scroll bar is called "<name>/Scroll bar", and
    // each row that shows "<name>/<index>".
    pub fn list(&mut self, name: &str, area: Rect, count: usize, row_height: f32, selected: &mut Option<usize>) -> ListView {
        let id = Id::new(name);
        let inner = area.inset(1.0);
        let visible = ((inner.height / row_height).floor() as usize).max(1);
        let most = count.saturating_sub(visible);
        let mut first = self.state.scroll.get(&id).copied().unwrap_or(0).min(most);
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
        let scroll = |first: &mut usize, frame: &mut Self, notches: f32| {
            let (last_id, part) = frame.state.wheel;
            let turned = notches * ROWS_PER_NOTCH + if last_id == Some(id) { part } else { 0.0 };
            let rows = turned.trunc();
            frame.state.wheel = (Some(id), turned - rows);
            // Turning away from the user scrolls up, to earlier rows.
            *first = (*first as f32 - rows).clamp(0.0, most as f32) as usize;
        };
        for event in self.events_for(id) {
            match event {
                Event::Wheel(_, notches) => scroll(&mut first, self, notches),
                Event::Key { key, .. } if count > 0 => {
                    keyed = true;
                    *selected = Some(match (key as i32, *selected) {
                        (VK_UP, Some(row)) => row.saturating_sub(1),
                        (VK_DOWN, Some(row)) => (row + 1).min(last),
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
        let scrollable = count > visible;
        let bar = Rect::new(inner.right() - SCROLL_BAR_WIDTH, inner.y, SCROLL_BAR_WIDTH, inner.height);
        let thumb_height = (bar.height * visible as f32 / count.max(1) as f32).max(MINIMUM_THUMB).min(bar.height);
        let thumb_y = |first: usize| {
            bar.y + (bar.height - thumb_height) * if most > 0 { first as f32 / most as f32 } else { 0.0 }
        };
        let bar_name = format!("{name}/Scroll bar");
        let bar_id = Id::new(&bar_name);
        let mut bar_response = Response::default();
        if scrollable {
            let bar_sense = Sense {
                click: true,
                wheel: true,
                ..Sense::HOVER
            };
            bar_response = self.interact_on(&bar_name, "ScrollBar", "", bar, bar_sense, Layer::Base, Some(id));
            // The first row for the thumb's top `grab` above `y`.
            let dragged = |y: f32, grab: f32| {
                let travel = (bar.height - thumb_height).max(1.0);
                let fraction = ((y - grab - bar.y) / travel).clamp(0.0, 1.0);
                (fraction * most as f32).round() as usize
            };
            for event in self.events_for(bar_id) {
                match event {
                    Event::Press([_, y]) => {
                        let top = thumb_y(first);
                        self.state.grab = if (top..top + thumb_height).contains(&y) {
                            y - top
                        } else {
                            thumb_height * 0.5
                        };
                        first = dragged(y, self.state.grab);
                    }
                    Event::Wheel(_, notches) => scroll(&mut first, self, notches),
                    _ => {}
                }
            }
            if bar_response.held
                && let Some([_, y]) = self.cursor()
            {
                first = dragged(y, self.state.grab);
            }
        }

        // A press on a row selects it, even one that the wheel scrolled away
        // in the same frame.
        for (index, event) in self.part_events(id) {
            if matches!(event, Event::Press(_)) && index < count {
                *selected = Some(index);
            }
        }
        if keyed && let Some(row) = *selected {
            if row < first {
                first = row;
            } else if row >= first + visible {
                first = row + 1 - visible;
            }
        }
        self.state.scroll.insert(id, first);
        response.changed = *selected != before;

        let enabled = self.enabled();
        let canvas = &mut self.canvas;
        canvas.rect(area.x, area.y, area.width, area.height, PANEL);
        let rows_width = inner.width - if scrollable { SCROLL_BAR_WIDTH } else { 0.0 };
        let row_sense = Sense {
            click: true,
            ..Sense::HOVER
        };
        let mut view = ListView {
            response,
            rows: Vec::new(),
            hovered: None,
            first,
            visible,
        };
        for index in first..count.min(first + visible) {
            let row = Rect::new(inner.x, inner.y + row_height * (index - first) as f32, rows_width, row_height);
            let row_response = self.interact_on(&format!("{name}/{index}"), "Row", "", row, row_sense, Layer::Base, Some(id));
            let canvas = &mut self.canvas;
            if Some(index) == *selected {
                canvas.rect(row.x, row.y, row.width, row.height, ACCENT.alpha(0.12));
                canvas.rect(row.x, row.y, 3.0, row.height, if enabled { ACCENT } else { FAINT });
            } else if row_response.hovered {
                canvas.rect(row.x, row.y, row.width, row.height, INSET);
            }
            if row_response.hovered {
                view.hovered = Some(index);
                view.response.hovered = true;
            }
            view.rows.push((index, row));
        }
        let canvas = &mut self.canvas;
        if scrollable {
            canvas.rect(bar.x, bar.y, bar.width, bar.height, INSET);
            let thumb_color = if bar_response.held || bar_response.hovered { DIM } else { FAINT };
            canvas.rect(bar.x + 2.0, thumb_y(first) + 2.0, bar.width - 4.0, thumb_height - 4.0, thumb_color);
        }
        let outline = if view.response.focused { ACCENT } else { BORDER };
        canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, outline);
        if self.shows_focus(&view.response) {
            focus_ring(&mut self.canvas, area);
        }
        view
    }
}
