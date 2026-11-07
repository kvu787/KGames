// What the editors share: where their cards and views are, the view of the
// ground that they pan and zoom, where points on the ground are on the screen
// for what they draw over the view, and the controls that change the
// document: sliders of its numbers, fields of its colors, and the controls
// that paint a model's slots with SimplePaint. They're Library/UserInterface's
// widgets, laid out with its Form, with colors from the theme's roles and
// sizes from its tokens.
//
// Each control is a node made at build, whose events the editor reads each
// frame and whose value sync pushes back; the overlay an editor draws over
// its view is an OverlayCanvas node of shapes in screen space, swapped each
// frame.
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
use user_interface::{Canvas, Color, ColorField, ComboBox, Cursor, Form, Key, MouseButton, Painter, Point, Rect, Response, Slider, Text, Ui, View};

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

    // The editor's View node, named like "Terrain/View".
    pub fn build(&self, ui: &mut Ui, name: &str, area: Rect) {
        ui.node(name, area, View::new());
    }

    // What the frame's input did to the view: its presses, which take
    // keyboard focus from a text field so that the keys after them, like
    // Space, are the view's, and the pan and zoom.
    pub fn apply(&mut self, ui: &mut Ui, name: &str) -> ViewInput {
        let size = window_size(ui);
        let space = ui.key_down(Key::SPACE);
        let response = ui.response(name);
        let cursor = if space || self.grab.is_some() { Some(Cursor::Move) } else { None };
        ui.get_mut::<View>(name).set_cursor(cursor);
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
// window, for what an editor draws over its view.
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

    // A closed outline through points on the ground, as an overlay shape.
    pub fn outline(&self, points: &[[f64; 2]], width: f32, role: Role) -> Shape {
        let mut screen: Vec<Point> = points.iter().map(|&point| self.point(point)).collect();
        if let Some(&first) = screen.first() {
            screen.push(first);
        }
        Shape::Polyline(screen, width, role)
    }
}

// A theme role that an overlay shape is colored with, resolved as it draws,
// so that it follows a theme switch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Role {
    Accent,
    Primary,
    Secondary,
    Acrylic,
}

impl Role {
    fn resolve(self, theme: &user_interface::Theme) -> Color {
        match self {
            Role::Accent => theme.accent,
            Role::Primary => theme.text_primary,
            Role::Secondary => theme.text_secondary,
            Role::Acrylic => theme.acrylic,
        }
    }
}

// What an editor draws over its view, in screen space.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Polyline(Vec<Point>, f32, Role),
    Circle(Point, f32, Role),
    Ring(Point, f32, f32, Role),
    Line(Point, Point, f32, Role),
}

// The overlay's node: shapes over the view, clipped to it, under the cards.
pub fn overlay_node(ui: &mut Ui, name: &str, view: Rect) {
    ui.node(name, view, Canvas::new(Vec::new(), draw_overlay));
}

pub fn set_overlay(ui: &mut Ui, name: &str, shapes: Vec<Shape>) {
    ui.get_mut::<Canvas<Vec<Shape>>>(name).data = shapes;
}

fn draw_overlay(ui: &mut Ui, view: Rect, shapes: &Vec<Shape>) {
    let theme = ui.theme().clone();
    let mut painter = ui.painter();
    painter.push_clip(view);
    for shape in shapes {
        draw_shape(&mut painter, &theme, shape);
    }
    painter.pop_clip();
}

fn draw_shape(painter: &mut Painter, theme: &user_interface::Theme, shape: &Shape) {
    match shape {
        Shape::Polyline(points, width, role) => painter.polyline(points, *width, role.resolve(theme)),
        Shape::Circle(center, radius, role) => painter.circle(*center, *radius, role.resolve(theme)),
        Shape::Ring(center, radius, width, role) => painter.ring(*center, *radius, *width, role.resolve(theme)),
        Shape::Line(from, to, width, role) => painter.line(*from, *to, *width, role.resolve(theme)),
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
    Slider::height(true)
}

// A slider of one of the document's numbers, with its label and its value
// above it, in `rect`, slider_height() tall.
pub fn slider_node(ui: &mut Ui, name: &str, rect: Rect, label: &str, value: f64, range: SliderRange) {
    let slider = Slider::new(value, range.start, range.end)
        .step(1.0 / range.steps)
        .allowed(range.minimum, range.maximum)
        .header(label)
        .show_value(range.decimals)
        .suffix(range.suffix);
    ui.node(name, rect, slider);
}

// A slider of its own row of a form.
pub fn slider_row(form: &mut Form, ui: &mut Ui, name: &str, label: &str, value: f64, range: SliderRange) {
    let rect = form.take(slider_height());
    slider_node(ui, name, rect, label, value, range);
}

// The value the slider moved to this frame, on its steps, if it moved.
pub fn slider_changed(ui: &Ui, name: &str, range: SliderRange) -> Option<f64> {
    ui.changed(name).map(|value| range.nearest(value))
}

// Pushes the document's value back into the slider, unless it's being
// dragged.
pub fn sync_slider(ui: &mut Ui, name: &str, value: f64) {
    if !ui.response(name).held {
        ui.get_mut::<Slider>(name).set_value(value);
    }
}

// A control's header above it, in a column of a form's row that's
// HEADED_HEIGHT tall, as Form::row puts it in a row of its own. Returns the
// control's rectangle.
pub fn header(ui: &mut Ui, name: &str, column: Rect, label: &str) -> Rect {
    ui.node(name, column.top_strip(BODY.line_height), Text::label(label));
    column.bottom_strip(CONTROL_HEIGHT)
}

// A field of one of the document's colors, like "#1CDA61", with a swatch of
// it, in `rect`. What's typed takes effect with Enter. It takes keyboard
// focus, unlike the editors' other controls.
pub fn color_field_node(ui: &mut Ui, name: &str, rect: Rect, color: FileColor) {
    ui.node(name, rect, ColorField::new(interface_color(color)));
    ui.set_focusable(name, true);
}

// The color the field took this frame, if it changed.
pub fn color_field_changed(ui: &Ui, name: &str, color: FileColor) -> Option<FileColor> {
    let changed = ui.color_changed(name)?;
    let [red, green, blue, _] = changed.to_bytes();
    ([red, green, blue] != color).then_some([red, green, blue])
}

pub fn sync_color_field(ui: &mut Ui, name: &str, color: FileColor) {
    let shown = interface_color(color);
    let field = ui.get_mut::<ColorField>(name);
    if field.color() != shown {
        field.set_color(shown);
    }
}

// The controls of a SimplePaint material, for one of a model's slots, which
// the combo box picks, with a swatch of its base color: the base color's
// channels and the dark point down the left, and the brightness, the shift,
// the rotation and the light point down the right.
pub struct PaintControls;

// The sliders of a paint's parameters, by part name.
const PAINT_PARTS: [(&str, &str); 8] = [
    ("Red", "Red"),
    ("Brightness", "Brightness"),
    ("Green", "Green"),
    ("Shift", "Shift"),
    ("Blue", "Blue"),
    ("Rotation", "Rotation"),
    ("DarkPoint", "Dark point"),
    ("LightPoint", "Light point"),
];

fn paint_range(part: &str) -> SliderRange {
    let unit = SliderRange::new(0.0, 1.0, 1000.0, 3);
    let (lowest, highest) = (simple_paint::MARGIN, simple_paint::INTERIOR_MAXIMUM);
    match part {
        "Red" | "Green" | "Blue" | "Brightness" => unit.between(lowest, highest),
        "Shift" | "DarkPoint" => unit.between(0.0, highest),
        "LightPoint" => unit.between(lowest, 1.0),
        _ => SliderRange::new(0.0, 360.0, 10.0, 1).between(0.0, 359.9).suffix("°"),
    }
}

fn paint_value(parameters: &Parameters, part: &str) -> f64 {
    match part {
        "Red" => parameters.base_color_srgb[0],
        "Green" => parameters.base_color_srgb[1],
        "Blue" => parameters.base_color_srgb[2],
        "Brightness" => parameters.brightness,
        "Shift" => parameters.shift,
        "DarkPoint" => parameters.dark_point,
        "LightPoint" => parameters.light_point,
        _ => parameters.rotation_degrees,
    }
}

fn set_paint_value(parameters: &mut Parameters, part: &str, value: f64) {
    match part {
        "Red" => parameters.base_color_srgb[0] = value,
        "Green" => parameters.base_color_srgb[1] = value,
        "Blue" => parameters.base_color_srgb[2] = value,
        "Brightness" => parameters.brightness = value,
        "Shift" => parameters.shift = value,
        "DarkPoint" => parameters.dark_point = value,
        "LightPoint" => parameters.light_point = value,
        _ => parameters.rotation_degrees = value,
    }
}

fn draw_paint_swatch(ui: &mut Ui, rect: Rect, color: &[f32; 3]) {
    ui.swatch(rect, Color::rgb(color[0], color[1], color[2]));
}

impl PaintControls {
    // The nodes, in the form: the slot's combo box with its swatch, then the
    // sliders.
    pub fn build(form: &mut Form, ui: &mut Ui, name: &str, slots: &[String], slot: usize, parameters: &Parameters) {
        let mut row = form.control(ui, "Slot");
        let swatch_area = row.cut_right(CONTROL_HEIGHT, SPACE_S);
        let names: Vec<&str> = slots.iter().map(String::as_str).collect();
        ui.node(&format!("{name}/Slot"), row, ComboBox::new(&names, slot));
        let color = parameters.base_color_srgb.map(|channel| channel as f32);
        ui.node(&format!("{name}/Swatch"), swatch_area, Canvas::new(color, draw_paint_swatch));
        for pair in PAINT_PARTS.chunks(2) {
            let columns = form.columns::<2>(slider_height());
            for ((part, label), column) in pair.iter().zip(columns) {
                slider_node(ui, &format!("{name}/{part}"), column, label, paint_value(parameters, part), paint_range(part));
            }
        }
    }

    // The slot picked and the parameters as the frame's events left them.
    // Returns whether the parameters changed.
    pub fn changed(ui: &Ui, name: &str, slot: &mut usize, parameters: &mut Parameters) -> bool {
        if let Some(picked) = ui.selected(&format!("{name}/Slot")) {
            *slot = picked;
        }
        let mut changed = false;
        for (part, _) in PAINT_PARTS {
            if let Some(value) = slider_changed(ui, &format!("{name}/{part}"), paint_range(part)) {
                set_paint_value(parameters, part, value);
                changed = true;
            }
        }
        changed
    }

    // Pushes the document's paint back into the controls.
    pub fn sync(ui: &mut Ui, name: &str, slots: &[String], slot: usize, parameters: &Parameters) {
        let names: Vec<&str> = slots.iter().map(String::as_str).collect();
        let combo = ui.get_mut::<ComboBox>(&format!("{name}/Slot"));
        combo.set_items(&names);
        combo.set_selected(slot);
        ui.get_mut::<Canvas<[f32; 3]>>(&format!("{name}/Swatch")).data = parameters.base_color_srgb.map(|channel| channel as f32);
        for (part, _) in PAINT_PARTS {
            sync_slider(ui, &format!("{name}/{part}"), paint_value(parameters, part));
        }
    }
}
