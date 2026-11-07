// The scene editor: arranges the current scene's decorations, which are
// models, and the vehicles parked in it, which a press in the view picks
// and a drag moves, and sets the scene's background, its fixed camera, and
// the paint of a decoration's model, which every use of the model shares.
//
// Its card's shape follows what's picked, so the program rebuilds the tree
// when the fingerprint changes.

use json::Value;
use user_interface::controls::LIST_ITEM_HEIGHT;
use user_interface::tokens::CONTROL_HEIGHT;
use user_interface::{Button, Card, ComboBox, Icon, ListRow, ListView, Rect, Ui, UiEventKind};

use crate::collision::Footprint;
use crate::document::Document;
use crate::editing::{self, HEADED_HEIGHT, PaintControls, Projection, Role, Shape, SliderRange, Viewport, header, slider_height, window_size};
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

const DEGREES: SliderRange = SliderRange::new(-180.0, 180.0, 10.0, 1).suffix("°");
const SCALES: SliderRange = SliderRange::new(SMALLEST_SCALE, LARGEST_SCALE, 100.0, 2);
const HEIGHTS: SliderRange = SliderRange::new(5.0, 200.0, 1.0, 0).suffix(" tall");

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
    // What's picked, as a kind, and whether a model's paint shows: the
    // card's shape, which a change to re-lays out.
    pub fn fingerprint(&self, document: &Document, models: &Models, scene: usize) -> String {
        let current = &document.scenes[scene];
        let (kind, paint) = match self.picked {
            Some(Picked::Decoration(index)) => {
                let model = current.decorations.get(index).and_then(|decoration| models.find(&decoration.model));
                (format!("decoration {:?}", current.decorations.get(index).map(|decoration| &decoration.model)), model)
            }
            Some(Picked::Parked(index)) => (format!("parked {:?}", current.parked.get(index).map(|parked| &parked.vehicle)), None),
            None => ("none".to_string(), None),
        };
        format!("{kind} paint {paint:?} slots {:?}", paint.map(|model| models.list[model].slots.len()))
    }

    // The view, the overlay over it, and the card.
    pub fn build(&self, ui: &mut Ui, view: Rect, document: &Document, models: &Models, scene: usize) {
        let viewport_name = "Scene/View";
        ui.node(viewport_name, view, user_interface::View::new());
        editing::overlay_node(ui, "Scene/Overlay", view);
        let page = ui.page();
        self.card(ui, page, document, models, scene);
    }

    fn card(&self, ui: &mut Ui, page: Rect, document: &Document, models: &Models, scene: usize) {
        let current = &document.scenes[scene];
        let name = current.name.clone();
        let card = Card::new("Scene").subtitle(&name).floating();
        let area = card.area(ui, editing::left_card(page));
        let mut bottom = area.content.y;
        ui.group("SceneCard", editing::left_card(page), Card::new("Scene").subtitle(&name).floating(), |ui| {
            let mut form = user_interface::Form::new("SceneForm", area.content);
            let model_names = models.names();
            let vehicle_names: Vec<String> = document.vehicles.list.iter().map(|vehicle| vehicle.name.clone()).collect();
            let [background, follow] = form.columns::<2>(HEADED_HEIGHT);
            let background = header(ui, "Scene/BackgroundLabel", background, "Background");
            editing::color_field_node(ui, "Scene/Background", background, current.background);
            // Its header lines up with the field's.
            let follow = follow.top_strip(slider_height());
            editing::slider_node(ui, "Scene/FollowHeight", follow, "Following view", current.follow_height, HEIGHTS);
            ui.node("Scene/Camera", form.take(CONTROL_HEIGHT), Button::new("Make this view the fixed camera's").icon(Icon::Camera));
            ui.set_tooltip("Scene/Camera", "The fixed camera looks where the middle of the view right of this card is");

            form.heading(ui, "Decorations and parked vehicles");
            let rows: Vec<ListRow> = current
                .decorations
                .iter()
                .map(|decoration| decoration.model.clone())
                .chain(current.parked.iter().map(|parked| format!("{}, parked", parked.vehicle)))
                .map(|text| ListRow::new(&text))
                .collect();
            let shown = rows.len().clamp(1, LIST_ROWS) as f32;
            let mut list = ListView::new(rows, LIST_ITEM_HEIGHT).selectable();
            list.set_selected(self.picked_row(current));
            ui.node("Scene/Objects", form.take(shown * LIST_ITEM_HEIGHT), list);
            let [model, vehicle] = form.columns::<2>(HEADED_HEIGHT);
            let model = header(ui, "Scene/ModelLabel", model, "Model");
            let vehicle = header(ui, "Scene/VehicleLabel", vehicle, "Vehicle");
            ui.node("Scene/Model", model, ComboBox::new(&model_names, self.model));
            let vehicle_names: Vec<&str> = vehicle_names.iter().map(String::as_str).collect();
            ui.node("Scene/Vehicle", vehicle, ComboBox::new(&vehicle_names, self.vehicle));
            let [add, park, delete] = form.columns::<3>(CONTROL_HEIGHT);
            ui.node("Scene/AddDecoration", add, Button::new("Add the model").icon(Icon::Add));
            ui.set_tooltip("Scene/AddDecoration", "Puts the model in the middle of the view");
            ui.node("Scene/Park", park, Button::new("Park the vehicle").icon(Icon::Car));
            ui.set_tooltip("Scene/Park", "Parks the vehicle in the middle of the view");
            ui.node("Scene/Delete", delete, Button::new("Delete").icon(Icon::Delete));
            ui.set_tooltip("Scene/Delete", "Deletes the one that's picked");

            let mut paint_model = None;
            match self.picked {
                Some(Picked::Decoration(index)) if index < current.decorations.len() => {
                    let decoration = &current.decorations[index];
                    form.heading(ui, &format!("The {}", decoration.model));
                    let [heading, scale] = form.columns::<2>(slider_height());
                    editing::slider_node(ui, "Scene/Heading", heading, "Heading", decoration.pose.heading, DEGREES);
                    editing::slider_node(ui, "Scene/Scale", scale, "Scale", decoration.scale, SCALES);
                    paint_model = models.find(&decoration.model);
                }
                Some(Picked::Parked(index)) if index < current.parked.len() => {
                    let parked = &current.parked[index];
                    form.heading(ui, &format!("The parked {}", parked.vehicle));
                    editing::slider_row(&mut form, ui, "Scene/Heading", "Heading", parked.pose.heading, DEGREES);
                    form.caption(ui, "The Vehicles tab paints vehicles.");
                }
                _ => form.caption(ui, "Press one in the view, or in the list, to pick it."),
            }
            if let Some(model) = paint_model {
                let name = models.list[model].name.clone();
                let slots = &models.list[model].slots;
                let slot = self.slot.min(slots.len() - 1);
                let parameters = document.model_materials(models, model)[slot];
                form.heading(ui, &format!("The {name}'s paint, everywhere it is"));
                PaintControls::build(&mut form, ui, "ModelPaint", slots, slot, &parameters);
            }
            bottom = form.bottom();
        });
        let rect = Rect::new(page.x, page.y, editing::CARD_WIDTH, card.height_around(bottom - area.content.y));
        ui.set_rect("SceneCard", rect);
    }

    fn picked_row(&self, current: &crate::scene::Scene) -> Option<usize> {
        self.picked.map(|picked| match picked {
            Picked::Decoration(index) => index,
            Picked::Parked(index) => current.decorations.len() + index,
        })
    }

    // The view, where a press picks what's under it and a drag moves it, the
    // frame's events, and what the card shows.
    pub fn update(&mut self, ui: &mut Ui, view: Rect, viewport: &mut Viewport, document: &mut Document, models: &Models, scene: usize) {
        let input = viewport.apply(ui, "Scene/View");
        let things = footprints(document, models, scene);
        {
            let current = &mut document.scenes[scene];
            // A pick that's gone, like after an undo, even one during a drag.
            self.picked = self.picked.filter(|picked| match *picked {
                Picked::Decoration(index) => index < current.decorations.len(),
                Picked::Parked(index) => index < current.parked.len(),
            });
            if self.picked.is_none() {
                self.grab = None;
            }
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
        }

        self.events(ui, view, viewport, document, models, scene);

        // Each thing's place, and the picked one's rectangle.
        let things = footprints(document, models, scene);
        let current = &document.scenes[scene];
        let decorations = current.decorations.len();
        let row_of = |row: usize| if row < decorations { Picked::Decoration(row) } else { Picked::Parked(row - decorations) };
        let projection = Projection::new(ui, viewport.eye);
        let mut shapes = Vec::new();
        for (row, (footprint, pose, scale)) in things.iter().enumerate() {
            let picked = self.picked == Some(row_of(row));
            if picked {
                shapes.push(projection.outline(&corners(*footprint, pose, *scale), 2.0, Role::Accent));
            }
            let center = projection.point(pose.position);
            shapes.push(Shape::Circle(center, 5.0, if picked { Role::Accent } else { Role::Primary }));
            shapes.push(Shape::Circle(center, 2.0, Role::Acrylic));
        }
        editing::set_overlay(ui, "Scene/Overlay", shapes);
        self.sync(ui, document, models, scene);
    }

    fn events(&mut self, ui: &mut Ui, view: Rect, viewport: &Viewport, document: &mut Document, models: &Models, scene: usize) {
        let middle = editing::visible(view).center();
        let center = viewport.eye.to_ground([middle.x, middle.y], window_size(ui));
        let model_names = models.names();
        let events: Vec<user_interface::UiEvent> = ui.events().to_vec();
        for event in events {
            let current = &mut document.scenes[scene];
            match (event.source.as_str(), &event.kind) {
                ("Scene/Background", UiEventKind::ColorChanged(color)) => {
                    let [red, green, blue, _] = color.to_bytes();
                    current.background = [red, green, blue];
                }
                ("Scene/FollowHeight", UiEventKind::Changed(value)) => current.follow_height = HEIGHTS.nearest(*value),
                ("Scene/Camera", UiEventKind::Clicked) => {
                    current.camera_center = center;
                    current.camera_height = viewport.eye.height;
                }
                ("Scene/Objects", UiEventKind::Selected(row)) => {
                    self.picked = Some(if *row < current.decorations.len() {
                        Picked::Decoration(*row)
                    } else {
                        Picked::Parked(*row - current.decorations.len())
                    });
                }
                ("Scene/Model", UiEventKind::Selected(index)) => self.model = *index,
                ("Scene/Vehicle", UiEventKind::Selected(index)) => self.vehicle = *index,
                ("Scene/AddDecoration", UiEventKind::Clicked) => {
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
                ("Scene/Park", UiEventKind::Clicked) => {
                    let names: Vec<String> = document.vehicles.list.iter().map(|vehicle| vehicle.name.clone()).collect();
                    let current = &mut document.scenes[scene];
                    current.parked.push(Parked {
                        vehicle: names[self.vehicle.min(names.len() - 1)].clone(),
                        pose: Pose {
                            position: center,
                            heading: 0.0,
                        },
                    });
                    self.picked = Some(Picked::Parked(current.parked.len() - 1));
                }
                ("Scene/Delete", UiEventKind::Clicked) => {
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
                ("Scene/Heading", UiEventKind::Changed(value)) => match self.picked {
                    Some(Picked::Decoration(index)) if index < current.decorations.len() => {
                        current.decorations[index].pose.heading = DEGREES.nearest(*value);
                    }
                    Some(Picked::Parked(index)) if index < current.parked.len() => {
                        current.parked[index].pose.heading = DEGREES.nearest(*value);
                    }
                    _ => {}
                },
                ("Scene/Scale", UiEventKind::Changed(value)) => {
                    if let Some(Picked::Decoration(index)) = self.picked
                        && index < current.decorations.len()
                    {
                        current.decorations[index].scale = SCALES.nearest(*value);
                    }
                }
                _ => {}
            }
        }
        // The picked decoration's model's paint.
        if let Some(Picked::Decoration(index)) = self.picked
            && let Some(model) = document.scenes[scene].decorations.get(index).and_then(|decoration| models.find(&decoration.model))
        {
            let name = models.list[model].name.clone();
            let slots = models.list[model].slots.clone();
            self.slot = self.slot.min(slots.len() - 1);
            let mut parameters = document.model_materials(models, model)[self.slot];
            if PaintControls::changed(ui, "ModelPaint", &mut self.slot, &mut parameters) {
                let slot = slots[self.slot.min(slots.len() - 1)].clone();
                set_slot(document.paint_mut(&name), &slots, &slot, parameters);
            }
        }
    }

    // Keeps the card's controls true to the document.
    fn sync(&mut self, ui: &mut Ui, document: &Document, models: &Models, scene: usize) {
        let current = &document.scenes[scene];
        editing::sync_color_field(ui, "Scene/Background", current.background);
        editing::sync_slider(ui, "Scene/FollowHeight", current.follow_height);
        let rows: Vec<String> = current
            .decorations
            .iter()
            .map(|decoration| decoration.model.clone())
            .chain(current.parked.iter().map(|parked| format!("{}, parked", parked.vehicle)))
            .collect();
        let list = ui.get_mut::<ListView>("Scene/Objects");
        let changed = list.rows().len() != rows.len() || list.rows().iter().zip(&rows).any(|(a, b)| a.text != *b);
        if changed {
            list.set_rows(rows.iter().map(|text| ListRow::new(text)).collect());
        }
        list.set_selected(self.picked_row(current));
        // The picked thing's controls, once a rebuild has made them.
        match self.picked {
            Some(Picked::Decoration(index)) if index < current.decorations.len() && ui.has_node("Scene/Scale") => {
                let decoration = &current.decorations[index];
                editing::sync_slider(ui, "Scene/Heading", decoration.pose.heading);
                editing::sync_slider(ui, "Scene/Scale", decoration.scale);
                if let Some(model) = models.find(&decoration.model)
                    && ui.has_node("ModelPaint/Slot")
                {
                    let slots = &models.list[model].slots;
                    let slot = self.slot.min(slots.len() - 1);
                    let parameters = document.model_materials(models, model)[slot];
                    PaintControls::sync(ui, "ModelPaint", slots, slot, &parameters);
                }
            }
            Some(Picked::Parked(index)) if index < current.parked.len() && ui.has_node("Scene/Heading") && !ui.has_node("Scene/Scale") => {
                editing::sync_slider(ui, "Scene/Heading", current.parked[index].pose.heading);
            }
            _ => {}
        }
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
