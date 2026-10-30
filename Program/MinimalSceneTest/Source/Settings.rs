// The test's own settings, after the ones every program has (see
// Library/Application), in Settings.json next to Cargo.toml: the SimplePaint
// material of each object, in Library/PaintSettings' format, which
// SimplePaintDemo saves, so a material can be tuned there and copied here.
//
// By default, the ground and the cube are lit, and the sphere is a constant
// color, which SimplePaint paints with equal Dark and Light Points.

use application::{ProgramSettings, Settings, WindowMode};
use json::Value;
use simple_paint::Parameters;

pub struct SceneSettings {
    pub sphere: Parameters,
    pub ground: Parameters,
    pub cube: Parameters,
}

// A lit material, painted with its color on faces that face up, as if lit
// from above, and darker sides.
fn lit(base_color_srgb: [f64; 3]) -> Parameters {
    Parameters {
        base_color_srgb,
        brightness: 0.5,
        shift: 0.0,
        rotation_degrees: 0.0,
        dark_point: 0.05,
        light_point: 0.7,
    }
}

impl ProgramSettings for SceneSettings {
    const NAMES: &'static [&'static str] = &["Sphere", "Ground", "Cube"];

    fn defaults() -> (Settings, Self) {
        (
            // Unlike other programs, it opens borderless fullscreen.
            Settings {
                window_mode: WindowMode::BorderlessFullscreen,
                ..Default::default()
            },
            Self {
                // Blue is the largest that SimplePaint's sRGB channels
                // accept, 1 - 1/1024.
                sphere: Parameters {
                    base_color_srgb: [0.17000002, 0.47433347, 0.9990234375],
                    brightness: 0.5,
                    shift: 0.0,
                    rotation_degrees: 0.0,
                    dark_point: 0.5,
                    light_point: 0.5,
                },
                ground: lit([0.08900002, 0.89, 0.48950002]),
                cube: lit([0.88, 0.1672, 0.17907982]),
            },
        )
    }

    fn apply(&mut self, name: &str, value: &Value) -> Result<(), String> {
        let material = match name {
            "Sphere" => &mut self.sphere,
            "Ground" => &mut self.ground,
            _ => &mut self.cube,
        };
        *material = paint_settings::parse(value, material)?;
        Ok(())
    }

    fn default_lines(&self) -> Vec<String> {
        [("Sphere", &self.sphere), ("Ground", &self.ground), ("Cube", &self.cube)]
            .into_iter()
            .map(|(name, material)| format!("\"{name}\": {}", paint_settings::format(material, 4)))
            .collect()
    }
}
