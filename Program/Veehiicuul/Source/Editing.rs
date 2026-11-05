// What the editors share: where their cards and views are, the view of the
// ground that they pan and zoom, rows of controls with labels, color fields,
// and the controls that paint a model's slots with SimplePaint. They follow
// Library/UserInterface's design system: colors from the theme's roles, and
// sizes from its tokens.
//
// Programs only support two window setups (see Library/Application), so the
// editors' cards have fixed places in pixels: the app bar at the top, with
// the tabs, a card down the left side, and in the vehicle editor, another
// down the right, the status bar at the bottom, and the 3D view in the rest,
// which fills the window under them.

use simple_paint::Parameters;
use user_interface::{
    Align, Button, CONTROL_HEIGHT, Choice, Color, Frame, MARGIN, PANELS_TOP, RADIUS_CONTROL, Rect,
    Response, SPACE_M, SPACE_S, Sense, SliderRange, Stack, TextField, TextStyle,
};

use crate::camera::{self, Eye};
use crate::files::{self, Color as FileColor};

pub const CARD_WIDTH: f32 = 560.0;
// The width of a row's label, and of the value after a slider.
const LABEL_WIDTH: f32 = 136.0;
const VALUE_WIDTH: f32 = 88.0;

// The editors' card, down the left side, between the app bar and the status
// bar.
pub fn card_area(frame: &Frame) -> Rect {
    Rect::from_edges(MARGIN, PANELS_TOP, MARGIN + CARD_WIDTH, frame.content_bottom() - MARGIN)
}

// The vehicle editor's second card, down the right side.
pub fn right_card_area(frame: &Frame) -> Rect {
    Rect::from_edges(frame.width - MARGIN - CARD_WIDTH, PANELS_TOP, frame.width - MARGIN, frame.content_bottom() - MARGIN)
}

// The area of the 3D view that the editors take input in: right of the
// card, between the app bar and the status bar.
pub fn view_area(frame: &Frame) -> Rect {
    Rect::from_edges(MARGIN * 2.0 + CARD_WIDTH, PANELS_TOP, frame.width, frame.content_bottom())
}

// The stack of rows in an editor's card.
pub fn rows(inner: Rect) -> Stack {
    Stack::down(inner, SPACE_S)
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
    // Whether there's a card down the right side, which the view stops at.
    pub right_card: bool,
    // The point on the ground that a pan holds under the cursor.
    grab: Option<[f64; 2]>,
}

impl Viewport {
    pub fn new(center: [f64; 2], height: f64) -> Viewport {
        Viewport {
            eye: Eye::top_down(center, height),
            right_card: false,
            grab: None,
        }
    }

    // Where the view takes input, and what's drawn over it shows.
    pub fn area(&self, frame: &Frame) -> Rect {
        let area = view_area(frame);
        if self.right_card {
            Rect::from_edges(area.x, area.y, right_card_area(frame).x - MARGIN, area.bottom())
        } else {
            area
        }
    }

    pub fn game_angle(&self) -> bool {
        self.eye.pitch < std::f64::consts::FRAC_PI_2 - 1e-6
    }

    pub fn set_game_angle(&mut self, game: bool) {
        self.eye.pitch = if game { camera::PITCH } else { std::f64::consts::FRAC_PI_2 };
    }

    pub fn interact(&mut self, frame: &mut Frame, name: &str) -> ViewInput {
        let area = self.area(frame);
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

// A line of text in the card.
pub fn label(frame: &mut Frame, stack: &mut Stack, text: &str, color: Color) {
    let area = stack.next(frame.canvas.line_height(TextStyle::Body));
    frame.label(area, text, TextStyle::Body, color, Align::Start);
}

// A heading in the card, under a divider, a little apart from what's above
// it.
pub fn heading(frame: &mut Frame, stack: &mut Stack, text: &str) {
    stack.skip(SPACE_S);
    let line = stack.next(1.0);
    frame.divider([line.x, line.y], line.width);
    stack.skip(SPACE_S);
    let area = stack.next(frame.canvas.line_height(TextStyle::BodyStrong));
    let color = frame.theme().text;
    frame.label(area, text, TextStyle::BodyStrong, color, Align::Start);
}

// A row's label, before its control, and the area left for the control.
fn row(frame: &mut Frame, stack: &mut Stack, label: &str) -> Rect {
    let area = stack.next(CONTROL_HEIGHT);
    let (label_area, rest) = area.split_left(LABEL_WIDTH, 0.0);
    let color = frame.theme().text_secondary;
    frame.label(label_area, label, TextStyle::Body, color, Align::Start);
    rest
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
    let rest = row(frame, stack, label);
    let (value_area, rail) = rest.split_right(VALUE_WIDTH, SPACE_M);
    let changed = frame.slider(name, rail, value, range).changed;
    let color = frame.theme().text;
    frame.label(value_area, &format(*value), TextStyle::Body, color, Align::End);
    changed
}

// A combo box with its label before it. Returns whether it changed.
pub fn select(frame: &mut Frame, stack: &mut Stack, name: &str, label: &str, labels: &[&str], selected: &mut usize) -> bool {
    let rest = row(frame, stack, label);
    let choices: Vec<Choice> = labels.iter().map(|label| Choice { label, detail: "" }).collect();
    frame.select(name, rest, &choices, selected).changed
}

// A text field with its label before it.
pub fn text_field(frame: &mut Frame, stack: &mut Stack, name: &str, label: &str, text: &mut String, placeholder: &str) -> Response {
    let rest = row(frame, stack, label);
    frame.text_field(name, rest, text, TextField::new(placeholder).limit(40))
}

// A field for a color, like "#1CDA61", with a swatch of it. The color
// changes as soon as the field holds one. `text` keeps what's typed.
pub fn color_field(frame: &mut Frame, stack: &mut Stack, name: &str, label: &str, text: &mut String, color: &mut FileColor) -> bool {
    let rest = row(frame, stack, label);
    let (swatch, field) = rest.split_right(CONTROL_HEIGHT, SPACE_S);
    // What's typed shows the color, unless it's being typed.
    if frame.focus_name() != Some(name) {
        *text = files::color_text(*color);
    }
    frame.text_field(name, field, text, TextField::new("#RRGGBB").limit(7));
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

// A file's color as the canvas takes it.
pub fn canvas_color([red, green, blue]: FileColor) -> Color {
    Color::rgb(((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
}

pub fn swatch_rect(frame: &mut Frame, area: Rect, color: FileColor) {
    let stroke = frame.theme().control_stroke;
    frame.canvas.fill_stroke(area, RADIUS_CONTROL, canvas_color(color), 1.0, stroke);
}

// A row of buttons, as wide as the card together. Returns the index of the
// one clicked, if any.
pub fn buttons(frame: &mut Frame, stack: &mut Stack, buttons: &[(&str, Button)]) -> Option<usize> {
    let area = stack.next(CONTROL_HEIGHT);
    let mut clicked = None;
    for (index, (column, (name, button))) in area.columns(buttons.len(), SPACE_S).into_iter().zip(buttons).enumerate() {
        if frame.button(name, column, *button).clicked {
            clicked = Some(index);
        }
    }
    clicked
}

// The sliders of a SimplePaint material, for one of a model's slots, which
// the combo box picks. Returns whether the material changed.
pub fn paint_controls(
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
        line(frame, viewport, from, to, width, color);
    }
}

// A line over the 3D view, cut where it leaves the editor's view, so that
// it doesn't cross the cards, the app bar or the status bar.
pub fn line(frame: &mut Frame, viewport: &Viewport, from: [f32; 2], to: [f32; 2], width: f32, color: Color) {
    let area = viewport.area(frame);
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

// A dot over the 3D view, if it's in the editor's view.
pub fn dot(frame: &mut Frame, viewport: &Viewport, center: [f32; 2], radius: f32, color: Color) {
    if viewport.area(frame).contains(center) {
        frame.canvas.circle(center, radius, color);
    }
}

pub fn ring(frame: &mut Frame, viewport: &Viewport, center: [f32; 2], radius: f32, width: f32, color: Color) {
    if viewport.area(frame).contains(center) {
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

// The card's inner area, under its title, for the editors that draw it.
pub fn card(frame: &mut Frame, title: &str, subtitle: &str) -> Rect {
    let area = card_area(frame);
    frame.card(area, title, subtitle)
}

// The same for the card down the right side.
pub fn right_card(frame: &mut Frame, title: &str, subtitle: &str) -> Rect {
    let area = right_card_area(frame);
    frame.card(area, title, subtitle)
}
