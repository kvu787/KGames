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
use user_interface::{
    Color, ColorField, ComboBox, Cursor, Dispatch, Form, Interaction, Key, MouseButton, Painter, Point, Rect, Sense, Slider, Swatch,
    Text, Tree, Ui, ViewInput, Widget,
};

use crate::camera::{self, Eye};
use crate::files::Color as FileColor;
use crate::program::Message;

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

// The editors' and the game's view of the ground: a target that hears the
// pointer every frame, sends what it did with whether Space is down, and
// takes keyboard focus from a text field when pressed, so that the keys
// after it, like Space, are the view's.
pub struct GroundView {
    // Whether a pan drag was under way as of the last frame, for the cursor.
    pub panning: bool,
}

impl Widget<Message> for GroundView {
    fn kind(&self) -> &'static str {
        "View"
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::VIEW)
    }

    fn always_interacts(&self) -> bool {
        true
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<Message>) {
        let hit = interaction.whole();
        if hit.pressed || hit.secondary_pressed || hit.middle_pressed {
            ctx.clear_focus();
        }
        let space = ctx.key_down(Key::SPACE);
        if (space || self.panning) && (hit.hovered || hit.held) {
            ctx.set_cursor(Cursor::Move);
        }
        let input = ViewInput {
            hovered: hit.hovered,
            pointer: interaction.pointer,
            pressed: hit.pressed,
            secondary_pressed: hit.secondary_pressed,
            middle_pressed: hit.middle_pressed,
            double_clicked: hit.double_clicked,
            held: hit.held,
            held_button: hit.held_button,
            drag_delta: interaction.drag_delta,
            released: hit.released,
            wheel: hit.wheel,
            modifiers: ctx.modifiers(),
        };
        ctx.send(Message::View { input, space });
    }
}

// What happened in an editor's view in a frame, on the ground.
#[derive(Clone, Copy, Debug)]
pub struct GroundInput {
    // The left button's press, and whether it's still down, which is what
    // edits and pans.
    pub pressed: bool,
    pub held: bool,
    // The point on the ground under the pointer, if it's on the view.
    pub ground: Option<[f64; 2]>,
    pub cursor: Option<[f32; 2]>,
    // Whether Space is down or a pan drag is under way, so that a drag pans
    // instead of editing.
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

    pub fn panning(&self) -> bool {
        self.grab.is_some()
    }

    // What the view's input did to the viewport: the pan and the zoom, and
    // what's left for the editor's tool, on the ground.
    pub fn apply(&mut self, input: &ViewInput, space: bool, size: [f32; 2]) -> GroundInput {
        let held = input.holding(MouseButton::Left);
        let cursor = input.pointer.map(|point| [point.x, point.y]);
        if input.wheel.y != 0.0
            && let Some(cursor) = cursor
        {
            self.eye.zoom_at(camera::NOTCH.powf(-f64::from(input.wheel.y)), cursor, size);
        }
        if input.pressed && space {
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
        GroundInput {
            pressed: input.pressed,
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
    pub fn new(eye: Eye, size: [f32; 2]) -> Projection {
        Projection { eye, size }
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
    Slider::<Message>::height(true)
}

// A slider of one of the document's numbers, with its label and its value
// above it, slider_height() tall. Only input moves the value, so that one
// from a file stays as it is until then; what moving it sends carries the
// nearest step.
pub fn slider(label: &str, value: f64, range: SliderRange, to: impl Fn(f64) -> Message + 'static) -> Slider<Message> {
    Slider::new(value, range.start, range.end)
        .step(1.0 / range.steps)
        .allowed(range.minimum, range.maximum)
        .header(label)
        .show_value(range.decimals)
        .suffix(range.suffix)
        .on_change(move |value| to(range.nearest(value)))
}

// A slider on its own row of a form.
pub fn slider_row(form: &mut Form, tree: &mut Tree<Message>, name: &str, label: &str, value: f64, range: SliderRange, to: impl Fn(f64) -> Message + 'static) {
    let rect = form.take(slider_height());
    tree.add(name, rect, slider(label, value, range, to));
}

// A control's header above it, in a column of a form's row that's
// HEADED_HEIGHT tall, as Form::row puts it in a row of its own. Returns the
// control's rectangle.
pub fn header(tree: &mut Tree<Message>, column: Rect, label: &str) -> Rect {
    tree.draw(column.top_strip(BODY.line_height), Text::label(label));
    column.bottom_strip(CONTROL_HEIGHT)
}

// A field of one of the document's colors, like "#1CDA61", with a swatch of
// it. What's typed takes effect with Enter. It takes keyboard focus, unlike
// the editors' other controls, and what a change sends carries the color.
pub fn color_field(tree: &mut Tree<Message>, name: &str, rect: Rect, color: FileColor, to: impl Fn(FileColor) -> Message + 'static) {
    let field = ColorField::new(interface_color(color)).on_change(move |color| {
        let [red, green, blue, _] = color.to_bytes();
        to([red, green, blue])
    });
    tree.add(name, rect, field).set_focusable(true);
}

// The controls of a SimplePaint material, for one of a model's slots, which
// the combo box picks, with a swatch of its base color: the base color's
// channels and the dark point down the left, and the brightness, the shift,
// the rotation and the light point down the right. A slider's change sends
// the whole material, with its field on the nearest step.
pub fn paint(
    form: &mut Form,
    tree: &mut Tree<Message>,
    name: &str,
    slots: &[String],
    slot: usize,
    parameters: Parameters,
    to_slot: impl Fn(usize) -> Message + 'static,
    to_paint: impl Fn(Parameters) -> Message + Clone + 'static,
) {
    let mut row = form.control(tree, "Slot");
    let swatch_area = row.cut_right(CONTROL_HEIGHT, SPACE_S);
    let names: Vec<&str> = slots.iter().map(String::as_str).collect();
    tree.add(&format!("{name}/Slot"), row, ComboBox::new(slot, &names).on_select(to_slot));
    let [red, green, blue] = parameters.base_color_srgb.map(|channel| channel as f32);
    tree.draw(swatch_area, Swatch::new(Color::rgb(red, green, blue)));
    let unit = SliderRange::new(0.0, 1.0, 1000.0, 3);
    let (lowest, highest) = (simple_paint::MARGIN, simple_paint::INTERIOR_MAXIMUM);
    let degrees = SliderRange::new(0.0, 360.0, 10.0, 1).between(0.0, 359.9).suffix("°");
    type Set = fn(&mut Parameters, f64);
    let rows: [[(&str, &str, f64, SliderRange, Set); 2]; 4] = [
        [
            ("Red", "Red", parameters.base_color_srgb[0], unit.between(lowest, highest), |p, v| p.base_color_srgb[0] = v),
            ("Brightness", "Brightness", parameters.brightness, unit.between(lowest, highest), |p, v| p.brightness = v),
        ],
        [
            ("Green", "Green", parameters.base_color_srgb[1], unit.between(lowest, highest), |p, v| p.base_color_srgb[1] = v),
            ("Shift", "Shift", parameters.shift, unit.between(0.0, highest), |p, v| p.shift = v),
        ],
        [
            ("Blue", "Blue", parameters.base_color_srgb[2], unit.between(lowest, highest), |p, v| p.base_color_srgb[2] = v),
            ("Rotation", "Rotation", parameters.rotation_degrees, degrees, |p, v| p.rotation_degrees = v),
        ],
        [
            ("DarkPoint", "Dark point", parameters.dark_point, unit.between(0.0, highest), |p, v| p.dark_point = v),
            ("LightPoint", "Light point", parameters.light_point, unit.between(lowest, 1.0), |p, v| p.light_point = v),
        ],
    ];
    for sliders in rows {
        let columns = form.columns::<2>(slider_height());
        for ((part, label, value, range, set), column) in sliders.into_iter().zip(columns) {
            let to = to_paint.clone();
            let widget = slider(label, value, range, move |value| {
                let mut edited = parameters;
                set(&mut edited, value);
                to(edited)
            });
            tree.add(&format!("{name}/{part}"), column, widget);
        }
    }
}
