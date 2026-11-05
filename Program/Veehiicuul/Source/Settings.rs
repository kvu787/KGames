// The game's own settings, after the ones every program has (see
// Library/Application), in Settings.json next to Cargo.toml: the track and
// the vehicle to start with, the camera, and the stick's dead zones.

use application::{ProgramSettings, Settings};
use json::Value;

use crate::driving::{DeadZone, DeadZones};

pub struct GameSettings {
    // The names of the track and the vehicle to start with.
    pub track: String,
    pub vehicle: String,
    // Whether the camera starts following the car, instead of fixed.
    pub follow: bool,
    pub dead_zones: DeadZones,
}

impl ProgramSettings for GameSettings {
    const NAMES: &'static [&'static str] = &["Track", "Vehicle", "Camera", "DeadZones"];

    fn defaults() -> (Settings, Self) {
        (
            Settings::default(),
            GameSettings {
                track: "Ribeye".to_string(),
                vehicle: "Yellow".to_string(),
                follow: false,
                dead_zones: DeadZones::default(),
            },
        )
    }

    fn apply(&mut self, name: &str, value: &Value) -> Result<(), String> {
        match (name, value) {
            ("Track", Value::String(track)) => self.track = track.clone(),
            ("Vehicle", Value::String(vehicle)) => self.vehicle = vehicle.clone(),
            ("Track" | "Vehicle", other) => return Err(format!("must be a name, not {}", other.describe())),
            ("Camera", Value::String(camera)) if camera.eq_ignore_ascii_case("Fixed") => self.follow = false,
            ("Camera", Value::String(camera)) if camera.eq_ignore_ascii_case("Follow") => self.follow = true,
            ("Camera", other) => return Err(format!("must be \"Fixed\" or \"Follow\", not {}", other.describe())),
            _ => self.dead_zones = dead_zones(value)?,
        }
        Ok(())
    }

    fn default_lines(&self) -> Vec<String> {
        let zone = |zone: DeadZone| format!("[{:?}, {:?}]", zone.inner, zone.outer);
        vec![
            format!("\"Track\": \"{}\"", self.track),
            format!("\"Vehicle\": \"{}\"", self.vehicle),
            format!("\"Camera\": \"{}\"", if self.follow { "Follow" } else { "Fixed" }),
            format!(
                "\"DeadZones\": {{\n        \"Sideways\": {},\n        \"ForwardAndBack\": {}\n    }}",
                zone(self.dead_zones.sideways),
                zone(self.dead_zones.forward_and_back)
            ),
        ]
    }
}

// Like {"Sideways": [0.1, 0.95], "ForwardAndBack": [0.05, 0.95]}. Applied
// only if every member is right, so that a wrong one keeps both as they were.
fn dead_zones(value: &Value) -> Result<DeadZones, String> {
    let Value::Object(members) = value else {
        return Err(format!("must be an object with Sideways and ForwardAndBack, not {}", value.describe()));
    };
    let mut dead_zones = DeadZones::default();
    for (member, value) in members {
        let zone = match member.as_str() {
            "Sideways" => &mut dead_zones.sideways,
            "ForwardAndBack" => &mut dead_zones.forward_and_back,
            _ => return Err(format!("{member} isn't a dead zone. They're Sideways and ForwardAndBack.")),
        };
        *zone = match value {
            Value::Array(items) => match items.as_slice() {
                [Value::Number(inner), Value::Number(outer)] if 0.0 <= *inner && inner < outer && *outer <= 1.0 => {
                    DeadZone {
                        inner: *inner,
                        outer: *outer,
                    }
                }
                _ => return Err(dead_zone_problem(member, value)),
            },
            _ => return Err(dead_zone_problem(member, value)),
        };
    }
    Ok(dead_zones)
}

fn dead_zone_problem(member: &str, value: &Value) -> String {
    format!(
        "{member} must be [inner, outer], from 0 to 1 with inner below outer, not {}",
        value.describe()
    )
}
