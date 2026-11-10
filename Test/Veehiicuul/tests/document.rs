// What the game's files are: the models from Blender, and the document of
// scenes, vehicles and paints, which read and write back the same, and the
// game's own settings in Settings.json.

use std::path::{Path, PathBuf};

use simple_paint::Material;
use user_interface::Settings;
use veehiicuul::document::Document;
use veehiicuul::driving::{Acceleration, DeadZone};
use veehiicuul::files;
use veehiicuul::models::{Models, paint_file, paint_from_color};
use veehiicuul::settings::GameSettings;
use veehiicuul::vehicles::Vehicles;

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Program/Veehiicuul")
}

fn load() -> (Models, Document) {
    let models = Models::load(&folder().join("Models")).unwrap_or_else(|problem| panic!("{problem}"));
    let document = Document::load(&folder(), &models).unwrap_or_else(|problem| panic!("{problem}"));
    (models, document)
}

// The models are the .glb files from Blender, each with its slots, which
// are its materials, painted as Blender says until the game paints them.
#[test]
fn models_come_from_blender_with_their_slots() {
    let (models, document) = load();
    assert_eq!(models.names(), ["BigCone", "SlopeCar"]);
    let car = &models.list[1];
    assert_eq!(car.slots, ["SlopeCarAxles", "SlopeCarBodyBlue", "SlopeCarCabinBlue", "SlopeCarHeadlights", "SlopeCarWheels"]);
    assert!((car.footprint.min[0] + 1.5).abs() < 1e-5 && (car.footprint.min[1] + 3.0).abs() < 1e-5);
    // Blender's paint is from each material's color.
    let cone = &models.list[0];
    let orange = cone.slots.iter().position(|slot| slot == "BigConeOrange").unwrap();
    assert_eq!(cone.blender_paints[orange], paint_from_color([1.0, 0.268_714_8, 0.0, 1.0]));
    // The car's paint file paints its slots, and the cone has none.
    let materials = document.model_materials(&models, 1);
    assert_eq!(materials[0].base_color_srgb, [0.2159, 0.2159, 0.2159]);
    assert_eq!(document.model_materials(&models, 0), cone.blender_paints);
    for parameters in materials {
        Material::compile(&parameters).unwrap();
    }
}

// Six vehicles, each faster than the one before, each painted its own way
// over the car, with the footprint that touches the track's edges.
#[test]
fn the_vehicles_each_look_and_drive_their_own_way() {
    let (models, document) = load();
    let vehicles = &document.vehicles;
    let names: Vec<&str> = vehicles.list.iter().map(|vehicle| vehicle.name.as_str()).collect();
    assert_eq!(names, ["Green", "Blue", "Red", "Yellow", "Cyan", "Magenta"]);
    assert_eq!(vehicles.list[3].acceleration, Acceleration { forward: 140.0, reverse: 280.0, left: 140.0, right: 140.0 });
    for pair in vehicles.list.windows(2) {
        assert!(pair[0].acceleration.forward < pair[1].acceleration.forward);
    }
    let mut bodies = Vec::new();
    for index in 0..vehicles.list.len() {
        let (model, materials) = document.vehicle_materials(&models, index).unwrap();
        assert_eq!(models.list[model].name, "SlopeCar");
        assert!(!bodies.contains(&materials[1].base_color_srgb), "{} has another's body", vehicles.list[index].name);
        bodies.push(materials[1].base_color_srgb);
        // The vehicle paints its body and cabin, and the car paints the rest.
        assert_eq!(materials[0], document.model_materials(&models, model)[0]);
    }
    let yellow = &vehicles.list[3].footprint;
    assert_eq!((yellow.min, yellow.max), ([-1.5, -3.0], [1.5, 2.9925]));
}

// The files read and write back the same, so that saving what didn't change
// changes nothing, and a change writes only the file that it's in.
#[test]
fn the_files_write_back_as_they_were_and_only_what_changed() {
    let (_, document) = load();
    for scene in &document.scenes {
        let text = files::read(&folder().join(format!("Scenes/{}.json", scene.name))).unwrap();
        assert_eq!(scene.to_json(), text, "Scenes/{}.json", scene.name);
    }
    assert_eq!(document.vehicles.to_json(), files::read(&folder().join("Vehicles.json")).unwrap());
    assert_eq!(paint_file(document.paint("SlopeCar")), files::read(&folder().join("Models/SlopeCar.json")).unwrap());
    assert!(document.changed_files(&document).is_empty());
    let mut changed = document.clone();
    changed.scenes[0].track.barrier_width = 2.0;
    changed.vehicles.list[0].acceleration.forward = 11.0;
    let paths: Vec<String> = changed.changed_files(&document).into_iter().map(|(path, _)| path).collect();
    assert_eq!(paths, ["Scenes/Ribeye.json", "Vehicles.json"]);
}

#[test]
fn says_what_a_document_names_that_is_not_there() {
    let (models, mut document) = load();
    assert!(document.missing(&models).is_empty());
    document.vehicles.list[0].model = "Boat".to_string();
    document.scenes[0].parked[1].vehicle = "Truck".to_string();
    assert_eq!(
        document.missing(&models),
        [
            "Vehicle Green's model, Boat, isn't in Models",
            "Scene Ribeye parks a Truck, which isn't in Vehicles.json"
        ]
    );
}

#[test]
fn says_what_is_wrong_with_the_vehicles_file() {
    let vehicle = |name: &str, footprint: &str| {
        format!(
            r#"{{ "Name": "{name}", "Model": "SlopeCar", "Acceleration": {{ "Forward": 1, "Reverse": 2, "Left": 3, "Right": 4 }}, "Footprint": {footprint} }}"#
        )
    };
    let footprint = r#"{ "Left": 1, "Right": 1, "Front": 2, "Back": 2 }"#;
    let file = |vehicles: &str| format!(r#"{{ "Vehicles": [{vehicles}] }}"#);
    let parsed = Vehicles::parse(&file(&vehicle("Gray", footprint))).unwrap();
    assert!(parsed.list[0].paint.is_empty());
    for (text, problem) in [
        (file(""), "Vehicles needs at least one vehicle"),
        (file(&format!("{}, {}", vehicle("Gray", footprint), vehicle("Gray", footprint))), "Vehicles[1]: another vehicle is called Gray"),
        (file(&vehicle("Gray", r#"{ "Left": 1, "Right": 1, "Front": 2 }"#)), "Vehicles[0]: needs Gray's Footprint.Back"),
        (file(&vehicle("Gray", r#"{ "Left": 0, "Right": 1, "Front": 2, "Back": 2 }"#)), "Vehicles[0]: Gray's Footprint.Left must be above 0, not 0"),
    ] {
        match Vehicles::parse(&text) {
            Ok(_) => panic!("{text} parsed"),
            Err(actual) => assert_eq!(actual, problem),
        }
    }
}

// Reads the game's settings from Settings.json's text, with what's wrong.
fn read(text: &str) -> (GameSettings, Vec<String>) {
    let mut settings = Settings::from_text(text);
    let game = GameSettings::read(&mut settings);
    (game, settings.problems().to_vec())
}

#[test]
fn settings_name_the_scene_vehicle_camera_and_dead_zones() {
    let (defaults, problems) = read("{}");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(defaults, GameSettings::default());
    assert_eq!((defaults.scene.as_str(), defaults.vehicle.as_str(), defaults.follow), ("Ribeye", "Yellow", false));
    let (settings, problems) = read(r#"{ "Camera": "follow", "DeadZones": { "Sideways": [0.2, 0.9] } }"#);
    assert!(problems.is_empty(), "{problems:?}");
    assert!(settings.follow);
    assert_eq!(settings.dead_zones.sideways, DeadZone { inner: 0.2, outer: 0.9 });
    assert_eq!(settings.dead_zones.forward_and_back, DeadZone { inner: 0.05, outer: 0.95 });
    // A setting that's wrong keeps its default, and says why.
    for (text, problem) in [
        (r#"{ "Camera": "Above" }"#, "Camera must be \"Fixed\" or \"Follow\", not \"Above\", so it's the default."),
        (r#"{ "Scene": 3 }"#, "Scene must be a name, not 3, so it's the default."),
        (
            r#"{ "DeadZones": { "Sideways": [0.9, 0.2] } }"#,
            "DeadZones has Sideways, which must be [inner, outer], from 0 to 1 with inner below outer, not a list of 2, so it's the default.",
        ),
    ] {
        let (settings, problems) = read(text);
        assert_eq!(problems, [problem]);
        assert_eq!(settings, GameSettings::default());
    }
    // The game's own Settings.json is right.
    let mut settings = Settings::load(&folder().join("Settings.json"));
    let read = GameSettings::read(&mut settings);
    assert!(settings.problems().is_empty(), "{:?}", settings.problems());
    assert_eq!((read.scene.as_str(), read.vehicle.as_str(), read.follow), ("Ribeye", "Yellow", false));
}

// A color's channels are two hex digits each, with no sign or space.
#[test]
fn colors_are_hex_digits_only() {
    assert_eq!(files::parse_color("#1CDA61"), Some([0x1C, 0xDA, 0x61]));
    assert_eq!(files::parse_color("#1cda61"), Some([0x1C, 0xDA, 0x61]));
    for bad in ["#+1+2+3", "#1CDA6", "#1CDA611", "1CDA61 ", "#1CDA6G", "# 1DA61"] {
        assert_eq!(files::parse_color(bad), None, "{bad:?}");
    }
    assert_eq!(files::color_text([0x1C, 0xDA, 0x61]), "#1CDA61");
}
