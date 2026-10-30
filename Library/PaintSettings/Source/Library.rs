// SimplePaint materials in programs' JSON files, in the format of the
// settings of Veehiicuul's C++ scene, so that a material can be copied
// between programs:
//
//     {
//         "BaseColor": [0.107, 0.223, 0.578],
//         "Brightness": 0.5,
//         "Shift": 0.0,
//         "RotationDegrees": 0.0,
//         "DarkPoint": 0.0,
//         "LightPoint": 1.0
//     }
//
// SimplePaint itself leaves serialization to its hosts.

use json::Value;
use simple_paint::{Parameters, validate_parameters};

const NAMES: [&str; 6] = [
    "BaseColor",
    "Brightness",
    "Shift",
    "RotationDegrees",
    "DarkPoint",
    "LightPoint",
];

// Reads a material from a JSON object. A member that's missing keeps its
// value from `defaults`. Fails with what's wrong, including any value that
// SimplePaint rejects.
pub fn parse(value: &Value, defaults: &Parameters) -> Result<Parameters, String> {
    let Value::Object(members) = value else {
        return Err(format!(
            "must be an object with {}, not {}",
            NAMES.join(", "),
            value.describe()
        ));
    };
    let mut parameters = *defaults;
    for (name, value) in members {
        let number = |value: &Value| match value {
            Value::Number(number) => Ok(*number),
            _ => Err(format!("{name} must be a number, not {}", value.describe())),
        };
        match name.as_str() {
            "BaseColor" => match value {
                Value::Array(channels) if channels.len() == 3 => {
                    for (channel, value) in channels.iter().enumerate() {
                        parameters.base_color_srgb[channel] = number(value)?;
                    }
                }
                _ => {
                    return Err(format!(
                        "BaseColor must be a list of 3 numbers, red, green and blue, not {}",
                        value.describe()
                    ));
                }
            },
            "Brightness" => parameters.brightness = number(value)?,
            "Shift" => parameters.shift = number(value)?,
            "RotationDegrees" => parameters.rotation_degrees = number(value)?,
            "DarkPoint" => parameters.dark_point = number(value)?,
            "LightPoint" => parameters.light_point = number(value)?,
            _ => {
                return Err(format!(
                    "{name} isn't a SimplePaint setting. They're {}.",
                    NAMES.join(", ")
                ));
            }
        }
    }
    validate_parameters(&parameters).map_err(|error| error.0)?;
    Ok(parameters)
}

// The material as a JSON value, in the same format.
pub fn to_value(parameters: &Parameters) -> Value {
    json::object([
        ("BaseColor", parameters.base_color_srgb.into()),
        ("Brightness", parameters.brightness.into()),
        ("Shift", parameters.shift.into()),
        ("RotationDegrees", parameters.rotation_degrees.into()),
        ("DarkPoint", parameters.dark_point.into()),
        ("LightPoint", parameters.light_point.into()),
    ])
}

// The material as a JSON object, over several lines, with the lines after
// the first indented by `indent` spaces, for nesting in a settings file.
// Numbers are written so that they read back exactly.
pub fn format(parameters: &Parameters, indent: usize) -> String {
    let outer = " ".repeat(indent);
    let inner = " ".repeat(indent + 4);
    let [red, green, blue] = parameters.base_color_srgb;
    let members = [
        format!("\"BaseColor\": [{red:?}, {green:?}, {blue:?}]"),
        format!("\"Brightness\": {:?}", parameters.brightness),
        format!("\"Shift\": {:?}", parameters.shift),
        format!("\"RotationDegrees\": {:?}", parameters.rotation_degrees),
        format!("\"DarkPoint\": {:?}", parameters.dark_point),
        format!("\"LightPoint\": {:?}", parameters.light_point),
    ];
    let members: Vec<String> = members.iter().map(|member| format!("{inner}{member}")).collect();
    format!("{{\n{}\n{outer}}}", members.join(",\n"))
}
