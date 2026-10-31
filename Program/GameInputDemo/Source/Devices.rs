// What the demo reads, as plain data: the devices, and readings of their
// state. Input.rs works on this data only, from an InputSource: GameInput
// itself (GameInputSource.rs), or a script or a test (ScriptedSource.rs), so
// that tests can give the demo any readings, like a gamepad's, without one.

use game_input::*;

use crate::scripted_source::ScriptedSource;

// A gamepad, keyboard or mouse that GameInput reports.
#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    // Tells devices apart, and stays the same while a device is connected.
    pub id: usize,
    // Its name, then its vendor and product IDs, like "Xbox Controller
    // (045E:0B13)".
    pub label: String,
    pub family: GameInputDeviceFamily,
    pub supported_input: GameInputKind,
    pub rumble_motors: GameInputRumbleMotors,
    pub xinput_index: Option<u32>,
    pub mouse_sample_rate: Option<u32>,
    pub gamepad_layout: Option<GameInputGamepadButtons>,
}

impl Device {
    // Matching on GameInput's constants needs their native names allowed.
    #[allow(non_upper_case_globals)]
    pub fn family_name(&self) -> &'static str {
        match self.family {
            GameInputFamilyVirtual => "virtual",
            GameInputFamilyXboxOne => "Xbox One",
            GameInputFamilyXbox360 => "Xbox 360",
            GameInputFamilyHid => "HID",
            GameInputFamilyI8042 => "i8042 (PS/2)",
            GameInputFamilyAggregate => "aggregate",
            _ => "unknown",
        }
    }

    pub fn kinds(&self) -> String {
        crate::input::kind_names(self.supported_input)
    }

    pub fn is_gamepad(&self) -> bool {
        self.supported_input & GameInputKindGamepad != 0
    }
}

// A snapshot of one device's whole state, at a time. GameInput keeps a
// short history of these, and doesn't send events.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reading {
    // The ID of the device it's from (see Device), or None when GameInput
    // doesn't say. The device's data comes with its device events, so a
    // reading doesn't copy it.
    pub device: Option<usize>,
    // On GameInput's clock, in microseconds.
    pub timestamp: u64,
    // Which kinds of input the reading has, like a gamepad's, or both a
    // keyboard's and a mouse's.
    pub kinds: GameInputKind,
    pub gamepad: GameInputGamepadState,
    pub keys: Vec<GameInputKeyState>,
    pub mouse: GameInputMouseState,
}

// Where devices and readings come from.
pub trait InputSource {
    // The GameInput runtime that was loaded, for the header, if any.
    fn runtime(&self) -> Option<&Runtime> {
        None
    }

    // The time now, on the readings' clock, in microseconds.
    fn now(&mut self) -> u64;

    // What happened to devices since the last call, oldest first: they
    // connected, disconnected, or changed.
    fn device_events(&mut self) -> Vec<(GameInputDeviceEvent, Device)>;

    // The readings with this kind of input since the last call, oldest
    // first. Input.rs compares each with the same device's reading before
    // it, which it keeps itself.
    fn readings(&mut self, kind: GameInputKind) -> Vec<Reading>;

    // Runs a gamepad's motors, or stops them with None.
    fn set_rumble(&mut self, device: usize, params: Option<&GameInputRumbleParams>);

    // The source, if it's a script's or a test's.
    fn as_scripted(&mut self) -> Option<&mut ScriptedSource> {
        None
    }
}
