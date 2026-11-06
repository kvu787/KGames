// What the editors share: where their cards and views are, the view of the
// ground that they pan and zoom, where points on the ground are on the screen
// for what they draw over the view, and the controls that change the
// document: sliders of its numbers, fields of its colors, and the controls
// that paint a model's slots with SimplePaint. They're Library/UserInterface's
// widgets, laid out with its Form, with colors from the theme's roles and
// sizes from its tokens.
//
// Programs only support two window setups (see Library/UserInterface's
// README), so the editors' cards have fixed places in pixels: the app bar at
// the top, with the tabs, a card at the top of the page's left side, as tall
// as what's in it, and in the vehicle editor, another at the top of its right
// side, the status bar at the bottom, and the 3D view under all of them,
// which fills the window. The cards float over the view, so it takes input
// only where they don't cover it.

use simple_paint::Parameters;
use user_interface::controls::TEXT_BOX_HEADER_GAP;
use user_interface::tokens::{APP_BAR_HEIGHT, BODY, CONTROL_HEIGHT, PAGE_PADDING, SPACE_S, STATUS_BAR_HEIGHT};
use user_interface::{Color, ColorField, ComboBox, Cursor, Form, Key, MouseButton, Painter, Point, Rect, Response, Slider, Ui, View};

use crate::camera::{self, Eye};
use crate::files::Color as FileColor;

// How wide the editors' cards are.
pub const CARD_WIDTH: f32 = 520.0;
// How tall a control is with its header above it, as Form::row puts it.
pub const HEADED_HEIGHT: f32 = BODY.line_height + TEXT_BOX_HEADER_GAP + CONTROL_HEIGHT;

// Where the editors' card down the left side of the page may go.
pub fn left_card(page: Rect) -> Rect {
    page.left_strip(CARD_WIDTH)
}

// Where the vehicle editor's second card, down the right side, may go.
pub fn right_card(page: Rect) -> Rect {
    page.right_strip(CARD_WIDTH)
}

// Where the 3D view takes input: the window between the app bar, under the
// PerfBar, and the status bar, all the way across. The cards, and
// the notices under the app bar, cover it.
pub fn view_area(ui: &Ui) -> Rect {
    let size = ui.size();
    let top = ui.content_top() + APP_BAR_HEIGHT;
    Rect::new(0.0, top, size.width, size.height - top - STATUS_BAR_HEIGHT)
}

// The part of the view that a card down the left side leaves, whose middle
// is where new things go.
pub fn visible(view: Rect) -> Rect {
    Rect::new(view.x + PAGE_PADDING + CARD_WIDTH, view.y, view.width - PAGE_PADDING - CARD_WIDTH, view.height)
}

pub fn window_size(ui: &Ui) -> [f32; 2] {
    [ui.size().width, ui.size().height]
}

// What happened in an editor's view in a frame.
pub struct ViewInput {
    pub response: Response,
    // Whether the left button's press on the view is still down, which is
    // what edits and pans.
    pub held: bool,
    // The point on the ground under the pointer, if it's on the view.
    pub ground: Option<[f64; 2]>,
    pub cursor: Option<[f32; 2]>,
    // Whether Space is down, so that a drag pans instead of editing.
    pub panning: bool,
}

// A view of the ground for editing: straight down, or at the game's angle.
// Space and a drag pan it, and the wheel zooms it around the pointer. It
// fills the window, so its middle is the window's.
pub struct Viewport {
    pub eye: Eye,
    // The point on the ground that a pan holds under the pointer.
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

    // The view as Library/UserInterface's View, named like "Terrain/View",
    // in `area`: its presses, which take keyboard focus from a text field so
    // that the keys after them, like Space, are the view's, and the pan and
    // zoom.
    pub fn interact(&mut self, ui: &mut Ui, name: &str, area: Rect) -> ViewInput {
        let size = window_size(ui);
        let space = ui.key_down(Key::SPACE);
        let view = if space || self.grab.is_some() { View::new().cursor(Cursor::Move) } else { View::new() };
        let response = ui.add(name, area, view);
        let held = response.held && response.held_button == Some(MouseButton::Left);
        let cursor = response.pointer.map(|point| [point.x, point.y]);
        if response.wheel.y != 0.0
            && let Some(cursor) = cursor
        {
            self.eye.zoom_at(camera::NOTCH.powf(-f64::from(response.wheel.y)), cursor, size);
        }
        if response.pressed && space {
            self.grab = cursor.map(|cursor| self.eye.to_ground(cursor, size));
        }
        if !held {
            self.grab = None;
        }
        if let (Some(grab), Some(cursor)) = (self.grab, cursor) {
            let under = self.eye.to_ground(cursor, size);
            self.eye.center[0] += grab[0] - under[0];
            self.eye.center[1] += grab[1] - under[1];
        }
        ViewInput {
            response,
            held,
            ground: cursor.map(|cursor| self.eye.to_ground(cursor, size)),
            cursor,
            panning: space || self.grab.is_some(),
        }
    }
}

// The nearest of some points on the ground to the pointer, within `reach`
// pixels of it on the screen.
pub fn nearest(eye: &Eye, size: [f32; 2], cursor: [f32; 2], points: &[[f64; 2]], reach: f32) -> Option<usize> {
    points
        .iter()
        .map(|&point| {
            let [x, y] = eye.ground_to_screen(point, size);
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

// A file's color as the interface takes it.
pub fn interface_color([red, green, blue]: FileColor) -> Color {
    Color::from_bytes([red, green, blue, 255])
}

// Where points on the ground are on the screen, in a view that fills the
// window, for what an editor draws over its view. It draws before its cards,
// clipped to the view, so that the cards and the bars go over it.
#[derive(Clone, Copy)]
pub struct Projection {
    eye: Eye,
    size: [f32; 2],
}

impl Projection {
    pub fn new(ui: &Ui, eye: Eye) -> Projection {
        Projection { eye, size: window_size(ui) }
    }

    pub fn point(&self, ground: [f64; 2]) -> Point {
        let [x, y] = self.eye.ground_to_screen(ground, self.size);
        Point::new(x, y)
    }

    // A closed outline through points on the ground.
    pub fn outline(&self, painter: &mut Painter, points: &[[f64; 2]], width: f32, color: Color) {
        let mut screen: Vec<Point> = points.iter().map(|&point| self.point(point)).collect();
        if let Some(&first) = screen.first() {
            screen.push(first);
        }
        painter.polyline(&screen, width, color);
    }
}

// The values a slider makes: from `start` at its left end to `end` at its
// right, a whole number of steps, between `minimum` and `maximum`, which can
// be inside its ends, like for a value that must be above 0, shown with
// `decimals` and the suffix. A value from a file can be between steps, and
// stays as it is until the slider moves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderRange {
    pub start: f64,
    pub end: f64,
    pub minimum: f64,
    pub maximum: f64,
    // Steps per unit, like 1000 for steps of 0.001.
    pub steps: f64,
    pub decimals: usize,
    pub suffix: &'static str,
}

impl SliderRange {
    pub const fn new(start: f64, end: f64, steps: f64, decimals: usize) -> SliderRange {
        SliderRange {
            start,
            end,
            minimum: start,
            maximum: end,
            steps,
            decimals,
            suffix: "",
        }
    }

    pub const fn between(self, minimum: f64, maximum: f64) -> SliderRange {
        SliderRange { minimum, maximum, ..self }
    }

    // What the value shows with after it, like "°".
    pub const fn suffix(self, suffix: &'static str) -> SliderRange {
        SliderRange { suffix, ..self }
    }

    // The step nearest `value`, within the range. Dividing whole numbers,
    // like 107 / 1000, gives the double nearest the decimal, so values save
    // as they show, like 0.107.
    pub fn nearest(&self, value: f64) -> f64 {
        ((value * self.steps).round() / self.steps).clamp(self.minimum, self.maximum)
    }
}

// How tall a slider is with its header.
pub fn slider_height() -> f32 {
    Slider::<f64>::height(true)
}

// A slider of one of the document's numbers, with its label and its value
// above it, in `rect`, slider_height() tall. Only input moves the value, so
// that one from a file stays as it is until then. Returns whether it changed
// the value.
pub fn slider(ui: &mut Ui, name: &str, rect: Rect, label: &str, value: &mut f64, range: SliderRange) -> bool {
    let before = *value;
    let slider = Slider::new(value, range.start, range.end)
        .step(1.0 / range.steps)
        .allowed(range.minimum, range.maximum)
        .header(label)
        .show_value(range.decimals)
        .suffix(range.suffix);
    if ui.add(name, rect, slider).changed {
        *value = range.nearest(*value);
    }
    *value != before
}

// A slider of its own row of a form.
pub fn slider_row(form: &mut Form, ui: &mut Ui, name: &str, label: &str, value: &mut f64, range: SliderRange) -> bool {
    let rect = form.take(slider_height());
    slider(ui, name, rect, label, value, range)
}

// A control's header above it, in a column of a form's row that's
// HEADED_HEIGHT tall, as Form::row puts it in a row of its own. Returns the
// control's rectangle.
pub fn header(ui: &mut Ui, column: Rect, label: &str) -> Rect {
    let color = ui.theme().text_primary;
    ui.text(column.top_strip(BODY.line_height), label, BODY, color);
    column.bottom_strip(CONTROL_HEIGHT)
}

// A field of one of the document's colors, like "#1CDA61", with a swatch of
// it, in `rect`. What's typed takes effect with Enter. It takes keyboard
// focus, unlike the editors' other controls. Returns whether it changed the
// color.
pub fn color_field(ui: &mut Ui, name: &str, rect: Rect, color: &mut FileColor) -> bool {
    let mut shown = interface_color(*color);
    ui.set_focusable(true);
    let response = ui.add(name, rect, ColorField::new(&mut shown));
    ui.set_focusable(false);
    let [red, green, blue, _] = shown.to_bytes();
    if response.changed && [red, green, blue] != *color {
        *color = [red, green, blue];
        return true;
    }
    false
}

// The controls of a SimplePaint material, for one of a model's slots, which
// the combo box picks, with a swatch of its base color: the base color's
// channels and the dark point down the left, and the brightness, the shift,
// the rotation and the light point down the right. Returns whether the
// material changed.
pub fn paint(form: &mut Form, ui: &mut Ui, name: &str, slots: &[String], slot: &mut usize, parameters: &mut Parameters) -> bool {
    let mut row = form.control(ui, "Slot");
    let swatch_area = row.cut_right(CONTROL_HEIGHT, SPACE_S);
    let names: Vec<&str> = slots.iter().map(String::as_str).collect();
    ui.add(&format!("{name}/Slot"), row, ComboBox::new(slot, &names));
    let [red, green, blue] = parameters.base_color_srgb.map(|channel| channel as f32);
    ui.swatch(swatch_area, Color::rgb(red, green, blue));
    let unit = SliderRange::new(0.0, 1.0, 1000.0, 3);
    let (lowest, highest) = (simple_paint::MARGIN, simple_paint::INTERIOR_MAXIMUM);
    let degrees = SliderRange::new(0.0, 360.0, 10.0, 1).between(0.0, 359.9).suffix("°");
    let [red, green, blue] = &mut parameters.base_color_srgb;
    let rows = [
        [
            ("Red", "Red", red, unit.between(lowest, highest)),
            ("Brightness", "Brightness", &mut parameters.brightness, unit.between(lowest, highest)),
        ],
        [
            ("Green", "Green", green, unit.between(lowest, highest)),
            ("Shift", "Shift", &mut parameters.shift, unit.between(0.0, highest)),
        ],
        [
            ("Blue", "Blue", blue, unit.between(lowest, highest)),
            ("Rotation", "Rotation", &mut parameters.rotation_degrees, degrees),
        ],
        [
            ("DarkPoint", "Dark point", &mut parameters.dark_point, unit.between(0.0, highest)),
            ("LightPoint", "Light point", &mut parameters.light_point, unit.between(lowest, 1.0)),
        ],
    ];
    let mut changed = false;
    for sliders in rows {
        let columns = form.columns::<2>(slider_height());
        for ((part, label, value, range), column) in sliders.into_iter().zip(columns) {
            changed |= slider(ui, &format!("{name}/{part}"), column, label, value, range);
        }
    }
    changed
}
