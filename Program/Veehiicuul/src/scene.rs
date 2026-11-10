// A scene: what a level of the game is, from its file in Scenes. It has the
// terrain, the track, the decorations, which are models, and vehicles parked
// as decorations, all on the ground, with the background's color where
// there's nothing, and the camera's view of it all.
//
//     {
//         "Background": "#1CDA61",
//         "Camera": { "Center": [-3.5, -15.3], "Height": 150 },
//         "FollowHeight": 45,
//         "Terrain": { ... },
//         "Track": { ... },
//         "Decorations": [{ "Model": "BigCone", "Position": [-53.4, -62.2], "Heading": 28, "Scale": 4.9 }],
//         "Vehicles": [{ "Vehicle": "Green", "Position": [37.6, -75.5], "Heading": -32.1 }]
//     }
//
// See terrain.rs and track.rs for theirs.

use std::path::Path;

use json::Value;

use crate::files::{self, Color};
use crate::terrain::Terrain;
use crate::track::{Pose, Track};

// A model put somewhere, turned, and scaled the same along every axis.
#[derive(Clone, Debug, PartialEq)]
pub struct Decoration {
    pub model: String,
    pub pose: Pose,
    pub scale: f64,
}

// A vehicle put somewhere, as a decoration, which doesn't move.
#[derive(Clone, Debug, PartialEq)]
pub struct Parked {
    pub vehicle: String,
    pub pose: Pose,
}

// The scale that decorations stay within, for SimplePaint.
pub const SMALLEST_SCALE: f64 = 0.01;
pub const LARGEST_SCALE: f64 = 100.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    // Its file's name, without ".json".
    pub name: String,
    pub background: Color,
    // The fixed camera looks at `camera_center`, at a view `camera_height`
    // units tall, and the camera that follows the car at one
    // `follow_height` tall.
    pub camera_center: [f64; 2],
    pub camera_height: f64,
    pub follow_height: f64,
    pub terrain: Terrain,
    pub track: Track,
    pub decorations: Vec<Decoration>,
    pub parked: Vec<Parked>,
}

const MEMBERS: [&str; 7] = ["Background", "Camera", "FollowHeight", "Terrain", "Track", "Decorations", "Vehicles"];

impl Scene {
    // A new scene, with a round track, called `name`.
    pub fn new(name: &str) -> Scene {
        Scene {
            name: name.to_string(),
            background: [0x1C, 0xDA, 0x61],
            camera_center: [0.0, 0.0],
            camera_height: 100.0,
            follow_height: 45.0,
            terrain: Terrain::default(),
            track: Track::round(),
            decorations: Vec::new(),
            parked: Vec::new(),
        }
    }

    pub fn parse(name: &str, text: &str) -> Result<Scene, String> {
        let value = files::parse(text)?;
        files::only(&value, &MEMBERS, "a scene")?;
        let member = |name: &str| files::member(&value, name, "");
        let camera = member("Camera")?;
        files::only(camera, &["Center", "Height"], "Camera")?;
        let decorations = match member("Decorations")? {
            Value::Array(items) => items
                .iter()
                .enumerate()
                .map(|(index, item)| decoration(item, &format!("Decorations[{index}]")))
                .collect::<Result<Vec<_>, _>>()?,
            other => return Err(format!("Decorations must be a list, not {}", other.describe())),
        };
        let parked = match member("Vehicles")? {
            Value::Array(items) => items
                .iter()
                .enumerate()
                .map(|(index, item)| parked(item, &format!("Vehicles[{index}]")))
                .collect::<Result<Vec<_>, _>>()?,
            other => return Err(format!("Vehicles must be a list, not {}", other.describe())),
        };
        Ok(Scene {
            name: name.to_string(),
            background: files::color(member("Background")?, "Background")?,
            camera_center: files::point(files::member(camera, "Center", "Camera.")?, "Camera.Center")?,
            camera_height: files::positive(files::member(camera, "Height", "Camera.")?, "Camera.Height")?,
            follow_height: files::positive(member("FollowHeight")?, "FollowHeight")?,
            terrain: Terrain::parse(member("Terrain")?)?,
            track: Track::parse(member("Track")?)?,
            decorations,
            parked,
        })
    }

    pub fn to_json(&self) -> String {
        let decorations = self
            .decorations
            .iter()
            .map(|decoration| {
                json::object([
                    ("Model", decoration.model.clone().into()),
                    ("Position", files::point_value(decoration.pose.position)),
                    ("Heading", files::rounded(decoration.pose.heading)),
                    ("Scale", files::rounded(decoration.scale)),
                ])
            })
            .collect();
        let parked = self
            .parked
            .iter()
            .map(|parked| {
                json::object([
                    ("Vehicle", parked.vehicle.clone().into()),
                    ("Position", files::point_value(parked.pose.position)),
                    ("Heading", files::rounded(parked.pose.heading)),
                ])
            })
            .collect();
        json::object([
            ("Background", files::color_text(self.background).into()),
            (
                "Camera",
                json::object([
                    ("Center", files::point_value(self.camera_center)),
                    ("Height", files::rounded(self.camera_height)),
                ]),
            ),
            ("FollowHeight", files::rounded(self.follow_height)),
            ("Terrain", self.terrain.to_value()),
            ("Track", self.track.to_value()),
            ("Decorations", Value::Array(decorations)),
            ("Vehicles", Value::Array(parked)),
        ])
        .to_json()
            + "\n"
    }

    pub fn load(path: &Path) -> Result<Scene, String> {
        let name = path.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
        Scene::parse(&name, &files::read(path)?).map_err(|problem| format!("{}: {problem}", path.display()))
    }

    // Every scene in a folder, from its .json files, by name.
    pub fn load_all(folder: &Path) -> Result<Vec<Scene>, String> {
        let entries = std::fs::read_dir(folder).map_err(|error| format!("Couldn't read {}: {error}", folder.display()))?;
        let mut paths: Vec<_> = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("json")))
            .collect();
        paths.sort();
        if paths.is_empty() {
            return Err(format!("{} has no scenes", folder.display()));
        }
        paths.iter().map(|path| Scene::load(path)).collect()
    }
}

fn pose(value: &Value, name: &str) -> Result<Pose, String> {
    Ok(Pose {
        position: files::point(files::member(value, "Position", &format!("{name}."))?, &format!("{name}.Position"))?,
        heading: files::number(files::member(value, "Heading", &format!("{name}."))?, &format!("{name}.Heading"))?,
    })
}

fn decoration(value: &Value, name: &str) -> Result<Decoration, String> {
    files::only(value, &["Model", "Position", "Heading", "Scale"], name)?;
    let scale = files::positive(files::member(value, "Scale", &format!("{name}."))?, &format!("{name}.Scale"))?;
    if !(SMALLEST_SCALE..=LARGEST_SCALE).contains(&scale) {
        return Err(format!("{name}.Scale must be from {SMALLEST_SCALE} to {LARGEST_SCALE}, not {scale}"));
    }
    Ok(Decoration {
        model: files::text(files::member(value, "Model", &format!("{name}."))?, &format!("{name}.Model"))?,
        pose: pose(value, name)?,
        scale,
    })
}

fn parked(value: &Value, name: &str) -> Result<Parked, String> {
    files::only(value, &["Vehicle", "Position", "Heading"], name)?;
    Ok(Parked {
        vehicle: files::text(files::member(value, "Vehicle", &format!("{name}."))?, &format!("{name}.Vehicle"))?,
        pose: pose(value, name)?,
    })
}
