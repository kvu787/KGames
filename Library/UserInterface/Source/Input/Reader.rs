// Turns a frame's GameInput readings (Platform/InputDevices.rs) into the
// interface's events (Input/Events.rs), in the order that they happened,
// with the window's focus changes between them:
//
// - Keys come from keyboards' readings, which are the keys held down. Shift,
//   Ctrl and Alt are the same keys on the left and the right. Typed text is
//   the keys, translated with Windows' keyboard layout, with Shift, AltGr and
//   dead keys. A held key repeats at Windows' keyboard delay and rate.
// - The mouse's buttons and wheels come from mice's readings, and the pointer
//   is where Windows' cursor is, so that it moves as Windows moves it.
// - The gamepad is the newest reading of any gamepad.
//
// Without focus, the window gets no input, though its devices' state is
// still followed. Keys and buttons held when focus comes count once they're
// let go of, like the click that gives the window focus, and a press while
// the pointer isn't on the window is another window's.

use game_input::*;
use windows_api::*;

use crate::drawing::geometry::Point;
use crate::input::devices::{Gamepad, GamepadButtons, Key, MouseButton as Button};
use crate::input::events::Event;
use crate::platform::clock;
use crate::platform::input_devices::{Devices, KINDS, Reading};
use crate::platform::window::{FocusChange, Window};

// What a frame read.
#[derive(Clone, Debug, Default)]
pub struct FrameInput {
    pub events: Vec<Event>,
    // The time of the newest reading that the frame took, as the counter's
    // ticks, if it took any: where its input latency starts.
    pub input_time: Option<i64>,
}

// A keyboard's held keys, with their scan codes.
type HeldKeys = Vec<(Key, u32)>;

struct Repeat {
    key: Key,
    scan_code: u32,
    // When it next repeats, as the counter's ticks.
    next: i64,
}

pub struct InputReader {
    // Whether the gamepad goes to the interface, as Options::gamepad says.
    gamepad: bool,
    focused: bool,
    // Each keyboard's held keys, with their scan codes, by device.
    keyboards: Vec<(Option<usize>, HeldKeys)>,
    keys_down: [bool; 256],
    scan_codes: [u32; 256],
    // Each mouse's last reading, by device.
    mice: Vec<(Option<usize>, GameInputMouseState)>,
    // The mouse buttons held, as bits by Button, on any mouse.
    buttons_down: u32,
    // Buttons whose presses the interface didn't get, whose releases it
    // doesn't either.
    ignored_buttons: u32,
    gamepad_state: Gamepad,
    gamepad_device: Option<usize>,
    repeat: Option<Repeat>,
    repeat_delay: i64,
    repeat_interval: i64,
    // Where the interface last heard the pointer was.
    pointer: Option<Point>,
}

// What the window says for a frame: where Windows' cursor is in it, whether
// it's on it, and when it got or lost focus.
#[derive(Clone, Debug, Default)]
pub struct WindowState {
    pub cursor: Point,
    pub inside: bool,
    pub focus_changes: Vec<FocusChange>,
}

impl InputReader {
    // Held keys repeat at Windows' keyboard delay and rate.
    pub fn new(gamepad: bool) -> Self {
        let (delay, rate) = keyboard_repeat();
        Self::with_repeat(gamepad, delay, rate)
    }

    // Held keys repeat after the delay, in seconds, at the rate, in repeats a
    // second.
    pub fn with_repeat(gamepad: bool, delay: f64, rate: f64) -> Self {
        Self {
            gamepad,
            focused: false,
            keyboards: Vec::new(),
            keys_down: [false; 256],
            scan_codes: [0; 256],
            mice: Vec::new(),
            buttons_down: 0,
            ignored_buttons: 0,
            gamepad_state: Gamepad::default(),
            gamepad_device: None,
            repeat: None,
            repeat_delay: (delay * clock::frequency() as f64) as i64,
            repeat_interval: (clock::frequency() as f64 / rate) as i64,
            pointer: None,
        }
    }

    // The frame's events, from the devices' readings since the last frame and
    // the window's focus changes, in time order. Captures the mouse while a
    // press that the interface got is held.
    pub fn frame(&mut self, devices: Option<&Devices>, window: &mut Window, focus_changes: &[FocusChange]) -> FrameInput {
        let (cursor, inside) = window.cursor();
        let state = WindowState {
            cursor,
            inside,
            focus_changes: focus_changes.to_vec(),
        };
        let mut readings = Vec::new();
        let mut disconnected = Vec::new();
        if let Some(devices) = devices {
            for (event, device) in devices.device_events() {
                if *event == GameInputDeviceEventDisconnected {
                    disconnected.push(device.id);
                }
            }
            for kind in KINDS {
                for reading in devices.readings(kind) {
                    readings.push((devices.qpc(reading.timestamp), kind, reading.clone()));
                }
            }
        }
        let frame = self.read(&readings, &disconnected, &state, clock::now());
        window.set_capture(self.capturing());
        frame
    }

    // Whether a press that the interface got is held.
    pub fn capturing(&self) -> bool {
        self.focused && self.buttons_down & !self.ignored_buttons != 0
    }

    // The frame's events from readings, each with its time on the counter and
    // its kind of input, from the devices that disconnected, and from what the
    // window says, at a time on the counter.
    #[allow(non_upper_case_globals)]
    pub fn read(&mut self, readings: &[(i64, GameInputKind, Reading)], disconnected: &[usize], window: &WindowState, now: i64) -> FrameInput {
        enum Item<'a> {
            Focus(bool),
            Reading(GameInputKind, &'a Reading),
        }
        let (cursor, inside) = (window.cursor, window.inside);
        let mut frame = FrameInput::default();
        // A device that's unplugged lets go of what it held: a gamepad's
        // state goes, and a keyboard's keys and a mouse's buttons are
        // released, unless another keyboard or mouse holds them.
        if self.gamepad_device.is_some_and(|device| disconnected.contains(&device)) {
            self.gamepad_state = Gamepad::default();
            self.gamepad_device = None;
        }
        let unplugged = |device: &Option<usize>| device.is_some_and(|device| disconnected.contains(&device));
        if self.keyboards.iter().any(|(device, _)| unplugged(device)) {
            self.keyboards.retain(|(device, _)| !unplugged(device));
            self.update_keys(now, &mut frame.events);
        }
        if self.mice.iter().any(|(device, _)| unplugged(device)) {
            self.mice.retain(|(device, _)| !unplugged(device));
            self.update_buttons(cursor, inside, &mut frame.events);
        }
        let mut items: Vec<(i64, Item)> = window.focus_changes.iter().map(|change| (change.time, Item::Focus(change.gained))).collect();
        for (time, kind, reading) in readings {
            items.push((*time, Item::Reading(*kind, reading)));
        }
        // Stable, so that a reading with two kinds of input keeps its order.
        items.sort_by_key(|(time, _)| *time);

        for (time, item) in items {
            match item {
                Item::Focus(true) => {
                    if !self.focused {
                        self.focused = true;
                        let held = (0..=255u8).filter(|code| self.keys_down[*code as usize]).map(Key).collect();
                        frame.events.push(Event::FocusGained(held));
                        self.ignored_buttons = self.buttons_down;
                        self.pointer = None;
                    }
                }
                Item::Focus(false) => {
                    if self.focused {
                        self.focused = false;
                        frame.events.push(Event::FocusLost);
                        self.repeat = None;
                        self.pointer = None;
                    }
                }
                Item::Reading(kind, reading) => {
                    match kind {
                        GameInputKindKeyboard => self.keyboard(reading, time, &mut frame.events),
                        GameInputKindMouse => self.mouse(reading, cursor, inside, &mut frame.events),
                        _ => {
                            self.gamepad_state = gamepad(&reading.gamepad);
                            self.gamepad_device = reading.device;
                        }
                    }
                    if self.focused && (kind != GameInputKindGamepad || self.gamepad) {
                        frame.input_time = Some(time);
                    }
                }
            }
        }

        // A held key repeats, at the times Windows' settings give it.
        if self.focused
            && let Some(repeat) = &mut self.repeat
        {
            let (key, scan_code) = (repeat.key, repeat.scan_code);
            let mut count = 0;
            while repeat.next <= now {
                repeat.next += self.repeat_interval.max(1);
                count += 1;
            }
            for _ in 0..count {
                frame.events.push(Event::KeyPressed { key, repeat: true });
                self.text(key, scan_code, &mut frame.events);
            }
        }
        self.point(cursor, inside, &mut frame.events);
        if self.gamepad {
            frame.events.push(Event::Gamepad(self.gamepad_state));
        }
        frame
    }

    // Tells the interface where the pointer is, if that changed: where the
    // cursor is, while it's on the window or a press that the interface got
    // is held, or else that it left.
    fn point(&mut self, cursor: Point, inside: bool, events: &mut Vec<Event>) {
        if !self.focused {
            return;
        }
        let pointer = (inside || self.buttons_down & !self.ignored_buttons != 0).then_some(cursor);
        if pointer != self.pointer {
            match pointer {
                Some(point) => events.push(Event::PointerMoved(point)),
                None => events.push(Event::PointerLeft),
            }
            self.pointer = pointer;
        }
    }

    fn keyboard(&mut self, reading: &Reading, time: i64, events: &mut Vec<Event>) {
        let keys: HeldKeys = reading.keys.iter().filter_map(|state| key(state).map(|key| (key, state.scanCode))).collect();
        match self.keyboards.iter_mut().find(|(device, _)| *device == reading.device) {
            Some((_, held)) => *held = keys,
            None => self.keyboards.push((reading.device, keys)),
        }
        self.update_keys(time, events);
    }

    // The presses and releases since the last time, from what every keyboard
    // holds now, at a time on the counter.
    fn update_keys(&mut self, time: i64, events: &mut Vec<Event>) {
        let mut down = [false; 256];
        for (_, held) in &self.keyboards {
            for (key, scan_code) in held {
                down[key.0 as usize] = true;
                self.scan_codes[key.0 as usize] = *scan_code;
            }
        }
        for (code, held) in down.iter().enumerate() {
            if self.keys_down[code] && !held {
                self.keys_down[code] = false;
                if self.focused {
                    events.push(Event::KeyReleased(Key(code as u8)));
                }
                if self.repeat.as_ref().is_some_and(|repeat| repeat.key.0 as usize == code) {
                    self.repeat = None;
                }
            }
        }
        for (code, held) in down.iter().enumerate() {
            if !self.keys_down[code] && *held {
                self.keys_down[code] = true;
                let key = Key(code as u8);
                if self.focused {
                    events.push(Event::KeyPressed { key, repeat: false });
                    self.text(key, self.scan_codes[code], events);
                    // Windows repeats the last key pressed, and not Shift,
                    // Ctrl or Alt here.
                    if !key.is_modifier() {
                        self.repeat = Some(Repeat {
                            key,
                            scan_code: self.scan_codes[code],
                            next: time + self.repeat_delay,
                        });
                    }
                }
            }
        }
    }

    fn mouse(&mut self, reading: &Reading, cursor: Point, inside: bool, events: &mut Vec<Event>) {
        let state = reading.mouse;
        let previous = match self.mice.iter_mut().find(|(device, _)| *device == reading.device) {
            Some((_, last)) => Some(std::mem::replace(last, state)),
            None => {
                self.mice.push((reading.device, state));
                None
            }
        };
        self.update_buttons(cursor, inside, events);
        // The wheels are running totals, in Windows' units of 120 a notch.
        if let Some(previous) = previous
            && self.focused
            && inside
        {
            let turned = Point::new(
                (state.wheelX - previous.wheelX) as f32 / WHEEL_DELTA as f32,
                (state.wheelY - previous.wheelY) as f32 / WHEEL_DELTA as f32,
            );
            if turned != Point::default() {
                self.point(cursor, inside, events);
                events.push(Event::Wheel(turned));
            }
        }
    }

    // The presses and releases since the last time, from what every mouse
    // holds now.
    fn update_buttons(&mut self, cursor: Point, inside: bool, events: &mut Vec<Event>) {
        let mut down = 0;
        for (_, mouse) in &self.mice {
            down |= button_bits(mouse.buttons);
        }
        for button in Button::ALL {
            let bit = 1 << button as u32;
            if down & bit != 0 && self.buttons_down & bit == 0 {
                self.buttons_down |= bit;
                if self.focused && inside && self.ignored_buttons & bit == 0 {
                    self.point(cursor, inside, events);
                    events.push(Event::ButtonPressed(button));
                } else {
                    self.ignored_buttons |= bit;
                }
            } else if down & bit == 0 && self.buttons_down & bit != 0 {
                self.buttons_down &= !bit;
                if self.ignored_buttons & bit != 0 {
                    self.ignored_buttons &= !bit;
                } else if self.focused {
                    self.point(cursor, inside, events);
                    events.push(Event::ButtonReleased(button));
                }
            }
        }
    }

    // The text that a key types, translated with Windows' keyboard layout,
    // like WM_CHAR. Alt's keys type nothing, unless Ctrl is held with it,
    // which is AltGr.
    fn text(&self, key: Key, scan_code: u32, events: &mut Vec<Event>) {
        let held = |key: Key| self.keys_down[key.0 as usize];
        if held(Key::ALT) && !held(Key::CONTROL) {
            return;
        }
        let mut state = [0u8; 256];
        for (state, down) in state.iter_mut().zip(self.keys_down) {
            if down {
                *state = 0x80;
            }
        }
        for (generic, left) in [(Key::SHIFT, 0xA0), (Key::CONTROL, 0xA2), (Key::ALT, 0xA4)] {
            if held(generic) {
                state[left] = 0x80;
            }
        }
        // Caps Lock's and Num Lock's toggles are Windows', which keeps them as
        // the window's messages come.
        for toggle in [Key::CAPS_LOCK.0, 0x90, 0x91] {
            if unsafe { GetKeyState(toggle as i32) } & 1 != 0 {
                state[toggle as usize] |= 1;
            }
        }
        let mut buffer = [0u16; 8];
        let count = unsafe {
            ToUnicodeEx(
                key.0 as u32,
                scan_code & 0xFF,
                state.as_ptr(),
                windows_core::PWSTR(buffer.as_mut_ptr()),
                buffer.len() as i32,
                0,
                Some(GetKeyboardLayout(0)),
            )
        };
        // Less than 0 for a dead key, which the next key's text has with it.
        if count > 0 {
            for character in char::decode_utf16(buffer[..count as usize].iter().copied()).flatten() {
                events.push(Event::Text(character));
            }
        }
    }
}

// The interface's key for a key that GameInput reads: its virtual key, with
// Shift, Ctrl and Alt the same on the left and the right.
fn key(state: &GameInputKeyState) -> Option<Key> {
    let mut code = state.virtualKey as u32;
    if code == 0 {
        code = unsafe { MapVirtualKeyW(state.scanCode, MAPVK_VSC_TO_VK_EX as u32) };
    }
    let code = match code {
        0xA0 | 0xA1 => Key::SHIFT.0,
        0xA2 | 0xA3 => Key::CONTROL.0,
        0xA4 | 0xA5 => Key::ALT.0,
        1..=0xFE => code as u8,
        _ => return None,
    };
    Some(Key(code))
}

// GameInput's mouse buttons as bits by Button.
fn button_bits(buttons: GameInputMouseButtons) -> u32 {
    let mut bits = 0;
    for (button, flag) in [
        (Button::Left, GameInputMouseLeftButton),
        (Button::Right, GameInputMouseRightButton),
        (Button::Middle, GameInputMouseMiddleButton),
    ] {
        if buttons & flag != 0 {
            bits |= 1 << button as u32;
        }
    }
    bits
}

pub fn gamepad(state: &GameInputGamepadState) -> Gamepad {
    Gamepad {
        connected: true,
        left_stick: Point::new(state.leftThumbstickX, state.leftThumbstickY),
        right_stick: Point::new(state.rightThumbstickX, state.rightThumbstickY),
        left_trigger: state.leftTrigger,
        right_trigger: state.rightTrigger,
        // GameInput's bits for these buttons are the interface's.
        buttons: GamepadButtons(state.buttons as u32 & 0x3FFF),
    }
}

// Windows' keyboard delay, in seconds, and repeat rate, in repeats a second,
// from its settings: the delay's 0 to 3 is 250 ms to 1 s, and the rate's 0 to
// 31 is about 2.5 to 30 a second.
fn keyboard_repeat() -> (f64, f64) {
    let mut delay = 1u32;
    let mut speed = 31u32;
    unsafe {
        let _ = SystemParametersInfoW(SPI_GETKEYBOARDDELAY as u32, 0, (&raw mut delay).cast(), 0);
        let _ = SystemParametersInfoW(SPI_GETKEYBOARDSPEED as u32, 0, (&raw mut speed).cast(), 0);
    }
    let delay = (delay.min(3) + 1) as f64 * 0.25;
    let rate = 2.5 + speed.min(31) as f64 * (30.0 - 2.5) / 31.0;
    (delay, rate)
}
