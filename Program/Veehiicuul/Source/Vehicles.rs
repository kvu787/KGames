// The vehicles, from Vehicles.json. Each is the car of Library/Meshes, with
// its own colors, so that it looks like no other, and its own acceleration,
// the same on every track. The colors are SimplePaint materials, in
// Library/PaintSettings' format: each vehicle's body and cabin, and the
// axles, headlights and wheels that they all share.
//
//     {
//         "Axles": { "BaseColor": [0.2159, 0.2159, 0.2159], ... },
//         "Headlights": { ... },
//         "Wheels": { ... },
//         "Vehicles": [
//             {
//                 "Name": "Green",
//                 "Acceleration": { "Forward": 10, "Reverse": 100, "Left": 40, "Right": 40 },
//                 "Body": { ... },
//                 "Cabin": { ... }
//             },
//             ...
//         ]
//     }

use std::path::Path;

use json::Value;
use meshes::car;
use simple_paint::Parameters;

use crate::driving::Acceleration;

pub struct Vehicle {
    pub name: String,
    pub acceleration: Acceleration,
    pub body: Parameters,
    pub cabin: Parameters,
}

pub struct Vehicles {
    pub axles: Parameters,
    pub headlights: Parameters,
    pub wheels: Parameters,
    pub list: Vec<Vehicle>,
}

// What a material that a file leaves out a member of keeps.
const DEFAULT_PAINT: Parameters = Parameters {
    base_color_srgb: [0.5, 0.5, 0.5],
    brightness: 0.5,
    shift: 0.0,
    rotation_degrees: 0.0,
    dark_point: 0.0,
    light_point: 1.0,
};

impl Vehicles {
    pub fn parse(text: &str) -> Result<Vehicles, String> {
        let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
        let value = json::parse(text).map_err(|error| {
            format!("line {}, column {}: {}", error.line, error.column, error.message)
        })?;
        let Value::Object(members) = &value else {
            return Err(format!("must be an object, not {}", value.describe()));
        };
        const MEMBERS: [&str; 4] = ["Axles", "Headlights", "Wheels", "Vehicles"];
        if let Some((member, _)) = members.iter().find(|(member, _)| !MEMBERS.contains(&member.as_str())) {
            return Err(format!("{member} isn't part of the vehicles file. They're {}", MEMBERS.join(", ")));
        }
        let Some(Value::Array(items)) = value.get("Vehicles") else {
            return Err("needs Vehicles, a list of vehicles".to_string());
        };
        let mut list = Vec::new();
        for (index, item) in items.iter().enumerate() {
            let vehicle = vehicle(item).map_err(|problem| format!("Vehicles[{index}]: {problem}"))?;
            if list.iter().any(|other: &Vehicle| other.name == vehicle.name) {
                return Err(format!("Vehicles[{index}]: another vehicle is called {}", vehicle.name));
            }
            list.push(vehicle);
        }
        if list.is_empty() {
            return Err("Vehicles needs at least one vehicle".to_string());
        }
        Ok(Vehicles {
            axles: paint(&value, "Axles")?,
            headlights: paint(&value, "Headlights")?,
            wheels: paint(&value, "Wheels")?,
            list,
        })
    }

    pub fn load(path: &Path) -> Result<Vehicles, String> {
        let text = std::fs::read_to_string(path).map_err(|error| format!("Couldn't read {}: {error}", path.display()))?;
        Vehicles::parse(&text).map_err(|problem| format!("{}: {problem}", path.display()))
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.list.iter().position(|vehicle| vehicle.name.eq_ignore_ascii_case(name))
    }

    // A vehicle's materials, in the car mesh's order.
    pub fn paints(&self, index: usize) -> [Parameters; car::MATERIAL_COUNT as usize] {
        let vehicle = &self.list[index];
        let mut paints = [DEFAULT_PAINT; car::MATERIAL_COUNT as usize];
        paints[car::AXLES as usize] = self.axles;
        paints[car::BODY as usize] = vehicle.body;
        paints[car::CABIN as usize] = vehicle.cabin;
        paints[car::HEADLIGHTS as usize] = self.headlights;
        paints[car::WHEELS as usize] = self.wheels;
        paints
    }
}

fn paint(value: &Value, name: &str) -> Result<Parameters, String> {
    let member = value.get(name).ok_or_else(|| format!("needs {name}, a SimplePaint material"))?;
    paint_settings::parse(member, &DEFAULT_PAINT).map_err(|problem| format!("{name} {problem}"))
}

fn vehicle(value: &Value) -> Result<Vehicle, String> {
    let Value::Object(members) = value else {
        return Err(format!("must be an object, not {}", value.describe()));
    };
    const MEMBERS: [&str; 4] = ["Name", "Acceleration", "Body", "Cabin"];
    if let Some((member, _)) = members.iter().find(|(member, _)| !MEMBERS.contains(&member.as_str())) {
        return Err(format!("{member} isn't part of a vehicle. They're {}", MEMBERS.join(", ")));
    }
    let name = match value.get("Name") {
        Some(Value::String(name)) if !name.trim().is_empty() => name.clone(),
        _ => return Err("needs a Name".to_string()),
    };
    let acceleration = value.get("Acceleration").unwrap_or(&Value::Null);
    let rate = |direction: &str| match acceleration.get(direction) {
        Some(Value::Number(rate)) if rate.is_finite() && *rate >= 0.0 => Ok(*rate),
        Some(other) => Err(format!("{name}'s Acceleration.{direction} must be 0 or more, not {}", other.describe())),
        None => Err(format!("{name} needs Acceleration.{direction}")),
    };
    Ok(Vehicle {
        acceleration: Acceleration {
            forward: rate("Forward")?,
            reverse: rate("Reverse")?,
            left: rate("Left")?,
            right: rate("Right")?,
        },
        body: paint(value, "Body").map_err(|problem| format!("{name}'s {problem}"))?,
        cabin: paint(value, "Cabin").map_err(|problem| format!("{name}'s {problem}"))?,
        name,
    })
}
