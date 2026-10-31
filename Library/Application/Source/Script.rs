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
// step is done, the program exits, with an error if its state differed.
//
//     {
//         "FrameSeconds": 0.01,
//         "Steps": [
//             { "Frame": 1, "Move": [1400, 300] },
//             { "Frame": 2, "Press": [1400, 300] },
//             { "Frame": 3, "Release": [1800, 300] },
//             { "Frame": 4, "Wheel": [1400, 300], "Notches": -2 },
//             { "Frame": 5, "Key": "S", "Control": true },
//             { "Frame": 6, "Capture": "AfterSave" },
//             { "Frame": 6, "Expect": { "Window": { "Fullscreen": false } } },
//             { "Frame": 7, "Program": { "Anything": "the program reads" } }
//         ]
//     }
//
// Frames count from 0. Clicks hit what the frame before them drew, so they
// start at frame 1. "Program" steps go to the program as they are, for
// input that doesn't come from the window, like GameInput readings.

use std::path::{Path, PathBuf};

use json::Value;

use crate::Event;

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
    pub checkpoints: Vec<Checkpoint>,
    pub program_steps: Vec<Value>,
}

enum Action {
    Move([f32; 2]),
    Event(Event),
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
            steps.push(step);
        }
        Ok(Script {
            steps,
            next: 0,
            frame: 0,
            frame_seconds,
            output: output.to_path_buf(),
            cursor: None,
        })
    }

    // Whether every step is done, so the program can exit.
    pub fn finished(&self) -> bool {
        self.next == self.steps.len()
    }

    // Moves to the next frame, and returns what happens in it.
    pub fn next_frame(&mut self) -> ScriptFrame {
        let mut frame = ScriptFrame {
            seconds: self.frame as f64 * self.frame_seconds,
            events: Vec::new(),
            cursor: None,
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
            "needs one of Move, Press, Release, Wheel, Key, Capture, Expect and Program".to_string(),
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
        "Key" => {
            let key = match value {
                Value::String(name) => virtual_key(name).ok_or_else(|| format!("{name} isn't a key"))?,
                Value::Number(code) if (1.0..255.0).contains(code) => *code as u32,
                other => return Err(format!("Key must be a key's name, like \"F11\", not {}", other.describe())),
            };
            Action::Event(Event::Key {
                key,
                control: flag("Control")?,
                shift: flag("Shift")?,
                repeat: flag("Repeat")?,
            })
        }
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

fn point(value: &Value) -> Result<[f32; 2], String> {
    match value {
        Value::Array(items) => match items.as_slice() {
            [Value::Number(x), Value::Number(y)] => Ok([*x as f32, *y as f32]),
            _ => Err(format!("a point must be [x, y], not {}", value.describe())),
        },
        _ => Err(format!("a point must be [x, y], not {}", value.describe())),
    }
}

// A key's virtual-key code, from its name, like "F11", "Left", "Escape",
// "Space", "S" or "7".
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
        "Delete" => Some(0x2E),
        _ => None,
    };
    if named.is_some() {
        return named;
    }
    if let Some(number) = name.strip_prefix('F').and_then(|digits| digits.parse::<u32>().ok()) {
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
// Numbers match within a billionth of their size, for rounding.
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
            let scale = expected_number.abs().max(actual_number.abs()).max(1.0);
            if (expected_number - actual_number).abs() > 1e-9 * scale {
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
