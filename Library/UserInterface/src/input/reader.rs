// Turns a frame's GameInput readings (platform/input_devices.rs) into the
// interface's events (input/events.rs), in the order of GameInput's stream,
// which is the order the input happened in, with the window's focus changes,
// which come from the window and not from GameInput, put among them by their
// times. Nothing here reorders the readings:
//
// - Keys come from keyboards' readings, which are the keys held down. Shift,
//   Ctrl and Alt are the same keys on the left and the right. Typed text is
//   the keys, translated with Windows' keyboard layout, with Shift, AltGr and
//   dead keys. A held key doesn't repeat: GameInput's readings don't, and the
//   reader doesn't make repeats of its own.
// - The mouse's buttons, wheels and motion come from mice's readings, and
//   the pointer is the reader's own, as SDL's relative mode keeps one: it
//   starts in the middle of the window the first time the window gets focus,
//   and carries on from where it was each time after, and each mouse's
//   motion moves it, its counts times MOUSE_SENSITIVITY, inside the window, where the window keeps Windows' cursor, hidden, while
//   it has focus (see platform/window.rs). It moves in the stream's order, so a press happens
//   where the pointer was as the button went down. Only mice's relative
//   motion moves it, not the absolute positions of a pen or a touch screen.
//   A mouse's motion and wheels are running totals, which only its readings
//   with relative positions have: GameInput gives each mouse a reading as
//   focus goes that has none, and whose totals are 0, and goes on adding up
//   a mouse's motion and wheels while the program doesn't have focus, as
//   measured with GameInput 4.0.306. So such a reading neither moves the
//   pointer nor turns the wheel, and each mouse starts over as focus goes:
//   its first reading with relative positions after that only says where its
//   totals are, and motion and turns made in another program never count.
// - Each gamepad keeps its own state, from its own readings, and the
//   interface's gamepad is all of them at once: a button is held while any
//   gamepad holds it, a trigger is pulled as far as any gamepad pulls it, and
//   each stick is the one that's pushed furthest, so that a gamepad at rest,
//   whose sticks may read thousands of times a second, never covers the one
//   in use. Each change of its buttons is an event at its reading, so that a
//   button pressed and let go of within a frame counts, and the frame's last
//   event is its state as the frame leaves it.
// - A gamepad is known from when it connects, and a device that's unplugged
//   lets go of what it held after the frame's readings, as GameInput doesn't
//   say when it went.
//
// Without focus, the window gets no input, and GameInput gives none, as the
// library leaves its background input off (see platform/input_devices.rs):
// as focus goes, it gives each device a reading at rest, which lets go of
// everything held, and the reader forgets the keys and mouse buttons held
// itself, in case that reading was lost. GameInput doesn't report keys and
// mouse buttons held from before focus at all, like the click that gives the
// window focus, but reports a gamepad's whole state, whose held buttons the
// reader ignores until they're let go of. A reading from before the window
// got focus is input without it even when it comes in a later frame than the
// focus change, like that reading of a gamepad if GameInput is late with it.

use game_input::*;
use windows_api::*;

use crate::drawing::geometry::{Point, Size};
use crate::input::devices::{Gamepad, GamepadButtons, Key, MouseButton as Button};
use crate::input::events::Event;
use crate::platform::input_devices::{Devices, Reading};
use crate::platform::window::{FocusChange, Window};

// How far the pointer moves for each count of a mouse's motion, in pixels.
pub const MOUSE_SENSITIVITY: f32 = 0.8;

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

// A mouse's buttons, and its motion's and wheels' running totals as its last
// reading with relative positions had them, which the next one's motion and
// turns are from: none until it has one, and none again once focus goes, or
// a reading without them comes.
#[derive(Clone, Copy, Debug, Default)]
struct Mouse {
    buttons: GameInputMouseButtons,
    totals: Option<Totals>,
}

#[derive(Clone, Copy, Debug)]
struct Totals {
    x: i64,
    y: i64,
    wheel_x: i64,
    wheel_y: i64,
}

pub struct InputReader {
    // Whether the gamepad goes to the interface, as Options::gamepad says.
    gamepad: bool,
    focused: bool,
    // When the window last got focus, as the counter's ticks.
    focus_time: i64,
    // Each keyboard's held keys, with their scan codes, by device.
    keyboards: Vec<(Option<usize>, HeldKeys)>,
    keys_down: [bool; 256],
    scan_codes: [u32; 256],
    // Each mouse, by device.
    mice: Vec<(Option<usize>, Mouse)>,
    // The mouse buttons held, as bits by Button, on any mouse.
    buttons_down: u32,
    // Buttons whose presses the interface didn't get, whose releases it
    // doesn't either.
    ignored_buttons: u32,
    // Each gamepad's state, by device.
    gamepads: Vec<(Option<usize>, Gamepad)>,
    // The gamepads' buttons held without focus, which the interface doesn't
    // get until they're let go of.
    gamepad_ignored: u32,
    // The buttons that the last Gamepad event told the interface of.
    gamepad_sent: u32,
    // The pointer, in the client area's pixels and parts of them, once the
    // window has had focus, and the client area's size, which it stays
    // inside, from 0 to its width or height less 1, as SDL keeps its own.
    position: Point,
    placed: bool,
    size: Size,
    // Where the interface last heard the pointer was.
    pointer: Option<Point>,
}

// What the window says for a frame: the size of its client area, and when it
// got or lost focus.
#[derive(Clone, Debug, Default)]
pub struct WindowState {
    pub size: Size,
    pub focus_changes: Vec<FocusChange>,
}

impl InputReader {
    pub fn new(gamepad: bool) -> Self {
        Self {
            gamepad,
            focused: false,
            focus_time: i64::MIN,
            keyboards: Vec::new(),
            keys_down: [false; 256],
            scan_codes: [0; 256],
            mice: Vec::new(),
            buttons_down: 0,
            ignored_buttons: 0,
            gamepads: Vec::new(),
            gamepad_ignored: 0,
            gamepad_sent: 0,
            position: Point::ZERO,
            placed: false,
            size: Size::default(),
            pointer: None,
        }
    }

    // The frame's events, from the devices' readings since the last frame, in
    // the stream's order, and the window's focus changes.
    pub fn frame(&mut self, devices: Option<&Devices>, window: &Window, focus_changes: &[FocusChange]) -> FrameInput {
        let (width, height) = window.client_size();
        let state = WindowState {
            size: Size::new(width as f32, height as f32),
            focus_changes: focus_changes.to_vec(),
        };
        let mut readings = Vec::new();
        let mut disconnected = Vec::new();
        if let Some(devices) = devices {
            // By each device's last event in the frame, so that one that
            // disconnects and connects again stays.
            let events = devices.device_events();
            for (index, (event, device)) in events.iter().enumerate() {
                if events[index + 1..].iter().any(|(_, later)| later.id == device.id) {
                    continue;
                }
                if *event == GameInputDeviceEventDisconnected {
                    disconnected.push(device.id);
                } else if *event == GameInputDeviceEventConnected && device.supported_input & GameInputKindGamepad != 0 {
                    self.gamepad_connected(device.id);
                }
            }
            for reading in devices.readings() {
                readings.push((devices.qpc(reading.timestamp), reading.clone()));
            }
        }
        self.read(&readings, &disconnected, &state)
    }

    // The frame's events from its readings, in the stream's order, each with
    // its time on the counter, from the devices that disconnected, and from
    // what the window says.
    pub fn read(&mut self, readings: &[(i64, Reading)], disconnected: &[usize], window: &WindowState) -> FrameInput {
        self.size = window.size;
        self.position = self.inside(self.position);
        let mut frame = FrameInput::default();
        // The window's focus changes go before the first reading that's
        // later, and the readings keep the stream's order, whatever their
        // times.
        let mut changes = window.focus_changes.clone();
        changes.sort_by_key(|change| change.time);
        let mut changes = changes.into_iter().peekable();
        for (time, reading) in readings {
            while let Some(change) = changes.next_if(|change| change.time <= *time) {
                self.focus_change(change, &mut frame.events);
            }
            self.reading(*time, reading, &mut frame);
        }
        for change in changes {
            self.focus_change(change, &mut frame.events);
        }

        // A device that's unplugged lets go of what it held: a gamepad's
        // state goes, and a keyboard's keys and a mouse's buttons are
        // released, unless another device holds them. GameInput's device
        // events have no times, so this comes after the frame's readings,
        // which are all from before the device went: a reading of its that
        // came in the same frame mustn't bring it back, with what it held
        // stuck down.
        let unplugged = |device: &Option<usize>| device.is_some_and(|device| disconnected.contains(&device));
        if self.gamepads.iter().any(|(device, _)| unplugged(device)) {
            self.gamepads.retain(|(device, _)| !unplugged(device));
            self.gamepads_changed(&mut frame.events);
        }
        if self.keyboards.iter().any(|(device, _)| unplugged(device)) {
            self.keyboards.retain(|(device, _)| !unplugged(device));
            self.update_keys(&mut frame.events);
        }
        if self.mice.iter().any(|(device, _)| unplugged(device)) {
            self.mice.retain(|(device, _)| !unplugged(device));
            self.update_buttons(&mut frame.events);
        }

        self.point(&mut frame.events);
        if self.gamepad {
            self.send_gamepad(&mut frame.events);
        }
        frame
    }

    // The window got or lost focus, at a time on the counter.
    fn focus_change(&mut self, change: FocusChange, events: &mut Vec<Event>) {
        if change.gained {
            if !self.focused {
                self.focused = true;
                self.focus_time = change.time;
                let held = (0..=255u8).filter(|code| self.keys_down[*code as usize]).map(Key).collect();
                events.push(Event::FocusGained(held));
                self.ignored_buttons = self.buttons_down;
                self.gamepad_ignored = self.combined_gamepad().buttons.0;
                // The pointer starts in the middle of the window, and carries
                // on from where it was after that. It's anywhere, not on the
                // pixels' grid, as the motion moves it by parts of a pixel.
                if !std::mem::replace(&mut self.placed, true) {
                    self.position = Point::new(self.size.width / 2.0, self.size.height / 2.0);
                }
                self.position = self.inside(self.position);
                // The interface hears where the pointer is right away, so
                // that a key or a button after focus came is where it is.
                self.pointer = None;
                self.point(events);
                // The interface takes the gamepad event right after focus
                // comes as what's held then, which counts once it's let go
                // of. Its buttons that count are none, as the reader ignores
                // all that are held, so that a press after this, in the same
                // frame, is a press.
                if self.gamepad {
                    self.send_gamepad(events);
                }
            }
        } else if self.focused {
            self.focused = false;
            events.push(Event::FocusLost);
            self.pointer = None;
            // Keys and mouse buttons go with focus: GameInput gives each
            // device a reading at rest as focus goes, and none of what's
            // held or let go of until focus comes back, so whatever the
            // reader last had is no longer so, even if that reading was lost.
            // Each mouse starts over too, as GameInput goes on adding up its
            // motion and wheels without focus.
            self.keyboards.clear();
            self.keys_down = [false; 256];
            for (_, mouse) in &mut self.mice {
                *mouse = Mouse::default();
            }
            self.buttons_down = 0;
            self.ignored_buttons = 0;
        }
    }

    // A gamepad that connected, which is known from then on, at rest until it
    // reads, so that the interface's gamepad is connected while one is.
    pub fn gamepad_connected(&mut self, device: usize) {
        if !self.gamepads.iter().any(|(known, _)| *known == Some(device)) {
            self.gamepads.push((
                Some(device),
                Gamepad {
                    connected: true,
                    ..Gamepad::default()
                },
            ));
        }
    }

    // A reading, at its time on the counter: a keyboard's, a mouse's or a
    // gamepad's, or a device's with more than one of them, like a keyboard
    // with a trackball, which is each.
    fn reading(&mut self, time: i64, reading: &Reading, frame: &mut FrameInput) {
        // A reading from before the window got focus is handled as input
        // without focus, whenever it comes.
        let focused = self.focused;
        self.focused = focused && time >= self.focus_time;
        let kinds = reading.kinds;
        if kinds & GameInputKindKeyboard != 0 {
            self.keyboard(reading, &mut frame.events);
        }
        if kinds & GameInputKindMouse != 0 {
            self.mouse(reading, &mut frame.events);
        }
        if kinds & GameInputKindGamepad != 0 {
            let state = gamepad(&reading.gamepad);
            match self.gamepads.iter_mut().find(|(device, _)| *device == reading.device) {
                Some((_, known)) => *known = state,
                None => self.gamepads.push((reading.device, state)),
            }
            self.gamepads_changed(&mut frame.events);
        }
        let counts = kinds & (GameInputKindKeyboard | GameInputKindMouse) != 0 || (self.gamepad && kinds & GameInputKindGamepad != 0);
        if self.focused && counts {
            frame.input_time = Some(frame.input_time.map_or(time, |newest| newest.max(time)));
        }
        self.focused = focused;
    }

    // All the gamepads at once: a button is held while any holds it, a
    // trigger is pulled as far as any pulls it, and each stick is the one
    // that's pushed furthest. At rest, and not connected, without any.
    fn combined_gamepad(&self) -> Gamepad {
        let mut combined = Gamepad {
            connected: !self.gamepads.is_empty(),
            ..Gamepad::default()
        };
        let pushed = |stick: Point| stick.x * stick.x + stick.y * stick.y;
        for (_, pad) in &self.gamepads {
            combined.buttons.0 |= pad.buttons.0;
            combined.left_trigger = combined.left_trigger.max(pad.left_trigger);
            combined.right_trigger = combined.right_trigger.max(pad.right_trigger);
            if pushed(pad.left_stick) > pushed(combined.left_stick) {
                combined.left_stick = pad.left_stick;
            }
            if pushed(pad.right_stick) > pushed(combined.right_stick) {
                combined.right_stick = pad.right_stick;
            }
        }
        combined
    }

    // After a gamepad's state changed, or one went: buttons held without
    // focus stay ignored until they're let go of, and a change of the
    // buttons that count is an event, at its reading.
    fn gamepads_changed(&mut self, events: &mut Vec<Event>) {
        let held = self.combined_gamepad().buttons.0;
        self.gamepad_ignored = if self.focused { self.gamepad_ignored & held } else { held };
        if self.gamepad && self.gamepad_buttons() != self.gamepad_sent {
            self.send_gamepad(events);
        }
    }

    // The gamepads' buttons that count: those held but not ignored.
    fn gamepad_buttons(&self) -> u32 {
        self.combined_gamepad().buttons.0 & !self.gamepad_ignored
    }

    // Tells the interface the gamepads' state, with the buttons that count.
    fn send_gamepad(&mut self, events: &mut Vec<Event>) {
        let mut state = self.combined_gamepad();
        state.buttons.0 = self.gamepad_buttons();
        self.gamepad_sent = state.buttons.0;
        events.push(Event::Gamepad(state));
    }

    // Tells the interface where the pointer is, if that changed, while the
    // window has focus.
    fn point(&mut self, events: &mut Vec<Event>) {
        if !self.focused || self.pointer == Some(self.position) {
            return;
        }
        events.push(Event::PointerMoved(self.position));
        self.pointer = Some(self.position);
    }

    // The point, kept inside the client area.
    fn inside(&self, point: Point) -> Point {
        Point::new(
            point.x.clamp(0.0, (self.size.width - 1.0).max(0.0)),
            point.y.clamp(0.0, (self.size.height - 1.0).max(0.0)),
        )
    }

    fn keyboard(&mut self, reading: &Reading, events: &mut Vec<Event>) {
        let keys: HeldKeys = reading.keys.iter().filter_map(|state| key(state).map(|key| (key, state.scanCode))).collect();
        match self.keyboards.iter_mut().find(|(device, _)| *device == reading.device) {
            Some((_, held)) => *held = keys,
            None => self.keyboards.push((reading.device, keys)),
        }
        self.update_keys(events);
    }

    // The presses and releases since the last time, from what every keyboard
    // holds now.
    fn update_keys(&mut self, events: &mut Vec<Event>) {
        let mut down = [false; 256];
        for (_, held) in &self.keyboards {
            for (key, scan_code) in held {
                down[key.0 as usize] = true;
                self.scan_codes[key.0 as usize] = *scan_code;
            }
        }
        // Keys that change in one reading are let go of before Shift, Ctrl
        // and Alt are, and pressed after them, as each step sees what's held
        // as its key goes: a Ctrl+Backspace, Shift+Tab or Ctrl+Enter that
        // one reading has is that, and not a Backspace, Tab or Enter.
        let modifiers = [Key::SHIFT.0, Key::CONTROL.0, Key::ALT.0];
        let others = (0..=255u8).filter(|code| !modifiers.contains(code));
        for code in others.clone().chain(modifiers).map(usize::from) {
            let held = down[code];
            if self.keys_down[code] && !held {
                self.keys_down[code] = false;
                if self.focused {
                    events.push(Event::KeyReleased(Key(code as u8)));
                }
            }
        }
        for code in modifiers.into_iter().chain(others).map(usize::from) {
            let held = down[code];
            if !self.keys_down[code] && held {
                self.keys_down[code] = true;
                let key = Key(code as u8);
                if self.focused {
                    events.push(Event::KeyPressed(key));
                    self.text(key, self.scan_codes[code], events);
                }
            }
        }
    }

    fn mouse(&mut self, reading: &Reading, events: &mut Vec<Event>) {
        let state = reading.mouse;
        let totals = (state.positions & GameInputMouseRelativePosition != 0).then_some(Totals {
            x: state.positionX,
            y: state.positionY,
            wheel_x: state.wheelX,
            wheel_y: state.wheelY,
        });
        let index = match self.mice.iter().position(|(device, _)| *device == reading.device) {
            Some(index) => index,
            None => {
                self.mice.push((reading.device, Mouse::default()));
                self.mice.len() - 1
            }
        };
        let mouse = &mut self.mice[index].1;
        mouse.buttons = state.buttons;
        let previous = std::mem::replace(&mut mouse.totals, totals);
        // The motion and the wheels are running totals, which go around an
        // i64's ends, the wheels in Windows' units of 120 a notch, from the
        // mouse's last reading with them. The motion comes first: a reading
        // with a press has the motion before it. The interface hears of each
        // move as it happens, so that what comes after it in the stream, like
        // a key, sees the pointer where it went.
        let change = match (previous, totals) {
            (Some(previous), Some(totals)) if self.focused => Some((previous, totals)),
            _ => None,
        };
        if let Some((previous, totals)) = change {
            let moved = Point::new(totals.x.wrapping_sub(previous.x) as f32, totals.y.wrapping_sub(previous.y) as f32);
            let moved = Point::new(moved.x * MOUSE_SENSITIVITY, moved.y * MOUSE_SENSITIVITY);
            self.position = self.inside(self.position + moved);
            self.point(events);
        }
        self.update_buttons(events);
        if let Some((previous, totals)) = change {
            let turned = Point::new(
                totals.wheel_x.wrapping_sub(previous.wheel_x) as f32 / WHEEL_DELTA as f32,
                totals.wheel_y.wrapping_sub(previous.wheel_y) as f32 / WHEEL_DELTA as f32,
            );
            if turned != Point::default() {
                self.point(events);
                events.push(Event::Wheel(turned));
            }
        }
    }

    // The presses and releases since the last time, from what every mouse
    // holds now.
    fn update_buttons(&mut self, events: &mut Vec<Event>) {
        let mut down = 0;
        for (_, mouse) in &self.mice {
            down |= button_bits(mouse.buttons);
        }
        for button in Button::ALL {
            let bit = 1 << button as u32;
            if down & bit != 0 && self.buttons_down & bit == 0 {
                self.buttons_down |= bit;
                if self.focused && self.ignored_buttons & bit == 0 {
                    self.point(events);
                    events.push(Event::ButtonPressed(button));
                } else {
                    self.ignored_buttons |= bit;
                }
            } else if down & bit == 0 && self.buttons_down & bit != 0 {
                self.buttons_down &= !bit;
                if self.ignored_buttons & bit != 0 {
                    self.ignored_buttons &= !bit;
                } else if self.focused {
                    self.point(events);
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
