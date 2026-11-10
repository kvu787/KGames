// The models that vehicles and decorations are made of: the .glb files in
// Models, which Blender exports, with Library/Gltf. A model's materials are
// its slots, each painted with a SimplePaint material. What Blender says a
// slot looks like is its first paint: the SimplePaint material in its
// custom property "SimplePaint", in Library/PaintSettings' format, if it has
// one, or else one with its base color. The game can paint over that, for
// the model, in Models/<Name>.json, and for a vehicle, in Vehicles.json.
//
//     {
//         "Paint": {
//             "SlopeCarAxles": { "BaseColor": [0.2159, 0.2159, 0.2159], ... }
//         }
//     }

use std::path::Path;

use json::Value;
use simple_paint::{Parameters, Vertex, validate_mesh};

use crate::collision::Footprint;
use crate::files;

// The most slots a model has, as SIMPLE_PAINT_MATERIAL_COUNT in build.rs
// says.
pub const MOST_SLOTS: usize = 16;

pub struct Model {
    pub name: String,
    pub slots: Vec<String>,
    // Each slot's paint, as Blender says.
    pub blender_paints: Vec<Parameters>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    // The rectangle around it seen from above.
    pub footprint: Footprint,
}

// A model's paint: some of its slots' SimplePaint materials, by the slots'
// names. The slots it leaves out keep their paint from before.
pub type Paint = Vec<(String, Parameters)>;

// The smallest and largest of SimplePaint's color channels.
const LOWEST: f64 = 1.0 / 1024.0;
const HIGHEST: f64 = 1.0 - 1.0 / 1024.0;

// A slot's paint from its base color, which glTF says as linear RGB: a
// little shaded, with the color itself where the surface faces sideways.
pub fn paint_from_color([red, green, blue, _]: [f32; 4]) -> Parameters {
    let srgb = |channel: f32| {
        let channel = channel.clamp(0.0, 1.0) as f64;
        let encoded = if channel <= 0.003_130_8 {
            channel * 12.92
        } else {
            1.055 * channel.powf(1.0 / 2.4) - 0.055
        };
        ((encoded * 10_000.0).round() / 10_000.0).clamp(LOWEST, HIGHEST)
    };
    Parameters {
        base_color_srgb: [srgb(red), srgb(green), srgb(blue)],
        brightness: 0.5,
        shift: 0.25,
        rotation_degrees: 35.0,
        dark_point: 0.0,
        light_point: 1.0,
    }
}

impl Model {
    pub fn read(name: &str, model: &gltf::Model) -> Result<Model, String> {
        if model.material_list.len() > MOST_SLOTS {
            return Err(format!("has {} materials, and a model has {MOST_SLOTS} at most", model.material_list.len()));
        }
        let mut blender_paints = Vec::new();
        for material in &model.material_list {
            let from_color = paint_from_color(material.base_color);
            let paint = match material.extras.as_ref().and_then(|extras| extras.get("SimplePaint")) {
                Some(paint) => paint_settings::parse(paint, &from_color)
                    .map_err(|problem| format!("material {}'s SimplePaint {problem}", material.name))?,
                None => from_color,
            };
            blender_paints.push(paint);
        }
        let vertices: Vec<Vertex> = (0..model.positions.len())
            .map(|index| Vertex {
                position: model.positions[index],
                normal: model.normals[index],
                material_index: model.materials[index],
            })
            .collect();
        validate_mesh(&vertices, &model.indices, model.material_list.len() as u32).map_err(|error| error.0)?;
        Ok(Model {
            name: name.to_string(),
            slots: model.material_list.iter().map(|material| material.name.clone()).collect(),
            blender_paints,
            vertices,
            indices: model.indices.clone(),
            footprint: Footprint::of_model(&model.positions),
        })
    }

    // Its slots' materials, with these paints over Blender's, each over
    // the ones before it.
    pub fn materials(&self, paints: &[&Paint]) -> Vec<Parameters> {
        let mut materials = self.blender_paints.clone();
        for paint in paints {
            for (slot, parameters) in paint.iter() {
                if let Some(index) = self.slots.iter().position(|name| name == slot) {
                    materials[index] = *parameters;
                }
            }
        }
        materials
    }
}

pub struct Models {
    pub list: Vec<Model>,
}

impl Models {
    // Every .glb file in a folder, by name.
    pub fn load(folder: &Path) -> Result<Models, String> {
        let entries = std::fs::read_dir(folder).map_err(|error| format!("Couldn't read {}: {error}", folder.display()))?;
        let mut paths: Vec<_> = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("glb")))
            .collect();
        paths.sort();
        let mut list = Vec::new();
        for path in paths {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            let model = gltf::load(&path)?;
            list.push(Model::read(&name, &model).map_err(|problem| format!("{}: {problem}", path.display()))?);
        }
        if list.is_empty() {
            return Err(format!("{} has no models", folder.display()));
        }
        Ok(Models { list })
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.list.iter().position(|model| model.name == name)
    }

    pub fn names(&self) -> Vec<&str> {
        self.list.iter().map(|model| model.name.as_str()).collect()
    }
}

// A paint, from its object of slots' materials.
pub fn parse_paint(value: &Value, name: &str) -> Result<Paint, String> {
    let Value::Object(members) = value else {
        return Err(format!("{name} must be an object of materials by their slots' names, not {}", value.describe()));
    };
    members
        .iter()
        .map(|(slot, value)| {
            paint_settings::parse(value, &paint_from_color([0.5, 0.5, 0.5, 1.0]))
                .map(|parameters| (slot.clone(), parameters))
                .map_err(|problem| format!("{name}.{slot} {problem}"))
        })
        .collect()
}

pub fn paint_value(paint: &Paint) -> Value {
    Value::Object(
        paint
            .iter()
            .map(|(slot, parameters)| (slot.clone(), paint_settings::to_value(parameters)))
            .collect(),
    )
}

// A model's paint file's text, like Models/SlopeCar.json.
pub fn paint_file(paint: &Paint) -> String {
    json::object([("Paint", paint_value(paint))]).to_json() + "\n"
}

pub fn parse_paint_file(text: &str) -> Result<Paint, String> {
    let value = files::parse(text)?;
    files::only(&value, &["Paint"], "a model's paint file")?;
    parse_paint(files::member(&value, "Paint", "")?, "Paint")
}

// Sets a slot's material in a paint, in the slots' order.
pub fn set_slot(paint: &mut Paint, slots: &[String], slot: &str, parameters: Parameters) {
    match paint.iter_mut().find(|(name, _)| name == slot) {
        Some((_, existing)) => *existing = parameters,
        None => {
            paint.push((slot.to_string(), parameters));
            paint.sort_by_key(|(name, _)| slots.iter().position(|other| other == name).unwrap_or(usize::MAX));
        }
    }
}
