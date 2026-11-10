// The test's own settings, after the ones every program has (see
// Library/UserInterface), in Settings.json next to Cargo.toml: the
// SimplePaint material of each object, in Library/PaintSettings' format,
// which SimplePaintDemo saves, so a material can be tuned there and copied
// here.
//
// By default, the ground and the cube are lit, and the sphere is a constant
// color, which SimplePaint paints with equal Dark and Light Points.

use simple_paint::Parameters;
use user_interface::Settings;

#[derive(Clone, Copy, Debug, PartialEq)]
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

impl Default for SceneSettings {
    fn default() -> Self {
        Self {
            // Blue is the largest that SimplePaint's sRGB channels accept,
            // 1 - 1/1024.
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
        }
    }
}

impl SceneSettings {
    // Reads the materials, "Sphere", "Ground" and "Cube". One that's missing
    // or wrong, like one that SimplePaint rejects, keeps its default, and
    // the settings say what's wrong with it.
    pub fn read(settings: &mut Settings) -> SceneSettings {
        let defaults = SceneSettings::default();
        let mut material = |name: &str, default: Parameters| {
            settings.value(name, paint_settings::to_value(&default), |value| {
                // The settings go on after what's wrong, so it ends without
                // the sentence's period.
                paint_settings::parse(value, &default).map_err(|error| error.trim_end_matches('.').to_string())
            })
        };
        SceneSettings {
            sphere: material("Sphere", defaults.sphere),
            ground: material("Ground", defaults.ground),
            cube: material("Cube", defaults.cube),
        }
    }
}
