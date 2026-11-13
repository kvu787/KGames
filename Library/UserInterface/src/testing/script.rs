// Scripts: input from a file instead of the user, on a fixed clock, which
// tests a program in its real window without touching the user's input or
// focus. RunInBackground runs them, and the README describes the format.
//
//     {
//       "FrameTime": 0.016666666666666666,
//       "Steps": [
//         { "Frame": 1, "Click": "Save" },
//         { "Frame": 3, "Drag": { "From": "Volume", "To": [400, 300], "Frames": 10 } },
//         { "Frame": 20, "Key": "Ctrl+S" },
//         { "Frame": 22, "Capture": "Saved" },
//         { "Frame": 22, "Expect": { "Saved": true, "Interface": { "Popup": null } } }
//       ]
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
use crate::testing::gestures::{self, Frames};

pub const DEFAULT_FRAME_TIME: f64 = 1.0 / 60.0;

const ACTIONS: [&str; 15] = [
    "Move", "Press", "Release", "Click", "DoubleClick", "Drag", "Scroll", "Key", "Hold", "Type", "Gamepad", "Focus", "Capture", "Expect", "Program",
];

// Where a step points: a target's middle, by its name; a place in the
// window; or a place in a target, as fractions of its size from its top
// left, like [0.25, 0.5] for a quarter of the way along a slider. Names are
// found in the frame before the step's.
#[derive(Clone, Debug)]
enum Place {
    Target(String),
    Window(Point),
    InTarget(String, Point),
}

// What a step does, as its script says.
#[derive(Clone, Debug)]
enum Action {
    Move(Place),
    Press(Place, MouseButton),
    Release(MouseButton),
    Click(Place),
    DoubleClick(Place),
    Drag { from: Place, to: Place, frames: usize, button: MouseButton },
    Scroll(Place, f32),
    Key(Shortcut),
    Hold(Vec<Key>),
    Type(String),
    Gamepad(Gamepad),
    Focus(bool),
    Capture(String),
    Expect(Value),
    Program(Value),
}

#[derive(Clone, Debug)]
struct Step {
    frame: u64,
    // The action's name, for what a failure says.
    name: String,
    action: Action,
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

    // Reads every step, so that one that's wrong says so before the run
    // starts.
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
            let [(name, value)] = actions.as_slice() else {
                return Err(format!("has step {index}, which must have one action besides its Frame"));
            };
            if !ACTIONS.contains(&name.as_str()) {
                return Err(format!("has step {index} with the action {name}, which isn't one of {}", ACTIONS.join(", ")));
            }
            let action = action(name, value).map_err(|error| format!("has step {index}, at frame {frame}, whose {name} {error}"))?;
            steps.push(Step { frame, name: name.clone(), action });
        }
        steps.sort_by_key(|step| step.frame);
        Ok(Script { frame_time, steps })
    }

    // The frame of the last step.
    pub fn last_frame(&self) -> u64 {
        self.steps.last().map_or(1, |step| step.frame)
    }
}

// The action that a step names, with its value.
fn action(name: &str, value: &Value) -> Result<Action, String> {
    let member = |name: &str| value.get(name).ok_or_else(|| format!("needs {name}"));
    Ok(match name {
        "Move" => Action::Move(place(value)?),
        "Press" => match value {
            Value::Object(_) if value.get("Target").is_none() => Action::Press(place(member("At")?)?, button_of(value.get("Button"))?),
            other => Action::Press(place(other)?, MouseButton::Left),
        },
        "Release" => Action::Release(match value {
            Value::Bool(true) => MouseButton::Left,
            other => button_of(Some(other))?,
        }),
        "Click" => Action::Click(place(value)?),
        "DoubleClick" => Action::DoubleClick(place(value)?),
        "Drag" => Action::Drag {
            from: place(member("From")?)?,
            to: place(member("To")?)?,
            frames: match value.get("Frames") {
                None => 10,
                Some(Value::Number(count)) if *count >= 1.0 => *count as usize,
                Some(other) => return Err(format!("has Frames of {}, which must be at least 1", other.describe())),
            },
            button: button_of(value.get("Button"))?,
        },
        "Scroll" => Action::Scroll(place(member("At")?)?, number(value.get("Notches")).ok_or("needs Notches, a number")? as f32),
        "Key" => {
            let Value::String(name) = value else {
                return Err("needs a key's name, like \"F11\" or \"Ctrl+S\"".to_string());
            };
            Action::Key(Shortcut::parse(name).ok_or_else(|| format!("names no key: {name}"))?)
        }
        "Hold" => {
            let Value::Array(names) = value else {
                return Err("needs a list of keys".to_string());
            };
            let keys = names.iter().map(|name| match name {
                Value::String(name) => Key::parse(name).ok_or_else(|| format!("names no key: {name}")),
                other => Err(format!("has {}, which isn't a key's name", other.describe())),
            });
            Action::Hold(keys.collect::<Result<_, _>>()?)
        }
        "Type" => match value {
            Value::String(text) => Action::Type(text.clone()),
            _ => return Err("needs text".to_string()),
        },
        "Gamepad" => Action::Gamepad(parse_gamepad(value)?),
        "Focus" => match value {
            Value::Bool(focused) => Action::Focus(*focused),
            other => return Err(format!("must be true or false, not {}", other.describe())),
        },
        "Capture" => match value {
            Value::String(name) => Action::Capture(name.clone()),
            _ => return Err("needs a name for the files".to_string()),
        },
        "Expect" => Action::Expect(value.clone()),
        "Program" => Action::Program(value.clone()),
        _ => unreachable!("the actions were checked first"),
    })
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
            let at = |place: &Place| point(place, ui).map_err(|error| format!("Frame {frame}: {} {error}", step.name));
            let gesture: Frames = match step.action {
                Action::Move(place) => vec![vec![Event::PointerMoved(at(&place)?)]],
                Action::Press(place, button) => gestures::press(at(&place)?, button),
                Action::Release(button) => vec![vec![Event::ButtonReleased(button)]],
                Action::Click(place) => gestures::click(at(&place)?, MouseButton::Left),
                Action::DoubleClick(place) => gestures::double_click(at(&place)?),
                Action::Drag { from, to, frames, button } => gestures::drag(at(&from)?, at(&to)?, frames, button),
                Action::Scroll(place, notches) => gestures::scroll(at(&place)?, notches),
                Action::Key(shortcut) => gestures::key(shortcut.modifiers, shortcut.key, &self.held),
                Action::Hold(keys) => {
                    let released = self.held.iter().filter(|key| !keys.contains(key)).map(|key| Event::KeyReleased(*key));
                    let pressed = keys.iter().filter(|key| !self.held.contains(key)).map(|key| Event::KeyPressed { key: *key, repeat: false });
                    let events = released.chain(pressed).collect();
                    self.held = keys;
                    vec![events]
                }
                Action::Type(text) => vec![text.chars().map(Event::Text).collect()],
                Action::Focus(true) => vec![vec![Event::FocusGained(self.held.clone())]],
                Action::Focus(false) => vec![vec![Event::FocusLost]],
                Action::Gamepad(gamepad) => {
                    self.gamepad = gamepad;
                    Vec::new()
                }
                Action::Capture(name) => {
                    output.captures.push(name);
                    Vec::new()
                }
                Action::Expect(expected) => {
                    output.expectations.push(expected);
                    Vec::new()
                }
                Action::Program(value) => {
                    output.program_steps.push(value);
                    Vec::new()
                }
            };
            // The gesture's first frame is this one, and the rest follow.
            for (index, events) in gesture.into_iter().enumerate() {
                match index {
                    0 => output.events.extend(events),
                    _ => self.scheduled.entry(frame + index as u64).or_default().extend(events),
                }
            }
        }
        // The gamepad reads every frame, like a real one.
        output.events.push(Event::Gamepad(self.gamepad));
        Ok(output)
    }

    // Checks an expectation against the state, and remembers what didn't
    // match.
    pub fn check(&mut self, frame: u64, expected: &Value, state: &Value) {
        let mut mismatches = Vec::new();
        compare(expected, state, &mut String::new(), &mut mismatches);
        self.failures.extend(mismatches.into_iter().map(|mismatch| format!("Frame {frame}: {mismatch}")));
    }
}

// A step's place, from its value: a target's name, [x, y] in the window, or
// { "Target": name, "At": [x, y] }.
fn place(value: &Value) -> Result<Place, String> {
    let pair = |items: &[Value]| match items {
        [Value::Number(x), Value::Number(y)] => Some(Point::new(*x as f32, *y as f32)),
        _ => None,
    };
    match value {
        Value::String(name) => Ok(Place::Target(name.clone())),
        Value::Array(items) => pair(items).map(Place::Window).ok_or_else(|| "needs [x, y]".to_string()),
        Value::Object(_) => {
            let Some(Value::String(name)) = value.get("Target") else {
                return Err("needs a Target's name with its At".to_string());
            };
            let fraction = match value.get("At") {
                Some(Value::Array(fractions)) => pair(fractions),
                _ => None,
            };
            fraction.map(|fraction| Place::InTarget(name.clone(), fraction)).ok_or_else(|| "needs At, as [x, y] fractions of the target".to_string())
        }
        other => Err(format!("needs a target's name or [x, y], not {}", other.describe())),
    }
}

// Where a place is in the window, with the targets that the last frame
// showed.
fn point(place: &Place, ui: &Interface) -> Result<Point, String> {
    let target = |name: &str| {
        ui.target(name).map(|target| target.rect).ok_or_else(|| {
            let names: Vec<&str> = ui.targets().iter().filter(|target| target.listed).map(|target| target.name.as_str()).collect();
            format!("points at {name}, which the last frame didn't show. It showed: {}", names.join(", "))
        })
    };
    match place {
        Place::Target(name) => Ok(target(name)?.center()),
        Place::Window(point) => Ok(*point),
        Place::InTarget(name, fraction) => Ok(target(name)?.at_fraction(*fraction)),
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
