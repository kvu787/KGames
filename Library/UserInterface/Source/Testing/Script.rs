// Scripts: input from a file instead of the user, on a fixed clock, which
// tests a program in its real window without touching the user's input or
// focus. RunInBackground runs them, and the README describes the format.
//
//     {
//         "FrameTime": 0.016666666666666666,
//         "Steps": [
//             { "Frame": 1, "Click": "Save" },
//             { "Frame": 3, "Drag": { "From": "Volume", "To": [400, 300], "Frames": 10 } },
//             { "Frame": 20, "Key": "Ctrl+S" },
//             { "Frame": 22, "Capture": "Saved" },
//             { "Frame": 22, "Expect": { "Saved": true, "Interface": { "Popup": null } } }
//         ]
//     }
//
// Frame N's steps happen before frame N's update, which sees them as input,
// except Capture and Expect, which happen after it. Frame N is at N times
// FrameTime seconds, a 60th of a second unless the script says. Frame 0,
// before the first step, gives the window focus and draws what steps can
// point at. The program exits after the last step's frame, and fails if an
// expectation wasn't met.

use std::collections::BTreeMap;
use std::path::Path;

use json::Value;

use crate::core::interface::Interface;
use crate::drawing::geometry::Point;
use crate::input::devices::{Gamepad, GamepadButtons, Key, MouseButton, Shortcut};
use crate::input::events::Event;

pub const DEFAULT_FRAME_TIME: f64 = 1.0 / 60.0;

const ACTIONS: [&str; 15] = [
    "Move", "Press", "Release", "Click", "DoubleClick", "Drag", "Scroll", "Key", "Hold", "Type", "Gamepad", "Focus", "Capture", "Expect", "Program",
];

#[derive(Clone, Debug)]
struct Step {
    frame: u64,
    action: String,
    value: Value,
}

#[derive(Debug)]
pub struct Script {
    pub frame_time: f64,
    steps: Vec<Step>,
}

// What a frame of a script does around the program's update.
#[derive(Default)]
pub struct ScriptFrame {
    pub events: Vec<Event>,
    pub program_steps: Vec<Value>,
    // These happen after the update.
    pub expectations: Vec<Value>,
    pub captures: Vec<String>,
}

impl Script {
    pub fn load(path: &Path) -> Result<Script, String> {
        let text = std::fs::read_to_string(path).map_err(|error| format!("Couldn't read the script {}: {error}", path.display()))?;
        Script::parse(&text).map_err(|error| format!("The script {} {error}", path.display()))
    }

    pub fn parse(text: &str) -> Result<Script, String> {
        let root = json::parse(text).map_err(|error| format!("isn't valid JSON, at line {}, column {}: {}", error.line, error.column, error.message))?;
        let frame_time = match root.get("FrameTime") {
            None => DEFAULT_FRAME_TIME,
            Some(Value::Number(time)) if *time > 0.0 => *time,
            Some(other) => return Err(format!("has a FrameTime of {}, which isn't a positive number", other.describe())),
        };
        let Some(Value::Array(items)) = root.get("Steps") else {
            return Err("must have Steps, a list".to_string());
        };
        let mut steps = Vec::new();
        for (index, item) in items.iter().enumerate() {
            let Value::Object(members) = item else {
                return Err(format!("has step {index}, which isn't an object"));
            };
            let frame = match item.get("Frame") {
                Some(Value::Number(frame)) if *frame >= 1.0 && frame.fract() == 0.0 => *frame as u64,
                _ => return Err(format!("has step {index} without a Frame, a whole number from 1")),
            };
            let actions: Vec<&(String, Value)> = members.iter().filter(|(name, _)| name != "Frame").collect();
            let [(action, value)] = actions.as_slice() else {
                return Err(format!("has step {index}, which must have one action besides its Frame"));
            };
            if !ACTIONS.contains(&action.as_str()) {
                return Err(format!("has step {index} with the action {action}, which isn't one of {}", ACTIONS.join(", ")));
            }
            steps.push(Step {
                frame,
                action: action.clone(),
                value: value.clone(),
            });
        }
        steps.sort_by_key(|step| step.frame);
        Ok(Script { frame_time, steps })
    }

    // The frame of the last step.
    pub fn last_frame(&self) -> u64 {
        self.steps.last().map_or(1, |step| step.frame)
    }
}

pub struct ScriptRunner {
    script: Script,
    // Events that steps put off to later frames, like a click's release.
    scheduled: BTreeMap<u64, Vec<Event>>,
    held: Vec<Key>,
    gamepad: Gamepad,
    pub failures: Vec<String>,
}

impl ScriptRunner {
    pub fn new(script: Script) -> Self {
        Self {
            script,
            scheduled: BTreeMap::new(),
            held: Vec::new(),
            gamepad: Gamepad::default(),
            failures: Vec::new(),
        }
    }

    pub fn frame_time(&self) -> f64 {
        self.script.frame_time
    }

    pub fn last_frame(&self) -> u64 {
        self.script.last_frame()
    }

    // What frame N does. `ui` has the last frame's targets, which names point
    // at.
    pub fn frame(&mut self, frame: u64, ui: &Interface) -> Result<ScriptFrame, String> {
        let mut output = ScriptFrame::default();
        if frame == 0 {
            output.events.push(Event::FocusGained(Vec::new()));
        }
        if let Some(events) = self.scheduled.remove(&frame) {
            output.events.extend(events);
        }
        let steps: Vec<Step> = self.script.steps.iter().filter(|step| step.frame == frame).cloned().collect();
        for step in steps {
            let fail = |error: String| format!("Frame {frame}: {} {error}", step.action);
            let value = &step.value;
            match step.action.as_str() {
                "Move" => output.events.push(Event::PointerMoved(point(value, ui).map_err(fail)?)),
                "Press" => {
                    let (at, button) = match value {
                        Value::Object(_) if value.get("Target").is_none() => {
                            (value.get("At").ok_or_else(|| fail("needs At".to_string()))?, button_of(value.get("Button")).map_err(fail)?)
                        }
                        other => (other, MouseButton::Left),
                    };
                    output.events.push(Event::PointerMoved(point(at, ui).map_err(fail)?));
                    output.events.push(Event::ButtonPressed(button));
                }
                "Release" => {
                    let button = match value {
                        Value::Bool(true) => MouseButton::Left,
                        other => button_of(Some(other)).map_err(fail)?,
                    };
                    output.events.push(Event::ButtonReleased(button));
                }
                "Click" | "DoubleClick" => {
                    output.events.push(Event::PointerMoved(point(value, ui).map_err(fail)?));
                    output.events.push(Event::ButtonPressed(MouseButton::Left));
                    if step.action == "Click" {
                        self.schedule(frame + 1, Event::ButtonReleased(MouseButton::Left));
                    } else {
                        output.events.push(Event::ButtonReleased(MouseButton::Left));
                        self.schedule(frame + 1, Event::ButtonPressed(MouseButton::Left));
                        self.schedule(frame + 1, Event::ButtonReleased(MouseButton::Left));
                    }
                }
                "Drag" => {
                    let from = point(value.get("From").ok_or_else(|| fail("needs From".to_string()))?, ui).map_err(fail)?;
                    let to = point(value.get("To").ok_or_else(|| fail("needs To".to_string()))?, ui).map_err(fail)?;
                    let frames = match value.get("Frames") {
                        None => 10,
                        Some(Value::Number(count)) if *count >= 1.0 => *count as u64,
                        Some(other) => return Err(fail(format!("has Frames of {}, which must be at least 1", other.describe()))),
                    };
                    let button = button_of(value.get("Button")).map_err(fail)?;
                    output.events.push(Event::PointerMoved(from));
                    output.events.push(Event::ButtonPressed(button));
                    for index in 1..=frames {
                        self.schedule(frame + index, Event::PointerMoved(from.lerp(to, index as f32 / frames as f32)));
                    }
                    self.schedule(frame + frames + 1, Event::ButtonReleased(button));
                }
                "Scroll" => {
                    let at = point(value.get("At").ok_or_else(|| fail("needs At".to_string()))?, ui).map_err(fail)?;
                    let notches = number(value.get("Notches")).ok_or_else(|| fail("needs Notches, a number".to_string()))?;
                    output.events.push(Event::PointerMoved(at));
                    output.events.push(Event::Wheel(Point::new(0.0, notches as f32)));
                }
                "Key" => {
                    let Value::String(name) = value else {
                        return Err(fail("needs a key's name, like \"F11\" or \"Ctrl+S\"".to_string()));
                    };
                    let shortcut = Shortcut::parse(name).ok_or_else(|| fail(format!("names no key: {name}")))?;
                    let mut keys = Vec::new();
                    for (held, key) in [(shortcut.modifiers.control, Key::CONTROL), (shortcut.modifiers.shift, Key::SHIFT), (shortcut.modifiers.alt, Key::ALT)] {
                        if held {
                            keys.push(key);
                        }
                    }
                    keys.push(shortcut.key);
                    let pressed: Vec<Key> = keys.iter().copied().filter(|key| !self.held.contains(key)).collect();
                    for key in &pressed {
                        output.events.push(Event::KeyPressed { key: *key, repeat: false });
                    }
                    for key in pressed.iter().rev() {
                        self.schedule(frame + 1, Event::KeyReleased(*key));
                    }
                }
                "Hold" => {
                    let Value::Array(names) = value else {
                        return Err(fail("needs a list of keys".to_string()));
                    };
                    let mut keys = Vec::new();
                    for name in names {
                        let Value::String(name) = name else {
                            return Err(fail(format!("has {}, which isn't a key's name", name.describe())));
                        };
                        keys.push(Key::parse(name).ok_or_else(|| fail(format!("names no key: {name}")))?);
                    }
                    for key in self.held.iter().filter(|key| !keys.contains(key)) {
                        output.events.push(Event::KeyReleased(*key));
                    }
                    for key in keys.iter().filter(|key| !self.held.contains(key)) {
                        output.events.push(Event::KeyPressed { key: *key, repeat: false });
                    }
                    self.held = keys;
                }
                "Type" => {
                    let Value::String(text) = value else {
                        return Err(fail("needs text".to_string()));
                    };
                    output.events.extend(text.chars().map(Event::Text));
                }
                "Gamepad" => self.gamepad = parse_gamepad(value).map_err(fail)?,
                "Focus" => match value {
                    Value::Bool(true) => output.events.push(Event::FocusGained(self.held.clone())),
                    Value::Bool(false) => output.events.push(Event::FocusLost),
                    other => return Err(fail(format!("must be true or false, not {}", other.describe()))),
                },
                "Capture" => {
                    let Value::String(name) = value else {
                        return Err(fail("needs a name for the files".to_string()));
                    };
                    output.captures.push(name.clone());
                }
                "Expect" => output.expectations.push(value.clone()),
                "Program" => output.program_steps.push(value.clone()),
                _ => unreachable!("the actions were checked as the script was parsed"),
            }
        }
        // The gamepad reads every frame, like a real one.
        output.events.push(Event::Gamepad(self.gamepad));
        Ok(output)
    }

    fn schedule(&mut self, frame: u64, event: Event) {
        self.scheduled.entry(frame).or_default().push(event);
    }

    // Checks an expectation against the state, and remembers what didn't
    // match.
    pub fn check(&mut self, frame: u64, expected: &Value, state: &Value) {
        let mut mismatches = Vec::new();
        compare(expected, state, &mut String::new(), &mut mismatches);
        self.failures.extend(mismatches.into_iter().map(|mismatch| format!("Frame {frame}: {mismatch}")));
    }
}

// A point: a target's name, which is its middle; [x, y] in the window; or
// { "Target": name, "At": [x, y] }, a place in a target as fractions of its
// size from its top left, like [0.25, 0.5] for a quarter of the way along a
// slider.
fn point(value: &Value, ui: &Interface) -> Result<Point, String> {
    let target = |name: &str| {
        ui.target(name).map(|target| target.rect).ok_or_else(|| {
            let names: Vec<&str> = ui.targets().iter().filter(|target| target.listed).map(|target| target.name.as_str()).collect();
            format!("points at {name}, which the last frame didn't show. It showed: {}", names.join(", "))
        })
    };
    match value {
        Value::String(name) => Ok(target(name)?.center()),
        Value::Array(items) => match items.as_slice() {
            [Value::Number(x), Value::Number(y)] => Ok(Point::new(*x as f32, *y as f32)),
            _ => Err("needs [x, y]".to_string()),
        },
        Value::Object(_) => {
            let Some(Value::String(name)) = value.get("Target") else {
                return Err("needs a Target's name with its At".to_string());
            };
            let Some(Value::Array(fractions)) = value.get("At") else {
                return Err("needs At, as [x, y] fractions of the target".to_string());
            };
            let [Value::Number(x), Value::Number(y)] = fractions.as_slice() else {
                return Err("needs At, as [x, y] fractions of the target".to_string());
            };
            Ok(target(name)?.at_fraction(Point::new(*x as f32, *y as f32)))
        }
        other => Err(format!("needs a target's name or [x, y], not {}", other.describe())),
    }
}

fn number(value: Option<&Value>) -> Option<f64> {
    match value {
        Some(Value::Number(number)) => Some(*number),
        _ => None,
    }
}

fn button_of(value: Option<&Value>) -> Result<MouseButton, String> {
    match value {
        None => Ok(MouseButton::Left),
        Some(Value::String(name)) => MouseButton::parse(name).ok_or_else(|| format!("names no button: {name}")),
        Some(other) => Err(format!("has a Button of {}, which isn't Left, Right or Middle", other.describe())),
    }
}

// A gamepad from a script's Gamepad step.
pub fn parse_gamepad(value: &Value) -> Result<Gamepad, String> {
    let mut gamepad = Gamepad {
        connected: !matches!(value.get("Connected"), Some(Value::Bool(false))),
        ..Gamepad::default()
    };
    let stick = |name: &str| -> Result<Point, String> {
        match value.get(name) {
            None => Ok(Point::ZERO),
            Some(Value::Array(items)) => match items.as_slice() {
                [Value::Number(x), Value::Number(y)] => Ok(Point::new(*x as f32, *y as f32)),
                _ => Err(format!("has {name}, which must be [x, y]")),
            },
            Some(other) => Err(format!("has {name} of {}, which must be [x, y]", other.describe())),
        }
    };
    gamepad.left_stick = stick("LeftStick")?;
    gamepad.right_stick = stick("RightStick")?;
    gamepad.left_trigger = number(value.get("LeftTrigger")).unwrap_or(0.0) as f32;
    gamepad.right_trigger = number(value.get("RightTrigger")).unwrap_or(0.0) as f32;
    if let Some(buttons) = value.get("Buttons") {
        let Value::Array(names) = buttons else {
            return Err("has Buttons, which must be a list".to_string());
        };
        for name in names {
            let Value::String(name) = name else {
                return Err(format!("has a button of {}, which isn't a name", name.describe()));
            };
            gamepad.buttons.0 |= GamepadButtons::parse(name).ok_or_else(|| format!("names no gamepad button: {name}"))?;
        }
    }
    Ok(gamepad)
}

// Whether `actual` has everything that `expected` has: objects match if each
// member that `expected` names matches, lists match item by item, and
// numbers match to a millionth. Each mismatch says where it is.
pub fn compare(expected: &Value, actual: &Value, path: &mut String, mismatches: &mut Vec<String>) {
    match (expected, actual) {
        (Value::Object(members), Value::Object(_)) => {
            for (name, value) in members {
                let length = path.len();
                if !path.is_empty() {
                    path.push('.');
                }
                path.push_str(name);
                match actual.get(name) {
                    Some(actual_value) => compare(value, actual_value, path, mismatches),
                    None => mismatches.push(format!("{path} is missing")),
                }
                path.truncate(length);
            }
        }
        (Value::Array(expected_items), Value::Array(actual_items)) if expected_items.len() == actual_items.len() => {
            for (index, (expected_item, actual_item)) in expected_items.iter().zip(actual_items).enumerate() {
                let length = path.len();
                path.push_str(&format!("[{index}]"));
                compare(expected_item, actual_item, path, mismatches);
                path.truncate(length);
            }
        }
        (Value::Number(expected), Value::Number(actual)) => {
            if (expected - actual).abs() > 1e-6 * expected.abs().max(1.0) {
                mismatches.push(format!("{path} is {actual}, not {expected}"));
            }
        }
        _ => {
            if expected != actual {
                mismatches.push(format!("{path} is {}, not {}", actual.to_json().replace('\n', " "), expected.to_json().replace('\n', " ")));
            }
        }
    }
}
