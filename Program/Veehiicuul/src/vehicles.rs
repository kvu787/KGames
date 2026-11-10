// The vehicles, from Vehicles.json. Each is a model, painted its own way,
// so that it looks like no other, with its own acceleration and footprint,
// the same on every track. Its paint goes over its model's, slot by slot.
//
//     {
//         "Vehicles": [
//             {
//                 "Name": "Green",
//                 "Model": "SlopeCar",
//                 "Acceleration": { "Forward": 10, "Reverse": 100, "Left": 40, "Right": 40 },
//                 "Footprint": { "Left": 1.5, "Right": 1.5, "Front": 2.9925, "Back": 3 },
//                 "Paint": { "SlopeCarBodyBlue": { ... } }
//             }
//         ]
//     }

use json::Value;

use crate::collision::Footprint;
use crate::driving::Acceleration;
use crate::files;
use crate::models::{Paint, paint_value, parse_paint};

#[derive(Clone, Debug, PartialEq)]
pub struct Vehicle {
    pub name: String,
    pub model: String,
    pub acceleration: Acceleration,
    pub footprint: Footprint,
    pub paint: Paint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vehicles {
    pub list: Vec<Vehicle>,
}

impl Vehicles {
    pub fn parse(text: &str) -> Result<Vehicles, String> {
        let value = files::parse(text)?;
        files::only(&value, &["Vehicles"], "the vehicles file")?;
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
        Ok(Vehicles { list })
    }

    pub fn to_json(&self) -> String {
        let list = self.list.iter().map(vehicle_value).collect();
        json::object([("Vehicles", Value::Array(list))]).to_json() + "\n"
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.list.iter().position(|vehicle| vehicle.name.eq_ignore_ascii_case(name))
    }

    // A name that no vehicle has yet, like "Vehicle 7".
    pub fn new_name(&self) -> String {
        (self.list.len() + 1..)
            .map(|number| format!("Vehicle {number}"))
            .find(|name| self.find(name).is_none())
            .unwrap()
    }
}

fn vehicle(value: &Value) -> Result<Vehicle, String> {
    files::only(value, &["Name", "Model", "Acceleration", "Footprint", "Paint"], "a vehicle")?;
    let name = files::text(files::member(value, "Name", "")?, "Name")?;
    let model = files::text(files::member(value, "Model", &format!("{name}'s "))?, &format!("{name}'s Model"))?;
    let acceleration = files::member(value, "Acceleration", &format!("{name}'s "))?;
    let rate = |direction: &str| match acceleration.get(direction) {
        Some(Value::Number(rate)) if rate.is_finite() && *rate >= 0.0 => Ok(*rate),
        Some(other) => Err(format!("{name}'s Acceleration.{direction} must be 0 or more, not {}", other.describe())),
        None => Err(format!("{name} needs Acceleration.{direction}")),
    };
    let footprint = files::member(value, "Footprint", &format!("{name}'s "))?;
    files::only(footprint, &["Left", "Right", "Front", "Back"], &format!("{name}'s Footprint"))?;
    let side = |side: &str| {
        files::positive(
            files::member(footprint, side, &format!("{name}'s Footprint."))?,
            &format!("{name}'s Footprint.{side}"),
        )
    };
    Ok(Vehicle {
        acceleration: Acceleration {
            forward: rate("Forward")?,
            reverse: rate("Reverse")?,
            left: rate("Left")?,
            right: rate("Right")?,
        },
        footprint: Footprint::from_sides(side("Left")?, side("Right")?, side("Front")?, side("Back")?),
        paint: match value.get("Paint") {
            Some(paint) => parse_paint(paint, &format!("{name}'s Paint"))?,
            None => Vec::new(),
        },
        model,
        name,
    })
}

fn vehicle_value(vehicle: &Vehicle) -> Value {
    let rates = vehicle.acceleration;
    let footprint = vehicle.footprint;
    json::object([
        ("Name", vehicle.name.clone().into()),
        ("Model", vehicle.model.clone().into()),
        (
            "Acceleration",
            json::object([
                ("Forward", files::rounded(rates.forward)),
                ("Reverse", files::rounded(rates.reverse)),
                ("Left", files::rounded(rates.left)),
                ("Right", files::rounded(rates.right)),
            ]),
        ),
        (
            "Footprint",
            json::object([
                ("Left", files::rounded(footprint.max[0])),
                ("Right", files::rounded(-footprint.min[0])),
                ("Front", files::rounded(footprint.max[1])),
                ("Back", files::rounded(-footprint.min[1])),
            ]),
        ),
        ("Paint", paint_value(&vehicle.paint)),
    ])
}
