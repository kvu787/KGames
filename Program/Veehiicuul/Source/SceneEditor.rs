// The scene editor: arranges the current scene's decorations, which are
// models, and the vehicles parked in it, which a press in the view picks
// and a drag moves, and sets the scene's background, its fixed camera, and
// the paint of a decoration's model, which every use of the model shares.

use json::Value;
use simple_paint::Parameters;
use user_interface::controls::LIST_ITEM_HEIGHT;
use user_interface::tokens::CONTROL_HEIGHT;
use user_interface::{Button, Canvas, Card, ComboBox, Form, Icon, ListView, Rect, Text, Tree, Ui};

use crate::collision::Footprint;
use crate::document::Document;
use crate::editing::{self, GroundInput, HEADED_HEIGHT, Projection, SliderRange, Viewport, header, slider_height, window_size};
use crate::files;
use crate::models::{Models, set_slot};
use crate::program::Message;
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

// What the card's controls do. What needs the view's middle on the ground
// carries it from the frame it was pressed on.
#[derive(Clone, Debug)]
pub enum SceneMsg {
    Pick(usize),
    Model(usize),
    Vehicle(usize),
    Add { center: [f64; 2] },
    Park { center: [f64; 2] },
    Delete,
    Camera { center: [f64; 2], height: f64 },
    Background(files::Color),
    FollowHeight(f64),
    Heading(f64),
    Scale(f64),
    Slot(usize),
    Paint(Parameters),
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
    // Drops a pick that's gone, like after an undo, even one during a drag.
    pub fn keep_picked(&mut self, document: &Document, scene: usize) {
        let current = &document.scenes[scene];
        self.picked = self.picked.filter(|picked| match *picked {
            Picked::Decoration(index) => index < current.decorations.len(),
            Picked::Parked(index) => index < current.parked.len(),
        });
        if self.picked.is_none() {
            self.grab = None;
        }
    }

    fn row_of(&self, document: &Document, scene: usize, row: usize) -> Picked {
        let decorations = document.scenes[scene].decorations.len();
        if row < decorations { Picked::Decoration(row) } else { Picked::Parked(row - decorations) }
    }

    // What the pointer in the view does: a press picks what's under it, and
    // a drag moves it.
    pub fn view_input(&mut self, input: GroundInput, document: &mut Document, models: &Models, scene: usize) {
        self.keep_picked(document, scene);
        let things = footprints(document, models, scene);
        if input.pressed
            && !input.panning
            && let Some(ground) = input.ground
        {
            let row = (0..things.len()).rev().find(|&row| {
                let (footprint, pose, scale) = things[row];
                under(footprint, &pose, scale, ground)
            });
            self.picked = row.map(|row| self.row_of(document, scene, row));
            self.grab = row.map(|row| {
                let position = things[row].1.position;
                [position[0] - ground[0], position[1] - ground[1]]
            });
        }
        // The drag follows the pointer to where it's let go of.
        if let (Some(grab), Some(ground), Some(picked)) = (self.grab, input.ground, self.picked) {
            let position = [ground[0] + grab[0], ground[1] + grab[1]];
            let current = &mut document.scenes[scene];
            match picked {
                Picked::Decoration(index) => current.decorations[index].pose.position = position,
                Picked::Parked(index) => current.parked[index].pose.position = position,
            }
        }
        if !input.held {
            self.grab = None;
        }
    }

    // A control on the card.
    pub fn apply(&mut self, message: SceneMsg, document: &mut Document, models: &Models, scene: usize) {
        self.keep_picked(document, scene);
        match message {
            SceneMsg::Pick(row) => self.picked = Some(self.row_of(document, scene, row)),
            SceneMsg::Model(model) => self.model = model,
            SceneMsg::Vehicle(vehicle) => self.vehicle = vehicle,
            SceneMsg::Add { center } => {
                let model_names = models.names();
                let model = self.model.min(model_names.len().saturating_sub(1));
                let current = &mut document.scenes[scene];
                current.decorations.push(Decoration {
                    model: model_names[model].to_string(),
                    pose: Pose {
                        position: center,
                        heading: 0.0,
                    },
                    scale: 1.0,
                });
                self.picked = Some(Picked::Decoration(current.decorations.len() - 1));
            }
            SceneMsg::Park { center } => {
                let vehicle = self.vehicle.min(document.vehicles.list.len() - 1);
                let name = document.vehicles.list[vehicle].name.clone();
                let current = &mut document.scenes[scene];
                current.parked.push(Parked {
                    vehicle: name,
                    pose: Pose {
                        position: center,
                        heading: 0.0,
                    },
                });
                self.picked = Some(Picked::Parked(current.parked.len() - 1));
            }
            SceneMsg::Delete => {
                let current = &mut document.scenes[scene];
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
            SceneMsg::Camera { center, height } => {
                let current = &mut document.scenes[scene];
                current.camera_center = center;
                current.camera_height = height;
            }
            SceneMsg::Background(color) => document.scenes[scene].background = color,
            SceneMsg::FollowHeight(height) => document.scenes[scene].follow_height = height,
            SceneMsg::Heading(heading) => {
                let current = &mut document.scenes[scene];
                match self.picked {
                    Some(Picked::Decoration(index)) => current.decorations[index].pose.heading = heading,
                    Some(Picked::Parked(index)) => current.parked[index].pose.heading = heading,
                    None => {}
                }
            }
            SceneMsg::Scale(scale) => {
                if let Some(Picked::Decoration(index)) = self.picked {
                    document.scenes[scene].decorations[index].scale = scale;
                }
            }
            SceneMsg::Slot(slot) => self.slot = slot,
            SceneMsg::Paint(parameters) => {
                if let Some(Picked::Decoration(index)) = self.picked
                    && let Some(model) = models.find(&document.scenes[scene].decorations[index].model)
                {
                    let name = models.list[model].name.clone();
                    let slots = &models.list[model].slots;
                    let slot = slots[self.slot.min(slots.len() - 1)].clone();
                    set_slot(document.paint_mut(&name), slots, &slot, parameters);
                }
            }
        }
    }

    // The overlay over the view, then the card, over both.
    #[allow(clippy::too_many_arguments)]
    pub fn view(&self, ui: &mut Ui, tree: &mut Tree<Message>, page: Rect, view: Rect, viewport: &Viewport, document: &Document, models: &Models, scene: usize) {
        // Each thing's place, and the picked one's rectangle.
        let things = footprints(document, models, scene);
        let decorations = document.scenes[scene].decorations.len();
        let row_of = |row: usize| if row < decorations { Picked::Decoration(row) } else { Picked::Parked(row - decorations) };
        let marks: Vec<([[f64; 2]; 4], [f64; 2], bool)> = things
            .iter()
            .enumerate()
            .map(|(row, (footprint, pose, scale))| (corners(*footprint, pose, *scale), pose.position, self.picked == Some(row_of(row))))
            .collect();
        let projection = Projection::new(viewport.eye, window_size(ui));
        tree.draw(
            view,
            Canvas::new(move |painter, rect| {
                let theme = painter.theme().clone();
                painter.push_clip(rect);
                for (corners, position, picked) in &marks {
                    if *picked {
                        projection.outline(painter, corners, 2.0, theme.accent);
                    }
                    let center = projection.point(*position);
                    painter.circle(center, 5.0, if *picked { theme.accent } else { theme.text_primary });
                    painter.circle(center, 2.0, theme.acrylic);
                }
                painter.pop_clip();
            }),
        );
        self.card(ui, tree, page, view, viewport, document, models, scene);
    }

    #[allow(clippy::too_many_arguments)]
    fn card(&self, ui: &mut Ui, tree: &mut Tree<Message>, page: Rect, view: Rect, viewport: &Viewport, document: &Document, models: &Models, scene: usize) {
        let name = document.scenes[scene].name.clone();
        let middle = editing::visible(view).center();
        let center = viewport.eye.to_ground([middle.x, middle.y], window_size(ui));
        let camera = SceneMsg::Camera {
            center,
            height: viewport.eye.height,
        };
        tree.sized_card(ui, None, editing::left_card(page), Card::new("Scene").subtitle(&name).floating(), |ui, card, area| {
            let mut form = Form::new(area.content);
            let vehicle_names: Vec<String> = document.vehicles.list.iter().map(|vehicle| vehicle.name.clone()).collect();
            let vehicle_names: Vec<&str> = vehicle_names.iter().map(String::as_str).collect();
            let model_names = models.names();
            let current = &document.scenes[scene];
            let [background, follow] = form.columns::<2>(HEADED_HEIGHT);
            let background = header(card, background, "Background");
            editing::color_field(card, "Scene/Background", background, current.background, |color| {
                Message::Scene(SceneMsg::Background(color))
            });
            let heights = SliderRange::new(5.0, 200.0, 1.0, 0).suffix(" tall");
            // Its header lines up with the field's.
            let follow = follow.top_strip(slider_height());
            card.add("Scene/FollowHeight", follow, editing::slider("Following view", current.follow_height, heights, |value| {
                Message::Scene(SceneMsg::FollowHeight(value))
            }));
            let button = Button::new("Make this view the fixed camera's").icon(Icon::Camera).on_press(Message::Scene(camera));
            card.add("Scene/Camera", form.take(CONTROL_HEIGHT), button)
                .set_tooltip("The fixed camera looks where the middle of the view right of this card is");

            form.heading(card, "Decorations and parked vehicles");
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
            let shown = rows.len().clamp(1, LIST_ROWS) as f32;
            let labels = rows.clone();
            let list = ListView::new(rows.len(), LIST_ITEM_HEIGHT)
                .selection(picked_row)
                .on_select(|row| Message::Scene(SceneMsg::Pick(row)))
                .labels(move |row| labels[row].clone());
            let list_rect = form.take(shown * LIST_ITEM_HEIGHT);
            let shown_rows = list.rows(ui, "Scene/Objects", list_rect);
            card.container(Some("Scene/Objects"), list_rect, list, |items| {
                for row in shown_rows.indices() {
                    items.draw(shown_rows.content(row), Text::label(&rows[row]));
                }
            });
            let [model, vehicle] = form.columns::<2>(HEADED_HEIGHT);
            let (model, vehicle) = (header(card, model, "Model"), header(card, vehicle, "Vehicle"));
            card.add("Scene/Model", model, ComboBox::new(self.model.min(model_names.len().saturating_sub(1)), &model_names).on_select(|model| {
                Message::Scene(SceneMsg::Model(model))
            }));
            card.add("Scene/Vehicle", vehicle, ComboBox::new(self.vehicle.min(vehicle_names.len() - 1), &vehicle_names).on_select(|vehicle| {
                Message::Scene(SceneMsg::Vehicle(vehicle))
            }));
            let [add, park, delete] = form.columns::<3>(CONTROL_HEIGHT);
            card.add("Scene/AddDecoration", add, Button::new("Add the model").icon(Icon::Add).on_press(Message::Scene(SceneMsg::Add { center })))
                .set_tooltip("Puts the model in the middle of the view");
            card.add("Scene/Park", park, Button::new("Park the vehicle").icon(Icon::Car).on_press(Message::Scene(SceneMsg::Park { center })))
                .set_tooltip("Parks the vehicle in the middle of the view");
            card.add("Scene/Delete", delete, Button::new("Delete").icon(Icon::Delete).on_press(Message::Scene(SceneMsg::Delete)))
                .set_tooltip("Deletes the one that's picked");

            let degrees = SliderRange::new(-180.0, 180.0, 10.0, 1).suffix("°");
            let mut paint_model = None;
            match self.picked {
                Some(Picked::Decoration(index)) => {
                    let decoration = &current.decorations[index];
                    form.heading(card, &format!("The {}", decoration.model));
                    let [heading, scale] = form.columns::<2>(slider_height());
                    card.add("Scene/Heading", heading, editing::slider("Heading", decoration.pose.heading, degrees, |value| {
                        Message::Scene(SceneMsg::Heading(value))
                    }));
                    let scales = SliderRange::new(SMALLEST_SCALE, LARGEST_SCALE, 100.0, 2);
                    card.add("Scene/Scale", scale, editing::slider("Scale", decoration.scale, scales, |value| {
                        Message::Scene(SceneMsg::Scale(value))
                    }));
                    paint_model = models.find(&decoration.model);
                }
                Some(Picked::Parked(index)) => {
                    let parked = &current.parked[index];
                    form.heading(card, &format!("The parked {}", parked.vehicle));
                    editing::slider_row(&mut form, card, "Scene/Heading", "Heading", parked.pose.heading, degrees, |value| {
                        Message::Scene(SceneMsg::Heading(value))
                    });
                    form.caption(ui, card, "The Vehicles tab paints vehicles.");
                }
                None => form.caption(ui, card, "Press one in the view, or in the list, to pick it."),
            }
            if let Some(model) = paint_model {
                let name = &models.list[model].name;
                let slots = &models.list[model].slots;
                let slot = self.slot.min(slots.len() - 1);
                let parameters = document.model_materials(models, model)[slot];
                form.heading(card, &format!("The {name}'s paint, everywhere it is"));
                editing::paint(
                    &mut form,
                    card,
                    "ModelPaint",
                    slots,
                    slot,
                    parameters,
                    |slot| Message::Scene(SceneMsg::Slot(slot)),
                    |parameters| Message::Scene(SceneMsg::Paint(parameters)),
                );
            }
            form.bottom()
        });
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
