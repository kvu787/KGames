// The settings editor's state: the material being edited, its sliders, and
// what the keyboard does to them. Panels.rs draws the sliders with
// Library/UserInterface, which handles what the mouse does to them.
//
// Sliders move in steps of 0.001, or 0.1 degrees for Rotation, and stop at
// the last step inside SimplePaint's accepted range, so the material they
// make is always valid. Values from Paint.json can be between steps.

use std::path::Path;

use application::Event;
use json::Value;
use simple_paint::{GpuMaterial, Material, Parameters};
use user_interface::SliderRange;
use windows_api::*;

pub struct Slider {
    pub name: &'static str,
    // The track runs from `start` to `end`.
    pub start: f64,
    pub end: f64,
    // The first and last steps inside SimplePaint's accepted range.
    pub minimum: f64,
    pub maximum: f64,
    // Steps per unit, so a value on a step is a whole number of them.
    pub steps: f64,
    pub decimals: usize,
    pub unit: &'static str,
    get: fn(&Parameters) -> f64,
    set: fn(&mut Parameters, f64),
}

impl Slider {
    pub fn value(&self, parameters: &Parameters) -> f64 {
        (self.get)(parameters)
    }

    pub fn range(&self) -> SliderRange {
        SliderRange {
            start: self.start,
            end: self.end,
            minimum: self.minimum,
            maximum: self.maximum,
            steps: self.steps,
        }
    }

    pub fn format(&self, value: f64) -> String {
        format!("{value:.0$}{1}", self.decimals, self.unit)
    }
}

macro_rules! unit_slider {
    ($name:expr, $minimum:expr, $maximum:expr, $field:ident $([$index:expr])?) => {
        Slider {
            name: $name,
            start: 0.0,
            end: 1.0,
            minimum: $minimum,
            maximum: $maximum,
            steps: 1000.0,
            decimals: 3,
            unit: "",
            get: |p| p.$field$([$index])?,
            set: |p, value| p.$field$([$index])? = value,
        }
    };
}

pub const SLIDERS: [Slider; 8] = [
    unit_slider!("R", 0.001, 0.999, base_color_srgb[0]),
    unit_slider!("G", 0.001, 0.999, base_color_srgb[1]),
    unit_slider!("B", 0.001, 0.999, base_color_srgb[2]),
    unit_slider!("Brightness", 0.001, 0.999, brightness),
    unit_slider!("Shift", 0.0, 0.999, shift),
    Slider {
        name: "Rotation",
        start: 0.0,
        end: 360.0,
        minimum: 0.0,
        maximum: 359.9,
        steps: 10.0,
        decimals: 1,
        unit: " deg",
        get: |p| p.rotation_degrees,
        set: |p, value| p.rotation_degrees = value,
    },
    unit_slider!("Dark Point", 0.0, 0.999, dark_point),
    unit_slider!("Light Point", 0.001, 1.0, light_point),
];

pub struct Editor {
    pub parameters: Parameters,
    // Compiled from parameters whenever they change.
    pub material: GpuMaterial,
    // The slider that the arrow keys move, which is highlighted.
    pub selected: usize,
    // The slider that the mouse drags, while the button is down.
    pub dragging: Option<usize>,
    // What happened at the last save, and whether it failed.
    pub status: Option<(String, bool)>,
}

impl Editor {
    pub fn new(parameters: Parameters) -> Self {
        Self {
            parameters,
            material: *Material::compile(&parameters)
                .expect("the editor starts with a valid material")
                .constants(),
            selected: 0,
            dragging: None,
            status: None,
        }
    }

    // Sets a slider to a value that's one of its steps, as the slider
    // widget makes them.
    pub fn change(&mut self, slider: usize, value: f64) {
        (SLIDERS[slider].set)(&mut self.parameters, value);
        // Sliders only make valid values, and set leaves the others alone.
        self.material = *Material::compile(&self.parameters)
            .expect("sliders only make valid materials")
            .constants();
    }

    // Moves a slider by a number of its steps. A value from Paint.json can be
    // past the slider's range, like a B of 0.9990234375 past 0.999, where
    // the nearest step back in it is the other way. Then it stays put.
    fn nudge(&mut self, slider: usize, steps: f64) {
        let current = SLIDERS[slider].value(&self.parameters);
        let next = SLIDERS[slider].range().nudge(current, steps);
        if next != current {
            self.change(slider, next);
        }
    }

    // Goes back to SimplePaint's default material, keeping the selection.
    pub fn reset(&mut self) {
        *self = Editor {
            selected: self.selected,
            ..Editor::new(Parameters::default())
        }
    }

    // Handles a key that no widget took. Up and Down select a slider, and
    // Left and Right move it a step, or 10 with Shift. Holding an arrow key
    // repeats it. Ctrl+S saves, and holding it does nothing more.
    pub fn key(&mut self, event: &Event, paint_path: &Path) {
        let Event::Key {
            key,
            control,
            shift,
            repeat,
        } = *event
        else {
            return;
        };
        let steps = if shift { 10.0 } else { 1.0 };
        match key as i32 {
            _ if control => {
                if !repeat && key == 'S' as u32 {
                    self.save(paint_path);
                }
            }
            VK_LEFT => self.nudge(self.selected, -steps),
            VK_RIGHT => self.nudge(self.selected, steps),
            VK_UP => self.selected = self.selected.saturating_sub(1),
            VK_DOWN => self.selected = (self.selected + 1).min(SLIDERS.len() - 1),
            _ => {}
        }
    }

    // What the editor holds, for scripted runs to save and check. The
    // interface's state, which the main loop adds, says where the sliders
    // and buttons are.
    pub fn state(&self) -> Value {
        let m = &self.material;
        json::object([
            ("Parameters", paint_settings::to_value(&self.parameters)),
            ("Selected", SLIDERS[self.selected].name.into()),
            ("Dragging", self.dragging.map(|index| SLIDERS[index].name).into()),
            ("Saved", self.status.as_ref().map(|(_, failed)| !failed).into()),
            (
                "Constants",
                json::object([
                    ("Warp", m.warp.into()),
                    ("NumeratorDark", m.numerator_dark.into()),
                    ("NumeratorLight", m.numerator_light.into()),
                    ("ComplementDark", m.complement_dark.into()),
                    ("ComplementLight", m.complement_light.into()),
                ]),
            ),
        ])
    }

    pub fn save(&mut self, path: &Path) {
        let text = paint_settings::format(&self.parameters, 0) + "\n";
        self.status = Some(match std::fs::write(path, text) {
            Ok(()) => (format!("Saved to {}", path.display()), false),
            Err(error) => (format!("Couldn't save to {}: {error}", path.display()), true),
        });
    }
}

// The material in Paint.json, if there is one. Problems go in `notices`,
// and then the default material is used.
pub fn load(path: &Path, notices: &mut Vec<String>) -> Parameters {
    let defaults = Parameters::default();
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return defaults,
        Err(error) => {
            notices.push(format!(
                "Couldn't read {}, so the default material is used: {error}",
                path.display()
            ));
            return defaults;
        }
    };
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(&text);
    let result = json::parse(text)
        .map_err(|error| format!("line {}, column {}: {}", error.line, error.column, error.message))
        .and_then(|value| paint_settings::parse(&value, &defaults));
    result.unwrap_or_else(|problem| {
        notices.push(format!(
            "{}: {problem}. The default material is used.",
            path.display()
        ));
        defaults
    })
}
