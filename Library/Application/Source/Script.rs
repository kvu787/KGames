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
//             { "Frame": 5, "Gamepad": { "RightStick": [0, 1], "Buttons": ["A"] } },
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
// "Gamepad" says where the gamepad's sticks and triggers are, and which of
// its buttons are held, from its frame until the next "Gamepad". What it
// leaves out is at rest, and null disconnects the gamepad. "Program" steps
// go to the program as they are, for input that doesn't come from the
// window or the gamepad, like other GameInput readings.
//
// Instead of a point, a step can name a widget, which is where the frame
// before drew it, at its middle, or at fractions of the way across and down
// it, so that scripts don't depend on where widgets are:
//
//     { "Frame": 1, "Press": "Volume" },
//     { "Frame": 2, "Release": { "Target": "Volume", "At": [0.75, 0.5] } },
//     { "Frame": 3, "Click": "Save" }
//
// "Click" presses and releases in the same frame, for a widget that a press
// takes away, like a choice in a combo box's list.

use std::path::{Path, PathBuf};

use json::Value;

use crate::{GamepadButton, Event, Gamepad};

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
    pub gamepad: Gamepad,
    pub checkpoints: Vec<Checkpoint>,
    pub program_steps: Vec<Value>,
    // Steps that named a widget that the frame before didn't draw.
    pub failures: Vec<String>,
}

// Where a step's pointer goes: a point in the client area, or a place in a
// widget, at fractions of the way across and down it.
#[derive(Clone, Debug, PartialEq)]
pub enum Place {
    At([f32; 2]),
    Target { name: String, at: [f32; 2] },
}

// What a step does with the pointer.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Pointer {
    Move,
    Press,
    Release,
    Click,
    Wheel(f32),
}

enum Action {
    Pointer(Pointer, Place),
    Event(Event),
    Hold(Vec<u32>),
    Gamepad(Gamepad),
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
    gamepad: Gamepad,
    // The gamepad's buttons held in the frame before.
    buttons_before: u32,
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
            gamepad: Gamepad::default(),
            buttons_before: 0,
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

    // Moves to the next frame, and returns what happens in it. `target`
    // says where the frame before drew the widget of a name, as its left,
    // top, width and height, for steps that name one.
    pub fn next_frame(&mut self, target: impl Fn(&str) -> Option<[f32; 4]>) -> ScriptFrame {
        let mut frame = ScriptFrame {
            seconds: self.frame as f64 * self.frame_seconds,
            events: Vec::new(),
            cursor: None,
            held_keys: Vec::new(),
            gamepad: Gamepad::default(),
            checkpoints: Vec::new(),
            program_steps: Vec::new(),
            failures: Vec::new(),
        };
        while let Some(step) = self.steps.get(self.next)
            && step.frame == self.frame
        {
            match &step.action {
                Action::Pointer(pointer, place) => {
                    let point = match place {
                        Place::At(point) => Some(*point),
                        Place::Target { name, at: [across, down] } => match target(name) {
                            Some([x, y, width, height]) => Some([x + width * across, y + height * down]),
                            None => {
                                frame.failures.push(format!("there's no widget called {name} to point at"));
                                None
                            }
                        },
                    };
                    if let Some(point) = point {
                        // Like the window's messages, a click moves the
                        // cursor to where it is.
                        self.cursor = Some(point);
                        match pointer {
                            Pointer::Move => {}
                            Pointer::Press => frame.events.push(Event::Press(point)),
                            Pointer::Release => frame.events.push(Event::Release(point)),
                            Pointer::Click => frame.events.extend([Event::Press(point), Event::Release(point)]),
                            Pointer::Wheel(notches) => frame.events.push(Event::Wheel(point, *notches)),
                        }
                    }
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
                Action::Gamepad(gamepad) => self.gamepad = *gamepad,
                Action::Event(event) => frame.events.push(*event),
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
        frame.gamepad = self.gamepad.after(self.buttons_before);
        self.buttons_before = self.gamepad.buttons;
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
            "needs one of Move, Press, Release, Click, Wheel, Key, Hold, Gamepad, Text, Capture, Expect and Program"
                .to_string(),
        );
    };
    let action = match name.as_str() {
        "Move" => Action::Pointer(Pointer::Move, place(value)?),
        "Press" => Action::Pointer(Pointer::Press, place(value)?),
        "Release" => Action::Pointer(Pointer::Release, place(value)?),
        "Click" => Action::Pointer(Pointer::Click, place(value)?),
        "Wheel" => {
            let notches = match item.get("Notches") {
                None => 1.0,
                Some(Value::Number(notches)) => *notches as f32,
                Some(other) => return Err(format!("Notches must be a number, not {}", other.describe())),
            };
            Action::Pointer(Pointer::Wheel(notches), place(value)?)
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
        "Gamepad" => Action::Gamepad(gamepad(value).map_err(|problem| format!("Gamepad's {problem}"))?),
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

// A gamepad that's connected, with its sticks, triggers and buttons as a
// Gamepad step says, or one that isn't, from null.
fn gamepad(value: &Value) -> Result<Gamepad, String> {
    let members = match value {
        Value::Null => return Ok(Gamepad::default()),
        Value::Object(members) => members,
        other => return Err(format!("state must be an object, or null, not {}", other.describe())),
    };
    let mut gamepad = Gamepad {
        connected: true,
        ..Gamepad::default()
    };
    for (name, value) in members {
        let stick = || match value {
            Value::Array(items) => match items.as_slice() {
                [Value::Number(x), Value::Number(y)] if x.abs() <= 1.0 && y.abs() <= 1.0 => {
                    Ok([*x as f32, *y as f32])
                }
                _ => Err(format!("{name} must be [x, y], each from -1 to 1, not {}", value.describe())),
            },
            other => Err(format!("{name} must be [x, y], each from -1 to 1, not {}", other.describe())),
        };
        let trigger = || match value {
            Value::Number(number) if (0.0..=1.0).contains(number) => Ok(*number as f32),
            other => Err(format!("{name} must be a number from 0 to 1, not {}", other.describe())),
        };
        match name.as_str() {
            "LeftStick" => gamepad.left_stick = stick()?,
            "RightStick" => gamepad.right_stick = stick()?,
            "LeftTrigger" => gamepad.left_trigger = trigger()?,
            "RightTrigger" => gamepad.right_trigger = trigger()?,
            "Buttons" => {
                let Value::Array(names) = value else {
                    return Err(format!("Buttons must be a list of buttons, like [\"A\"], not {}", value.describe()));
                };
                for name in names {
                    let button = match name {
                        Value::String(text) => GamepadButton::from_name(text),
                        _ => None,
                    };
                    let Some(button) = button else {
                        let names: Vec<&str> = GamepadButton::ALL.iter().map(|button| button.name()).collect();
                        return Err(format!("{} isn't a button. They're {}", name.to_json(), names.join(", ")));
                    };
                    gamepad.buttons |= button.bit();
                }
            }
            _ => {
                return Err(format!(
                    "{name} isn't part of a gamepad. They're LeftStick, RightStick, LeftTrigger, RightTrigger and Buttons"
                ));
            }
        }
    }
    Ok(gamepad)
}

// A point, [x, y], or a widget's name, or an object with the widget's name
// as "Target", and where in it as "At", in fractions across and down it.
fn place(value: &Value) -> Result<Place, String> {
    match value {
        Value::String(name) if !name.is_empty() => Ok(Place::Target {
            name: name.clone(),
            at: [0.5, 0.5],
        }),
        Value::Object(_) => {
            let Some(Value::String(name)) = value.get("Target") else {
                return Err(format!("a place in a widget needs its name as Target, not {}", value.describe()));
            };
            let at = match value.get("At") {
                None => [0.5, 0.5],
                Some(at) => point(at)?,
            };
            Ok(Place::Target { name: name.clone(), at })
        }
        _ => Ok(Place::At(point(value)?)),
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

// The keys with names of their own, and their virtual-key codes.
const NAMED_KEYS: [(&str, u32); 17] = [
    ("Backspace", 0x08),
    ("Tab", 0x09),
    ("Enter", 0x0D),
    ("Shift", 0x10),
    ("Control", 0x11),
    ("Alt", 0x12),
    ("Escape", 0x1B),
    ("Space", 0x20),
    ("Left", 0x25),
    ("Up", 0x26),
    ("Right", 0x27),
    ("Down", 0x28),
    ("PageUp", 0x21),
    ("PageDown", 0x22),
    ("End", 0x23),
    ("Home", 0x24),
    ("Delete", 0x2E),
];

// A key's virtual-key code, from its name, like "F11", "Left", "Home",
// "Escape", "Space", "S" or "7".
pub fn virtual_key(name: &str) -> Option<u32> {
    if let Some((_, key)) = NAMED_KEYS.iter().find(|(named, _)| *named == name) {
        return Some(*key);
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

// A key's name, as virtual_key reads it, or its code in hexadecimal, like
// "0xBA", if it has none.
pub fn key_name(key: u32) -> String {
    if let Some((name, _)) = NAMED_KEYS.iter().find(|(_, named)| *named == key) {
        return name.to_string();
    }
    match key {
        0x70..=0x87 => format!("F{}", key - 0x70 + 1),
        0x30..=0x39 | 0x41..=0x5A => char::from_u32(key).unwrap().to_string(),
        _ => format!("0x{key:02X}"),
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
