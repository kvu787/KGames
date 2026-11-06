// The paints that color the mesh: each face's material is a paint, which is a
// name and SimplePaint's parameters. The flat shader colors with the same
// paints' base colors. A document has up to MAXIMUM_PAINTS, which is how many
// the shaders are compiled for.

use json::Value;
use simple_paint::{GpuMaterial, INTERIOR_MAXIMUM, MARGIN, Material, Parameters};
use user_interface::Color;

pub const MAXIMUM_PAINTS: usize = 16;

#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    pub name: String,
    pub parameters: Parameters,
}

impl Paint {
    pub fn new(name: &str, color: u32) -> Self {
        let mut paint = Paint {
            name: name.to_string(),
            parameters: Parameters::default(),
        };
        paint.set_color(Color::hex(color));
        paint
    }

    // The base color, as the interface shows it.
    pub fn color(&self) -> Color {
        let [r, g, b] = self.parameters.base_color_srgb;
        Color::rgb(r as f32, g as f32, b as f32)
    }

    // A base color, kept inside what SimplePaint accepts: each channel at
    // least MARGIN and at most INTERIOR_MAXIMUM.
    pub fn set_color(&mut self, color: Color) {
        let channel = |value: f32| (value as f64).clamp(MARGIN, INTERIOR_MAXIMUM);
        self.parameters.base_color_srgb = [channel(color.r), channel(color.g), channel(color.b)];
    }

    // SimplePaint's constants for the paint. The interface keeps every
    // parameter in its range, so it compiles; if one didn't, the paint shows
    // as a neutral gray instead of failing the frame.
    pub fn material(&self) -> GpuMaterial {
        match Material::compile(&self.parameters) {
            Ok(material) => *material.constants(),
            Err(_) => *Material::compile(&Parameters::default()).expect("the default paint compiles").constants(),
        }
    }

    // The base color in linear light, for the flat shader.
    pub fn linear_color(&self) -> [f32; 4] {
        let [r, g, b] = self.color().to_linear();
        [r, g, b, 1.0]
    }

    // As a document has it: { "Name": ..., "Paint": PaintSettings' object }.
    pub fn to_value(&self) -> Value {
        Value::Object(vec![
            ("Name".to_string(), Value::from(self.name.as_str())),
            ("Paint".to_string(), paint_settings::to_value(&self.parameters)),
        ])
    }

    pub fn from_value(value: &Value) -> Result<Paint, String> {
        let name = match value.get("Name") {
            Some(Value::String(name)) => name.clone(),
            Some(other) => return Err(format!("A paint's Name must be text, not {}.", other.describe())),
            None => return Err("A paint must have a Name.".to_string()),
        };
        let parameters = match value.get("Paint") {
            Some(paint) => paint_settings::parse(paint, &Parameters::default()).map_err(|error| format!("The paint {name}'s {error}."))?,
            None => Parameters::default(),
        };
        Ok(Paint { name, parameters })
    }
}

// The paints that a new document starts with.
pub fn default_paints() -> Vec<Paint> {
    vec![Paint::new("Blue", 0x1B3994), Paint::new("Coral", 0xE8624A), Paint::new("Sand", 0xD9C09A), Paint::new("Moss", 0x4E7A3A)]
}

// A color for a new paint, after the ones before it, from Fluent's shared
// colors.
pub fn next_color(count: usize) -> u32 {
    const COLORS: [u32; 8] = [0x0078D4, 0xCA5010, 0x107C10, 0x8764B8, 0x038387, 0xD13438, 0xC19C00, 0xE3008C];
    COLORS[count % COLORS.len()]
}
