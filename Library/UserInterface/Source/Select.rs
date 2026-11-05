// A combo box, as Fluent calls it: a box that shows one of a list of choices,
// and opens the list, as a popup under it, to pick another. A press on the
// box opens or closes the list, and a press on the list picks what it's on.
// A press anywhere else closes it, as does Esc, or the window losing focus.
//
// With keyboard focus, Space and Enter open and close the list, and Up,
// Down, Home and End pick another choice, whether it's open or not.

use application::Event;
use canvas::{Align, Fonts, Icon, Rect, TextStyle};
use windows_api::*;

use crate::interface::{Id, Layer, Sense};
use crate::surfaces::{focus_ring, raised, shadow};
use crate::tokens::*;
use crate::{Frame, Response};

const KEYS: &[i32] = &[VK_SPACE, VK_RETURN, VK_UP, VK_DOWN, VK_HOME, VK_END];
// A choice in the open list.
const CHOICE_HEIGHT: f32 = 32.0;
// Around the choices in the list.
const LIST_PADDING: f32 = 4.0;
// The chevron's room at the box's right.
const CHEVRON_WIDTH: f32 = 32.0;

// A choice: its label, which the box shows while it's chosen, and a few
// words about it, which the list shows after the label.
#[derive(Clone, Copy, Debug)]
pub struct Choice<'a> {
    pub label: &'a str,
    pub detail: &'a str,
}

// How wide a combo box is for a label: the label, with room for the chevron.
pub fn select_width(fonts: &Fonts, label: &str) -> f32 {
    TEXT_INSET + fonts.measure(label, TextStyle::Body).ceil() + CHEVRON_WIDTH
}

impl Frame<'_> {
    // A combo box in `area`, which shows choices[*selected]. Its list is
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

        // The box looks pressed while its list is open.
        let (hover, press) = self.hover_and_press(id, &response);
        let press = press.max(open as u8 as f32 * 0.6);
        let enabled = self.enabled();
        let theme = self.theme();
        let canvas = &mut self.canvas;
        let (text, chevron) = if enabled {
            let fill = theme.control.mix(theme.control_hover, hover).mix(theme.control_pressed, press);
            raised(canvas, area, RADIUS_CONTROL, fill, theme.control_stroke, theme.control_stroke_bottom);
            (theme.text, theme.text_secondary)
        } else {
            canvas.fill_stroke(area, RADIUS_CONTROL, theme.control_disabled, 1.0, theme.control_stroke);
            (theme.text_disabled, theme.text_disabled)
        };
        let label = choices.get(*selected).map_or("", |choice| choice.label);
        let text_area = Rect::from_edges(area.x + TEXT_INSET, area.y, area.right() - CHEVRON_WIDTH, area.bottom());
        canvas.text_in(text_area, label, TextStyle::Body, text, Align::Start);
        // The chevron nudges down while the box is pressed.
        let chevron_center = [area.right() - CHEVRON_WIDTH * 0.5, area.center()[1] + press.round()];
        canvas.icon(chevron_center, Icon::ChevronDown, 12.0, chevron);
        if self.shows_focus(&response) {
            focus_ring(&mut self.canvas, theme, area, RADIUS_CONTROL);
        }
        if open {
            self.select_list(name, id, area, choices, *selected);
        }
        response
    }

    // The open list of a combo box, under it, or over it if there isn't
    // room under it, and inside the window.
    fn select_list(&mut self, name: &str, id: Id, area: Rect, choices: &[Choice], selected: usize) {
        let (width, height) = (self.width, self.height);
        let theme = self.theme();
        let canvas = &self.popup;
        let label_width = choices
            .iter()
            .map(|choice| canvas.measure(choice.label, TextStyle::Body))
            .fold(0.0, f32::max)
            .ceil();
        let detail_width = choices
            .iter()
            .map(|choice| canvas.measure(choice.detail, TextStyle::Body))
            .fold(0.0, f32::max)
            .ceil();
        let gap = if detail_width > 0.0 { SPACE_XXL } else { 0.0 };
        let list_width = (LIST_PADDING * 2.0 + TEXT_INSET * 2.0 + label_width + gap + detail_width)
            .max(area.width)
            .min(width - MARGIN - area.x);
        let list_height = CHOICE_HEIGHT * choices.len() as f32 + LIST_PADDING * 2.0;
        let below = area.bottom() + SPACE_XS;
        let y = if below + list_height <= height - MARGIN {
            below
        } else {
            (area.y - SPACE_XS - list_height).max(MARGIN)
        };
        let list = Rect::new(area.x, y, list_width, list_height);
        // Presses on the list's padding land on it, not on what's under it.
        self.interact_on(&format!("{name}/List"), "List", "", list, Sense::CLICK, Layer::Popup, Some(id));
        let canvas = &mut self.popup;
        shadow(canvas, theme, list, RADIUS_CARD, FLYOUT);
        canvas.fill_stroke(list, RADIUS_CARD, theme.flyout, 1.0, theme.flyout_stroke);

        let detail_x = list.x + LIST_PADDING + TEXT_INSET + label_width + gap;
        for (index, choice) in choices.iter().enumerate() {
            let row = Rect::new(
                list.x + LIST_PADDING,
                list.y + LIST_PADDING + CHOICE_HEIGHT * index as f32,
                list.width - LIST_PADDING * 2.0,
                CHOICE_HEIGHT,
            );
            let part = format!("{name}/{index}");
            let response = self.interact_on(&part, "Choice", choice.label, row, Sense::CLICK, Layer::Popup, Some(id));
            let (hover, press) = self.hover_and_press(Id::new(&part), &response);
            let chosen = index == selected;
            let canvas = &mut self.popup;
            let face = row.inset_by(0.0, 2.0);
            let base = if chosen { theme.subtle_selected } else { canvas::Color::TRANSPARENT };
            canvas.fill(face, RADIUS_CONTROL, base.mix(theme.subtle_hover, hover).mix(theme.subtle_pressed, press));
            if chosen {
                let [pill_width, pill_height] = PILL_SIZE;
                let pill = Rect::new(face.x, face.center()[1] - pill_height * 0.5, pill_width, pill_height);
                canvas.fill(pill, pill_width * 0.5, theme.accent);
            }
            let label_area = Rect::from_edges(row.x + TEXT_INSET, row.y, detail_x - gap * 0.5, row.bottom());
            canvas.text_in(label_area, choice.label, TextStyle::Body, theme.text, Align::Start);
            if !choice.detail.is_empty() {
                let detail_area = Rect::from_edges(detail_x, row.y, row.right() - TEXT_INSET, row.bottom());
                canvas.text_in(detail_area, choice.detail, TextStyle::Body, theme.text_secondary, Align::Start);
            }
        }
    }
}
