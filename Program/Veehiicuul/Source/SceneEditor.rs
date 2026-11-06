// The scene editor: arranges the current scene's decorations, which are
// models, and the vehicles parked in it, which a press in the view picks
// and a drag moves, and sets the scene's background, its fixed camera, and
// the paint of a decoration's model, which every use of the model shares.

use json::Value;
use user_interface::controls::LIST_ITEM_HEIGHT;
use user_interface::tokens::{BODY, CONTROL_HEIGHT};
use user_interface::{Button, Card, ComboBox, Form, Icon, ListView, Rect, Ui};

use crate::collision::Footprint;
use crate::document::Document;
use crate::editing::{self, HEADED_HEIGHT, Projection, SliderRange, Viewport, header, slider_height, window_size};
use crate::models::{Models, set_slot};
use crate::scene::{Decoration, LARGEST_SCALE, Parked, SMALLEST_SCALE};
use crate::track::Pose;

// What's picked: a decoration, or a parked vehicle, by its index.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Picked {
    Decoration(usize),
    Parked(usize),
}

// The most rows the list shows without scrolling.
const LIST_ROWS: usize = 6;

#[derive(Default)]
pub struct SceneEditor {
    pub picked: Option<Picked>,
    // How far the picked thing is from the point on the ground that a drag
    // grabbed it at.
    grab: Option<[f64; 2]>,
    pub model: usize,
    pub vehicle: usize,
    pub slot: usize,
}

// The rectangle that a thing takes on the ground, as its corners.
fn corners(footprint: Footprint, pose: &Pose, scale: f64) -> [[f64; 2]; 4] {
    let scaled = Footprint {
        min: footprint.min.map(|value| value * scale),
        max: footprint.max.map(|value| value * scale),
    };
    scaled.corners(pose.position, pose.radians())
}

// Whether a point is in a thing's rectangle.
fn under(footprint: Footprint, pose: &Pose, scale: f64, [x, z]: [f64; 2]) -> bool {
    let (sin, cos) = pose.radians().sin_cos();
    let [dx, dz] = [x - pose.position[0], z - pose.position[1]];
    let left = (dx * cos - dz * sin) / scale;
    let ahead = (dx * sin + dz * cos) / scale;
    footprint.min[0] <= left && left <= footprint.max[0] && footprint.min[1] <= ahead && ahead <= footprint.max[1]
}

// The decorations, then the parked vehicles, as rows, with their footprints.
fn footprints(document: &Document, models: &Models, scene: usize) -> Vec<(Footprint, Pose, f64)> {
    let current = &document.scenes[scene];
    let unknown = Footprint::from_sides(1.0, 1.0, 1.0, 1.0);
    let decorations = current.decorations.iter().map(|decoration| {
        let footprint = models.find(&decoration.model).map_or(unknown, |model| models.list[model].footprint);
        (footprint, decoration.pose, decoration.scale)
    });
    let parked = current.parked.iter().map(|parked| {
        let footprint = document
            .vehicles
            .find(&parked.vehicle)
            .map_or(unknown, |vehicle| document.vehicles.list[vehicle].footprint);
        (footprint, parked.pose, 1.0)
    });
    decorations.chain(parked).collect()
}

impl SceneEditor {
    // The view, where a press picks what's under it and a drag moves it,
    // then the card, over it.
    pub fn update(&mut self, ui: &mut Ui, view: Rect, viewport: &mut Viewport, document: &mut Document, models: &Models, scene: usize) {
        let input = viewport.interact(ui, "Scene/View", view);
        let things = footprints(document, models, scene);
        let current = &mut document.scenes[scene];
        let decorations = current.decorations.len();
        let row_of = |row: usize| if row < decorations { Picked::Decoration(row) } else { Picked::Parked(row - decorations) };
        if input.response.pressed
            && !input.panning
            && let Some(ground) = input.ground
        {
            let row = (0..things.len()).rev().find(|&row| {
                let (footprint, pose, scale) = things[row];
                under(footprint, &pose, scale, ground)
            });
            self.picked = row.map(row_of);
            self.grab = row.map(|row| {
                let position = things[row].1.position;
                [position[0] - ground[0], position[1] - ground[1]]
            });
        }
        // The drag follows the pointer to where it's let go of.
        if let (Some(grab), Some(ground), Some(picked)) = (self.grab, input.ground, self.picked) {
            let position = [ground[0] + grab[0], ground[1] + grab[1]];
            match picked {
                Picked::Decoration(index) => current.decorations[index].pose.position = position,
                Picked::Parked(index) => current.parked[index].pose.position = position,
            }
        }
        if !input.held {
            self.grab = None;
        }

        // Each thing's place, and the picked one's rectangle.
        let things = footprints(document, models, scene);
        let projection = Projection::new(ui, viewport.eye);
        let mut painter = ui.painter();
        let theme = painter.theme();
        painter.push_clip(view);
        for (row, (footprint, pose, scale)) in things.iter().enumerate() {
            let picked = self.picked == Some(row_of(row));
            if picked {
                projection.outline(&mut painter, &corners(*footprint, pose, *scale), 2.0, theme.accent);
            }
            let center = projection.point(pose.position);
            painter.circle(center, 5.0, if picked { theme.accent } else { theme.text_primary });
            painter.circle(center, 2.0, theme.acrylic);
        }
        painter.pop_clip();
        let page = ui.page();
        self.card(ui, page, view, viewport, document, models, scene);
    }

    #[allow(clippy::too_many_arguments)]
    fn card(&mut self, ui: &mut Ui, page: Rect, view: Rect, viewport: &Viewport, document: &mut Document, models: &Models, scene: usize) {
        let name = document.scenes[scene].name.clone();
        let open = ui.begin_card(editing::left_card(page), Card::new("Scene").subtitle(&name).floating());
        let mut form = Form::new(open.area.content);
        let vehicle_names: Vec<String> = document.vehicles.list.iter().map(|vehicle| vehicle.name.clone()).collect();
        let vehicle_names: Vec<&str> = vehicle_names.iter().map(String::as_str).collect();
        let model_names = models.names();
        let current = &mut document.scenes[scene];
        let [background, follow] = form.columns::<2>(HEADED_HEIGHT);
        let background = header(ui, background, "Background");
        editing::color_field(ui, "Scene/Background", background, &mut current.background);
        let heights = SliderRange::new(5.0, 200.0, 1.0, 0).suffix(" tall");
        // Its header lines up with the field's.
        let follow = follow.top_strip(slider_height());
        editing::slider(ui, "Scene/FollowHeight", follow, "Following view", &mut current.follow_height, heights);
        let camera = Button::new("Make this view the fixed camera's").icon(Icon::Camera);
        let response = ui.add("Scene/Camera", form.take(CONTROL_HEIGHT), camera);
        ui.tooltip(&response, "The fixed camera looks where the middle of the view right of this card is");
        if response.clicked {
            let middle = editing::visible(view).center();
            current.camera_center = viewport.eye.to_ground([middle.x, middle.y], window_size(ui));
            current.camera_height = viewport.eye.height;
        }

        form.heading(ui, "Decorations and parked vehicles");
        let rows: Vec<String> = current
            .decorations
            .iter()
            .map(|decoration| decoration.model.clone())
            .chain(current.parked.iter().map(|parked| format!("{}, parked", parked.vehicle)))
            .collect();
        let picked_row = self.picked.map(|picked| match picked {
            Picked::Decoration(index) => index,
            Picked::Parked(index) => current.decorations.len() + index,
        });
        let mut selected = picked_row;
        let shown = rows.len().clamp(1, LIST_ROWS) as f32;
        let text = ui.theme().text_primary;
        let list = ListView::new(rows.len(), LIST_ITEM_HEIGHT, |ui, index, rect, _| {
            ui.text(rect, &rows[index], BODY, text);
            ui.set_label(&format!("Scene/Objects/{index}"), &rows[index]);
        });
        ui.add("Scene/Objects", form.take(shown * LIST_ITEM_HEIGHT), list.selection(&mut selected));
        if selected != picked_row {
            self.picked = selected.map(|row| {
                if row < current.decorations.len() {
                    Picked::Decoration(row)
                } else {
                    Picked::Parked(row - current.decorations.len())
                }
            });
        }
        let middle = editing::visible(view).center();
        let center = viewport.eye.to_ground([middle.x, middle.y], window_size(ui));
        let [model, vehicle] = form.columns::<2>(HEADED_HEIGHT);
        let (model, vehicle) = (header(ui, model, "Model"), header(ui, vehicle, "Vehicle"));
        ui.add("Scene/Model", model, ComboBox::new(&mut self.model, &model_names));
        ui.add("Scene/Vehicle", vehicle, ComboBox::new(&mut self.vehicle, &vehicle_names));
        let [add, park, delete] = form.columns::<3>(CONTROL_HEIGHT);
        let add = ui.add("Scene/AddDecoration", add, Button::new("Add the model").icon(Icon::Add));
        ui.tooltip(&add, "Puts the model in the middle of the view");
        let park = ui.add("Scene/Park", park, Button::new("Park the vehicle").icon(Icon::Car));
        ui.tooltip(&park, "Parks the vehicle in the middle of the view");
        let delete = ui.add("Scene/Delete", delete, Button::new("Delete").icon(Icon::Delete));
        ui.tooltip(&delete, "Deletes the one that's picked");
        if add.clicked {
            current.decorations.push(Decoration {
                model: model_names[self.model].to_string(),
                pose: Pose {
                    position: center,
                    heading: 0.0,
                },
                scale: 1.0,
            });
            self.picked = Some(Picked::Decoration(current.decorations.len() - 1));
        }
        if park.clicked {
            current.parked.push(Parked {
                vehicle: vehicle_names[self.vehicle].to_string(),
                pose: Pose {
                    position: center,
                    heading: 0.0,
                },
            });
            self.picked = Some(Picked::Parked(current.parked.len() - 1));
        }
        if delete.clicked {
            match self.picked {
                Some(Picked::Decoration(index)) if index < current.decorations.len() => {
                    current.decorations.remove(index);
                }
                Some(Picked::Parked(index)) if index < current.parked.len() => {
                    current.parked.remove(index);
                }
                _ => {}
            }
            self.picked = None;
        }
        // A pick that's gone, like after an undo.
        self.picked = self.picked.filter(|picked| match *picked {
            Picked::Decoration(index) => index < current.decorations.len(),
            Picked::Parked(index) => index < current.parked.len(),
        });

        let degrees = SliderRange::new(-180.0, 180.0, 10.0, 1).suffix("°");
        let mut paint_model = None;
        match self.picked {
            Some(Picked::Decoration(index)) => {
                let decoration = &mut current.decorations[index];
                form.heading(ui, &format!("The {}", decoration.model));
                let [heading, scale] = form.columns::<2>(slider_height());
                editing::slider(ui, "Scene/Heading", heading, "Heading", &mut decoration.pose.heading, degrees);
                let scales = SliderRange::new(SMALLEST_SCALE, LARGEST_SCALE.min(20.0), 100.0, 2);
                editing::slider(ui, "Scene/Scale", scale, "Scale", &mut decoration.scale, scales);
                paint_model = models.find(&decoration.model);
            }
            Some(Picked::Parked(index)) => {
                let parked = &mut current.parked[index];
                form.heading(ui, &format!("The parked {}", parked.vehicle));
                editing::slider_row(&mut form, ui, "Scene/Heading", "Heading", &mut parked.pose.heading, degrees);
                form.caption(ui, "The Vehicles tab paints vehicles.");
            }
            None => form.caption(ui, "Press one in the view, or in the list, to pick it."),
        }
        if let Some(model) = paint_model {
            let name = models.list[model].name.clone();
            let slots = &models.list[model].slots;
            self.slot = self.slot.min(slots.len() - 1);
            let mut parameters = document.model_materials(models, model)[self.slot];
            form.heading(ui, &format!("The {name}'s paint, everywhere it is"));
            if editing::paint(&mut form, ui, "ModelPaint", slots, &mut self.slot, &mut parameters) {
                let slot = slots[self.slot].clone();
                set_slot(document.paint_mut(&name), slots, &slot, parameters);
            }
        }
        ui.end_card(open, form.bottom());
    }

    pub fn state(&self, document: &Document, scene: usize) -> Value {
        let current = &document.scenes[scene];
        let picked = match self.picked {
            Some(Picked::Decoration(index)) => format!("Decoration {index}").into(),
            Some(Picked::Parked(index)) => format!("Parked {index}").into(),
            None => Value::Null,
        };
        let decorations = current.decorations.iter().map(|decoration| {
            json::object([
                ("Model", decoration.model.clone().into()),
                ("Pose", decoration.pose.to_value()),
                ("Scale", decoration.scale.into()),
            ])
        });
        let parked = current
            .parked
            .iter()
            .map(|parked| json::object([("Vehicle", parked.vehicle.clone().into()), ("Pose", parked.pose.to_value())]));
        json::object([
            ("Picked", picked),
            ("Decorations", Value::Array(decorations.collect())),
            ("Parked", Value::Array(parked.collect())),
            ("Background", crate::files::color_text(current.background).into()),
            ("CameraHeight", current.camera_height.into()),
        ])
    }
}
