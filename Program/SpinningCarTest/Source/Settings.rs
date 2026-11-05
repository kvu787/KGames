// The test's own settings, after the ones every program has (see
// Library/UserInterface), in Settings.json next to Cargo.toml: the
// SimplePaint material of each part of the car and of the sphere, in
// Library/PaintSettings' format, and the sphere's resolution.

use json::Value;
use simple_paint::Parameters;
use user_interface::Settings;

// The car's materials, in its mesh's order, then the sphere's.
pub const MATERIAL_NAMES: [&str; 6] = [
    "SimplePaintShader_Axles",
    "SimplePaintShader_Body",
    "SimplePaintShader_Cabin",
    "SimplePaintShader_Headlights",
    "SimplePaintShader_Wheels",
    "SimplePaintShader_Sphere",
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneSettings {
    pub materials: [Parameters; 6],
    // Vertices around each of the sphere's rings, and bands from pole to pole.
    pub sphere_resolution: [u32; 2],
}

fn material(
    base_color_srgb: [f64; 3],
    brightness: f64,
    shift: f64,
    rotation_degrees: f64,
    dark_point: f64,
    light_point: f64,
) -> Parameters {
    Parameters {
        base_color_srgb,
        brightness,
        shift,
        rotation_degrees,
        dark_point,
        light_point,
    }
}

impl Default for SceneSettings {
    fn default() -> Self {
        Self {
            materials: [
                material([0.2159017, 0.21590173, 0.2159017], 0.5, 0.0, 0.0, 0.0, 1.0),
                material([0.103999995, 0.3592001, 0.8], 0.20996094, 0.4140625, 35.600586, 0.0, 1.0),
                material([0.37600002, 0.5314668, 0.8], 0.26464844, 0.36132813, 28.935547, 0.2000000095, 1.0),
                material([0.98, 0.74676, 0.15680003], 0.48242188, 0.6484375, 49.7666, 0.0, 0.8999766052025),
                material([0.27022192, 0.27022192, 0.27022186], 0.41796875, 0.2841797, 224.49902, 0.12890625, 0.78222656),
                material([0.107, 0.223, 0.578], 0.126, 0.0, 0.0, 0.0, 1.0),
            ],
            sphere_resolution: [64, 32],
        }
    }
}

impl SceneSettings {
    // Reads the materials, then "Sphere", with the sphere's resolution, like
    // {"UResolution": 64, "VResolution": 32}. One that's missing or wrong,
    // like a material that SimplePaint rejects, keeps its default, and the
    // settings say what's wrong with it.
    pub fn read(settings: &mut Settings) -> SceneSettings {
        let defaults = SceneSettings::default();
        let materials = std::array::from_fn(|index| {
            let default = defaults.materials[index];
            settings.value(MATERIAL_NAMES[index], paint_settings::to_value(&default), |value| {
                // The settings go on after what's wrong, so it ends without
                // the sentence's period.
                paint_settings::parse(value, &default).map_err(|error| error.trim_end_matches('.').to_string())
            })
        });
        let [u, v] = defaults.sphere_resolution;
        let sphere = json::object([("UResolution", u.into()), ("VResolution", v.into())]);
        let sphere_resolution = settings.value("Sphere", sphere, |value| parse_resolution(value, defaults.sphere_resolution));
        SceneSettings {
            materials,
            sphere_resolution,
        }
    }
}

// The sphere's resolution, whose members that are missing keep theirs from
// `defaults`. It's the defaults unless every member is right.
fn parse_resolution(value: &Value, defaults: [u32; 2]) -> Result<[u32; 2], String> {
    let Value::Object(members) = value else {
        return Err(format!("must be an object with UResolution and VResolution, not {}", value.describe()));
    };
    let mut resolution = defaults;
    for (member, value) in members {
        let (index, range) = match member.as_str() {
            "UResolution" => (0, 4..=512),
            "VResolution" => (1, 2..=512),
            _ => return Err(format!("has {member}, which isn't one of UResolution and VResolution")),
        };
        match value {
            Value::Number(number) if number.fract() == 0.0 && range.contains(&(*number as i64)) => {
                resolution[index] = *number as u32;
            }
            _ => {
                return Err(format!(
                    "{member} must be a whole number from {} to {}, not {}",
                    range.start(),
                    range.end(),
                    value.describe()
                ));
            }
        }
    }
    Ok(resolution)
}
