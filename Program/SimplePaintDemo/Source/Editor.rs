// The settings editor's state: the material being edited, its sliders, and
// what the mouse and keyboard do to them. Panels.rs draws it.
//
// Sliders move in steps of 0.001, or 0.1 degrees for Rotation, and stop at
// the last step inside SimplePaint's accepted range, so the material they
// make is always valid. Values from Paint.json can be between steps.

use std::path::Path;

use application::Event;
use json::Value;
use simple_paint::{GpuMaterial, Material, Parameters};
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

    // Where the value is along the track, from 0 to 1.
    pub fn fraction(&self, parameters: &Parameters) -> f64 {
        ((self.value(parameters) - self.start) / (self.end - self.start)).clamp(0.0, 1.0)
    }

    pub fn format(&self, value: f64) -> String {
        format!("{value:.0$}{1}", self.decimals, self.unit)
    }

    // Sets the value to the step nearest `value`, within the range.
    fn set(&self, parameters: &mut Parameters, value: f64) {
        // Dividing whole numbers, like 107 / 1000, gives the double nearest
        // the decimal, so values save as written, like 0.107.
        let stepped = (value * self.steps).round() / self.steps;
        (self.set)(parameters, stepped.clamp(self.minimum, self.maximum));
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

// What a click does at a place in the window.
#[derive(Clone, Copy)]
pub enum Click {
    // Sets a slider from where the click is, and drags it.
    Slider(usize),
    Reset,
    Save,
}

// A place that reacts to clicks, and where a slider's track is along it.
#[derive(Clone, Copy)]
pub struct Target {
    pub area: canvas::Rect,
    pub click: Click,
    // The track's left and right edges, for a slider.
    pub track: [f32; 2],
}

// What the main loop does, besides drawing the editor.
pub enum Action {
    None,
    ToggleFullscreen,
}

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
    // The slider that the wheel last turned over, and how much it turned
    // there that didn't make a whole step yet.
    wheel: (usize, f32),
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
            wheel: (0, 0.0),
        }
    }

    fn change(&mut self, slider: usize, value: f64) {
        SLIDERS[slider].set(&mut self.parameters, value);
        // Sliders only make valid values, and set leaves the others alone.
        self.material = *Material::compile(&self.parameters)
            .expect("sliders only make valid materials")
            .constants();
    }

    // Moves a slider by a number of its steps.
    fn nudge(&mut self, slider: usize, steps: f64) {
        let current = SLIDERS[slider].value(&self.parameters);
        self.change(slider, current + steps / SLIDERS[slider].steps);
    }

    // Sets a slider from where the cursor is along its track.
    fn drag_to(&mut self, slider: usize, x: f32, targets: &[Target]) {
        let Some(target) = targets
            .iter()
            .find(|target| matches!(target.click, Click::Slider(index) if index == slider))
        else {
            return;
        };
        let [left, right] = target.track;
        let fraction = ((x - left) / (right - left)).clamp(0.0, 1.0) as f64;
        let range = &SLIDERS[slider];
        self.change(slider, range.start + fraction * (range.end - range.start));
    }

    // Handles one event from the window, with the places that react to
    // clicks in the last frame drawn.
    pub fn handle(&mut self, event: &Event, targets: &[Target], paint_path: &Path) -> Action {
        let at = |point: [f32; 2]| targets.iter().rev().find(|target| target.area.contains(point));
        match *event {
            Event::Press(point) => match at(point).map(|target| target.click) {
                Some(Click::Slider(slider)) => {
                    self.selected = slider;
                    self.dragging = Some(slider);
                    self.drag_to(slider, point[0], targets);
                }
                Some(Click::Reset) => {
                    *self = Editor {
                        selected: self.selected,
                        ..Editor::new(Parameters::default())
                    }
                }
                Some(Click::Save) => self.save(paint_path),
                None => {}
            },
            Event::Release(_) => self.dragging = None,
            // A notch is a step. Touchpads and smooth-scrolling mice turn
            // by parts of a notch, which add up over the same slider.
            Event::Wheel(point, notches) => {
                if let Some(Click::Slider(slider)) = at(point).map(|target| target.click) {
                    self.selected = slider;
                    let (last, part) = self.wheel;
                    let turned = notches + if last == slider { part } else { 0.0 };
                    let steps = turned.trunc();
                    self.wheel = (slider, turned - steps);
                    if steps != 0.0 {
                        self.nudge(slider, steps as f64);
                    }
                }
            }
            // Holding an arrow key repeats it. Holding F11 or Ctrl+S does
            // nothing more.
            Event::Key {
                key,
                control,
                shift,
                repeat,
            } => {
                let steps = if shift { 10.0 } else { 1.0 };
                match key as i32 {
                    VK_F11 if !repeat => return Action::ToggleFullscreen,
                    VK_LEFT => self.nudge(self.selected, -steps),
                    VK_RIGHT => self.nudge(self.selected, steps),
                    VK_UP => self.selected = self.selected.saturating_sub(1),
                    VK_DOWN => self.selected = (self.selected + 1).min(SLIDERS.len() - 1),
                    _ if control && !repeat && key == 'S' as u32 => self.save(paint_path),
                    _ => {}
                }
            }
        }
        Action::None
    }

    // Follows the cursor with the slider being dragged. `cursor` is None
    // when the window doesn't have focus, which ends the drag, as the
    // button's release then never comes.
    pub fn follow(&mut self, cursor: Option<[f32; 2]>, targets: &[Target]) {
        match (self.dragging, cursor) {
            (Some(slider), Some([x, _])) => self.drag_to(slider, x, targets),
            (Some(_), None) => self.dragging = None,
            _ => {}
        }
    }

    // What the editor holds, for scripted runs to save and check, with
    // where the last frame's clickable places are, for writing scripts.
    pub fn state(&self, targets: &[Target]) -> Value {
        let m = &self.material;
        let targets: Vec<Value> = targets
            .iter()
            .map(|target| {
                let name = match target.click {
                    Click::Slider(index) => SLIDERS[index].name.to_string(),
                    Click::Reset => "Reset".to_string(),
                    Click::Save => "Save".to_string(),
                };
                let area = target.area;
                json::object([
                    ("Name", name.into()),
                    ("Area", [area.x, area.y, area.width, area.height].into()),
                    ("Track", target.track.into()),
                ])
            })
            .collect();
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
            ("Targets", Value::Array(targets)),
        ])
    }

    fn save(&mut self, path: &Path) {
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
