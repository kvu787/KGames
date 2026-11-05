// A script of input for a program, so that a coding agent can test how the
// program handles input without the user's mouse and keyboard, which belong
// to the window that the user works in. RunInBackground passes one with
// --script, and a folder for what it makes with --output.
//
// While a script runs, the program sees only the script's input: input from
// the window's messages is dropped, and the window acts as if it had focus.
// Time is a fixed clock, FrameSeconds a frame, so every run draws the same
// frames. At checkpoints, the program saves what it drew and its state, or
// checks its state against what the script expects. Once the script's last
// step is done, the program exits, with an error if its state differed. If
// it exits before that, like when its window is closed, that's an error too.
//
//     {
//         "FrameSeconds": 0.01,
//         "Steps": [
//             { "Frame": 1, "Move": [1400, 300] },
//             { "Frame": 2, "Press": [1400, 300] },
//             { "Frame": 3, "Release": [1800, 300] },
//             { "Frame": 4, "Wheel": [1400, 300], "Notches": -2 },
//             { "Frame": 5, "Key": "S", "Control": true },
//             { "Frame": 5, "Text": "Hello" },
//             { "Frame": 5, "Hold": ["Up", "Space"] },
//             { "Frame": 6, "Capture": "AfterSave" },
//             { "Frame": 6, "Expect": { "Window": { "Fullscreen": false } } },
//             { "Frame": 7, "Program": { "Anything": "the program reads" } }
//         ]
//     }
//
// Frames count from 0. Clicks hit what the frame before them drew, so they
// start at frame 1. "Key" presses a key, which doesn't stay held. "Hold"
// says which keys are held down from its frame until the next "Hold": each
// that wasn't held already is pressed then, and the others let go of. "Text"
// types characters, as Character events, without the keys that type them.
// "Program" steps go to the program as they are, for input that doesn't
// come from the window, like GameInput readings.

use std::path::{Path, PathBuf};

use json::Value;

use crate::Event;

const VK_SHIFT: u32 = 0x10;
const VK_CONTROL: u32 = 0x11;

// What a program does at a checkpoint.
pub enum Checkpoint {
    // Saves the frame as a PNG at `image`, and the program's state as JSON
    // at `state`.
    Capture { image: PathBuf, state: PathBuf },
    // Checks the program's state against this, which lists only the parts
    // that matter. See compare.
    Expect(Value),
}

// What a script makes happen in one frame.
pub struct ScriptFrame {
    pub seconds: f64,
    pub events: Vec<Event>,
    pub cursor: Option<[f32; 2]>,
    pub held_keys: Vec<u32>,
    pub checkpoints: Vec<Checkpoint>,
    pub program_steps: Vec<Value>,
}

enum Action {
    Move([f32; 2]),
    Event(Event),
    Hold(Vec<u32>),
    Text(String),
    Capture(String),
    Expect(Value),
    Program(Value),
}

struct Step {
    frame: u64,
    action: Action,
}

pub struct Script {
    steps: Vec<Step>,
    // The first step not done yet.
    next: usize,
    // The next frame's number.
    frame: u64,
    frame_seconds: f64,
    output: PathBuf,
    cursor: Option<[f32; 2]>,
    // The keys held down, in the order they went down.
    held_keys: Vec<u32>,
}

impl Script {
    pub fn load(path: &Path, output: &Path) -> Result<Script, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("Couldn't read the script {}: {error}", path.display()))?;
        Script::parse(&text, output).map_err(|problem| format!("{}: {problem}", path.display()))
    }

    pub fn parse(text: &str, output: &Path) -> Result<Script, String> {
        let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
        let value = json::parse(text).map_err(|error| {
            format!("line {}, column {}: {}", error.line, error.column, error.message)
        })?;
        let frame_seconds = match value.get("FrameSeconds") {
            None => 1.0 / 60.0,
            Some(Value::Number(seconds)) if *seconds > 0.0 && seconds.is_finite() => *seconds,
            Some(other) => {
                return Err(format!(
                    "FrameSeconds must be a number of seconds above 0, not {}",
                    other.describe()
                ));
            }
        };
        let Some(Value::Array(items)) = value.get("Steps") else {
            return Err("a script needs \"Steps\", a list of steps".to_string());
        };
        let mut steps = Vec::new();
        for (index, item) in items.iter().enumerate() {
            let step = parse_step(item).map_err(|problem| format!("step {}: {problem}", index + 1))?;
            if steps.last().is_some_and(|last: &Step| last.frame > step.frame) {
                return Err(format!("step {} comes before the step above it", index + 1));
            }
            // A frame saves one PNG, and each capture has its own files.
            if let Action::Capture(name) = &step.action {
                for other in &steps {
                    let Action::Capture(other_name) = &other.action else {
                        continue;
                    };
                    if other.frame == step.frame {
                        return Err(format!("step {}: frame {} has a Capture already", index + 1, step.frame));
                    }
                    if other_name == name {
                        return Err(format!("step {}: another Capture is called {name}", index + 1));
                    }
                }
            }
            steps.push(step);
        }
        Ok(Script {
            steps,
            next: 0,
            frame: 0,
            frame_seconds,
            output: output.to_path_buf(),
            cursor: None,
            held_keys: Vec::new(),
        })
    }

    // Whether every step is done, so the program can exit.
    pub fn finished(&self) -> bool {
        self.next == self.steps.len()
    }

    // The frame of the first step that isn't done yet, if any.
    pub fn pending_frame(&self) -> Option<u64> {
        self.steps.get(self.next).map(|step| step.frame)
    }

    // Moves to the next frame, and returns what happens in it.
    pub fn next_frame(&mut self) -> ScriptFrame {
        let mut frame = ScriptFrame {
            seconds: self.frame as f64 * self.frame_seconds,
            events: Vec::new(),
            cursor: None,
            held_keys: Vec::new(),
            checkpoints: Vec::new(),
            program_steps: Vec::new(),
        };
        while let Some(step) = self.steps.get(self.next)
            && step.frame == self.frame
        {
            match &step.action {
                Action::Move(point) => self.cursor = Some(*point),
                Action::Event(event) => {
                    // Like the window's messages, a click moves the cursor
                    // to where it is.
                    if let Event::Press(point) | Event::Release(point) | Event::Wheel(point, _) = event {
                        self.cursor = Some(*point);
                    }
                    frame.events.push(*event);
                }
                Action::Hold(keys) => {
                    // Like in the window, a key that goes down is pressed,
                    // with the keys held with it.
                    for &key in keys.iter().filter(|key| !self.held_keys.contains(key)) {
                        frame.events.push(Event::Key {
                            key,
                            control: keys.contains(&VK_CONTROL),
                            shift: keys.contains(&VK_SHIFT),
                            repeat: false,
                        });
                    }
                    self.held_keys = keys.clone();
                }
                Action::Text(text) => frame.events.extend(text.chars().map(Event::Character)),
                Action::Capture(name) => frame.checkpoints.push(Checkpoint::Capture {
                    image: self.output.join(format!("{name}.png")),
                    state: self.output.join(format!("{name}.json")),
                }),
                Action::Expect(expected) => frame.checkpoints.push(Checkpoint::Expect(expected.clone())),
                Action::Program(value) => frame.program_steps.push(value.clone()),
            }
            self.next += 1;
        }
        frame.cursor = self.cursor;
        frame.held_keys = self.held_keys.clone();
        self.frame += 1;
        frame
    }
}

fn parse_step(item: &Value) -> Result<Step, String> {
    let Value::Object(members) = item else {
        return Err(format!("must be an object, not {}", item.describe()));
    };
    let frame = match item.get("Frame") {
        Some(Value::Number(frame)) if *frame >= 0.0 && frame.fract() == 0.0 => *frame as u64,
        Some(other) => return Err(format!("Frame must be a whole number, not {}", other.describe())),
        None => return Err("needs a Frame".to_string()),
    };
    let flag = |name: &str| match item.get(name) {
        None => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        Some(other) => Err(format!("{name} must be true or false, not {}", other.describe())),
    };
    let actions: Vec<&(String, Value)> = members
        .iter()
        .filter(|(name, _)| !matches!(name.as_str(), "Frame" | "Notches" | "Control" | "Shift" | "Repeat"))
        .collect();
    let [(name, value)] = actions.as_slice() else {
        return Err(
            "needs one of Move, Press, Release, Wheel, Key, Hold, Text, Capture, Expect and Program".to_string(),
        );
    };
    let action = match name.as_str() {
        "Move" => Action::Move(point(value)?),
        "Press" => Action::Event(Event::Press(point(value)?)),
        "Release" => Action::Event(Event::Release(point(value)?)),
        "Wheel" => {
            let notches = match item.get("Notches") {
                None => 1.0,
                Some(Value::Number(notches)) => *notches as f32,
                Some(other) => return Err(format!("Notches must be a number, not {}", other.describe())),
            };
            Action::Event(Event::Wheel(point(value)?, notches))
        }
        "Key" => Action::Event(Event::Key {
            key: key(value, "Key")?,
            control: flag("Control")?,
            shift: flag("Shift")?,
            repeat: flag("Repeat")?,
        }),
        "Hold" => {
            let Value::Array(names) = value else {
                return Err(format!(
                    "Hold must be a list of the keys held down, like [\"Up\", \"Space\"], not {}",
                    value.describe()
                ));
            };
            let mut keys = Vec::new();
            for name in names {
                let code = key(name, "Each of Hold's keys")?;
                if keys.contains(&code) {
                    return Err(format!("Hold has {} twice", name.to_json()));
                }
                keys.push(code);
            }
            Action::Hold(keys)
        }
        "Text" => match value {
            Value::String(text) if !text.is_empty() && !text.chars().any(char::is_control) => {
                Action::Text(text.clone())
            }
            other => {
                return Err(format!(
                    "Text must be characters to type, without control characters, not {}",
                    other.describe()
                ));
            }
        },
        "Capture" => match value {
            Value::String(name)
                if !name.is_empty() && name.chars().all(|character| character.is_ascii_alphanumeric()) =>
            {
                Action::Capture(name.clone())
            }
            other => {
                return Err(format!(
                    "Capture must be a name of letters and digits, for its files, not {}",
                    other.describe()
                ));
            }
        },
        "Expect" => Action::Expect(value.clone()),
        "Program" => Action::Program(value.clone()),
        other => return Err(format!("{other} isn't a step")),
    };
    Ok(Step { frame, action })
}

// A key's virtual-key code, from its name, or the code itself. `what` is
// what the value is, for the message if it's neither.
fn key(value: &Value, what: &str) -> Result<u32, String> {
    match value {
        Value::String(name) => virtual_key(name).ok_or_else(|| format!("{name} isn't a key")),
        Value::Number(code) if (1.0..255.0).contains(code) && code.fract() == 0.0 => Ok(*code as u32),
        other => Err(format!("{what} must be a key's name, like \"F11\", not {}", other.describe())),
    }
}

fn point(value: &Value) -> Result<[f32; 2], String> {
    match value {
        Value::Array(items) => match items.as_slice() {
            [Value::Number(x), Value::Number(y)] => Ok([*x as f32, *y as f32]),
            _ => Err(format!("a point must be [x, y], not {}", value.describe())),
        },
        _ => Err(format!("a point must be [x, y], not {}", value.describe())),
    }
}

// A key's virtual-key code, from its name, like "F11", "Left", "Home",
// "Escape", "Space", "S" or "7".
pub fn virtual_key(name: &str) -> Option<u32> {
    let named = match name {
        "Backspace" => Some(0x08),
        "Tab" => Some(0x09),
        "Enter" => Some(0x0D),
        "Shift" => Some(0x10),
        "Control" => Some(0x11),
        "Alt" => Some(0x12),
        "Escape" => Some(0x1B),
        "Space" => Some(0x20),
        "Left" => Some(0x25),
        "Up" => Some(0x26),
        "Right" => Some(0x27),
        "Down" => Some(0x28),
        "PageUp" => Some(0x21),
        "PageDown" => Some(0x22),
        "End" => Some(0x23),
        "Home" => Some(0x24),
        "Delete" => Some(0x2E),
        _ => None,
    };
    if named.is_some() {
        return named;
    }
    if let Some(number) = name
        .strip_prefix('F')
        .filter(|digits| !digits.is_empty() && digits.bytes().all(|digit| digit.is_ascii_digit()))
        .and_then(|digits| digits.parse::<u32>().ok())
    {
        return (1..=24).contains(&number).then(|| 0x70 + number - 1);
    }
    let mut characters = name.chars();
    match (characters.next(), characters.next()) {
        (Some(character), None) if character.is_ascii_uppercase() || character.is_ascii_digit() => {
            Some(character as u32)
        }
        _ => None,
    }
}

// How `actual` differs from `expected`, one difference per item, or nothing
// if it matches. Only what `expected` has matters: an object can have
// members that `expected` leaves out. Lists must have the same length.
// Numbers match within a billionth, for rounding, or within a billionth of
// the expected number's size if it's above 1. An infinite or NaN number in
// the state matches nothing, as a script's numbers are always finite.
pub fn compare(expected: &Value, actual: &Value) -> Vec<String> {
    let mut differences = Vec::new();
    compare_at("", expected, actual, &mut differences);
    differences
}

fn compare_at(path: &str, expected: &Value, actual: &Value, differences: &mut Vec<String>) {
    let at = if path.is_empty() { "the state" } else { path };
    match (expected, actual) {
        (Value::Object(members), Value::Object(_)) => {
            for (name, value) in members {
                let path = if path.is_empty() {
                    name.clone()
                } else {
                    format!("{path}.{name}")
                };
                match actual.get(name) {
                    Some(actual) => compare_at(&path, value, actual, differences),
                    None => differences.push(format!("{path} is missing")),
                }
            }
        }
        (Value::Array(expected_items), Value::Array(actual_items)) => {
            if expected_items.len() != actual_items.len() {
                differences.push(format!(
                    "{at} has {} items, not {}: {}",
                    actual_items.len(),
                    expected_items.len(),
                    actual.to_json().replace('\n', " ")
                ));
                return;
            }
            for (index, (expected, actual)) in expected_items.iter().zip(actual_items).enumerate() {
                compare_at(&format!("{path}[{index}]"), expected, actual, differences);
            }
        }
        (Value::Number(expected_number), Value::Number(actual_number)) => {
            let tolerance = 1e-9 * expected_number.abs().max(1.0);
            if !actual_number.is_finite() || (expected_number - actual_number).abs() > tolerance {
                differences.push(format!("{at} is {actual_number}, not {expected_number}"));
            }
        }
        _ if expected == actual => {}
        _ => differences.push(format!(
            "{at} is {}, not {}",
            actual.to_json().replace('\n', " "),
            expected.to_json().replace('\n', " ")
        )),
    }
}
