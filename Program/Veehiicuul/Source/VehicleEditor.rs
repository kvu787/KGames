// The vehicle editor, and its footprint creator: picks a vehicle's model,
// its acceleration, and its paint over the model's, and draws its footprint,
// the rectangle that touches the track's edges, over its model seen from
// above. A side of the footprint follows a drag from it in the view, and the
// footprint can fit the model's own rectangle.

use json::Value;
use user_interface::theme::{ACCENT, DIM, TEXT, WARNING};
use user_interface::{Color, Frame, SliderRange, Stack};

use crate::collision::Footprint;
use crate::document::Document;
use crate::editing::{self, Viewport, heading, label, slider};
use crate::models::{Models, set_slot};

// The vehicle faces up the screen, which is this way on the ground.
pub const SHOWN_HEADING: f64 = -135.0;
// How near the cursor a side counts as under it, in pixels.
const REACH: f32 = 10.0;
const ROW_HEIGHT: f32 = 24.0;
pub const SIDES: [&str; 4] = ["Left", "Right", "Front", "Back"];

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

impl VehicleEditor {
    pub fn frame(&mut self, frame: &mut Frame, document: &mut Document, models: &Models, vehicle: &mut usize) {
        let inner = frame.panel(editing::panel_area(frame), "Vehicles", "each vehicle's model, driving, footprint and paint");
        let mut stack = Stack::new(inner, 8.0);
        let focusable = frame.set_focusable(false);
        let count = document.vehicles.list.len();
        let rows = count.min(6) as f32;
        let mut selected = Some(*vehicle);
        let list = frame.list("Vehicles/List", stack.next(rows * ROW_HEIGHT + 2.0), count, ROW_HEIGHT, &mut selected);
        for (index, row) in &list.rows {
            let name = &document.vehicles.list[*index].name;
            frame.canvas.text(row.x + 8.0, row.y + (row.height - frame.line_height()) / 2.0, name, TEXT);
        }
        if let Some(selected) = selected {
            *vehicle = selected.min(count - 1);
        }
        match editing::buttons(frame, &mut stack, &[("Vehicles/Add", "Add a vehicle"), ("Vehicles/Delete", "Delete it")]) {
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
            label(frame, &mut stack, &self.message, WARNING);
        }

        heading(frame, &mut stack, "Vehicle");
        frame.set_focusable(focusable);
        // The name, which a rename also changes where it's parked.
        let area = stack.next(frame.control_height());
        let (label_area, field) = area.split_left(150.0, 0.0);
        frame.canvas.text(label_area.x, area.y + (area.height - frame.line_height()) / 2.0, "Name", DIM);
        if frame.focus_name() != Some("Vehicles/Name") {
            self.name_text = document.vehicles.list[*vehicle].name.clone();
        }
        frame.text_field("Vehicles/Name", field, &mut self.name_text, "Name", 40);
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
        let focusable = frame.set_focusable(false);
        let names = models.names();
        // The models' paints, which the vehicle's paint goes over.
        let model_paints = document.paints.clone();
        let current = &mut document.vehicles.list[*vehicle];
        let mut model = models.find(&current.model).unwrap_or(0);
        if editing::select(frame, &mut stack, "Vehicles/Model", "Model", &names, &mut model) {
            current.model = names[model].to_string();
        }
        let rates = SliderRange::new(0.0, 2000.0, 1.0);
        let whole = |value: f64| format!("{value:.0}");
        let acceleration = &mut current.acceleration;
        for (name, label, value) in [
            ("Forward", "Forward", &mut acceleration.forward),
            ("Reverse", "Reverse", &mut acceleration.reverse),
            ("Left", "Left", &mut acceleration.left),
            ("Right", "Right", &mut acceleration.right),
        ] {
            slider(frame, &mut stack, &format!("Vehicles/{name}"), label, value, &rates, whole);
        }

        heading(frame, &mut stack, "Footprint");
        let mut values = sides(&current.footprint);
        let range = SliderRange::new(0.05, 10.0, 200.0);
        for (index, side) in SIDES.iter().enumerate() {
            slider(frame, &mut stack, &format!("Footprint/{side}"), side, &mut values[index], &range, |value| format!("{value:.3}"));
        }
        let model_footprint = models.list[model].footprint;
        if editing::buttons(frame, &mut stack, &[("Footprint/Fit", "Fit it to the model")]).is_some() {
            values = sides(&model_footprint);
        }
        label(frame, &mut stack, "Drag a side in the view to move it.", DIM);

        heading(frame, &mut stack, "Paint");
        let slots = &models.list[model].slots;
        self.slot = self.slot.min(slots.len() - 1);
        let none = Vec::new();
        let model_paint = model_paints.iter().find(|(name, _)| name == names[model]).map_or(&none, |(_, paint)| paint);
        let mut materials = models.list[model].materials(&[model_paint, &current.paint]);
        let mut parameters = materials[self.slot];
        if editing::paint_panel(frame, &mut stack, "VehiclePaint", slots, &mut self.slot, &mut parameters) {
            let slot = slots[self.slot.min(slots.len() - 1)].clone();
            set_slot(&mut current.paint, slots, &slot, parameters);
            materials[self.slot] = parameters;
        }
        let painted = current.paint.iter().any(|(slot, _)| *slot == slots[self.slot]);
        if editing::buttons(frame, &mut stack, &[("VehiclePaint/Reset", "Use the model's paint")]).is_some() {
            current.paint.retain(|(slot, _)| *slot != slots[self.slot]);
        }
        label(
            frame,
            &mut stack,
            if painted { "This vehicle paints this slot its own way." } else { "This slot has the model's paint." },
            DIM,
        );
        frame.set_focusable(focusable);

        // The footprint's sides, dragged in the view.
        let input = self.viewport.interact(frame, "Vehicles/View");
        let heading = SHOWN_HEADING.to_radians();
        let footprint = Footprint::from_sides(values[0], values[1], values[2], values[3]);
        let corners = footprint.corners([0.0, 0.0], heading);
        // Each side's corners, as SIDES has them.
        let side_corners = [[corners[1], corners[2]], [corners[3], corners[0]], [corners[2], corners[3]], [corners[0], corners[1]]];
        let screen = |frame: &Frame, point: [f64; 2]| self.viewport.screen(frame, point);
        let under = input.cursor.filter(|_| !input.panning).and_then(|cursor| {
            (0..4).find(|&side| {
                let [a, b] = side_corners[side].map(|corner| screen(frame, corner));
                distance_to_segment(cursor, a, b) <= REACH
            })
        });
        if input.response.pressed && !input.panning {
            self.drag = under;
        }
        // The drag follows the cursor to where it's let go of.
        if let (Some(side), Some(ground)) = (self.drag, input.ground) {
            let (sin, cos) = heading.sin_cos();
            let left = ground[0] * cos - ground[1] * sin;
            let ahead = ground[0] * sin + ground[1] * cos;
            values[side] = [left, -left, ahead, -ahead][side].clamp(0.05, 10.0);
        }
        if !input.response.held {
            self.drag = None;
        }
        current.footprint = Footprint::from_sides(values[0], values[1], values[2], values[3]);
        let model_corners = model_footprint.corners([0.0, 0.0], heading);
        editing::outline(frame, &self.viewport, &model_corners, 1.0, Color::rgb(0x646C78));
        let corners = current.footprint.corners([0.0, 0.0], heading);
        editing::outline(frame, &self.viewport, &corners, 2.0, ACCENT);
        let side_corners = [[corners[1], corners[2]], [corners[3], corners[0]], [corners[2], corners[3]], [corners[0], corners[1]]];
        for (side, [a, b]) in side_corners.iter().enumerate() {
            let middle = screen(frame, [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]);
            let active = self.drag == Some(side) || (self.drag.is_none() && under == Some(side));
            editing::dot(frame, middle, if active { 7.0 } else { 5.0 }, ACCENT);
        }
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
    let along = [b[0] - a[0], b[1] - a[1]];
    let length = along[0] * along[0] + along[1] * along[1];
    let t = if length > 0.0 {
        (((point[0] - a[0]) * along[0] + (point[1] - a[1]) * along[1]) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let nearest = [a[0] + along[0] * t, a[1] + along[1] * t];
    (point[0] - nearest[0]).hypot(point[1] - nearest[1])
}
