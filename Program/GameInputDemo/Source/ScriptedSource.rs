// Devices and readings from a test, or from a script's "Program" steps (see
// Library/Application's Script.rs), instead of GameInput, so that the demo
// can be tested without a gamepad, and without the user's keyboard and mouse.
// Its readings are snapshots of a device's whole state, like GameInput's,
// and it keeps the rumble that the demo sends, for checking.
//
// A script's steps, each one's "Program" member:
//
//     { "Connect": { "Id": 1, "Name": "Pad", "Input": ["Gamepad"],
//                    "Motors": ["LowFrequency", "HighFrequency"] } }
//     { "Gamepad": { "Id": 1, "LeftStick": [0.5, 0], "RightStick": [0, 0],
//                    "LeftTrigger": 0, "RightTrigger": 1, "Buttons": ["A"] } }
//     { "Keyboard": { "Id": 2, "Down": ["F1", "A"] } }
//     { "Mouse": { "Id": 3, "Motion": [10, -5], "Wheel": [0, 120],
//                  "Buttons": ["Left"] } }
//     { "Disconnect": 1 }
//
// "Gamepad", "Keyboard" and "Mouse" give that input's whole state: what's
// left out is at rest, except that "Motion" and "Wheel" are added to the
// mouse's running totals, as a mouse moves. "Input" can also have
// "Keyboard" and "Mouse", and "Motors" "LeftTrigger" and "RightTrigger".

use application::virtual_key;
use game_input::*;
use json::Value;
use windows_api::{MAPVK_VK_TO_VSC_EX, MapVirtualKeyW};

use crate::devices::{Device, InputSource, Reading};
use crate::input::{GAMEPAD_BUTTONS, MOUSE_BUTTONS};

#[derive(Default)]
pub struct ScriptedSource {
    // The time now, in microseconds. The main loop sets it from the frame's.
    pub now: u64,
    connected: Vec<Device>,
    events: Vec<(GameInputDeviceEvent, Device)>,
    // Readings not handed out yet, by kind of input, oldest first.
    queues: Vec<(GameInputKind, Vec<Reading>)>,
    // Each device's newest reading, which the next one starts from.
    latest: Vec<Reading>,
    // Each gamepad's rumble, as the demo last set it.
    rumble: Vec<(usize, Option<GameInputRumbleParams>)>,
}

impl ScriptedSource {
    pub fn connect(&mut self, device: Device) {
        self.connected.retain(|known| known.id != device.id);
        self.connected.push(device.clone());
        self.events.push((GameInputDeviceEventConnected, device));
    }

    pub fn disconnect(&mut self, id: usize) {
        if let Some(index) = self.connected.iter().position(|device| device.id == id) {
            let device = self.connected.remove(index);
            self.events.push((GameInputDeviceEventDisconnected, device));
        }
    }

    // A new reading of a connected device's state: its last state, changed
    // by `change`.
    pub fn read(&mut self, id: usize, change: impl FnOnce(&mut Reading)) -> Result<(), String> {
        let supported_input = self
            .connected
            .iter()
            .find(|device| device.id == id)
            .map(|device| device.supported_input)
            .ok_or_else(|| format!("device {id} isn't connected"))?;
        let mut reading = self
            .latest
            .iter()
            .find(|reading| reading.device == Some(id))
            .cloned()
            .unwrap_or_default();
        // Later than every earlier reading, so the order stays.
        let newest = self.latest.iter().map(|reading| reading.timestamp).max();
        reading.timestamp = newest.map_or(self.now, |newest| self.now.max(newest + 1));
        reading.kinds = supported_input;
        reading.device = Some(id);
        change(&mut reading);
        self.latest.retain(|known| known.device != Some(id));
        self.latest.push(reading.clone());
        for kind in [GameInputKindGamepad, GameInputKindKeyboard, GameInputKindMouse] {
            if reading.kinds & kind == 0 {
                continue;
            }
            match self.queues.iter_mut().find(|(queued, _)| *queued == kind) {
                Some((_, queue)) => queue.push(reading.clone()),
                None => self.queues.push((kind, vec![reading.clone()])),
            }
        }
        Ok(())
    }

    // The gamepad's rumble, as the demo last set it: None once it's stopped.
    pub fn rumble(&self, id: usize) -> Option<GameInputRumbleParams> {
        self.rumble
            .iter()
            .find(|(device, _)| *device == id)
            .and_then(|(_, params)| *params)
    }

    // Does what a script's "Program" step says.
    pub fn apply(&mut self, step: &Value) -> Result<(), String> {
        let Value::Object(members) = step else {
            return Err(format!("a step for the demo must be an object, not {}", step.describe()));
        };
        let [(name, value)] = members.as_slice() else {
            return Err("a step for the demo has one of Connect, Disconnect, Gamepad, Keyboard and Mouse".to_string());
        };
        match name.as_str() {
            "Connect" => {
                let supported_input = flags(
                    value,
                    "Input",
                    &[
                        (GameInputKindGamepad, "Gamepad"),
                        (GameInputKindKeyboard, "Keyboard"),
                        (GameInputKindMouse, "Mouse"),
                    ],
                )?;
                let rumble_motors = flags(
                    value,
                    "Motors",
                    &[
                        (GameInputRumbleLowFrequency, "LowFrequency"),
                        (GameInputRumbleHighFrequency, "HighFrequency"),
                        (GameInputRumbleLeftTrigger, "LeftTrigger"),
                        (GameInputRumbleRightTrigger, "RightTrigger"),
                    ],
                )?;
                let label = match value.get("Name") {
                    Some(Value::String(name)) => name.clone(),
                    _ => return Err("Connect needs a Name".to_string()),
                };
                self.connect(Device {
                    id: id(value)?,
                    label,
                    family: GameInputFamilyVirtual,
                    supported_input,
                    rumble_motors,
                    xinput_index: None,
                    mouse_sample_rate: None,
                    gamepad_layout: (supported_input & GameInputKindGamepad != 0).then_some(0),
                });
                Ok(())
            }
            "Disconnect" => match value {
                Value::Number(id) if *id >= 1.0 && id.fract() == 0.0 => {
                    self.disconnect(*id as usize);
                    Ok(())
                }
                other => Err(format!("Disconnect needs a device's Id, not {}", other.describe())),
            },
            "Gamepad" => {
                let [left_x, left_y] = pair(value, "LeftStick")?;
                let [right_x, right_y] = pair(value, "RightStick")?;
                let state = GameInputGamepadState {
                    buttons: flags(value, "Buttons", GAMEPAD_BUTTONS)?,
                    leftTrigger: number(value, "LeftTrigger")? as f32,
                    rightTrigger: number(value, "RightTrigger")? as f32,
                    leftThumbstickX: left_x as f32,
                    leftThumbstickY: left_y as f32,
                    rightThumbstickX: right_x as f32,
                    rightThumbstickY: right_y as f32,
                };
                self.read(id(value)?, |reading| reading.gamepad = state)
            }
            "Keyboard" => {
                let keys = strings(value, "Down")?
                    .iter()
                    .map(|name| key_state(name).ok_or_else(|| format!("{name} isn't a key")))
                    .collect::<Result<Vec<_>, _>>()?;
                self.read(id(value)?, |reading| reading.keys = keys)
            }
            "Mouse" => {
                let [motion_x, motion_y] = pair(value, "Motion")?;
                let [wheel_x, wheel_y] = pair(value, "Wheel")?;
                let buttons = flags(value, "Buttons", MOUSE_BUTTONS)?;
                self.read(id(value)?, |reading| {
                    let mouse = &mut reading.mouse;
                    mouse.buttons = buttons;
                    mouse.positions = GameInputMouseRelativePosition;
                    mouse.positionX += motion_x as i64;
                    mouse.positionY += motion_y as i64;
                    mouse.wheelX += wheel_x as i64;
                    mouse.wheelY += wheel_y as i64;
                })
            }
            other => Err(format!("{other} isn't a step for the demo")),
        }
    }
}

impl InputSource for ScriptedSource {
    fn now(&mut self) -> u64 {
        self.now
    }

    fn device_events(&mut self) -> Vec<(GameInputDeviceEvent, Device)> {
        std::mem::take(&mut self.events)
    }

    fn readings(&mut self, kind: GameInputKind) -> Vec<Reading> {
        self.queues
            .iter_mut()
            .find(|(queued, _)| *queued == kind)
            .map(|(_, queue)| std::mem::take(queue))
            .unwrap_or_default()
    }

    fn set_rumble(&mut self, device: usize, params: Option<&GameInputRumbleParams>) {
        self.rumble.retain(|(known, _)| *known != device);
        self.rumble.push((device, params.copied()));
    }

    fn as_scripted(&mut self) -> Option<&mut ScriptedSource> {
        Some(self)
    }
}

// The key, as GameInput reports it held, from its name, like "F1" or "A".
// Its scan code comes from the keyboard layout, and has the 0xE0 prefix for
// extended keys, like GameInput's.
pub fn key_state(name: &str) -> Option<GameInputKeyState> {
    let key = virtual_key(name)?;
    let mut scan_code = unsafe { MapVirtualKeyW(key, MAPVK_VK_TO_VSC_EX as u32) };
    // Windows gives the arrow keys and Delete the scan codes of their twins
    // on the numeric keypad, without the prefix that the keys themselves send.
    if matches!(key, 0x25..=0x28 | 0x2E) {
        scan_code |= 0xE000;
    }
    let character = match key {
        0x41..=0x5A => key + 0x20,
        0x30..=0x39 | 0x20 => key,
        _ => 0,
    };
    Some(GameInputKeyState {
        scanCode: scan_code,
        codePoint: character,
        virtualKey: key as u8,
        isDeadKey: false,
    })
}

fn id(value: &Value) -> Result<usize, String> {
    match value.get("Id") {
        Some(Value::Number(id)) if *id >= 1.0 && id.fract() == 0.0 => Ok(*id as usize),
        _ => Err("a device's Id must be a whole number from 1".to_string()),
    }
}

// A member that's a number, or 0 if it's left out.
fn number(value: &Value, name: &str) -> Result<f64, String> {
    match value.get(name) {
        None => Ok(0.0),
        Some(Value::Number(number)) => Ok(*number),
        Some(other) => Err(format!("{name} must be a number, not {}", other.describe())),
    }
}

// A member that's [x, y], or [0, 0] if it's left out.
fn pair(value: &Value, name: &str) -> Result<[f64; 2], String> {
    match value.get(name) {
        None => Ok([0.0, 0.0]),
        Some(Value::Array(items)) => match items.as_slice() {
            [Value::Number(x), Value::Number(y)] => Ok([*x, *y]),
            _ => Err(format!("{name} must be [x, y], not a list of {}", items.len())),
        },
        Some(other) => Err(format!("{name} must be [x, y], not {}", other.describe())),
    }
}

// A member that's a list of names, or none if it's left out.
fn strings(value: &Value, name: &str) -> Result<Vec<String>, String> {
    let not_names = |other: &Value| format!("{name} must be a list of names, not {}", other.describe());
    match value.get(name) {
        None => Ok(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| match item {
                Value::String(text) => Ok(text.clone()),
                other => Err(not_names(other)),
            })
            .collect(),
        Some(other) => Err(not_names(other)),
    }
}

// The flags named in a member that's a list of names, from `choices`.
fn flags(value: &Value, name: &str, choices: &[(i32, &str)]) -> Result<i32, String> {
    strings(value, name)?.iter().try_fold(0, |flags, item| {
        choices
            .iter()
            .find(|(_, choice)| choice == item)
            .map(|(flag, _)| flags | flag)
            .ok_or_else(|| format!("{name} can't have {item}"))
    })
}
