// The vehicle editor, and its footprint creator: picks a vehicle's model,
// its acceleration, and its paint over the model's, and draws its footprint,
// the rectangle that touches the track's edges, over its model seen from
// above. A side of the footprint follows a drag from it in the view, and the
// footprint can fit the model's own rectangle. The paint has a card of its
// own, down the right side, and the vehicle is between the two.

use json::Value;
use user_interface::tokens::{
    BODY, BODY_STRONG, CAPTION, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, CONTROL_HEIGHT, LIST_ROW_HEIGHT, SPACE_M,
    SPACE_S,
};
use user_interface::{Button, Icon, Point, Rect, Ui};

use crate::collision::Footprint;
use crate::document::Document;
use crate::editing::{self, Form, Overlay, SliderRange, Viewport, subtitle, window_size};
use crate::models::{Models, set_slot};

// The vehicle faces up the screen, which is this way on the ground.
pub const SHOWN_HEADING: f64 = -135.0;
// How near the pointer a side counts as under it, in pixels.
const REACH: f32 = 10.0;
pub const SIDES: [&str; 4] = ["Left", "Right", "Front", "Back"];
// The most rows the list shows without scrolling.
const LIST_ROWS: usize = 6;
// How tall the paint's card is, for what's in it: the slot, eight sliders,
// a button, and a caption.
const PAINT_CARD_HEIGHT: f32 =
    CARD_PADDING * 2.0 + CARD_HEADER_HEIGHT + CARD_HEADER_GAP + (CONTROL_HEIGHT + SPACE_S) * 10.0 + CAPTION.line_height;

pub struct VehicleEditor {
    pub viewport: Viewport,
    pub slot: usize,
    // The side of the footprint being dragged, as an index of SIDES.
    drag: Option<usize>,
    name_text: String,
    pub message: String,
}

impl Default for VehicleEditor {
    fn default() -> Self {
        VehicleEditor {
            viewport: Viewport::new([0.0, 0.0], 12.0),
            slot: 0,
            drag: None,
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
    // The view, where a drag moves a side of the footprint, then the cards,
    // over it.
    pub fn update(&mut self, ui: &mut Ui, view: Rect, document: &mut Document, models: &Models, vehicle: &mut usize) {
        let input = self.viewport.interact(ui, "Vehicles/View", view);
        let size = window_size(ui);
        let eye = self.viewport.eye;
        let heading = SHOWN_HEADING.to_radians();
        let current = &mut document.vehicles.list[*vehicle];
        let mut values = sides(&current.footprint);
        let corners = current.footprint.corners([0.0, 0.0], heading);
        let under = input.cursor.filter(|_| !input.panning).and_then(|cursor| {
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
        if !input.response.held {
            self.drag = None;
        }

        // The model's rectangle, thin, and the footprint, with a handle on
        // each side.
        let theme = ui.theme().clone();
        let model_footprint = models.find(&current.model).map(|model| models.list[model].footprint);
        let corners = current.footprint.corners([0.0, 0.0], heading);
        let mut overlay = Overlay::new(ui, eye, view);
        if let Some(footprint) = model_footprint {
            overlay.outline(&footprint.corners([0.0, 0.0], heading), 1.0, theme.text_secondary);
        }
        overlay.outline(&corners, 2.0, theme.accent);
        for (side, [a, b]) in side_corners(corners).iter().enumerate() {
            let middle = overlay.screen([(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]);
            let active = self.drag == Some(side) || (self.drag.is_none() && under == Some(side));
            overlay.dot(middle, if active { 7.0 } else { 5.0 }, theme.accent);
            overlay.dot(middle, if active { 3.0 } else { 2.0 }, theme.flyout);
        }
        drop(overlay);
        self.card(ui, document, models, vehicle);
        self.paint_card(ui, document, models, *vehicle);
    }

    fn card(&mut self, ui: &mut Ui, document: &mut Document, models: &Models, vehicle: &mut usize) {
        let page = ui.page();
        let card = ui.floating_card(editing::left_card(page), "Vehicles");
        subtitle(ui, card.header, "How each looks and drives");
        let mut form = Form::new(card.content);
        let names: Vec<String> = document.vehicles.list.iter().map(|vehicle| vehicle.name.clone()).collect();
        let shown = names.len().clamp(1, LIST_ROWS) as f32;
        let mut selected = Some(*vehicle);
        let text = ui.theme().text_primary;
        ui.list_view("Vehicles/List", form.next(shown * LIST_ROW_HEIGHT), names.len(), LIST_ROW_HEIGHT, &mut selected, |ui, index, rect, state| {
            let style = if state.selected { BODY_STRONG } else { BODY };
            ui.text(rect.inset_by(SPACE_M + 4.0, 0.0, SPACE_M, 0.0), &names[index], style, text);
        });
        if let Some(selected) = selected {
            *vehicle = selected.min(names.len() - 1);
        }
        let add = Button::new("Add a vehicle").icon(Icon::Add);
        let delete = Button::new("Delete it").icon(Icon::Delete);
        match form.buttons(ui, &[("Vehicles/Add", add), ("Vehicles/Delete", delete)]) {
            Some(0) => {
                let mut copy = document.vehicles.list[*vehicle].clone();
                copy.name = document.vehicles.new_name();
                document.vehicles.list.push(copy);
                *vehicle = document.vehicles.list.len() - 1;
                self.message.clear();
            }
            Some(1) => {
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
            _ => {}
        }
        if !self.message.is_empty() {
            form.warning(ui, "Vehicles/Message", "", &self.message.clone());
        }

        form.heading(ui, "Vehicle");
        // The name, which a rename also changes where it's parked.
        if !ui.is_focused("Vehicles/Name") {
            self.name_text = document.vehicles.list[*vehicle].name.clone();
        }
        form.text_box(ui, "Vehicles/Name", "Name", &mut self.name_text, "Name");
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
        let model_names = models.names();
        let current = &mut document.vehicles.list[*vehicle];
        let mut model = models.find(&current.model).unwrap_or(0);
        if form.combo(ui, "Vehicles/Model", "Model", &model_names, &mut model) {
            current.model = model_names[model].to_string();
        }
        let rates = SliderRange::new(0.0, 2000.0, 1.0);
        let whole = |value: f64| format!("{value:.0}");
        let acceleration = &mut current.acceleration;
        for (part, value) in [
            ("Forward", &mut acceleration.forward),
            ("Reverse", &mut acceleration.reverse),
            ("Left", &mut acceleration.left),
            ("Right", &mut acceleration.right),
        ] {
            form.slider(ui, &format!("Vehicles/{part}"), part, value, rates, whole);
        }
        form.caption(ui, "Acceleration each way, in units a second, a second.");

        form.heading(ui, "Footprint");
        let mut values = sides(&current.footprint);
        let range = SliderRange::new(0.05, 10.0, 200.0);
        for (index, side) in SIDES.iter().enumerate() {
            form.slider(ui, &format!("Footprint/{side}"), side, &mut values[index], range, |value| format!("{value:.3}"));
        }
        if form.buttons(ui, &[("Footprint/Fit", Button::new("Fit it to the model"))]).is_some() {
            values = sides(&models.list[model].footprint);
        }
        current.footprint = Footprint::from_sides(values[0], values[1], values[2], values[3]);
        form.caption(ui, "Drag a side in the view to move it.");
    }

    // The card down the right side, with the vehicle's paint over its
    // model's.
    fn paint_card(&mut self, ui: &mut Ui, document: &mut Document, models: &Models, vehicle: usize) {
        let page = ui.page();
        let card = ui.floating_card(editing::right_card(page).top_strip(PAINT_CARD_HEIGHT), "Paint");
        subtitle(ui, card.header, "This vehicle's, over its model's");
        let mut form = Form::new(card.content);
        let model_paints = document.paints.clone();
        let current = &mut document.vehicles.list[vehicle];
        let model = models.find(&current.model).unwrap_or(0);
        let slots = &models.list[model].slots;
        self.slot = self.slot.min(slots.len() - 1);
        let none = Vec::new();
        let model_name = &models.list[model].name;
        let model_paint = model_paints.iter().find(|(name, _)| name == model_name).map_or(&none, |(_, paint)| paint);
        let materials = models.list[model].materials(&[model_paint, &current.paint]);
        let mut parameters = materials[self.slot];
        if form.paint(ui, "VehiclePaint", slots, &mut self.slot, &mut parameters) {
            let slot = slots[self.slot].clone();
            set_slot(&mut current.paint, slots, &slot, parameters);
        }
        let painted = current.paint.iter().any(|(slot, _)| *slot == slots[self.slot]);
        let reset = Button::new("Use the model's paint").icon(Icon::Refresh);
        if form.buttons(ui, &[("VehiclePaint/Reset", reset)]).is_some() {
            current.paint.retain(|(slot, _)| *slot != slots[self.slot]);
        }
        form.caption(ui, if painted { "This vehicle paints this slot its own way." } else { "This slot has the model's paint." });
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
