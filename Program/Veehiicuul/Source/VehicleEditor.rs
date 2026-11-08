// The vehicle editor, and its footprint creator: picks a vehicle's model,
// its acceleration, and its paint over the model's, and draws its footprint,
// the rectangle that touches the track's edges, over its model seen from
// above. A side of the footprint follows a drag from it in the view, and the
// footprint can fit the model's own rectangle. The paint has a card of its
// own, down the right side, and the vehicle is between the two.

use json::Value;
use simple_paint::Parameters;
use user_interface::controls::LIST_ITEM_HEIGHT;
use user_interface::tokens::CONTROL_HEIGHT;
use user_interface::{Button, Canvas, Card, ComboBox, Form, Icon, InfoBar, ListView, Point, Rect, Severity, Text, TextBox, Tree, Ui};

use crate::collision::Footprint;
use crate::document::Document;
use crate::editing::{self, GroundInput, HEADED_HEIGHT, Projection, SliderRange, Viewport, header, slider_height, window_size};
use crate::models::{Models, set_slot};
use crate::program::Message;

// The vehicle faces up the screen, which is this way on the ground.
pub const SHOWN_HEADING: f64 = -135.0;
// How near the pointer a side counts as under it, in pixels.
const REACH: f32 = 10.0;
pub const SIDES: [&str; 4] = ["Left", "Right", "Front", "Back"];
// The most rows the list shows without scrolling.
const LIST_ROWS: usize = 6;

// What the cards' controls do.
#[derive(Clone, Debug)]
pub enum VehicleMsg {
    Select(usize),
    Add,
    Delete,
    // What the name's field holds, which renames the vehicle while the name
    // is free.
    Name(String),
    Model(usize),
    // One of the acceleration's rates, as `RATES` has them.
    Rate(usize, f64),
    // How far the footprint goes to one side, as SIDES has them.
    Side(usize, f64),
    Fit,
    Slot(usize),
    Paint(Parameters),
    ResetPaint,
}

// The acceleration's rates, as Rate's index has them.
pub const RATES: [&str; 4] = ["Forward", "Reverse", "Left", "Right"];

pub struct VehicleEditor {
    pub viewport: Viewport,
    pub slot: usize,
    // The side of the footprint being dragged, as an index of SIDES.
    drag: Option<usize>,
    // The side under the pointer, for the handles the overlay draws.
    under: Option<usize>,
    name_text: String,
    pub message: String,
}

impl Default for VehicleEditor {
    fn default() -> Self {
        VehicleEditor {
            viewport: Viewport::new([0.0, 0.0], 12.0),
            slot: 0,
            drag: None,
            under: None,
            name_text: String::new(),
            message: String::new(),
        }
    }
}

// How far the footprint goes to each side, as SIDES has them.
fn sides(footprint: &Footprint) -> [f64; 4] {
    [footprint.max[0], -footprint.min[0], footprint.max[1], -footprint.min[1]]
}

// Each side's corners, as SIDES has them, from the footprint's corners.
fn side_corners(corners: [[f64; 2]; 4]) -> [[[f64; 2]; 2]; 4] {
    [[corners[1], corners[2]], [corners[3], corners[0]], [corners[2], corners[3]], [corners[0], corners[1]]]
}

impl VehicleEditor {
    // Keeps the name's field on the vehicle's name while it isn't being
    // edited, so that editing starts from it.
    pub fn sync_name(&mut self, document: &Document, vehicle: usize) {
        self.name_text = document.vehicles.list[vehicle].name.clone();
    }

    // What the pointer in the view does: a drag moves a side of the
    // footprint.
    pub fn view_input(&mut self, input: GroundInput, document: &mut Document, vehicle: usize, size: [f32; 2]) {
        let eye = self.viewport.eye;
        let heading = SHOWN_HEADING.to_radians();
        let current = &mut document.vehicles.list[vehicle];
        let mut values = sides(&current.footprint);
        let corners = current.footprint.corners([0.0, 0.0], heading);
        self.under = input.cursor.filter(|_| !input.panning).and_then(|cursor| {
            (0..4).find(|&side| {
                let [a, b] = side_corners(corners)[side].map(|corner| eye.ground_to_screen(corner, size));
                distance_to_segment(cursor, a, b) <= REACH
            })
        });
        if input.pressed && !input.panning {
            self.drag = self.under;
        }
        // The drag follows the pointer to where it's let go of.
        if let (Some(side), Some(ground)) = (self.drag, input.ground) {
            let (sin, cos) = heading.sin_cos();
            let left = ground[0] * cos - ground[1] * sin;
            let ahead = ground[0] * sin + ground[1] * cos;
            values[side] = [left, -left, ahead, -ahead][side].clamp(0.05, 10.0);
            current.footprint = Footprint::from_sides(values[0], values[1], values[2], values[3]);
        }
        if !input.held {
            self.drag = None;
        }
    }

    // A control on the cards.
    pub fn apply(&mut self, message: VehicleMsg, document: &mut Document, models: &Models, vehicle: &mut usize) {
        match message {
            VehicleMsg::Select(index) => *vehicle = index.min(document.vehicles.list.len() - 1),
            VehicleMsg::Add => {
                let mut copy = document.vehicles.list[*vehicle].clone();
                copy.name = document.vehicles.new_name();
                document.vehicles.list.push(copy);
                *vehicle = document.vehicles.list.len() - 1;
                self.message.clear();
            }
            VehicleMsg::Delete => {
                let name = document.vehicles.list[*vehicle].name.clone();
                let parked = document.scenes.iter().find(|scene| scene.parked.iter().any(|parked| parked.vehicle == name));
                self.message = if document.vehicles.list.len() == 1 {
                    "The game needs a vehicle, so the last can't go.".to_string()
                } else if let Some(scene) = parked {
                    format!("{name} is parked in {}, so it can't go.", scene.name)
                } else {
                    document.vehicles.list.remove(*vehicle);
                    *vehicle = (*vehicle).min(document.vehicles.list.len() - 1);
                    String::new()
                };
            }
            VehicleMsg::Name(text) => {
                self.name_text = text;
                // A rename also changes the name where it's parked.
                let name = self.name_text.trim().to_string();
                let old = document.vehicles.list[*vehicle].name.clone();
                if !name.is_empty() && name != old && document.vehicles.find(&name).is_none() {
                    for scene in &mut document.scenes {
                        for parked in &mut scene.parked {
                            if parked.vehicle == old {
                                parked.vehicle = name.clone();
                            }
                        }
                    }
                    document.vehicles.list[*vehicle].name = name;
                }
            }
            VehicleMsg::Model(model) => {
                let names = models.names();
                document.vehicles.list[*vehicle].model = names[model.min(names.len() - 1)].to_string();
            }
            VehicleMsg::Rate(index, value) => {
                let acceleration = &mut document.vehicles.list[*vehicle].acceleration;
                match index {
                    0 => acceleration.forward = value,
                    1 => acceleration.reverse = value,
                    2 => acceleration.left = value,
                    _ => acceleration.right = value,
                }
            }
            VehicleMsg::Side(side, value) => {
                let current = &mut document.vehicles.list[*vehicle];
                let mut values = sides(&current.footprint);
                values[side] = value;
                current.footprint = Footprint::from_sides(values[0], values[1], values[2], values[3]);
            }
            VehicleMsg::Fit => {
                let current = &mut document.vehicles.list[*vehicle];
                let model = models.find(&current.model).unwrap_or(0);
                current.footprint = models.list[model].footprint;
            }
            VehicleMsg::Slot(slot) => self.slot = slot,
            VehicleMsg::Paint(parameters) => {
                let current = &mut document.vehicles.list[*vehicle];
                let model = models.find(&current.model).unwrap_or(0);
                let slots = &models.list[model].slots;
                let slot = slots[self.slot.min(slots.len() - 1)].clone();
                set_slot(&mut current.paint, slots, &slot, parameters);
            }
            VehicleMsg::ResetPaint => {
                let current = &mut document.vehicles.list[*vehicle];
                let model = models.find(&current.model).unwrap_or(0);
                let slots = &models.list[model].slots;
                let slot = slots[self.slot.min(slots.len() - 1)].clone();
                current.paint.retain(|(painted, _)| *painted != slot);
            }
        }
    }

    // The overlay over the view, then the cards, over both.
    pub fn view(&self, ui: &mut Ui, tree: &mut Tree<Message>, page: Rect, view: Rect, document: &Document, models: &Models, vehicle: usize) {
        // The model's rectangle, thin, and the footprint, with a handle on
        // each side.
        let heading = SHOWN_HEADING.to_radians();
        let current = &document.vehicles.list[vehicle];
        let model_corners = models
            .find(&current.model)
            .map(|model| models.list[model].footprint.corners([0.0, 0.0], heading));
        let corners = current.footprint.corners([0.0, 0.0], heading);
        let (drag, under) = (self.drag, self.under);
        let projection = Projection::new(self.viewport.eye, window_size(ui));
        tree.draw(
            view,
            Canvas::new(move |painter, rect| {
                let theme = painter.theme().clone();
                painter.push_clip(rect);
                if let Some(model_corners) = model_corners {
                    projection.outline(painter, &model_corners, 1.0, theme.text_secondary);
                }
                projection.outline(painter, &corners, 2.0, theme.accent);
                for (side, [a, b]) in side_corners(corners).iter().enumerate() {
                    let middle = projection.point([(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]);
                    let active = drag == Some(side) || (drag.is_none() && under == Some(side));
                    painter.circle(middle, if active { 7.0 } else { 5.0 }, theme.accent);
                    painter.circle(middle, if active { 3.0 } else { 2.0 }, theme.acrylic);
                }
                painter.pop_clip();
            }),
        );
        self.card(ui, tree, page, document, models, vehicle);
        self.paint_card(ui, tree, page, document, models, vehicle);
    }

    fn card(&self, ui: &mut Ui, tree: &mut Tree<Message>, page: Rect, document: &Document, models: &Models, vehicle: usize) {
        let card = Card::new("Vehicles").subtitle("How each looks and drives").floating();
        tree.sized_card(ui, None, editing::left_card(page), card, |ui, card, area| {
            let mut form = Form::new(area.content);
            let names: Vec<String> = document.vehicles.list.iter().map(|vehicle| vehicle.name.clone()).collect();
            let shown = names.len().clamp(1, LIST_ROWS) as f32;
            let labels = names.clone();
            let list = ListView::new(names.len(), LIST_ITEM_HEIGHT)
                .selection(Some(vehicle))
                .on_select(|index| Message::Vehicle(VehicleMsg::Select(index)))
                .labels(move |row| labels[row].clone());
            let list_rect = form.take(shown * LIST_ITEM_HEIGHT);
            let rows = list.rows(ui, "Vehicles/List", list_rect);
            card.container(Some("Vehicles/List"), list_rect, list, |items| {
                for row in rows.indices() {
                    items.draw(rows.content(row), Text::label(&names[row]));
                }
            });
            let [add, delete] = form.columns::<2>(CONTROL_HEIGHT);
            card.add("Vehicles/Add", add, Button::new("Add a vehicle").icon(Icon::Add).on_press(Message::Vehicle(VehicleMsg::Add)))
                .set_tooltip("Adds a copy of this vehicle, with a name of its own");
            card.add("Vehicles/Delete", delete, Button::new("Delete it").icon(Icon::Delete).on_press(Message::Vehicle(VehicleMsg::Delete)))
                .set_tooltip("Deletes this vehicle, unless it's parked in a scene");
            if !self.message.is_empty() {
                let bar = InfoBar::new(Severity::Warning, "", &self.message);
                let height = bar.height(ui, form.rest().width);
                card.add("Vehicles/Message", form.take(height), bar);
            }

            form.heading(card, "Vehicle");
            // The name, which a rename also changes where it's parked, and
            // the model. The field holds what's typed while it has focus,
            // and the vehicle's name otherwise, which Prepare keeps it on.
            let current = &document.vehicles.list[vehicle];
            let shown_name = if ui.is_focused("Vehicles/Name") { self.name_text.clone() } else { current.name.clone() };
            let [name_column, model_column] = form.columns::<2>(HEADED_HEIGHT);
            let field = header(card, name_column, "Name");
            card.add("Vehicles/Name", field, TextBox::new(shown_name).placeholder("Name").max_length(40).on_change(|name| {
                Message::Vehicle(VehicleMsg::Name(name))
            }))
            .set_focusable(true);
            let model_names = models.names();
            let model = models.find(&current.model).unwrap_or(0);
            let model_area = header(card, model_column, "Model");
            card.add("Vehicles/Model", model_area, ComboBox::new(model, &model_names).on_select(|model| {
                Message::Vehicle(VehicleMsg::Model(model))
            }));
            let rates = SliderRange::new(0.0, 2000.0, 1.0, 0);
            let acceleration = &current.acceleration;
            let values = [acceleration.forward, acceleration.reverse, acceleration.left, acceleration.right];
            for row in [[0, 1], [2, 3]] {
                let columns = form.columns::<2>(slider_height());
                for (index, column) in row.into_iter().zip(columns) {
                    card.add(&format!("Vehicles/{}", RATES[index]), column, editing::slider(RATES[index], values[index], rates, move |value| {
                        Message::Vehicle(VehicleMsg::Rate(index, value))
                    }));
                }
            }
            form.caption(ui, card, "Acceleration each way, in units a second, a second.");

            form.heading(card, "Footprint");
            let values = sides(&current.footprint);
            let range = SliderRange::new(0.05, 10.0, 200.0, 3);
            for row in [[0, 1], [2, 3]] {
                let columns = form.columns::<2>(slider_height());
                for (side, column) in row.into_iter().zip(columns) {
                    card.add(&format!("Footprint/{}", SIDES[side]), column, editing::slider(SIDES[side], values[side], range, move |value| {
                        Message::Vehicle(VehicleMsg::Side(side, value))
                    }));
                }
            }
            card.add("Footprint/Fit", form.take(CONTROL_HEIGHT), Button::new("Fit it to the model").on_press(Message::Vehicle(VehicleMsg::Fit)))
                .set_tooltip("Makes the footprint the rectangle around the model, seen from above");
            form.caption(ui, card, "Drag a side in the view to move it.");
            form.bottom()
        });
    }

    // The card down the right side, with the vehicle's paint over its
    // model's.
    fn paint_card(&self, ui: &mut Ui, tree: &mut Tree<Message>, page: Rect, document: &Document, models: &Models, vehicle: usize) {
        let card = Card::new("Paint").subtitle("This vehicle's, over its model's").floating();
        tree.sized_card(ui, None, editing::right_card(page), card, |ui, card, area| {
            let mut form = Form::new(area.content);
            let current = &document.vehicles.list[vehicle];
            let model = models.find(&current.model).unwrap_or(0);
            let slots = &models.list[model].slots;
            let slot = self.slot.min(slots.len() - 1);
            let none = Vec::new();
            let model_name = &models.list[model].name;
            let model_paint = document.paints.iter().find(|(name, _)| name == model_name).map_or(&none, |(_, paint)| paint);
            let materials = models.list[model].materials(&[model_paint, &current.paint]);
            editing::paint(
                &mut form,
                card,
                "VehiclePaint",
                slots,
                slot,
                materials[slot],
                |slot| Message::Vehicle(VehicleMsg::Slot(slot)),
                |parameters| Message::Vehicle(VehicleMsg::Paint(parameters)),
            );
            let painted = current.paint.iter().any(|(painted, _)| *painted == slots[slot]);
            let reset = Button::new("Use the model's paint").icon(Icon::Refresh).on_press(Message::Vehicle(VehicleMsg::ResetPaint));
            card.add("VehiclePaint/Reset", form.take(CONTROL_HEIGHT), reset)
                .set_enabled(painted)
                .set_tooltip("Paints this slot as the model is painted everywhere");
            form.caption(ui, card, if painted { "This vehicle paints this slot its own way." } else { "This slot has the model's paint." });
            form.bottom()
        });
    }

    pub fn state(&self, document: &Document, vehicle: usize) -> Value {
        let current = &document.vehicles.list[vehicle];
        let sides = sides(&current.footprint);
        json::object([
            ("Selected", current.name.clone().into()),
            ("Model", current.model.clone().into()),
            ("Names", Value::Array(document.vehicles.list.iter().map(|vehicle| vehicle.name.clone().into()).collect())),
            (
                "Footprint",
                json::object([
                    ("Left", sides[0].into()),
                    ("Right", sides[1].into()),
                    ("Front", sides[2].into()),
                    ("Back", sides[3].into()),
                ]),
            ),
            ("PaintedSlots", Value::Array(current.paint.iter().map(|(slot, _)| slot.clone().into()).collect())),
            ("Message", self.message.clone().into()),
        ])
    }
}

fn distance_to_segment(point: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (point, a, b) = (Point::new(point[0], point[1]), Point::new(a[0], a[1]), Point::new(b[0], b[1]));
    let along = b - a;
    let length = along.x * along.x + along.y * along.y;
    let t = if length > 0.0 {
        (((point.x - a.x) * along.x + (point.y - a.y) * along.y) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    point.distance(a.lerp(b, t))
}
