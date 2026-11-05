// The game's own settings, after the ones every program has (see
// Library/UserInterface's README), in Settings.json next to Cargo.toml: the
// scene and the vehicle to start with, the camera, and the push's dead
// zones. Each one that's wrong keeps its default, and a notice under the app
// bar says what's wrong with it.

use json::Value;
use user_interface::Settings;

use crate::driving::{DeadZone, DeadZones};

#[derive(Clone, Debug, PartialEq)]
pub struct GameSettings {
    // The names of the scene and the vehicle to start with.
    pub scene: String,
    pub vehicle: String,
    // Whether the camera starts following the car, instead of fixed.
    pub follow: bool,
    pub dead_zones: DeadZones,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            scene: "Ribeye".to_string(),
            vehicle: "Yellow".to_string(),
            follow: false,
            dead_zones: DeadZones::default(),
        }
    }
}

impl GameSettings {
    // Reads the game's settings, in this order, which a missing
    // Settings.json is written in.
    pub fn read(settings: &mut Settings) -> GameSettings {
        let defaults = GameSettings::default();
        let scene = settings.value("Scene", Value::from(defaults.scene.as_str()), name);
        let vehicle = settings.value("Vehicle", Value::from(defaults.vehicle.as_str()), name);
        let follow = settings.choice("Camera", "Fixed", &["Fixed", "Follow"]) == "Follow";
        let dead_zones = settings.value("DeadZones", dead_zones_value(&defaults.dead_zones), dead_zones);
        GameSettings {
            scene,
            vehicle,
            follow,
            dead_zones,
        }
    }
}

fn name(value: &Value) -> Result<String, String> {
    match value {
        Value::String(name) if !name.trim().is_empty() => Ok(name.clone()),
        other => Err(format!("must be a name, not {}", other.describe())),
    }
}

fn dead_zones_value(dead_zones: &DeadZones) -> Value {
    let zone = |zone: DeadZone| Value::from([zone.inner, zone.outer]);
    json::object([
        ("Sideways", zone(dead_zones.sideways)),
        ("ForwardAndBack", zone(dead_zones.forward_and_back)),
    ])
}

// Like {"Sideways": [0.1, 0.95], "ForwardAndBack": [0.05, 0.95]}. A member
// that's left out keeps its default, and one that's wrong makes both keep
// theirs.
fn dead_zones(value: &Value) -> Result<DeadZones, String> {
    let Value::Object(members) = value else {
        return Err(format!("must be an object with Sideways and ForwardAndBack, not {}", value.describe()));
    };
    let mut dead_zones = DeadZones::default();
    for (member, value) in members {
        let zone = match member.as_str() {
            "Sideways" => &mut dead_zones.sideways,
            "ForwardAndBack" => &mut dead_zones.forward_and_back,
            _ => return Err(format!("has {member}, which isn't a dead zone. They're Sideways and ForwardAndBack")),
        };
        *zone = match value {
            Value::Array(items) => match items.as_slice() {
                [Value::Number(inner), Value::Number(outer)] if 0.0 <= *inner && inner < outer && *outer <= 1.0 => DeadZone {
                    inner: *inner,
                    outer: *outer,
                },
                _ => return Err(dead_zone_problem(member, value)),
            },
            _ => return Err(dead_zone_problem(member, value)),
        };
    }
    Ok(dead_zones)
}

fn dead_zone_problem(member: &str, value: &Value) -> String {
    format!(
        "has {member}, which must be [inner, outer], from 0 to 1 with inner below outer, not {}",
        value.describe()
    )
}
