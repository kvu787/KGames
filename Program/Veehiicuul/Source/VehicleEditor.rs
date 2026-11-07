// The vehicle editor, and its footprint creator: picks a vehicle's model,
// its acceleration, and its paint over the model's, and draws its footprint,
// the rectangle that touches the track's edges, over its model seen from
// above. A side of the footprint follows a drag from it in the view, and the
// footprint can fit the model's own rectangle. The paint has a card of its
// own, down the right side, and the vehicle is between the two.

use json::Value;
use user_interface::controls::LIST_ITEM_HEIGHT;
use user_interface::tokens::CONTROL_HEIGHT;
use user_interface::{Button, Card, ComboBox, Form, Icon, InfoBar, ListRow, ListView, Point, Rect, Text, TextBox, Ui, UiEventKind};

use crate::collision::Footprint;
use crate::document::Document;
use crate::editing::{self, HEADED_HEIGHT, PaintControls, Projection, Role, Shape, SliderRange, Viewport, header, slider_height, window_size};
use crate::models::{Models, set_slot};

// The vehicle faces up the screen, which is this way on the ground.
pub const SHOWN_HEADING: f64 = -135.0;
// How near the pointer a side counts as under it, in pixels.
const REACH: f32 = 10.0;
pub const SIDES: [&str; 4] = ["Left", "Right", "Front", "Back"];
// The most rows the list shows without scrolling.
const LIST_ROWS: usize = 6;

const RATES: SliderRange = SliderRange::new(0.0, 2000.0, 1.0, 0);
const SIDE_RANGE: SliderRange = SliderRange::new(0.05, 10.0, 200.0, 3);
const ACCELERATION_PARTS: [&str; 4] = ["Forward", "Reverse", "Left", "Right"];

pub struct VehicleEditor {
    pub viewport: Viewport,
    pub slot: usize,
    // The side of the footprint being dragged, as an index of SIDES.
    drag: Option<usize>,
    pub message: String,
}

impl Default for VehicleEditor {
    fn default() -> Self {
        VehicleEditor {
            viewport: Viewport::new([0.0, 0.0], 12.0),
            slot: 0,
            drag: None,
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
    // The vehicles, the message, and the slots shape the cards.
    pub fn fingerprint(&self, document: &Document, models: &Models, vehicle: usize) -> String {
        let current = &document.vehicles.list[vehicle];
        let model = models.find(&current.model).unwrap_or(0);
        format!(
            "vehicles {} message {:?} slots {} painted {}",
            document.vehicles.list.len(),
            self.message,
            models.list[model].slots.len(),
            current.paint.len(),
        )
    }

    pub fn build(&self, ui: &mut Ui, view: Rect, document: &Document, models: &Models, vehicle: usize) {
        ui.node("Vehicles/View", view, user_interface::View::new());
        editing::overlay_node(ui, "Vehicles/Overlay", view);
        let page = ui.page();
        self.card(ui, page, document, models, vehicle);
        self.paint_card(ui, page, document, models, vehicle);
    }

    fn card(&self, ui: &mut Ui, page: Rect, document: &Document, models: &Models, vehicle: usize) {
        let card = Card::new("Vehicles").subtitle("How each looks and drives").floating();
        let area = card.area(ui, editing::left_card(page));
        let mut bottom = area.content.y;
        ui.group("VehiclesCard", editing::left_card(page), Card::new("Vehicles").subtitle("How each looks and drives").floating(), |ui| {
            let mut form = Form::new("VehiclesForm", area.content);
            let names: Vec<String> = document.vehicles.list.iter().map(|vehicle| vehicle.name.clone()).collect();
            let shown = names.len().clamp(1, LIST_ROWS) as f32;
            let mut list = ListView::new(names.iter().map(|name| ListRow::new(name)).collect(), LIST_ITEM_HEIGHT).selectable();
            list.set_selected(Some(vehicle));
            ui.node("Vehicles/List", form.take(shown * LIST_ITEM_HEIGHT), list);
            let [add, delete] = form.columns::<2>(CONTROL_HEIGHT);
            ui.node("Vehicles/Add", add, Button::new("Add a vehicle").icon(Icon::Add));
            ui.set_tooltip("Vehicles/Add", "Adds a copy of this vehicle, with a name of its own");
            ui.node("Vehicles/Delete", delete, Button::new("Delete it").icon(Icon::Delete));
            ui.set_tooltip("Vehicles/Delete", "Deletes this vehicle, unless it's parked in a scene");
            if !self.message.is_empty() {
                let bar = InfoBar::new(user_interface::Severity::Warning, "", &self.message);
                let height = bar.height(ui, form.width());
                ui.node("Vehicles/Message", form.take(height), bar);
            }

            form.heading(ui, "Vehicle");
            // The name, which a rename also changes where it's parked, and
            // the model.
            let current = &document.vehicles.list[vehicle];
            let [name_column, model_column] = form.columns::<2>(HEADED_HEIGHT);
            let field = header(ui, "Vehicles/NameLabel", name_column, "Name");
            ui.node("Vehicles/Name", field, TextBox::new(&current.name).placeholder("Name").max_length(40));
            ui.set_focusable("Vehicles/Name", true);
            let model_names = models.names();
            let model = models.find(&current.model).unwrap_or(0);
            let model_area = header(ui, "Vehicles/ModelLabel", model_column, "Model");
            ui.node("Vehicles/Model", model_area, ComboBox::new(&model_names, model));
            let acceleration = &current.acceleration;
            let values = [acceleration.forward, acceleration.reverse, acceleration.left, acceleration.right];
            for pair in ACCELERATION_PARTS.chunks(2).zip([[values[0], values[1]], [values[2], values[3]]]) {
                let (parts, row) = pair;
                let columns = form.columns::<2>(slider_height());
                for ((part, value), column) in parts.iter().zip(row).zip(columns) {
                    editing::slider_node(ui, &format!("Vehicles/{part}"), column, part, value, RATES);
                }
            }
            form.caption(ui, "Acceleration each way, in units a second, a second.");

            form.heading(ui, "Footprint");
            let values = sides(&current.footprint);
            for (pair, row) in SIDES.chunks(2).zip([[values[0], values[1]], [values[2], values[3]]]) {
                let columns = form.columns::<2>(slider_height());
                for ((side, value), column) in pair.iter().zip(row).zip(columns) {
                    editing::slider_node(ui, &format!("Footprint/{side}"), column, side, value, SIDE_RANGE);
                }
            }
            ui.node("Footprint/Fit", form.take(CONTROL_HEIGHT), Button::new("Fit it to the model"));
            ui.set_tooltip("Footprint/Fit", "Makes the footprint the rectangle around the model, seen from above");
            form.caption(ui, "Drag a side in the view to move it.");
            bottom = form.bottom();
        });
        let rect = Rect::new(page.x, page.y, editing::CARD_WIDTH, card.height_around(bottom - area.content.y));
        ui.set_rect("VehiclesCard", rect);
    }

    // The card down the right side, with the vehicle's paint over its
    // model's.
    fn paint_card(&self, ui: &mut Ui, page: Rect, document: &Document, models: &Models, vehicle: usize) {
        let card = Card::new("Paint").subtitle("This vehicle's, over its model's").floating();
        let area = card.area(ui, editing::right_card(page));
        let mut bottom = area.content.y;
        ui.group("PaintCard", editing::right_card(page), Card::new("Paint").subtitle("This vehicle's, over its model's").floating(), |ui| {
            let mut form = Form::new("PaintForm", area.content);
            let current = &document.vehicles.list[vehicle];
            let model = models.find(&current.model).unwrap_or(0);
            let slots = &models.list[model].slots;
            let slot = self.slot.min(slots.len() - 1);
            let materials = self.materials(document, models, vehicle);
            PaintControls::build(&mut form, ui, "VehiclePaint", slots, slot, &materials[slot]);
            ui.node("VehiclePaint/Reset", form.take(CONTROL_HEIGHT), Button::new("Use the model's paint").icon(Icon::Refresh));
            ui.set_tooltip("VehiclePaint/Reset", "Paints this slot as the model is painted everywhere");
            let painted = current.paint.iter().any(|(painted, _)| *painted == slots[slot]);
            ui.set_enabled("VehiclePaint/Reset", painted);
            let caption = if painted { "This vehicle paints this slot its own way." } else { "This slot has the model's paint." };
            let line = form.take(user_interface::tokens::CAPTION.line_height);
            ui.node("VehiclePaint/Caption", line, Text::caption(caption));
            bottom = form.bottom();
        });
        let right = editing::right_card(page);
        let rect = Rect::new(right.x, right.y, editing::CARD_WIDTH, card.height_around(bottom - area.content.y));
        ui.set_rect("PaintCard", rect);
    }

    // The materials of the vehicle's model's slots, with the vehicle's own
    // paint over the model's.
    fn materials(&self, document: &Document, models: &Models, vehicle: usize) -> Vec<simple_paint::Parameters> {
        let current = &document.vehicles.list[vehicle];
        let model = models.find(&current.model).unwrap_or(0);
        let none = Vec::new();
        let model_name = &models.list[model].name;
        let model_paint = document.paints.iter().find(|(name, _)| name == model_name).map_or(&none, |(_, paint)| paint);
        models.list[model].materials(&[model_paint, &current.paint])
    }

    // The view, where a drag moves a side of the footprint, the frame's
    // events, and what the cards show.
    pub fn update(&mut self, ui: &mut Ui, view: Rect, document: &mut Document, models: &Models, vehicle: &mut usize) {
        let _ = view;
        let input = self.viewport.apply(ui, "Vehicles/View");
        let size = window_size(ui);
        let eye = self.viewport.eye;
        let heading = SHOWN_HEADING.to_radians();
        let under;
        {
            let current = &mut document.vehicles.list[*vehicle];
            let mut values = sides(&current.footprint);
            let corners = current.footprint.corners([0.0, 0.0], heading);
            under = input.cursor.filter(|_| !input.panning).and_then(|cursor| {
                (0..4).find(|&side| {
                    let [a, b] = side_corners(corners)[side].map(|corner| eye.ground_to_screen(corner, size));
                    distance_to_segment(cursor, a, b) <= REACH
                })
            });
            if input.response.pressed && !input.panning {
                self.drag = under;
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
        self.events(ui, document, models, vehicle);

        // The model's rectangle, thin, and the footprint, with a handle on
        // each side.
        let current = &document.vehicles.list[*vehicle];
        let model_footprint = models.find(&current.model).map(|model| models.list[model].footprint);
        let corners = current.footprint.corners([0.0, 0.0], heading);
        let projection = Projection::new(ui, eye);
        let mut shapes = Vec::new();
        if let Some(footprint) = model_footprint {
            shapes.push(projection.outline(&footprint.corners([0.0, 0.0], heading), 1.0, Role::Secondary));
        }
        shapes.push(projection.outline(&corners, 2.0, Role::Accent));
        for (side, [a, b]) in side_corners(corners).iter().enumerate() {
            let middle = projection.point([(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]);
            let active = self.drag == Some(side) || (self.drag.is_none() && under == Some(side));
            shapes.push(Shape::Circle(middle, if active { 7.0 } else { 5.0 }, Role::Accent));
            shapes.push(Shape::Circle(middle, if active { 3.0 } else { 2.0 }, Role::Acrylic));
        }
        editing::set_overlay(ui, "Vehicles/Overlay", shapes);
        self.sync(ui, document, models, *vehicle);
    }

    fn events(&mut self, ui: &mut Ui, document: &mut Document, models: &Models, vehicle: &mut usize) {
        let model_names = models.names();
        let events: Vec<user_interface::UiEvent> = ui.events().to_vec();
        for event in events {
            match (event.source.as_str(), &event.kind) {
                ("Vehicles/List", UiEventKind::Selected(index)) => {
                    *vehicle = (*index).min(document.vehicles.list.len() - 1);
                }
                ("Vehicles/Add", UiEventKind::Clicked) => {
                    let mut copy = document.vehicles.list[*vehicle].clone();
                    copy.name = document.vehicles.new_name();
                    document.vehicles.list.push(copy);
                    *vehicle = document.vehicles.list.len() - 1;
                    self.message.clear();
                }
                ("Vehicles/Delete", UiEventKind::Clicked) => {
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
                ("Vehicles/Name", UiEventKind::TextChanged(text)) => {
                    // A rename also changes where it's parked.
                    let name = text.trim().to_string();
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
                ("Vehicles/Model", UiEventKind::Selected(index)) => {
                    document.vehicles.list[*vehicle].model = model_names[(*index).min(model_names.len() - 1)].to_string();
                }
                (source, UiEventKind::Changed(value)) if source.starts_with("Vehicles/") => {
                    let acceleration = &mut document.vehicles.list[*vehicle].acceleration;
                    let value = RATES.nearest(*value);
                    match source {
                        "Vehicles/Forward" => acceleration.forward = value,
                        "Vehicles/Reverse" => acceleration.reverse = value,
                        "Vehicles/Left" => acceleration.left = value,
                        "Vehicles/Right" => acceleration.right = value,
                        _ => {}
                    }
                }
                (source, UiEventKind::Changed(value)) if source.starts_with("Footprint/") => {
                    let current = &mut document.vehicles.list[*vehicle];
                    let mut values = sides(&current.footprint);
                    let value = SIDE_RANGE.nearest(*value);
                    match source {
                        "Footprint/Left" => values[0] = value,
                        "Footprint/Right" => values[1] = value,
                        "Footprint/Front" => values[2] = value,
                        "Footprint/Back" => values[3] = value,
                        _ => {}
                    }
                    current.footprint = Footprint::from_sides(values[0], values[1], values[2], values[3]);
                }
                ("Footprint/Fit", UiEventKind::Clicked) => {
                    let current = &mut document.vehicles.list[*vehicle];
                    let model = models.find(&current.model).unwrap_or(0);
                    current.footprint = models.list[model].footprint;
                }
                ("VehiclePaint/Reset", UiEventKind::Clicked) => {
                    let current = &mut document.vehicles.list[*vehicle];
                    let model = models.find(&current.model).unwrap_or(0);
                    let slots = &models.list[model].slots;
                    let slot = self.slot.min(slots.len() - 1);
                    let name = slots[slot].clone();
                    current.paint.retain(|(painted, _)| *painted != name);
                }
                _ => {}
            }
        }
        // The vehicle's paint over the model's.
        {
            let current = &document.vehicles.list[*vehicle];
            let model = models.find(&current.model).unwrap_or(0);
            let slots = models.list[model].slots.clone();
            self.slot = self.slot.min(slots.len() - 1);
            let mut parameters = self.materials(document, models, *vehicle)[self.slot];
            if PaintControls::changed(ui, "VehiclePaint", &mut self.slot, &mut parameters) {
                let slot = slots[self.slot.min(slots.len() - 1)].clone();
                let current = &mut document.vehicles.list[*vehicle];
                set_slot(&mut current.paint, &slots, &slot, parameters);
            }
        }
    }

    fn sync(&mut self, ui: &mut Ui, document: &Document, models: &Models, vehicle: usize) {
        let names: Vec<String> = document.vehicles.list.iter().map(|vehicle| vehicle.name.clone()).collect();
        let list = ui.get_mut::<ListView>("Vehicles/List");
        let changed = list.rows().len() != names.len() || list.rows().iter().zip(&names).any(|(a, b)| a.text != *b);
        if changed {
            list.set_rows(names.iter().map(|name| ListRow::new(name)).collect());
        }
        list.set_selected(Some(vehicle));
        let current = &document.vehicles.list[vehicle];
        if !ui.is_focused("Vehicles/Name") && ui.get::<TextBox>("Vehicles/Name").text() != current.name {
            let name = current.name.clone();
            ui.get_mut::<TextBox>("Vehicles/Name").set_text(&name);
        }
        let model_names = models.names();
        let model = models.find(&current.model).unwrap_or(0);
        let combo = ui.get_mut::<ComboBox>("Vehicles/Model");
        combo.set_items(&model_names);
        combo.set_selected(model);
        let acceleration = &current.acceleration;
        for (part, value) in ACCELERATION_PARTS
            .iter()
            .zip([acceleration.forward, acceleration.reverse, acceleration.left, acceleration.right])
        {
            editing::sync_slider(ui, &format!("Vehicles/{part}"), value);
        }
        let values = sides(&current.footprint);
        for (side, value) in SIDES.iter().zip(values) {
            editing::sync_slider(ui, &format!("Footprint/{side}"), value);
        }
        let slots = &models.list[model].slots;
        let slot = self.slot.min(slots.len() - 1);
        let materials = self.materials(document, models, vehicle);
        PaintControls::sync(ui, "VehiclePaint", slots, slot, &materials[slot]);
        let painted = current.paint.iter().any(|(painted, _)| *painted == slots[slot]);
        ui.set_enabled("VehiclePaint/Reset", painted);
        let caption = if painted { "This vehicle paints this slot its own way." } else { "This slot has the model's paint." };
        ui.get_mut::<Text>("VehiclePaint/Caption").set_text(caption);
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
