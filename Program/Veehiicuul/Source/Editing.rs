// What the editors share: where their panels and views are, the view of the
// ground that they pan and zoom, controls with labels, color fields, and the
// panel that paints a model's slots with SimplePaint.
//
// Programs only support two window setups (see Library/Application), so the
// editors' panels have fixed places in pixels: the tabs at the top, a panel
// down the left side, and the 3D view in the rest, which fills the window
// under them.

use simple_paint::Parameters;
use user_interface::theme::{BORDER, DIM, TEXT, TITLE};
use user_interface::{Choice, Color, Frame, MARGIN, Rect, Response, Sense, SliderRange, Stack};

use crate::camera::{self, Eye};
use crate::files::{self, Color as FileColor};

pub const PANEL_WIDTH: f32 = 560.0;
pub const TOP: f32 = 64.0;
// The panel's controls' labels, and the values after sliders.
const LABEL_WIDTH: f32 = 150.0;
const VALUE_WIDTH: f32 = 90.0;

// The editors' panel, down the left side under the tabs.
pub fn panel_area(frame: &Frame) -> Rect {
    Rect::new(MARGIN, TOP, PANEL_WIDTH, frame.height - TOP - MARGIN)
}

// The area of the 3D view that the editors take input in: right of the
// panel, and under the tabs, to the window's edges.
pub fn view_area(frame: &Frame) -> Rect {
    let left = MARGIN * 2.0 + PANEL_WIDTH;
    Rect::new(left, TOP, frame.width - left, frame.height - TOP)
}

// What happened in an editor's view in a frame.
pub struct ViewInput {
    pub response: Response,
    // The point on the ground under the cursor, if it's over the view.
    pub ground: Option<[f64; 2]>,
    pub cursor: Option<[f32; 2]>,
    // Whether Space is down, so that a drag pans instead of editing.
    pub panning: bool,
}

// A view of the ground for editing: straight down, or at the game's angle.
// Space and a drag pan it, and the wheel zooms it around the cursor.
pub struct Viewport {
    pub eye: Eye,
    // The point on the ground that a pan holds under the cursor.
    grab: Option<[f64; 2]>,
}

impl Viewport {
    pub fn new(center: [f64; 2], height: f64) -> Viewport {
        Viewport {
            eye: Eye::top_down(center, height),
            grab: None,
        }
    }

    pub fn game_angle(&self) -> bool {
        self.eye.pitch < std::f64::consts::FRAC_PI_2 - 1e-6
    }

    pub fn set_game_angle(&mut self, game: bool) {
        self.eye.pitch = if game { camera::PITCH } else { std::f64::consts::FRAC_PI_2 };
    }

    pub fn interact(&mut self, frame: &mut Frame, name: &str) -> ViewInput {
        let area = view_area(frame);
        let size = [frame.width, frame.height];
        let sense = Sense {
            click: true,
            wheel: true,
            ..Sense::HOVER
        };
        let response = frame.interact(name, "View", area, sense);
        let cursor = frame.cursor().filter(|&cursor| response.hovered || response.held || area.contains(cursor));
        if response.wheel != 0.0
            && let Some(cursor) = cursor
        {
            self.eye.zoom_at(camera::NOTCH.powf(-response.wheel as f64), cursor, size);
        }
        let panning = frame.held(0x20);
        if response.pressed && panning {
            self.grab = cursor.map(|cursor| self.eye.to_ground(cursor, size));
        }
        if !response.held {
            self.grab = None;
        }
        if let (Some(grab), Some(cursor)) = (self.grab, cursor) {
            let under = self.eye.to_ground(cursor, size);
            self.eye.center[0] += grab[0] - under[0];
            self.eye.center[1] += grab[1] - under[1];
        }
        ViewInput {
            response,
            ground: cursor.map(|cursor| self.eye.to_ground(cursor, size)),
            cursor,
            panning: panning || self.grab.is_some(),
        }
    }

    // Where a point on the ground is on the screen.
    pub fn screen(&self, frame: &Frame, point: [f64; 2]) -> [f32; 2] {
        self.eye.ground_to_screen(point, [frame.width, frame.height])
    }
}

// A line of text in the panel.
pub fn label(frame: &mut Frame, stack: &mut Stack, text: &str, color: Color) {
    let area = stack.next(frame.line_height());
    frame.canvas.text(area.x, area.y, text, color);
}

// A heading in the panel, a little apart from what's above it.
pub fn heading(frame: &mut Frame, stack: &mut Stack, text: &str) {
    stack.skip(6.0);
    let area = stack.next(frame.line_height());
    frame.canvas.text(area.x, area.y, text, TITLE);
    frame.canvas.rect(area.x, area.bottom() + 2.0, area.width, 1.0, BORDER);
    stack.skip(4.0);
}

// A slider with its label before it and its value after it. Returns whether
// it changed the value.
pub fn slider(
    frame: &mut Frame,
    stack: &mut Stack,
    name: &str,
    label: &str,
    value: &mut f64,
    range: &SliderRange,
    format: impl Fn(f64) -> String,
) -> bool {
    let area = stack.next(frame.control_height());
    let (label_area, rest) = area.split_left(LABEL_WIDTH, 0.0);
    let (value_area, track) = rest.split_right(VALUE_WIDTH, 8.0);
    let text_y = area.y + (area.height - frame.line_height()) / 2.0;
    frame.canvas.text(label_area.x, text_y, label, DIM);
    let changed = frame.slider(name, track, value, range).changed;
    frame.canvas.text(value_area.x, text_y, &format(*value), TEXT);
    changed
}

// A select box with its label before it. Returns whether it changed.
pub fn select(frame: &mut Frame, stack: &mut Stack, name: &str, label: &str, labels: &[&str], selected: &mut usize) -> bool {
    let area = stack.next(frame.control_height());
    let (label_area, rest) = area.split_left(LABEL_WIDTH, 0.0);
    let text_y = area.y + (area.height - frame.line_height()) / 2.0;
    frame.canvas.text(label_area.x, text_y, label, DIM);
    let choices: Vec<Choice> = labels.iter().map(|label| Choice { label, detail: "" }).collect();
    frame.select(name, rest, &choices, selected).changed
}

// A field for a color, like "#1CDA61", with a swatch of it. The color
// changes as soon as the field holds one. `text` keeps what's typed.
pub fn color_field(frame: &mut Frame, stack: &mut Stack, name: &str, label: &str, text: &mut String, color: &mut FileColor) -> bool {
    let area = stack.next(frame.control_height());
    let (label_area, rest) = area.split_left(LABEL_WIDTH, 0.0);
    let (swatch, field) = rest.split_right(area.height, 8.0);
    let text_y = area.y + (area.height - frame.line_height()) / 2.0;
    frame.canvas.text(label_area.x, text_y, label, DIM);
    // What's typed shows the color, unless it's being typed.
    let focused = frame.focus_name() == Some(name);
    if !focused {
        *text = files::color_text(*color);
    }
    frame.text_field(name, field, text, "#RRGGBB", 7);
    let mut changed = false;
    if let Some(parsed) = files::parse_color(text.trim())
        && parsed != *color
    {
        *color = parsed;
        changed = true;
    }
    swatch_rect(frame, swatch, *color);
    changed
}

pub fn swatch_rect(frame: &mut Frame, area: Rect, color: FileColor) {
    let [red, green, blue] = color;
    let fill = Color::rgb(((red as u32) << 16) | ((green as u32) << 8) | blue as u32);
    frame.canvas.rect(area.x, area.y, area.width, area.height, fill);
    frame.canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
}

// A row of buttons, as wide as the panel together. Returns the index of the
// one clicked, if any.
pub fn buttons(frame: &mut Frame, stack: &mut Stack, names: &[(&str, &str)]) -> Option<usize> {
    let area = stack.next(frame.control_height());
    let mut clicked = None;
    for (index, (column, (name, label))) in area.columns(names.len(), 8.0).into_iter().zip(names).enumerate() {
        if frame.button(name, column, label).clicked {
            clicked = Some(index);
        }
    }
    clicked
}

// The sliders of a SimplePaint material, for one of a model's slots, which
// the select box picks. Returns whether the material changed.
pub fn paint_panel(
    frame: &mut Frame,
    stack: &mut Stack,
    name: &str,
    slots: &[String],
    slot: &mut usize,
    parameters: &mut Parameters,
) -> bool {
    let labels: Vec<&str> = slots.iter().map(String::as_str).collect();
    select(frame, stack, &format!("{name}/Slot"), "Slot", &labels, slot);
    let lowest = 1.0 / 1024.0;
    let highest = 1.0 - 1.0 / 1024.0;
    let unit = |minimum: f64, maximum: f64| SliderRange {
        start: 0.0,
        end: 1.0,
        minimum,
        maximum,
        steps: 1000.0,
    };
    let fraction = |value: f64| format!("{value:.3}");
    let mut changed = false;
    let [red, green, blue] = &mut parameters.base_color_srgb;
    for (part, label, value) in [("Red", "Red", red), ("Green", "Green", green), ("Blue", "Blue", blue)] {
        changed |= slider(frame, stack, &format!("{name}/{part}"), label, value, &unit(lowest, highest), fraction);
    }
    changed |= slider(frame, stack, &format!("{name}/Brightness"), "Brightness", &mut parameters.brightness, &unit(lowest, highest), fraction);
    changed |= slider(frame, stack, &format!("{name}/Shift"), "Shift", &mut parameters.shift, &unit(0.0, highest), fraction);
    let degrees = SliderRange {
        start: 0.0,
        end: 360.0,
        minimum: 0.0,
        maximum: 359.9,
        steps: 10.0,
    };
    changed |= slider(frame, stack, &format!("{name}/Rotation"), "Rotation", &mut parameters.rotation_degrees, &degrees, |value| format!("{value:.1} deg"));
    changed |= slider(frame, stack, &format!("{name}/DarkPoint"), "Dark point", &mut parameters.dark_point, &unit(0.0, highest), fraction);
    changed |= slider(frame, stack, &format!("{name}/LightPoint"), "Light point", &mut parameters.light_point, &unit(lowest, 1.0), fraction);
    changed
}

// Draws a closed outline of points on the ground over the 3D view.
pub fn outline(frame: &mut Frame, viewport: &Viewport, points: &[[f64; 2]], width: f32, color: Color) {
    for (index, &point) in points.iter().enumerate() {
        let next = points[(index + 1) % points.len()];
        let [from, to] = [viewport.screen(frame, point), viewport.screen(frame, next)];
        line(frame, from, to, width, color);
    }
}

// A line over the 3D view, cut where it leaves the editors' view, so that
// it doesn't cross the panel or the tabs.
pub fn line(frame: &mut Frame, from: [f32; 2], to: [f32; 2], width: f32, color: Color) {
    let area = view_area(frame);
    // Liang and Barsky's clipping, along the line from 0 to 1.
    let [dx, dy] = [to[0] - from[0], to[1] - from[1]];
    let (mut start, mut end) = (0.0_f32, 1.0_f32);
    for (p, q) in [
        (-dx, from[0] - area.x),
        (dx, area.right() - from[0]),
        (-dy, from[1] - area.y),
        (dy, area.bottom() - from[1]),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                start = start.max(t);
            } else {
                end = end.min(t);
            }
        }
    }
    if start <= end {
        let at = |t: f32| [from[0] + dx * t, from[1] + dy * t];
        frame.canvas.line(at(start), at(end), width, color);
    }
}

// A dot over the 3D view, if it's in the editors' view.
pub fn dot(frame: &mut Frame, center: [f32; 2], radius: f32, color: Color) {
    if view_area(frame).contains(center) {
        frame.canvas.circle(center, radius, color);
    }
}

pub fn ring(frame: &mut Frame, center: [f32; 2], radius: f32, width: f32, color: Color) {
    if view_area(frame).contains(center) {
        frame.canvas.ring(center, radius, width, color);
    }
}

// The nearest of some points on the ground to the cursor on the screen,
// within `reach` pixels.
pub fn nearest(frame: &Frame, viewport: &Viewport, cursor: [f32; 2], points: &[[f64; 2]], reach: f32) -> Option<usize> {
    points
        .iter()
        .map(|&point| {
            let [x, y] = viewport.screen(frame, point);
            (x - cursor[0]).hypot(y - cursor[1])
        })
        .enumerate()
        .filter(|(_, distance)| *distance <= reach)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(index, _)| index)
}

// The heading, in degrees from +Z toward +X, from one point toward another.
pub fn heading_toward(from: [f64; 2], to: [f64; 2]) -> f64 {
    (to[0] - from[0]).atan2(to[1] - from[1]).to_degrees()
}
