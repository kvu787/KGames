// Everything that the editors change, and Save writes: the scenes, the
// vehicles, and the models' paints. The models' meshes come from Blender,
// so they're not part of it.

use std::path::Path;

use simple_paint::Parameters;

use crate::files;
use crate::models::{Models, Paint, paint_file, parse_paint_file};
use crate::scene::Scene;
use crate::vehicles::Vehicles;

#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub scenes: Vec<Scene>,
    pub vehicles: Vehicles,
    // Each model's paint, by its name, in the models' order.
    pub paints: Vec<(String, Paint)>,
}

impl Document {
    // The document in the game's folder: Scenes, Vehicles.json, and each
    // model's paint file in Models, if it has one.
    pub fn load(folder: &Path, models: &Models) -> Result<Document, String> {
        let scenes = Scene::load_all(&folder.join("Scenes"))?;
        let path = folder.join("Vehicles.json");
        let vehicles = Vehicles::parse(&files::read(&path)?).map_err(|problem| format!("{}: {problem}", path.display()))?;
        let mut paints = Vec::new();
        for model in &models.list {
            let path = folder.join("Models").join(format!("{}.json", model.name));
            let paint = if path.is_file() {
                parse_paint_file(&files::read(&path)?).map_err(|problem| format!("{}: {problem}", path.display()))?
            } else {
                Vec::new()
            };
            paints.push((model.name.clone(), paint));
        }
        let document = Document {
            scenes,
            vehicles,
            paints,
        };
        match document.missing(models).first() {
            Some(problem) => Err(problem.clone()),
            None => Ok(document),
        }
    }

    // What the document names that isn't there: models of vehicles and
    // decorations, and parked vehicles.
    pub fn missing(&self, models: &Models) -> Vec<String> {
        let mut problems = Vec::new();
        for vehicle in &self.vehicles.list {
            if models.find(&vehicle.model).is_none() {
                problems.push(format!("Vehicle {}'s model, {}, isn't in Models", vehicle.name, vehicle.model));
            }
        }
        for scene in &self.scenes {
            for decoration in &scene.decorations {
                if models.find(&decoration.model).is_none() {
                    problems.push(format!("Scene {} has a {}, which isn't in Models", scene.name, decoration.model));
                }
            }
            for parked in &scene.parked {
                if self.vehicles.find(&parked.vehicle).is_none() {
                    problems.push(format!("Scene {} parks a {}, which isn't in Vehicles.json", scene.name, parked.vehicle));
                }
            }
        }
        problems
    }

    pub fn paint(&self, model: &str) -> &Paint {
        static NONE: Paint = Vec::new();
        self.paints.iter().find(|(name, _)| name == model).map_or(&NONE, |(_, paint)| paint)
    }

    pub fn paint_mut(&mut self, model: &str) -> &mut Paint {
        if let Some(index) = self.paints.iter().position(|(name, _)| name == model) {
            return &mut self.paints[index].1;
        }
        self.paints.push((model.to_string(), Vec::new()));
        &mut self.paints.last_mut().unwrap().1
    }

    // A model's slots' materials, with its paint.
    pub fn model_materials(&self, models: &Models, model: usize) -> Vec<Parameters> {
        let model = &models.list[model];
        model.materials(&[self.paint(&model.name)])
    }

    // A vehicle's model, and its slots' materials, with its model's paint
    // and its own.
    pub fn vehicle_materials(&self, models: &Models, vehicle: usize) -> Option<(usize, Vec<Parameters>)> {
        let vehicle = self.vehicles.list.get(vehicle)?;
        let index = models.find(&vehicle.model)?;
        let model = &models.list[index];
        Some((index, model.materials(&[self.paint(&model.name), &vehicle.paint])))
    }

    // The files that differ from what's in `saved`, each as its path in the
    // game's folder and its text.
    pub fn changed_files(&self, saved: &Document) -> Vec<(String, String)> {
        let mut files = Vec::new();
        for scene in &self.scenes {
            if saved.scenes.iter().find(|other| other.name == scene.name) != Some(scene) {
                files.push((format!("Scenes/{}.json", scene.name), scene.to_json()));
            }
        }
        if self.vehicles != saved.vehicles {
            files.push(("Vehicles.json".to_string(), self.vehicles.to_json()));
        }
        for (model, paint) in &self.paints {
            if saved.paint(model) != paint {
                files.push((format!("Models/{model}.json"), paint_file(paint)));
            }
        }
        files
    }

    // The files that save should delete: scenes in `saved` that the document
    // no longer has, like after undoing a new scene.
    pub fn removed_files(&self, saved: &Document) -> Vec<String> {
        saved
            .scenes
            .iter()
            .filter(|scene| !self.scenes.iter().any(|kept| kept.name == scene.name))
            .map(|scene| format!("Scenes/{}.json", scene.name))
            .collect()
    }
}
