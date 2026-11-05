// The gamepad, read with GameInput, for programs that a gamepad controls,
// like a game: its sticks, its triggers and its buttons, as of each frame.
//
// Of the gamepads that are connected, the one that counts is the one used
// last: the first one that GameInput has a reading of, and then any other
// whose button goes down, or whose stick or trigger moves past halfway. So a
// second gamepad that lies still, whose sticks drift a little, doesn't take
// over.
//
// Like all input, a gamepad's is ignored while the window doesn't have
// focus, and frames have it at rest then. GameInput gives it to windows
// without focus, so this drops it. A button that's still down when the
// window gets focus back only counts once it's pressed again, but the sticks
// and triggers count right away.

use std::sync::Mutex;

use game_input::*;
use windows_core::{Interface, Result};

// A gamepad's buttons, named as GameInputDemo names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    A,
    B,
    X,
    Y,
    View,
    Menu,
    LeftShoulder,
    RightShoulder,
    LeftStick,
    RightStick,
    Up,
    Down,
    Left,
    Right,
}

impl Button {
    pub const ALL: [Button; 14] = [
        Button::A,
        Button::B,
        Button::X,
        Button::Y,
        Button::View,
        Button::Menu,
        Button::LeftShoulder,
        Button::RightShoulder,
        Button::LeftStick,
        Button::RightStick,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
    ];

    // Its name, like "A", "LB" or "Up", which scripts use too.
    pub fn name(self) -> &'static str {
        match self {
            Button::A => "A",
            Button::B => "B",
            Button::X => "X",
            Button::Y => "Y",
            Button::View => "View",
            Button::Menu => "Menu",
            Button::LeftShoulder => "LB",
            Button::RightShoulder => "RB",
            Button::LeftStick => "LS",
            Button::RightStick => "RS",
            Button::Up => "Up",
            Button::Down => "Down",
            Button::Left => "Left",
            Button::Right => "Right",
        }
    }

    pub fn from_name(name: &str) -> Option<Button> {
        Button::ALL.into_iter().find(|button| button.name() == name)
    }

    // Its bit in Gamepad's buttons, which is GameInput's.
    pub const fn bit(self) -> u32 {
        (match self {
            Button::A => GameInputGamepadA,
            Button::B => GameInputGamepadB,
            Button::X => GameInputGamepadX,
            Button::Y => GameInputGamepadY,
            Button::View => GameInputGamepadView,
            Button::Menu => GameInputGamepadMenu,
            Button::LeftShoulder => GameInputGamepadLeftShoulder,
            Button::RightShoulder => GameInputGamepadRightShoulder,
            Button::LeftStick => GameInputGamepadLeftThumbstick,
            Button::RightStick => GameInputGamepadRightThumbstick,
            Button::Up => GameInputGamepadDPadUp,
            Button::Down => GameInputGamepadDPadDown,
            Button::Left => GameInputGamepadDPadLeft,
            Button::Right => GameInputGamepadDPadRight,
        }) as u32
    }
}

// Every button's bit.
const BUTTONS: u32 = {
    let mut bits = 0;
    let mut index = 0;
    while index < Button::ALL.len() {
        bits |= Button::ALL[index].bit();
        index += 1;
    }
    bits
};

// The gamepad as of a frame. Without one, or without focus, it's at rest.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gamepad {
    // Whether a gamepad is connected, with or without focus.
    pub connected: bool,
    // Where each stick is, from -1 to 1 along each axis, +X right and +Y up,
    // as the gamepad says, without a dead zone.
    pub left_stick: [f32; 2],
    pub right_stick: [f32; 2],
    // How far each trigger is pulled, from 0 to 1.
    pub left_trigger: f32,
    pub right_trigger: f32,
    // The buttons held down, as Button's bits.
    pub buttons: u32,
    // The buttons that went down since the frame before.
    pub went_down: u32,
}

impl Gamepad {
    pub fn held(&self, button: Button) -> bool {
        self.buttons & button.bit() != 0
    }

    // Whether the button went down since the frame before.
    pub fn pressed(&self, button: Button) -> bool {
        self.went_down & button.bit() != 0
    }

    // The buttons held down, by name.
    pub fn held_names(&self) -> Vec<&'static str> {
        Button::ALL
            .into_iter()
            .filter(|button| self.held(*button))
            .map(Button::name)
            .collect()
    }

    // The gamepad in a frame after one in which `before` were held: the
    // buttons that weren't went down.
    pub fn after(self, before: u32) -> Gamepad {
        Gamepad {
            went_down: self.buttons & !before,
            ..self
        }
    }
}

// The device callback runs on one of GameInput's threads. It only queues the
// event, and the main thread takes it in Gamepads::read.
static DEVICE_EVENTS: Mutex<Vec<(GameInputDeviceEvent, IGameInputDevice)>> = Mutex::new(Vec::new());

unsafe extern "system" fn on_device_event(
    _token: GameInputCallbackToken,
    _context: *mut core::ffi::c_void,
    event: GameInputDeviceEvent,
    device: *mut core::ffi::c_void,
) {
    if let Some(device) = unsafe { IGameInputDevice::from_raw_borrowed(&device) } {
        DEVICE_EVENTS.lock().unwrap().push((event, device.clone()));
    }
}

// What frames see of the gamepad that's read: it's at rest without focus,
// and buttons that were down without focus only count once they're let go
// of.
#[derive(Clone, Copy, Debug)]
pub struct FocusFilter {
    // Buttons that were down without focus.
    ignored: u32,
    // The buttons held in the frame before.
    before: u32,
}

impl Default for FocusFilter {
    // Buttons held when the program starts are ignored too.
    fn default() -> Self {
        FocusFilter {
            ignored: BUTTONS,
            before: 0,
        }
    }
}

impl FocusFilter {
    // The gamepad that a frame sees, from the gamepad read, `read`, at rest
    // if it isn't connected.
    pub fn frame(&mut self, read: Gamepad, focused: bool) -> Gamepad {
        if !focused {
            self.lose_focus();
            return Gamepad {
                connected: read.connected,
                ..Gamepad::default()
            };
        }
        self.ignored &= read.buttons;
        let buttons = read.buttons & !self.ignored;
        let gamepad = Gamepad { buttons, ..read }.after(self.before);
        self.before = buttons;
        gamepad
    }

    // The window lost focus without a frame to see it, like when it's
    // minimized: the buttons down then only count once they're let go of.
    pub fn lose_focus(&mut self) {
        self.ignored = BUTTONS;
        self.before = 0;
    }
}

// Reads the gamepad with GameInput, once a frame. One per process.
pub struct Gamepads {
    game_input: IGameInput,
    callback: GameInputCallbackToken,
    // The gamepads connected, from the device callback. GameInput keeps one
    // object per device, so its address tells devices apart.
    connected: Vec<IGameInputDevice>,
    // The one used last, whose readings count.
    used: Option<IGameInputDevice>,
    filter: FocusFilter,
}

impl Gamepads {
    // Loads the newest GameInput runtime, and starts listening for gamepads.
    // The callback also reports the ones already connected, soon after.
    pub fn new() -> Result<Self> {
        let (game_input, _runtime) = game_input::create()?;
        let callback =
            unsafe { game_input.RegisterDeviceCallback(None, GameInputKindGamepad, None, Some(on_device_event))? };
        Ok(Gamepads {
            game_input,
            callback,
            connected: Vec::new(),
            used: None,
            filter: FocusFilter::default(),
        })
    }

    // The gamepad as of now, at rest without focus.
    #[allow(non_upper_case_globals)]
    pub fn read(&mut self, focused: bool) -> Gamepad {
        for (event, device) in DEVICE_EVENTS.lock().unwrap().drain(..) {
            match event {
                GameInputDeviceEventConnected if !self.connected.contains(&device) => self.connected.push(device),
                GameInputDeviceEventDisconnected => {
                    self.connected.retain(|connected| *connected != device);
                    if self.used.as_ref() == Some(&device) {
                        self.used = None;
                    }
                }
                _ => {}
            }
        }
        let state = unsafe {
            // The newest reading of any gamepad says which was used last.
            if let Ok(reading) = self.game_input.GetCurrentReading(GameInputKindGamepad, None)
                && let Some(device) = reading.GetDevice()
                && self.connected.contains(&device)
                && self.used.as_ref() != Some(&device)
                && (self.used.is_none() || in_use(&state(&reading)))
            {
                self.used = Some(device);
            }
            self.used
                .as_ref()
                .and_then(|device| self.game_input.GetCurrentReading(GameInputKindGamepad, Some(device)).ok())
                .map(|reading| state(&reading))
        };
        let read = match state {
            Some(state) => Gamepad {
                connected: true,
                left_stick: [state.leftThumbstickX, state.leftThumbstickY],
                right_stick: [state.rightThumbstickX, state.rightThumbstickY],
                left_trigger: state.leftTrigger,
                right_trigger: state.rightTrigger,
                buttons: state.buttons as u32 & BUTTONS,
                went_down: 0,
            },
            None => Gamepad {
                connected: !self.connected.is_empty(),
                ..Gamepad::default()
            },
        };
        self.filter.frame(read, focused)
    }

    pub fn lose_focus(&mut self) {
        self.filter.lose_focus();
    }
}

impl Drop for Gamepads {
    fn drop(&mut self) {
        unsafe { self.game_input.UnregisterCallback(self.callback) };
    }
}

unsafe fn state(reading: &IGameInputReading) -> GameInputGamepadState {
    let mut state = GameInputGamepadState::default();
    unsafe { reading.GetGamepadState(&mut state) };
    state
}

// Whether someone is using the gamepad, as a stick that lies still drifts.
fn in_use(state: &GameInputGamepadState) -> bool {
    let far = |value: f32| value.abs() > 0.5;
    state.buttons as u32 & BUTTONS != 0
        || [
            state.leftThumbstickX,
            state.leftThumbstickY,
            state.rightThumbstickX,
            state.rightThumbstickY,
            state.leftTrigger,
            state.rightTrigger,
        ]
        .into_iter()
        .any(far)
}
