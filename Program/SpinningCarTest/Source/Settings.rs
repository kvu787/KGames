// The test's own settings, after the ones every program has (see
// Library/Application), in Settings.json next to Cargo.toml: the SimplePaint
// material of each part of the car and of the sphere, in
// Library/PaintSettings' format, and the sphere's resolution.

use application::{ProgramSettings, Settings};
use json::Value;
use simple_paint::Parameters;

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

impl ProgramSettings for SceneSettings {
    // The car's materials, in its mesh's order, then the sphere's, and the
    // sphere's resolution.
    const NAMES: &'static [&'static str] = &[
        "SimplePaintShader_Axles",
        "SimplePaintShader_Body",
        "SimplePaintShader_Cabin",
        "SimplePaintShader_Headlights",
        "SimplePaintShader_Wheels",
        "SimplePaintShader_Sphere",
        "Sphere",
    ];

    fn defaults() -> (Settings, Self) {
        (
            Settings::default(),
            Self {
                materials: [
                    material([0.2159017, 0.21590173, 0.2159017], 0.5, 0.0, 0.0, 0.0, 1.0),
                    material([0.103999995, 0.3592001, 0.8], 0.20996094, 0.4140625, 35.600586, 0.0, 1.0),
                    material(
                        [0.37600002, 0.5314668, 0.8],
                        0.26464844,
                        0.36132813,
                        28.935547,
                        0.2000000095,
                        1.0,
                    ),
                    material(
                        [0.98, 0.74676, 0.15680003],
                        0.48242188,
                        0.6484375,
                        49.7666,
                        0.0,
                        0.8999766052025,
                    ),
                    material(
                        [0.27022192, 0.27022192, 0.27022186],
                        0.41796875,
                        0.2841797,
                        224.49902,
                        0.12890625,
                        0.78222656,
                    ),
                    material([0.107, 0.223, 0.578], 0.126, 0.0, 0.0, 0.0, 1.0),
                ],
                sphere_resolution: [64, 32],
            },
        )
    }

    fn apply(&mut self, name: &str, value: &Value) -> Result<(), String> {
        if let Some(index) = Self::NAMES[..6].iter().position(|&material| material == name) {
            self.materials[index] = paint_settings::parse(value, &self.materials[index])?;
            return Ok(());
        }
        // "Sphere", with its resolution, like {"UResolution": 64, "VResolution": 32}.
        let Value::Object(members) = value else {
            return Err(format!(
                "must be an object with UResolution and VResolution, not {}",
                value.describe()
            ));
        };
        // Applied only if every member is right, so that a wrong one keeps
        // both at their defaults.
        let mut resolution = self.sphere_resolution;
        for (member, value) in members {
            let (index, range) = match member.as_str() {
                "UResolution" => (0, 3..=512),
                "VResolution" => (1, 2..=512),
                _ => return Err(format!("{member} isn't a sphere setting. They're UResolution and VResolution.")),
            };
            match value {
                Value::Number(number)
                    if number.fract() == 0.0 && range.contains(&(*number as i64)) =>
                {
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
        self.sphere_resolution = resolution;
        Ok(())
    }

    fn default_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Self::NAMES[..6]
            .iter()
            .zip(&self.materials)
            .map(|(name, material)| format!("\"{name}\": {}", paint_settings::format(material, 4)))
            .collect();
        let [u, v] = self.sphere_resolution;
        lines.push(format!(
            "\"Sphere\": {{\n        \"UResolution\": {u},\n        \"VResolution\": {v}\n    }}"
        ));
        lines
    }
}
