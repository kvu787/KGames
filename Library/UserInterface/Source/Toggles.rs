// Widgets that are on or off, or one of a few choices: checkboxes, switches,
// radio buttons, segmented controls and tabs. Each acts on the press, as it
// comes sooner than the release. A switch's knob, and the mark of a segmented
// control's or tabs' choice, slide to where they go.

use application::Event;
use canvas::{Align, Color, Fonts, Icon, Rect, TextStyle};
use windows_api::*;

use crate::interface::{Id, Layer, Sense};
use crate::surfaces::{focus_ring, raised, shadow};
use crate::tokens::*;
use crate::{Frame, Response};

// A group's keys while it has focus.
const GROUP_KEYS: &[i32] = &[VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN, VK_HOME, VK_END];

// How wide a checkbox or a switch is with its label, for its area.
pub fn toggle_width(fonts: &Fonts, label: &str, switch: bool) -> f32 {
    let mark = if switch { SWITCH_SIZE[0] + SPACE_M } else { MARK_SIZE + LABEL_GAP };
    mark + fonts.measure(label, TextStyle::Body).ceil()
}

// How wide each of a row of tabs is: its label, with padding.
pub fn tab_widths(fonts: &Fonts, labels: &[&str]) -> Vec<f32> {
    labels
        .iter()
        .map(|label| fonts.measure(label, TextStyle::BodyStrong).ceil() + TEXT_INSET * 2.0)
        .collect()
}

impl Frame<'_> {
    // A box that a press checks or unchecks, with a label after it.
    pub fn checkbox(&mut self, name: &str, area: Rect, label: &str, value: &mut bool) -> Response {
        let id = Id::new(name);
        let mut response = self.interact_on(name, "Checkbox", "", area, Sense::BUTTON, Layer::Base, None);
        if response.clicked {
            *value = !*value;
            response.changed = true;
        }
        let (hover, press) = self.hover_and_press(id, &response);
        let checked = self.ease(id.part(3), *value as u8 as f32);
        let enabled = self.enabled();
        let theme = self.theme();
        let canvas = &mut self.canvas;
        let mark = Rect::new(area.x, (area.y + (area.height - MARK_SIZE) * 0.5).round(), MARK_SIZE, MARK_SIZE);
        if enabled {
            let off = theme.control_alt.mix(theme.subtle_hover.over(theme.control_alt), hover);
            let on = theme.accent.mix(theme.accent_hover, hover).mix(theme.accent_pressed, press);
            let fill = off.mix(on, checked);
            let stroke = theme.control_strong.mix(on, checked);
            canvas.fill_stroke(mark, RADIUS_CONTROL, fill, 1.0, stroke);
        } else if *value {
            canvas.fill(mark, RADIUS_CONTROL, theme.accent_disabled);
        } else {
            canvas.stroke(mark, RADIUS_CONTROL, 1.0, theme.text_disabled);
        }
        if *value {
            let color = if enabled { theme.on_accent } else { theme.card };
            canvas.icon(mark.center(), Icon::CheckMark, 12.0, color.alpha(checked.max(0.4)));
        }
        let text = Rect::from_edges(mark.right() + LABEL_GAP, area.y, area.right(), area.bottom());
        let color = if enabled { theme.text } else { theme.text_disabled };
        let end = canvas.text_in(text, label, TextStyle::Body, color, Align::Start);
        if self.shows_focus(&response) {
            let ring = Rect::from_edges(area.x, mark.y - 4.0, end + 4.0, mark.bottom() + 4.0);
            focus_ring(&mut self.canvas, theme, ring, RADIUS_CONTROL);
        }
        response
    }

    // A switch that a press turns on or off, with a label after it.
    pub fn switch(&mut self, name: &str, area: Rect, label: &str, value: &mut bool) -> Response {
        let id = Id::new(name);
        let mut response = self.interact_on(name, "Switch", "", area, Sense::BUTTON, Layer::Base, None);
        if response.clicked {
            *value = !*value;
            response.changed = true;
        }
        let (hover, press) = self.hover_and_press(id, &response);
        let on = self.ease(id.part(3), *value as u8 as f32);
        let enabled = self.enabled();
        let theme = self.theme();
        let canvas = &mut self.canvas;
        let [width, height] = SWITCH_SIZE;
        let track = Rect::new(area.x, (area.y + (area.height - height) * 0.5).round(), width, height);
        let radius = height * 0.5;
        let (on_fill, off_knob) = if enabled {
            (theme.accent.mix(theme.accent_hover, hover).mix(theme.accent_pressed, press), theme.text_secondary)
        } else {
            (theme.accent_disabled, theme.text_disabled)
        };
        let off_fill = theme.control_alt.mix(theme.subtle_hover.over(theme.control_alt), hover);
        let off_stroke = if enabled { theme.control_strong } else { theme.text_disabled };
        canvas.fill_stroke(track, radius, off_fill.mix(on_fill, on), 1.0, off_stroke.mix(on_fill, on));
        // The knob grows while the cursor is over it, and stretches while
        // it's pressed, toward where it goes.
        let size = 12.0 + 2.0 * hover;
        let stretch = 3.0 * press;
        let left = track.x + radius;
        let right = track.right() - radius;
        let x = left + (right - left) * on;
        let knob = Rect::new(x - size * 0.5 - stretch * on, track.center()[1] - size * 0.5, size + stretch, size);
        canvas.fill(knob, size * 0.5, off_knob.mix(theme.on_accent, on));
        let text = Rect::from_edges(track.right() + SPACE_M, area.y, area.right(), area.bottom());
        let color = if enabled { theme.text } else { theme.text_disabled };
        let end = canvas.text_in(text, label, TextStyle::Body, color, Align::Start);
        if self.shows_focus(&response) {
            let ring = Rect::from_edges(area.x, track.y - 4.0, end + 4.0, track.bottom() + 4.0);
            focus_ring(&mut self.canvas, theme, ring, RADIUS_CONTROL);
        }
        response
    }

    // A column of radio buttons, one for each label, each a control's
    // height. The group takes keyboard focus, and the arrow keys move its
    // choice. Each button is called "<name>/<label>".
    pub fn radio(&mut self, name: &str, area: Rect, labels: &[&str], selected: &mut usize) -> Response {
        let rows = area.rows(labels.len(), CONTROL_HEIGHT, 0.0);
        let group = self.group(name, "Radio", "RadioButton", area, labels, &rows, selected);
        let enabled = self.enabled();
        let theme = self.theme();
        for (index, (row, label)) in rows.iter().zip(labels).enumerate() {
            let part = Id::new(&format!("{name}/{label}"));
            let chosen = index == *selected;
            let response = &group.parts[index];
            let (hover, press) = self.hover_and_press(part, response);
            let on = self.ease(part.part(3), chosen as u8 as f32);
            let canvas = &mut self.canvas;
            let center = [row.x + MARK_SIZE * 0.5, row.center()[1].round()];
            let radius = MARK_SIZE * 0.5;
            if enabled {
                let off = theme.control_alt.mix(theme.subtle_hover.over(theme.control_alt), hover);
                let accent = theme.accent.mix(theme.accent_hover, hover).mix(theme.accent_pressed, press);
                let circle = Rect::new(center[0] - radius, center[1] - radius, MARK_SIZE, MARK_SIZE);
                canvas.fill_stroke(circle, radius, off.mix(accent, on), 1.0, theme.control_strong.mix(accent, on));
            } else if chosen {
                canvas.circle(center, radius, theme.accent_disabled);
            } else {
                canvas.ring(center, radius - 0.5, 1.0, theme.text_disabled);
            }
            // The dot in the middle grows while the cursor is over it, and
            // shrinks while it's pressed.
            let dot = (8.0 + 2.0 * hover - 4.0 * press) * on;
            if dot > 0.5 {
                let color = if enabled { theme.on_accent } else { theme.card };
                canvas.circle(center, dot * 0.5, color);
            }
            let text = Rect::from_edges(row.x + MARK_SIZE + LABEL_GAP, row.y, row.right(), row.bottom());
            let color = if enabled { theme.text } else { theme.text_disabled };
            let end = canvas.text_in(text, label, TextStyle::Body, color, Align::Start);
            if chosen && self.shows_focus(&group.response) {
                let ring = Rect::from_edges(row.x, center[1] - radius - 4.0, end + 4.0, center[1] + radius + 4.0);
                focus_ring(&mut self.canvas, theme, ring, RADIUS_CONTROL);
            }
        }
        group.response
    }

    // A segmented control: a row of choices as wide as each other, of which
    // one stands out, raised, and slides to where a press or the arrow keys
    // move it. Each choice is called "<name>/<label>".
    pub fn segmented(&mut self, name: &str, area: Rect, labels: &[&str], selected: &mut usize) -> Response {
        let id = Id::new(name);
        let track = area.inset(2.0);
        let segments = track.columns(labels.len(), 0.0);
        let group = self.group(name, "Segmented", "Segment", area, labels, &segments, selected);
        let enabled = self.enabled();
        let theme = self.theme();
        let position = self.ease(id.part(3), *selected as f32);
        self.canvas.fill(area, RADIUS_CONTROL + 2.0, theme.subtle_selected.over(theme.card));
        // The raised mark under the choice, between segments while it
        // slides.
        if let Some(first) = segments.first() {
            let x = first.x + first.width * position;
            let mark = Rect::new(x, track.y, first.width, track.height);
            let canvas = &mut self.canvas;
            if enabled {
                shadow(canvas, theme, mark, RADIUS_CONTROL, RAISED);
                raised(canvas, mark, RADIUS_CONTROL, theme.control, theme.control_stroke, theme.control_stroke_bottom);
            } else {
                canvas.fill(mark, RADIUS_CONTROL, theme.control_disabled);
            }
        }
        for (index, (segment, label)) in segments.iter().zip(labels).enumerate() {
            let part = Id::new(&format!("{name}/{label}"));
            let (hover, _) = self.hover_and_press(part, &group.parts[index]);
            let chosen = index == *selected;
            let canvas = &mut self.canvas;
            if !chosen && hover > 0.0 {
                canvas.fill(segment.inset(1.0), RADIUS_CONTROL, theme.subtle_hover.alpha(hover));
            }
            let color = match (enabled, chosen) {
                (false, _) => theme.text_disabled,
                (true, true) => theme.text,
                (true, false) => theme.text_secondary,
            };
            let style = if chosen { TextStyle::BodyStrong } else { TextStyle::Body };
            canvas.text_in(segment.inset_by(SPACE_S, 0.0), label, style, color, Align::Center);
        }
        if self.shows_focus(&group.response) {
            focus_ring(&mut self.canvas, theme, area, RADIUS_CONTROL + 2.0);
        }
        group.response
    }

    // A row of tabs from the left of `area`, each as wide as its label, of
    // which one is chosen, with a mark under it that slides to where a press
    // or the arrow keys move it. Each tab is called "<name>/<label>".
    pub fn tabs(&mut self, name: &str, area: Rect, labels: &[&str], selected: &mut usize) -> Response {
        let id = Id::new(name);
        let widths = tab_widths(self.fonts(), labels);
        let mut x = area.x;
        let tabs: Vec<Rect> = widths
            .iter()
            .map(|width| {
                let tab = Rect::new(x, area.y, *width, area.height);
                x += width;
                tab
            })
            .collect();
        let group = self.group(name, "Tabs", "Tab", area, labels, &tabs, selected);
        let enabled = self.enabled();
        let theme = self.theme();
        for (index, (tab, label)) in tabs.iter().zip(labels).enumerate() {
            let part = Id::new(&format!("{name}/{label}"));
            let (hover, press) = self.hover_and_press(part, &group.parts[index]);
            let chosen = index == *selected;
            let canvas = &mut self.canvas;
            let face = tab.inset_by(2.0, 4.0);
            canvas.fill(face, RADIUS_CONTROL, Color::TRANSPARENT.mix(theme.subtle_hover, hover).mix(theme.subtle_pressed, press));
            let color = match (enabled, chosen) {
                (false, _) => theme.text_disabled,
                (true, true) => theme.text,
                (true, false) => theme.text_secondary.mix(theme.text, hover),
            };
            let style = if chosen { TextStyle::BodyStrong } else { TextStyle::Body };
            canvas.text_in(*tab, label, style, color, Align::Center);
        }
        // The mark under the chosen tab: a pill as wide as its label, which
        // slides between tabs.
        if let Some(chosen) = tabs.get(*selected) {
            let left = self.ease(id.part(3), chosen.x + TEXT_INSET);
            let right = self.ease(id.part(4), chosen.right() - TEXT_INSET);
            let color = if enabled { theme.accent } else { theme.accent_disabled };
            let mark = Rect::from_edges(left, area.bottom() - 3.0, right, area.bottom());
            self.canvas.fill(mark, 1.5, color);
        }
        if self.shows_focus(&group.response)
            && let Some(chosen) = tabs.get(*selected)
        {
            focus_ring(&mut self.canvas, theme, chosen.inset_by(2.0, 4.0), RADIUS_CONTROL);
        }
        group.response
    }

    // A group of parts, one for each label, of which one is chosen: radio
    // buttons, segments, or tabs. The group is a target under its parts,
    // which takes keyboard focus, and a press on a part gives the group
    // focus. Returns the group's response, with each part's.
    #[allow(clippy::too_many_arguments)]
    fn group(
        &mut self,
        name: &str,
        kind: &'static str,
        part_kind: &'static str,
        area: Rect,
        labels: &[&str],
        parts: &[Rect],
        selected: &mut usize,
    ) -> Group {
        let id = Id::new(name);
        let sense = Sense {
            focus: true,
            keys: GROUP_KEYS,
            ..Sense::HOVER
        };
        let label = labels.get(*selected).copied().unwrap_or("");
        let mut response = self.interact_on(name, kind, label, area, sense, Layer::Base, None);
        let before = *selected;
        let last = labels.len().saturating_sub(1);
        for event in self.events_for(id) {
            if let Event::Key { key, .. } = event {
                *selected = match key as i32 {
                    VK_LEFT | VK_UP => selected.saturating_sub(1),
                    VK_RIGHT | VK_DOWN => (*selected + 1).min(last),
                    VK_HOME => 0,
                    VK_END => last,
                    _ => *selected,
                };
            }
        }
        // Each part's press first, so that the frame shows the choice.
        let names: Vec<String> = labels.iter().map(|label| format!("{name}/{label}")).collect();
        for (index, part_name) in names.iter().enumerate() {
            if self.events_for(Id::new(part_name)).iter().any(|event| matches!(event, Event::Press(_))) {
                *selected = index;
            }
        }
        self.relabel(id, labels.get(*selected).copied().unwrap_or(""));
        let mut responses = Vec::with_capacity(parts.len());
        for (part, part_name) in parts.iter().zip(&names) {
            let part_response = self.interact_on(part_name, part_kind, "", *part, Sense::CLICK, Layer::Base, Some(id));
            response.hovered |= part_response.hovered;
            response.pressed |= part_response.pressed;
            response.held |= part_response.held;
            responses.push(part_response);
        }
        response.changed = *selected != before;
        Group {
            response,
            parts: responses,
        }
    }
}

struct Group {
    response: Response,
    parts: Vec<Response>,
}
