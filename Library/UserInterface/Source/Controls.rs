// The simple widgets: buttons, checkboxes, switches, radio buttons, tabs,
// chips and progress bars. Each fills the area it's given, which is where
// presses go, and acts on the press, as it comes sooner than the release.

use application::Event;
use canvas::theme::{ACCENT, BORDER, DIM, ERROR, FAINT, INSET, ON_COLOR, PANEL, SUCCESS, TEXT};
use canvas::{Canvas, Color, Rect};
use windows_api::*;

use crate::interface::{Id, Layer, Sense};
use crate::panels::{PADDING, control_height};
use crate::{Frame, Response};

// How a button stands out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Normal,
    // The action that a panel is for, in the accent color.
    Primary,
    // An action that throws something away.
    Danger,
}

// What a chip shows: a label, and whether it's on.
#[derive(Clone, Copy, Debug)]
pub struct Chip<'a> {
    pub label: &'a str,
    pub on: bool,
    // Faint, for what the device or setting doesn't have.
    pub enabled: bool,
}

// The side of a checkbox's box, and of a radio button's ring.
const MARK_SIZE: f32 = 16.0;
// A switch's track.
const SWITCH_WIDTH: f32 = 34.0;
const SWITCH_HEIGHT: f32 = 18.0;
// Between a mark and its label.
const LABEL_GAP: f32 = 8.0;
// A group's keys while it has focus.
const GROUP_KEYS: &[i32] = &[VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN, VK_HOME, VK_END];

// Darkens what's under it, for a colored control that the cursor is over.
const DARKEN: Color = Color(0x1F00_0000);

// A ring around a widget that has keyboard focus, from Tab.
pub(crate) fn focus_ring(canvas: &mut Canvas, area: Rect) {
    let ring = area.outset(3.0);
    canvas.rect_outline(ring.x, ring.y, ring.width, ring.height, 2.0, ACCENT);
}

// Text centered across an area, and down it, cut short if it doesn't fit.
pub(crate) fn centered_text(canvas: &mut Canvas, area: Rect, text: &str, color: Color) {
    let width = canvas.text_width(text).min(area.width);
    let x = (area.x + (area.width - width) * 0.5).round();
    let y = area.y + ((area.height - canvas.line_height()) * 0.5).round();
    canvas.text_clipped(x, y, text, area.right() - x, color);
}

// Places chips with these labels left to right, wrapping onto new rows to
// fit in `width`. Returns each chip's offset from the top left and its
// width, and the height of all the rows.
pub fn chip_layout(canvas: &Canvas, width: f32, labels: &[&str]) -> (Vec<([f32; 2], f32)>, f32) {
    let line_height = canvas.line_height();
    let gap = (line_height * 0.3).round();
    let (mut x, mut y) = (0.0, 0.0);
    let mut chips = Vec::new();
    for label in labels {
        let chip_width = canvas.text_width(label) + line_height * 0.8;
        if x + chip_width > width && x > 0.0 {
            x = 0.0;
            y += control_height(canvas) + gap;
        }
        chips.push(([x, y], chip_width));
        x += chip_width + gap;
    }
    (chips, y + control_height(canvas))
}

// Draws chips as chip_layout places them, as far down `area` as they fit,
// in `on_color` while they're on, and returns their height.
pub fn chips(canvas: &mut Canvas, area: Rect, items: &[Chip], on_color: Color) -> f32 {
    let labels: Vec<&str> = items.iter().map(|item| item.label).collect();
    let (layout, height) = chip_layout(canvas, area.width, &labels);
    for (item, ([x, y], width)) in items.iter().zip(layout) {
        let chip = Rect::new(area.x + x, area.y + y, width, control_height(canvas));
        if chip.bottom() > area.bottom() + 0.5 {
            break;
        }
        draw_chip(canvas, chip, item, on_color, false);
    }
    height
}

fn draw_chip(canvas: &mut Canvas, chip: Rect, item: &Chip, on_color: Color, hovered: bool) {
    let line_height = canvas.line_height();
    let text_y = chip.y + ((chip.height - line_height) * 0.5).round();
    let text_x = chip.x + line_height * 0.4;
    if item.on {
        canvas.rect(chip.x, chip.y, chip.width, chip.height, on_color);
        if hovered {
            canvas.rect(chip.x, chip.y, chip.width, chip.height, DARKEN);
        }
        canvas.text(text_x, text_y, item.label, ON_COLOR);
    } else {
        if hovered {
            canvas.rect(chip.x, chip.y, chip.width, chip.height, INSET);
        }
        canvas.rect_outline(chip.x, chip.y, chip.width, chip.height, 1.0, BORDER);
        canvas.text(text_x, text_y, item.label, if item.enabled { DIM } else { FAINT });
    }
}

impl Frame<'_> {
    // How big a button is for its label: a line of text with padding.
    pub fn button_size(&self, label: &str) -> [f32; 2] {
        [
            self.text_width(label) + PADDING * 2.0,
            self.line_height() + PADDING,
        ]
    }

    pub fn button(&mut self, name: &str, area: Rect, label: &str) -> Response {
        self.button_with(name, area, label, Tone::Normal)
    }

    pub fn button_with(&mut self, name: &str, area: Rect, label: &str, tone: Tone) -> Response {
        let response = self.interact_on(name, "Button", "", area, Sense::BUTTON, Layer::Base, None);
        let enabled = self.enabled();
        let lit = response.hovered || response.held;
        let canvas = &mut self.canvas;
        let text = match (enabled, tone) {
            (false, _) => {
                canvas.rect(area.x, area.y, area.width, area.height, INSET);
                canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
                FAINT
            }
            (true, Tone::Normal) => {
                let fill = if response.held {
                    BORDER
                } else if response.hovered {
                    INSET
                } else {
                    PANEL
                };
                canvas.rect(area.x, area.y, area.width, area.height, fill);
                canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
                TEXT
            }
            (true, Tone::Primary | Tone::Danger) => {
                let fill = if tone == Tone::Primary { ACCENT } else { ERROR };
                canvas.rect(area.x, area.y, area.width, area.height, fill);
                if lit {
                    canvas.rect(area.x, area.y, area.width, area.height, DARKEN);
                }
                ON_COLOR
            }
        };
        centered_text(canvas, area, label, text);
        if self.shows_focus(&response) {
            focus_ring(&mut self.canvas, area);
        }
        response
    }

    // A box that a press checks or unchecks, with a label after it.
    pub fn checkbox(&mut self, name: &str, area: Rect, label: &str, value: &mut bool) -> Response {
        let mut response = self.interact_on(name, "Checkbox", "", area, Sense::BUTTON, Layer::Base, None);
        if response.clicked {
            *value = !*value;
            response.changed = true;
        }
        let enabled = self.enabled();
        let canvas = &mut self.canvas;
        let mark = Rect::new(area.x, (area.y + (area.height - MARK_SIZE) * 0.5).round(), MARK_SIZE, MARK_SIZE);
        if *value {
            canvas.rect(mark.x, mark.y, mark.width, mark.height, if enabled { ACCENT } else { FAINT });
            if response.hovered {
                canvas.rect(mark.x, mark.y, mark.width, mark.height, DARKEN);
            }
            let (x, y) = (mark.x, mark.y);
            canvas.line([x + 3.5, y + 8.5], [x + 6.5, y + 11.5], 2.0, ON_COLOR);
            canvas.line([x + 6.5, y + 11.5], [x + 12.5, y + 4.5], 2.0, ON_COLOR);
        } else {
            canvas.rect(mark.x, mark.y, mark.width, mark.height, if enabled { PANEL } else { INSET });
            let outline = if response.hovered { DIM } else { BORDER };
            canvas.rect_outline(mark.x, mark.y, mark.width, mark.height, 1.0, outline);
        }
        label_after(canvas, area, mark.right(), label, enabled);
        if self.shows_focus(&response) {
            focus_ring(&mut self.canvas, mark);
        }
        response
    }

    // A switch that a press turns on or off, with a label after it.
    pub fn switch(&mut self, name: &str, area: Rect, label: &str, value: &mut bool) -> Response {
        let mut response = self.interact_on(name, "Switch", "", area, Sense::BUTTON, Layer::Base, None);
        if response.clicked {
            *value = !*value;
            response.changed = true;
        }
        let enabled = self.enabled();
        let canvas = &mut self.canvas;
        let track = Rect::new(
            area.x,
            (area.y + (area.height - SWITCH_HEIGHT) * 0.5).round(),
            SWITCH_WIDTH,
            SWITCH_HEIGHT,
        );
        let radius = SWITCH_HEIGHT * 0.5;
        let color = match (enabled, *value) {
            (false, _) => BORDER,
            (true, true) => ACCENT,
            (true, false) => FAINT,
        };
        let middle = track.y + radius;
        canvas.rect(track.x + radius, track.y, track.width - radius * 2.0, track.height, color);
        canvas.circle([track.x + radius, middle], radius, color);
        canvas.circle([track.right() - radius, middle], radius, color);
        if response.hovered && enabled {
            canvas.rect(track.x + radius, track.y, track.width - radius * 2.0, track.height, DARKEN);
        }
        let knob = if *value { track.right() - radius } else { track.x + radius };
        canvas.circle([knob, middle], radius - 2.0, PANEL);
        label_after(canvas, area, track.right(), label, enabled);
        if self.shows_focus(&response) {
            focus_ring(&mut self.canvas, track);
        }
        response
    }

    // A column of radio buttons, one for each label, each a control's
    // height. The group takes keyboard focus, and Up and Down move its
    // choice. Each button is called "<name>/<label>".
    pub fn radio(&mut self, name: &str, area: Rect, labels: &[&str], selected: &mut usize) -> Response {
        let row_height = self.control_height();
        let rows = area.rows(labels.len(), row_height, 0.0);
        self.group(name, "Radio", area, labels, &rows, selected, |frame, row, label, chosen, response| {
            let enabled = frame.enabled();
            let canvas = &mut frame.canvas;
            let center = [row.x + MARK_SIZE * 0.5, row.center()[1].round()];
            let ring = if chosen && enabled {
                ACCENT
            } else if response.hovered {
                DIM
            } else {
                BORDER
            };
            canvas.circle(center, MARK_SIZE * 0.5, if enabled { PANEL } else { INSET });
            canvas.ring(center, MARK_SIZE * 0.5 - 0.5, 1.5, ring);
            if chosen {
                canvas.circle(center, 4.0, if enabled { ACCENT } else { FAINT });
            }
            label_after(canvas, row, row.x + MARK_SIZE, label, enabled);
        })
    }

    // A row of tabs, one for each label, as wide as each other, of which
    // one is chosen. The row takes keyboard focus, and Left and Right move
    // its choice. Each tab is called "<name>/<label>".
    pub fn tabs(&mut self, name: &str, area: Rect, labels: &[&str], selected: &mut usize) -> Response {
        let columns = area.columns(labels.len(), 0.0);
        let response = self.group(name, "Tabs", area, labels, &columns, selected, |frame, tab, label, chosen, response| {
            let enabled = frame.enabled();
            let canvas = &mut frame.canvas;
            let (fill, text) = match (chosen, enabled) {
                (true, true) => (ACCENT, ON_COLOR),
                (true, false) => (FAINT, ON_COLOR),
                (false, _) if response.hovered => (INSET, TEXT),
                (false, true) => (PANEL, TEXT),
                (false, false) => (PANEL, FAINT),
            };
            canvas.rect(tab.x, tab.y, tab.width, tab.height, fill);
            centered_text(canvas, tab, label, text);
        });
        let canvas = &mut self.canvas;
        for tab in columns.iter().skip(1) {
            canvas.rect(tab.x, area.y, 1.0, area.height, BORDER);
        }
        canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
        if self.shows_focus(&response) {
            focus_ring(&mut self.canvas, area);
        }
        response
    }

    // A group of parts, one for each label, of which one is chosen: radio
    // buttons, or tabs. The group is a target under its parts, which takes
    // keyboard focus, and a press on a part gives the group focus.
    #[allow(clippy::too_many_arguments)]
    fn group(
        &mut self,
        name: &str,
        kind: &'static str,
        area: Rect,
        labels: &[&str],
        parts: &[Rect],
        selected: &mut usize,
        mut draw: impl FnMut(&mut Self, Rect, &str, bool, &Response),
    ) -> Response {
        let id = Id::new(name);
        let sense = Sense {
            focus: true,
            keys: GROUP_KEYS,
            ..Sense::HOVER
        };
        let mut response = self.interact_on(name, kind, labels.get(*selected).copied().unwrap_or(""), area, sense, Layer::Base, None);
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
        let part_sense = Sense {
            click: true,
            ..Sense::HOVER
        };
        // Each part's press first, so that the frame shows the choice.
        let names: Vec<String> = labels.iter().map(|label| format!("{name}/{label}")).collect();
        for (index, part_name) in names.iter().enumerate() {
            if self.events_for(Id::new(part_name)).iter().any(|event| matches!(event, Event::Press(_))) {
                *selected = index;
            }
        }
        self.relabel(id, labels.get(*selected).copied().unwrap_or(""));
        for (index, (part, part_name)) in parts.iter().zip(&names).enumerate() {
            let part_response = self.interact_on(part_name, kind_of_part(kind), "", *part, part_sense, Layer::Base, Some(id));
            response.hovered |= part_response.hovered;
            response.pressed |= part_response.pressed;
            response.held |= part_response.held;
            draw(self, *part, labels[index], index == *selected, &part_response);
        }
        response.changed = *selected != before;
        response
    }

    // Chips that show what's on, like the buttons that are held, which
    // wrap onto more rows to fit across `area`. Returns their height.
    pub fn chips(&mut self, area: Rect, items: &[Chip]) -> f32 {
        chips(&mut self.canvas, area, items, SUCCESS)
    }

    // Chips that a press turns on or off, one for each label, which wrap
    // onto more rows to fit across `area`. Each is called "<name>/<label>".
    // Returns whether any changed, and their height.
    pub fn chip_switches(&mut self, name: &str, area: Rect, labels: &[&str], values: &mut [bool]) -> (bool, f32) {
        let (layout, height) = chip_layout(&self.canvas, area.width, labels);
        let mut changed = false;
        let enabled = self.enabled();
        for ((label, value), ([x, y], width)) in labels.iter().zip(values.iter_mut()).zip(layout) {
            let chip = Rect::new(area.x + x, area.y + y, width, self.control_height());
            let response = self.interact_on(&format!("{name}/{label}"), "Chip", "", chip, Sense::BUTTON, Layer::Base, None);
            if response.clicked {
                *value = !*value;
                changed = true;
            }
            let item = Chip { label, on: *value, enabled };
            draw_chip(&mut self.canvas, chip, &item, if enabled { ACCENT } else { FAINT }, response.hovered);
            if self.shows_focus(&response) {
                focus_ring(&mut self.canvas, chip);
            }
        }
        (changed, height)
    }

    // A bar filled from the left to `fraction`, from 0 to 1, in `color`.
    pub fn progress(&mut self, area: Rect, fraction: f32, color: Color) {
        let canvas = &mut self.canvas;
        canvas.rect(area.x, area.y, area.width, area.height, INSET);
        let filled = ((area.width - 2.0) * fraction.clamp(0.0, 1.0)).round();
        canvas.rect(area.x + 1.0, area.y + 1.0, filled, area.height - 2.0, color);
        canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
    }
}

fn kind_of_part(kind: &str) -> &'static str {
    match kind {
        "Tabs" => "Tab",
        _ => "RadioButton",
    }
}

// A control's label, after its mark, centered down its area.
fn label_after(canvas: &mut Canvas, area: Rect, mark_right: f32, label: &str, enabled: bool) {
    let x = mark_right + LABEL_GAP;
    let y = area.y + ((area.height - canvas.line_height()) * 0.5).round();
    canvas.text_clipped(x, y, label, area.right() - x, if enabled { TEXT } else { FAINT });
}
