// A select box: a box that shows one of a list of choices, and opens the
// list, as a popup under it, to pick another. A press on the box opens or
// closes the list, and a press on the list picks what it's on. A press
// anywhere else closes it, as does Esc, or the window losing focus.
//
// With keyboard focus, Space and Enter open and close the list, and Up,
// Down, Home and End pick another choice, whether it's open or not.

use application::Event;
use canvas::theme::{ACCENT, BORDER, DIM, FAINT, INSET, TEXT};
use canvas::{Canvas, Color, Rect};
use windows_api::*;

use crate::controls::focus_ring;
use crate::interface::{Id, Layer, Sense};
use crate::panels::{MARGIN, control_height, raised_box};
use crate::{Frame, Response};

const KEYS: &[i32] = &[VK_SPACE, VK_RETURN, VK_UP, VK_DOWN, VK_HOME, VK_END];

// A choice: its label, which the box shows while it's chosen, and a few
// words about it, which the list shows after the label.
#[derive(Clone, Copy, Debug)]
pub struct Choice<'a> {
    pub label: &'a str,
    pub detail: &'a str,
}

// A small V that points down, or up while the list it opens is open.
pub fn chevron(canvas: &mut Canvas, [x, y]: [f32; 2], size: f32, up: bool, color: Color) {
    let tip = if up { -size * 0.5 } else { size * 0.5 };
    canvas.line([x - size, y - tip], [x, y + tip], 1.5, color);
    canvas.line([x, y + tip], [x + size, y - tip], 1.5, color);
}

// How wide a select box is for a label: the label, with room for the
// chevron.
pub fn select_width(canvas: &Canvas, label: &str) -> f32 {
    let line_height = canvas.line_height();
    canvas.text_width(label) + (line_height * 0.4).round() * 2.0 + line_height
}

impl Frame<'_> {
    // A select box in `area`, which shows choices[*selected]. Its list is
    // called "<name>/List", and each choice in it "<name>/<index>".
    pub fn select(&mut self, name: &str, area: Rect, choices: &[Choice], selected: &mut usize) -> Response {
        let id = Id::new(name);
        let last = choices.len().saturating_sub(1);
        let before = *selected;
        let label = choices.get(*selected).map_or("", |choice| choice.label);
        let sense = Sense {
            click: true,
            focus: true,
            keys: KEYS,
            ..Sense::HOVER
        };
        let mut response = self.interact_on(name, "Select", label, area, sense, Layer::Base, None);
        let mut open = self.is_open(name);
        for event in self.events_for(id) {
            match event {
                Event::Press(_) => open = !open,
                Event::Key { key, .. } => match key as i32 {
                    VK_SPACE | VK_RETURN => open = !open,
                    VK_UP => *selected = selected.saturating_sub(1),
                    VK_DOWN => *selected = (*selected + 1).min(last),
                    VK_HOME => *selected = 0,
                    VK_END => *selected = last,
                    _ => {}
                },
                _ => {}
            }
        }
        // A press on a choice picks it, and closes the list.
        for (index, event) in self.part_events(id) {
            if matches!(event, Event::Press(_)) && index < choices.len() {
                *selected = index;
                open = false;
            }
        }
        if !self.enabled() {
            open = false;
        }
        self.set_popup(id, open);
        response.changed = *selected != before;
        self.relabel(id, choices.get(*selected).map_or("", |choice| choice.label));

        let enabled = self.enabled();
        let canvas = &mut self.canvas;
        let line_height = canvas.line_height();
        let inset = (line_height * 0.4).round();
        let arrow_width = line_height;
        canvas.rect(area.x, area.y, area.width, area.height, INSET);
        let outline = if (response.hovered || open) && enabled { DIM } else { BORDER };
        canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, outline);
        let text_y = area.y + ((area.height - line_height) * 0.5).round();
        let label = choices.get(*selected).map_or("", |choice| choice.label);
        canvas.text_clipped(
            area.x + inset,
            text_y,
            label,
            area.width - inset - arrow_width,
            if enabled { TEXT } else { FAINT },
        );
        chevron(
            canvas,
            [area.right() - arrow_width * 0.5, area.y + area.height * 0.5],
            (line_height * 0.2).round(),
            open,
            if enabled { DIM } else { FAINT },
        );
        if self.shows_focus(&response) {
            focus_ring(&mut self.canvas, area);
        }
        if open {
            self.select_list(name, id, area, choices, *selected);
        }
        response
    }

    // The open list of a select box, under it, or over it if there isn't
    // room under it, and inside the window.
    fn select_list(&mut self, name: &str, id: Id, area: Rect, choices: &[Choice], selected: usize) {
        let (width, height) = (self.width, self.height);
        let canvas = &self.popup;
        let line_height = canvas.line_height();
        let char_width = canvas.char_width();
        let row_height = control_height(canvas);
        let inset = (line_height * 0.4).round();
        let label_width = choices
            .iter()
            .map(|choice| canvas.text_width(choice.label))
            .fold(0.0, f32::max);
        let detail_width = choices
            .iter()
            .map(|choice| canvas.text_width(choice.detail))
            .fold(0.0, f32::max);
        // With a pixel on each side for the outline.
        let list_width = (inset * 2.0 + label_width + char_width * 2.0 + detail_width + 2.0)
            .max(area.width)
            .min(width - MARGIN - area.x);
        let list_height = row_height * choices.len() as f32 + 2.0;
        let below = area.bottom() + 2.0;
        let y = if below + list_height <= height - MARGIN {
            below
        } else {
            (area.y - 2.0 - list_height).max(MARGIN)
        };
        let list = Rect::new(area.x, y, list_width, list_height);
        // Presses on the list's outline land on it, not on what's under it.
        let background = Sense {
            click: true,
            ..Sense::HOVER
        };
        self.interact_on(&format!("{name}/List"), "List", "", list, background, Layer::Popup, Some(id));
        raised_box(&mut self.popup, list, DIM);

        let detail_x = list.x + inset + label_width + char_width * 2.0;
        for (index, choice) in choices.iter().enumerate() {
            let row = Rect::new(
                list.x + 1.0,
                list.y + 1.0 + row_height * index as f32,
                list.width - 2.0,
                row_height,
            );
            let response = self.interact_on(
                &format!("{name}/{index}"),
                "Choice",
                choice.label,
                row,
                background,
                Layer::Popup,
                Some(id),
            );
            let chosen = index == selected;
            let canvas = &mut self.popup;
            if response.hovered {
                canvas.rect(row.x, row.y, row.width, row.height, BORDER);
            }
            if chosen {
                canvas.rect(row.x, row.y, 3.0, row.height, ACCENT);
            }
            let text_y = row.y + ((row_height - line_height) * 0.5).round();
            canvas.text_clipped(
                row.x + inset,
                text_y,
                choice.label,
                detail_x - char_width - row.x - inset,
                if chosen { ACCENT } else { TEXT },
            );
            canvas.text_clipped(detail_x, text_y, choice.detail, row.right() - inset - detail_x, DIM);
        }
    }
}
