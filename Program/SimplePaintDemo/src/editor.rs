// The settings editor's state: the material being edited, its settings, and
// what the keys and the wheel do to them. cards.rs draws a slider for each
// setting with Library/UserInterface, which handles what dragging it does.
//
// Settings move in steps of 0.001, or 0.1 degrees for Rotation, and stop at
// the last step inside SimplePaint's accepted range, so the material they
// make is always valid. Values from Paint.json can be between steps.

use std::path::Path;

use json::Value;
use simple_paint::{GpuMaterial, Material, Parameters};
use user_interface::{Key, Modifiers, Slider};


pub struct Setting {
    pub name: &'static str,
    // The slider's rail runs from `start` to `end`.
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

impl Setting {
    pub fn value(&self, parameters: &Parameters) -> f64 {
        (self.get)(parameters)
    }

    // The slider that shows `value`, over its rail, a step at a time, inside
    // the range. Its label, which scripts see, is the value as it's shown,
    // and its wheel accumulates parts of notches by itself.
    pub fn slider<M: Clone + 'static>(&self, value: f64) -> Slider<M> {
        Slider::new(value, self.start, self.end)
            .step(1.0 / self.steps)
            .allowed(self.minimum, self.maximum)
            .decimals(self.decimals)
            .suffix(self.unit)
    }

    // The step nearest `value`, inside the range. Dividing whole numbers,
    // like 107 / 1000, gives the double nearest the decimal, so values save
    // as written, like 0.107.
    pub fn nearest(&self, value: f64) -> f64 {
        let stepped = (value * self.steps).round() / self.steps;
        stepped.clamp(self.minimum, self.maximum)
    }

    // The value `steps` steps from `value`. A value from Paint.json can be
    // past the range, like 0.9990234375 past 0.999, where the nearest step
    // back in it is the other way. Then it stays put.
    pub fn nudge(&self, value: f64, steps: f64) -> f64 {
        let next = self.nearest(value + steps / self.steps);
        if (next - value) * steps > 0.0 { next } else { value }
    }

    pub fn format(&self, value: f64) -> String {
        format!("{value:.0$}{1}", self.decimals, self.unit)
    }

    // Like "0.001 to 0.999".
    pub fn range(&self) -> String {
        format!("{:.2$} to {}", self.minimum, self.format(self.maximum), self.decimals)
    }
}

macro_rules! unit_setting {
    ($name:expr, $minimum:expr, $maximum:expr, $field:ident $([$index:expr])?) => {
        Setting {
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

pub const SETTINGS: [Setting; 8] = [
    unit_setting!("R", 0.001, 0.999, base_color_srgb[0]),
    unit_setting!("G", 0.001, 0.999, base_color_srgb[1]),
    unit_setting!("B", 0.001, 0.999, base_color_srgb[2]),
    unit_setting!("Brightness", 0.001, 0.999, brightness),
    unit_setting!("Shift", 0.0, 0.999, shift),
    Setting {
        name: "Rotation",
        start: 0.0,
        end: 360.0,
        minimum: 0.0,
        maximum: 359.9,
        steps: 10.0,
        decimals: 1,
        unit: "\u{B0}",
        get: |p| p.rotation_degrees,
        set: |p, value| p.rotation_degrees = value,
    },
    unit_setting!("Dark Point", 0.0, 0.999, dark_point),
    unit_setting!("Light Point", 0.001, 1.0, light_point),
];

// What a key does to the editor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    // Selects the setting this many below the selected one, or above it if
    // it's less than 0.
    Select(i32),
    // Moves the selected setting by this many steps.
    Nudge(f64),
    Save,
}

// What a key does, pressed with the modifiers held. Up and Down select a
// setting, and Left and Right move it a step, or 10 with Shift. Holding an
// arrow key repeats it. Ctrl+S saves, and holding it does nothing more.
pub fn command(key: Key, repeat: bool, modifiers: Modifiers) -> Option<Command> {
    if modifiers == Modifiers::CONTROL {
        return (key == Key::S && !repeat).then_some(Command::Save);
    }
    if modifiers.control || modifiers.alt {
        return None;
    }
    let steps = if modifiers.shift { 10.0 } else { 1.0 };
    match key {
        Key::UP => Some(Command::Select(-1)),
        Key::DOWN => Some(Command::Select(1)),
        Key::LEFT => Some(Command::Nudge(-steps)),
        Key::RIGHT => Some(Command::Nudge(steps)),
        _ => None,
    }
}

pub struct Editor {
    pub parameters: Parameters,
    // Compiled from parameters whenever they change.
    pub material: GpuMaterial,
    // The setting that the arrow keys move, which is highlighted.
    pub selected: usize,
    // The setting whose slider the pointer drags, while the button is down.
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

    // Sets a setting to a value that's one of its steps, if it isn't already
    // that.
    pub fn set(&mut self, setting: usize, value: f64) {
        if value == SETTINGS[setting].value(&self.parameters) {
            return;
        }
        (SETTINGS[setting].set)(&mut self.parameters, value);
        // Settings only take valid values, and set leaves the others alone.
        self.material = *Material::compile(&self.parameters)
            .expect("settings only make valid materials")
            .constants();
    }

    // Moves a setting by a number of its steps.
    pub fn nudge(&mut self, setting: usize, steps: f64) {
        let current = SETTINGS[setting].value(&self.parameters);
        self.set(setting, SETTINGS[setting].nudge(current, steps));
    }

    pub fn apply(&mut self, command: Command, paint_path: &Path) {
        match command {
            Command::Select(offset) => {
                self.selected = (self.selected as i32 + offset).clamp(0, SETTINGS.len() as i32 - 1) as usize;
            }
            Command::Nudge(steps) => self.nudge(self.selected, steps),
            Command::Save => self.save(paint_path),
        }
    }

    // Goes back to SimplePaint's default material, keeping the selection.
    pub fn reset(&mut self) {
        *self = Editor {
            selected: self.selected,
            ..Editor::new(Parameters::default())
        }
    }

    // What the editor holds, for scripted runs to save and check. The
    // interface's state, which the main loop adds, says where the sliders
    // and buttons are.
    pub fn state(&self) -> Value {
        let m = &self.material;
        json::object([
            ("Parameters", paint_settings::to_value(&self.parameters)),
            ("Selected", SETTINGS[self.selected].name.into()),
            ("Dragging", self.dragging.map(|index| SETTINGS[index].name).into()),
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
