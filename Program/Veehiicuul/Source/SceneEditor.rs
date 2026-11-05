// The scene editor: arranges the current scene's decorations, which are
// models, and the vehicles parked in it, which a press in the view picks
// and a drag moves, and sets the scene's background, its fixed camera, and
// the paint of a decoration's model, which every use of the model shares.

use json::Value;
use user_interface::{Align, Button, Frame, Icon, ROW_HEIGHT, SPACE_XS, SliderRange, TEXT_INSET, TextStyle};

use crate::collision::Footprint;
use crate::document::Document;
use crate::editing::{self, Viewport, heading, label, slider};
use crate::models::{Models, set_slot};
use crate::scene::{Decoration, LARGEST_SCALE, Parked, SMALLEST_SCALE};
use crate::track::Pose;

// What's picked: a decoration, or a parked vehicle, by its index.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Picked {
    Decoration(usize),
    Parked(usize),
}

#[derive(Default)]
pub struct SceneEditor {
    pub picked: Option<Picked>,
    // How far the picked thing is from the point on the ground that a drag
    // grabbed it at.
    grab: Option<[f64; 2]>,
    pub model: usize,
    pub vehicle: usize,
    pub slot: usize,
    background_text: String,
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

impl SceneEditor {
    pub fn frame(&mut self, frame: &mut Frame, viewport: &mut Viewport, document: &mut Document, models: &Models, scene: usize) {
        let theme = frame.theme();
        let title = format!("Scene: {}", document.scenes[scene].name);
        let inner = editing::card(frame, &title, "the decorations, parked vehicles, background and camera");
        let mut stack = editing::rows(inner);
        let vehicle_names: Vec<String> = document.vehicles.list.iter().map(|vehicle| vehicle.name.clone()).collect();
        let model_names = models.names();
        let current = &mut document.scenes[scene];
        editing::color_field(frame, &mut stack, "Scene/Background", "Background", &mut self.background_text, &mut current.background);
        let focusable = frame.set_focusable(false);
        slider(
            frame,
            &mut stack,
            "Scene/FollowHeight",
            "Following view",
            &mut current.follow_height,
            &SliderRange::new(5.0, 200.0, 1.0),
            |value| format!("{value:.0} tall"),
        );
        if editing::buttons(frame, &mut stack, &[("Scene/Camera", Button::new("Make this view the fixed camera's"))]).is_some() {
            let size = [frame.width, frame.height];
            current.camera_center = viewport.eye.to_ground(editing::view_area(frame).center(), size);
            current.camera_height = viewport.eye.height;
        }

        heading(frame, &mut stack, "Decorations and parked vehicles");
        let count = current.decorations.len() + current.parked.len();
        let picked_row = self.picked.map(|picked| match picked {
            Picked::Decoration(index) => index,
            Picked::Parked(index) => current.decorations.len() + index,
        });
        let mut selected = picked_row;
        let rows = (count.clamp(1, 6)) as f32;
        let (decorations, parked) = (&current.decorations, &current.parked);
        frame.list("Scene/Objects", stack.next(rows * ROW_HEIGHT), count, ROW_HEIGHT, &mut selected, |canvas, row| {
            let text = if row.index < decorations.len() {
                decorations[row.index].model.clone()
            } else {
                format!("{}, parked", parked[row.index - decorations.len()].vehicle)
            };
            let area = row.area.inset_by(TEXT_INSET + SPACE_XS, 0.0);
            let style = if row.selected { TextStyle::BodyStrong } else { TextStyle::Body };
            canvas.text_in(area, &text, style, theme.text, Align::Start);
        });
        if selected != picked_row {
            self.picked = selected.map(|row| {
                if row < current.decorations.len() {
                    Picked::Decoration(row)
                } else {
                    Picked::Parked(row - current.decorations.len())
                }
            });
        }
        let size = [frame.width, frame.height];
        let center = viewport.eye.to_ground(editing::view_area(frame).center(), size);
        editing::select(frame, &mut stack, "Scene/Model", "Model", &model_names, &mut self.model);
        let vehicle_labels: Vec<&str> = vehicle_names.iter().map(String::as_str).collect();
        self.vehicle = self.vehicle.min(vehicle_labels.len() - 1);
        editing::select(frame, &mut stack, "Scene/Vehicle", "Vehicle", &vehicle_labels, &mut self.vehicle);
        match editing::buttons(
            frame,
            &mut stack,
            &[
                ("Scene/AddDecoration", Button::new("Add the model").icon(Icon::Add)),
                ("Scene/Park", Button::new("Park the vehicle").icon(Icon::Add)),
                ("Scene/Delete", Button::new("Delete").icon(Icon::Delete)),
            ],
        ) {
            Some(0) => {
                current.decorations.push(Decoration {
                    model: model_names[self.model.min(model_names.len() - 1)].to_string(),
                    pose: Pose {
                        position: center,
                        heading: 0.0,
                    },
                    scale: 1.0,
                });
                self.picked = Some(Picked::Decoration(current.decorations.len() - 1));
            }
            Some(1) => {
                current.parked.push(Parked {
                    vehicle: vehicle_names[self.vehicle].clone(),
                    pose: Pose {
                        position: center,
                        heading: 0.0,
                    },
                });
                self.picked = Some(Picked::Parked(current.parked.len() - 1));
            }
            Some(2) => {
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
            _ => {}
        }
        // A pick that's gone, like after an undo.
        self.picked = self.picked.filter(|picked| match *picked {
            Picked::Decoration(index) => index < current.decorations.len(),
            Picked::Parked(index) => index < current.parked.len(),
        });

        let degrees = SliderRange::new(-180.0, 180.0, 10.0);
        let mut paint_model = None;
        match self.picked {
            Some(Picked::Decoration(index)) => {
                let decoration = &mut current.decorations[index];
                heading(frame, &mut stack, &format!("The {}", decoration.model));
                slider(frame, &mut stack, "Scene/Heading", "Heading", &mut decoration.pose.heading, &degrees, |value| format!("{value:.1} deg"));
                let scale = SliderRange::new(SMALLEST_SCALE, LARGEST_SCALE.min(20.0), 100.0);
                slider(frame, &mut stack, "Scene/Scale", "Scale", &mut decoration.scale, &scale, |value| format!("{value:.2}"));
                paint_model = models.find(&decoration.model);
            }
            Some(Picked::Parked(index)) => {
                let parked = &mut current.parked[index];
                heading(frame, &mut stack, &format!("The parked {}", parked.vehicle));
                slider(frame, &mut stack, "Scene/Heading", "Heading", &mut parked.pose.heading, &degrees, |value| format!("{value:.1} deg"));
                label(frame, &mut stack, "The Vehicles tab paints vehicles.", theme.text_secondary);
            }
            None => label(frame, &mut stack, "Press one in the view, or in the list, to pick it.", theme.text_secondary),
        }
        if let Some(model) = paint_model {
            let name = models.list[model].name.clone();
            let slots = &models.list[model].slots;
            self.slot = self.slot.min(slots.len() - 1);
            let mut parameters = document.model_materials(models, model)[self.slot];
            heading(frame, &mut stack, &format!("The {name}'s paint, everywhere"));
            if editing::paint_controls(frame, &mut stack, "ModelPaint", slots, &mut self.slot, &mut parameters) {
                let slot = slots[self.slot].clone();
                set_slot(document.paint_mut(&name), slots, &slot, parameters);
            }
        }
        frame.set_focusable(focusable);

        // A press picks what's under it, and a drag moves it.
        let input = viewport.interact(frame, "Scene/View");
        let current = &mut document.scenes[scene];
        let footprints: Vec<(Footprint, Pose, f64)> = current
            .decorations
            .iter()
            .map(|decoration| {
                let footprint = models.find(&decoration.model).map_or(Footprint::from_sides(1.0, 1.0, 1.0, 1.0), |model| models.list[model].footprint);
                (footprint, decoration.pose, decoration.scale)
            })
            .chain(current.parked.iter().map(|parked| {
                let footprint = document.vehicles.find(&parked.vehicle).map_or(Footprint::from_sides(1.0, 1.0, 1.0, 1.0), |vehicle| document.vehicles.list[vehicle].footprint);
                (footprint, parked.pose, 1.0)
            }))
            .collect();
        if input.response.pressed
            && !input.panning
            && let Some(ground) = input.ground
        {
            let row = (0..footprints.len()).rev().find(|&row| {
                let (footprint, pose, scale) = footprints[row];
                under(footprint, &pose, scale, ground)
            });
            let decorations = current.decorations.len();
            self.picked = row.map(|row| if row < decorations { Picked::Decoration(row) } else { Picked::Parked(row - decorations) });
            self.grab = row.map(|row| {
                let position = footprints[row].1.position;
                [position[0] - ground[0], position[1] - ground[1]]
            });
        }
        // The drag follows the cursor to where it's let go of.
        if let (Some(grab), Some(ground), Some(picked)) = (self.grab, input.ground, self.picked) {
            let position = [ground[0] + grab[0], ground[1] + grab[1]];
            match picked {
                Picked::Decoration(index) => current.decorations[index].pose.position = position,
                Picked::Parked(index) => current.parked[index].pose.position = position,
            }
        }
        if !input.response.held {
            self.grab = None;
        }
        for (row, (footprint, pose, scale)) in footprints.iter().enumerate() {
            let decorations = current.decorations.len();
            let this = if row < decorations { Picked::Decoration(row) } else { Picked::Parked(row - decorations) };
            let picked = self.picked == Some(this);
            let color = if picked { theme.accent } else { theme.text };
            if picked {
                editing::outline(frame, viewport, &corners(*footprint, pose, *scale), 2.0, theme.accent);
            }
            let center = viewport.screen(frame, pose.position);
            editing::dot(frame, viewport, center, 4.0, color);
        }
    }

    pub fn state(&self, document: &Document, scene: usize) -> Value {
        let current = &document.scenes[scene];
        let picked = match self.picked {
            Some(Picked::Decoration(index)) => format!("Decoration {index}").into(),
            Some(Picked::Parked(index)) => format!("Parked {index}").into(),
            None => Value::Null,
        };
        json::object([
            ("Picked", picked),
            ("Decorations", Value::Array(current.decorations.iter().map(|decoration| {
                json::object([("Model", decoration.model.clone().into()), ("Pose", decoration.pose.to_value()), ("Scale", decoration.scale.into())])
            }).collect())),
            ("Parked", Value::Array(current.parked.iter().map(|parked| {
                json::object([("Vehicle", parked.vehicle.clone().into()), ("Pose", parked.pose.to_value())])
            }).collect())),
            ("Background", crate::files::color_text(current.background).into()),
            ("CameraHeight", current.camera_height.into()),
        ])
    }
}
