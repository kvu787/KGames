// The vehicles file, and the game's own settings.

use std::path::Path;

use application::{ProgramSettings, Settings};
use json::Value;
use meshes::car;
use simple_paint::Material;
use veehiicuul::driving::{Acceleration, DeadZone};
use veehiicuul::settings::GameSettings;
use veehiicuul::vehicles::Vehicles;

fn vehicles() -> Vehicles {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Program/Veehiicuul/Vehicles.json");
    Vehicles::load(&path).unwrap_or_else(|problem| panic!("{problem}"))
}

// Six vehicles, each faster than the one before, and each in colors of its
// own, which SimplePaint takes.
#[test]
fn the_vehicles_each_look_and_drive_their_own_way() {
    let vehicles = vehicles();
    let names: Vec<&str> = vehicles.list.iter().map(|vehicle| vehicle.name.as_str()).collect();
    assert_eq!(names, ["Green", "Blue", "Red", "Yellow", "Cyan", "Magenta"]);
    assert_eq!(
        vehicles.list[3].acceleration,
        Acceleration {
            forward: 140.0,
            reverse: 280.0,
            left: 140.0,
            right: 140.0
        }
    );
    for pair in vehicles.list.windows(2) {
        assert!(pair[0].acceleration.forward < pair[1].acceleration.forward);
    }
    for (index, vehicle) in vehicles.list.iter().enumerate() {
        let paints = vehicles.paints(index);
        assert_eq!(paints[car::BODY as usize], vehicle.body);
        assert_eq!(paints[car::CABIN as usize], vehicle.cabin);
        assert_eq!(paints[car::WHEELS as usize], vehicles.wheels);
        for paint in paints {
            Material::compile(&paint).unwrap();
        }
        let others = vehicles.list.iter().filter(|other| other.body.base_color_srgb == vehicle.body.base_color_srgb);
        assert_eq!(others.count(), 1, "{} has another's body color", vehicle.name);
    }
    assert_eq!(vehicles.find("yellow"), Some(3));
    assert_eq!(vehicles.find("Purple"), None);
}

#[test]
fn says_what_is_wrong_with_the_vehicles_file() {
    let paint = r#"{ "BaseColor": [0.5, 0.5, 0.5] }"#;
    let file = |vehicles: &str| {
        format!(r#"{{ "Axles": {paint}, "Headlights": {paint}, "Wheels": {paint}, "Vehicles": [{vehicles}] }}"#)
    };
    let vehicle = |name: &str, acceleration: &str| {
        format!(r#"{{ "Name": "{name}", "Acceleration": {acceleration}, "Body": {paint}, "Cabin": {paint} }}"#)
    };
    let rates = r#"{ "Forward": 1, "Reverse": 2, "Left": 3, "Right": 4 }"#;
    Vehicles::parse(&file(&vehicle("Gray", rates))).unwrap();
    for (text, problem) in [
        (file(""), "Vehicles needs at least one vehicle".to_string()),
        (
            file(&format!("{}, {}", vehicle("Gray", rates), vehicle("Gray", rates))),
            "Vehicles[1]: another vehicle is called Gray".to_string(),
        ),
        (
            file(&vehicle("Gray", r#"{ "Forward": 1, "Reverse": -2, "Left": 3, "Right": 4 }"#)),
            "Vehicles[0]: Gray's Acceleration.Reverse must be 0 or more, not -2".to_string(),
        ),
        (
            file(&vehicle("Gray", r#"{ "Forward": 1, "Left": 3, "Right": 4 }"#)),
            "Vehicles[0]: Gray needs Acceleration.Reverse".to_string(),
        ),
        (
            file(&vehicle("Gray", rates)).replace("0.5, 0.5, 0.5] }, \"Vehicles\"", "0.5, 0.5, 2] }, \"Vehicles\""),
            "Wheels SimplePaint B must be finite and in [0.0009765625, 0.9990234375].".to_string(),
        ),
    ] {
        match Vehicles::parse(&text) {
            Ok(_) => panic!("{text} parsed"),
            Err(actual) => assert_eq!(actual, problem),
        }
    }
}

#[test]
fn settings_name_the_track_vehicle_camera_and_dead_zones() {
    let (_, mut settings) = GameSettings::defaults();
    assert_eq!((settings.track.as_str(), settings.vehicle.as_str(), settings.follow), ("Ribeye", "Yellow", false));
    settings.apply("Camera", &"follow".into()).unwrap();
    assert!(settings.follow);
    let zones = json::parse(r#"{ "Sideways": [0.2, 0.9] }"#).unwrap();
    settings.apply("DeadZones", &zones).unwrap();
    assert_eq!(settings.dead_zones.sideways, DeadZone { inner: 0.2, outer: 0.9 });
    assert_eq!(settings.dead_zones.forward_and_back, DeadZone { inner: 0.05, outer: 0.95 });
    for (name, value, problem) in [
        ("Camera", "\"Above\"", "must be \"Fixed\" or \"Follow\", not \"Above\""),
        ("Vehicle", "3", "must be a name, not 3"),
        ("DeadZones", r#"{ "Sideways": [0.9, 0.2] }"#, "Sideways must be [inner, outer], from 0 to 1 with inner below outer, not a list of 2"),
        ("DeadZones", r#"{ "Up": [0.1, 0.9] }"#, "Up isn't a dead zone. They're Sideways and ForwardAndBack."),
    ] {
        let value: Value = json::parse(value).unwrap();
        assert_eq!(settings.apply(name, &value), Err(problem.to_string()));
    }
    // The settings file in the game's folder has the defaults.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Program/Veehiicuul/Settings.json");
    let mut notices = Vec::new();
    let (_, read): (Settings, GameSettings) = Settings::load(&path, &mut notices);
    assert!(notices.is_empty(), "{notices:?}");
    assert_eq!((read.track, read.vehicle, read.follow), ("Ribeye".to_string(), "Yellow".to_string(), false));
}
