// Input as the interface sees it: keys, the pointer, the wheel, typed text
// and the gamepad, gathered into what happened this frame and what's held.
//
// The window turns its messages into events, and scripts and tests make up
// events of their own, so every program handles both the same way.

use crate::geometry::Point;

// A key, by its Windows virtual-key code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key(pub u8);

impl Key {
    pub const BACKSPACE: Key = Key(0x08);
    pub const TAB: Key = Key(0x09);
    pub const ENTER: Key = Key(0x0D);
    pub const SHIFT: Key = Key(0x10);
    pub const CONTROL: Key = Key(0x11);
    pub const ALT: Key = Key(0x12);
    pub const PAUSE: Key = Key(0x13);
    pub const CAPS_LOCK: Key = Key(0x14);
    pub const ESCAPE: Key = Key(0x1B);
    pub const SPACE: Key = Key(0x20);
    pub const PAGE_UP: Key = Key(0x21);
    pub const PAGE_DOWN: Key = Key(0x22);
    pub const END: Key = Key(0x23);
    pub const HOME: Key = Key(0x24);
    pub const LEFT: Key = Key(0x25);
    pub const UP: Key = Key(0x26);
    pub const RIGHT: Key = Key(0x27);
    pub const DOWN: Key = Key(0x28);
    pub const INSERT: Key = Key(0x2D);
    pub const DELETE: Key = Key(0x2E);
    pub const A: Key = Key(b'A');
    pub const C: Key = Key(b'C');
    pub const D: Key = Key(b'D');
    pub const E: Key = Key(b'E');
    pub const Q: Key = Key(b'Q');
    pub const R: Key = Key(b'R');
    pub const S: Key = Key(b'S');
    pub const V: Key = Key(b'V');
    pub const W: Key = Key(b'W');
    pub const X: Key = Key(b'X');
    pub const Y: Key = Key(b'Y');
    pub const Z: Key = Key(b'Z');
    pub const F1: Key = Key(0x70);
    pub const F2: Key = Key(0x71);
    pub const F3: Key = Key(0x72);
    pub const F4: Key = Key(0x73);
    pub const F5: Key = Key(0x74);
    pub const F6: Key = Key(0x75);
    pub const F7: Key = Key(0x76);
    pub const F8: Key = Key(0x77);
    pub const F9: Key = Key(0x78);
    pub const F10: Key = Key(0x79);
    pub const F11: Key = Key(0x7A);
    pub const F12: Key = Key(0x7B);
    pub const LEFT_WINDOWS: Key = Key(0x5B);
    pub const RIGHT_WINDOWS: Key = Key(0x5C);

    // A letter or a digit's key.
    pub fn character(character: char) -> Option<Key> {
        let upper = character.to_ascii_uppercase();
        upper.is_ascii_alphanumeric().then_some(Key(upper as u8))
    }

    const NAMES: [(Key, &'static str); 36] = [
        (Key::BACKSPACE, "Backspace"),
        (Key::TAB, "Tab"),
        (Key::ENTER, "Enter"),
        (Key::SHIFT, "Shift"),
        (Key::CONTROL, "Ctrl"),
        (Key::ALT, "Alt"),
        (Key::PAUSE, "Pause"),
        (Key::CAPS_LOCK, "CapsLock"),
        (Key::ESCAPE, "Esc"),
        (Key::SPACE, "Space"),
        (Key::PAGE_UP, "PageUp"),
        (Key::PAGE_DOWN, "PageDown"),
        (Key::END, "End"),
        (Key::HOME, "Home"),
        (Key::LEFT, "Left"),
        (Key::UP, "Up"),
        (Key::RIGHT, "Right"),
        (Key::DOWN, "Down"),
        (Key::INSERT, "Insert"),
        (Key::DELETE, "Delete"),
        (Key::LEFT_WINDOWS, "Windows"),
        (Key::RIGHT_WINDOWS, "RightWindows"),
        (Key(0xA0), "LeftShift"),
        (Key(0xA1), "RightShift"),
        (Key(0xA2), "LeftCtrl"),
        (Key(0xA3), "RightCtrl"),
        (Key(0xA4), "LeftAlt"),
        (Key(0xA5), "RightAlt"),
        (Key(0xBA), "Semicolon"),
        (Key(0xBB), "Plus"),
        (Key(0xBC), "Comma"),
        (Key(0xBD), "Minus"),
        (Key(0xBE), "Period"),
        (Key(0xBF), "Slash"),
        (Key(0xC0), "Backtick"),
        (Key(0xDE), "Quote"),
    ];

    // Like "A", "F11", "Left" or "Space".
    pub fn name(self) -> String {
        if let Some((_, name)) = Key::NAMES.iter().find(|(key, _)| *key == self) {
            return name.to_string();
        }
        match self.0 {
            b'0'..=b'9' | b'A'..=b'Z' => (self.0 as char).to_string(),
            0x70..=0x87 => format!("F{}", self.0 - 0x6F),
            0x60..=0x69 => format!("Numpad{}", self.0 - 0x60),
            code => format!("Key{code:02X}"),
        }
    }

    // A key from its name, without regard to case.
    pub fn parse(name: &str) -> Option<Key> {
        if let Some((key, _)) = Key::NAMES.iter().find(|(_, known)| known.eq_ignore_ascii_case(name)) {
            return Some(*key);
        }
        let upper = name.to_ascii_uppercase();
        if upper == "CONTROL" {
            return Some(Key::CONTROL);
        }
        if upper == "ESCAPE" {
            return Some(Key::ESCAPE);
        }
        if upper.len() == 1 {
            return Key::character(upper.chars().next()?);
        }
        if let Some(number) = upper.strip_prefix('F').and_then(|digits| digits.parse::<u8>().ok())
            && (1..=24).contains(&number)
        {
            return Some(Key(0x6F + number));
        }
        if let Some(number) = upper.strip_prefix("NUMPAD").and_then(|digits| digits.parse::<u8>().ok())
            && number <= 9
        {
            return Some(Key(0x60 + number));
        }
        None
    }

    pub fn is_modifier(self) -> bool {
        matches!(self.0, 0x10..=0x12 | 0x5B | 0x5C | 0xA0..=0xA5)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
}

impl Modifiers {
    pub const NONE: Modifiers = Modifiers {
        shift: false,
        control: false,
        alt: false,
    };
    pub const SHIFT: Modifiers = Modifiers {
        shift: true,
        control: false,
        alt: false,
    };
    pub const CONTROL: Modifiers = Modifiers {
        shift: false,
        control: true,
        alt: false,
    };
    pub const CONTROL_SHIFT: Modifiers = Modifiers {
        shift: true,
        control: true,
        alt: false,
    };
}

// A key with the modifiers held for it, like Ctrl+S, from its name.
pub fn parse_shortcut(text: &str) -> Option<(Modifiers, Key)> {
    let mut modifiers = Modifiers::NONE;
    let parts: Vec<&str> = text.split('+').map(str::trim).collect();
    let (last, held) = parts.split_last()?;
    for part in held {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => modifiers.control = true,
            "shift" => modifiers.shift = true,
            "alt" => modifiers.alt = true,
            _ => return None,
        }
    }
    Some((modifiers, Key::parse(last)?))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    Left,
    Right,
    Middle,
}

impl Button {
    pub const ALL: [Button; 3] = [Button::Left, Button::Right, Button::Middle];

    fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Button::Left => "Left",
            Button::Right => "Right",
            Button::Middle => "Middle",
        }
    }

    pub fn parse(name: &str) -> Option<Button> {
        Button::ALL.into_iter().find(|button| button.name().eq_ignore_ascii_case(name))
    }
}

// The gamepad's buttons, as bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GamepadButtons(pub u32);

impl GamepadButtons {
    pub const MENU: u32 = 1 << 0;
    pub const VIEW: u32 = 1 << 1;
    pub const A: u32 = 1 << 2;
    pub const B: u32 = 1 << 3;
    pub const X: u32 = 1 << 4;
    pub const Y: u32 = 1 << 5;
    pub const DPAD_UP: u32 = 1 << 6;
    pub const DPAD_DOWN: u32 = 1 << 7;
    pub const DPAD_LEFT: u32 = 1 << 8;
    pub const DPAD_RIGHT: u32 = 1 << 9;
    pub const LEFT_SHOULDER: u32 = 1 << 10;
    pub const RIGHT_SHOULDER: u32 = 1 << 11;
    pub const LEFT_THUMBSTICK: u32 = 1 << 12;
    pub const RIGHT_THUMBSTICK: u32 = 1 << 13;

    pub const NAMES: [(u32, &'static str); 14] = [
        (GamepadButtons::A, "A"),
        (GamepadButtons::B, "B"),
        (GamepadButtons::X, "X"),
        (GamepadButtons::Y, "Y"),
        (GamepadButtons::LEFT_SHOULDER, "LeftShoulder"),
        (GamepadButtons::RIGHT_SHOULDER, "RightShoulder"),
        (GamepadButtons::VIEW, "View"),
        (GamepadButtons::MENU, "Menu"),
        (GamepadButtons::LEFT_THUMBSTICK, "LeftThumbstick"),
        (GamepadButtons::RIGHT_THUMBSTICK, "RightThumbstick"),
        (GamepadButtons::DPAD_UP, "Up"),
        (GamepadButtons::DPAD_DOWN, "Down"),
        (GamepadButtons::DPAD_LEFT, "Left"),
        (GamepadButtons::DPAD_RIGHT, "Right"),
    ];

    pub fn contains(self, button: u32) -> bool {
        self.0 & button != 0
    }

    pub fn names(self) -> Vec<&'static str> {
        GamepadButtons::NAMES
            .iter()
            .filter(|(bit, _)| self.contains(*bit))
            .map(|(_, name)| *name)
            .collect()
    }

    pub fn parse(name: &str) -> Option<u32> {
        GamepadButtons::NAMES
            .iter()
            .find(|(_, known)| known.eq_ignore_ascii_case(name))
            .map(|(bit, _)| *bit)
    }
}

// What a gamepad reads: sticks from -1 to 1, with +y up, and triggers from
// 0 to 1, raw, without dead zones.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gamepad {
    pub connected: bool,
    pub left_stick: Point,
    pub right_stick: Point,
    pub left_trigger: f32,
    pub right_trigger: f32,
    pub buttons: GamepadButtons,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    PointerMoved(Point),
    // The pointer left the window, with no button held.
    PointerLeft,
    ButtonPressed(Button),
    ButtonReleased(Button),
    // In notches of the wheel, with +y away from the user.
    Wheel(Point),
    KeyPressed { key: Key, repeat: bool },
    KeyReleased(Key),
    Text(char),
    // With the keys held when focus came, which count once let go of.
    FocusGained(Vec<Key>),
    FocusLost,
    Gamepad(Gamepad),
}

#[derive(Clone, Copy, Debug, Default)]
struct ButtonState {
    down: bool,
    pressed: bool,
    released: bool,
    // How many presses in a row, close in time and place: 2 for a double-click.
    clicks: u32,
    press_time: f64,
    press_position: Point,
}

// How close in time and place a press must be to the last to count as part
// of a double-click.
pub const DOUBLE_CLICK_SECONDS: f64 = 0.5;
const DOUBLE_CLICK_DISTANCE: f32 = 4.0;

// What happened this frame, and what's held.
#[derive(Clone, Debug)]
pub struct Input {
    pub pointer: Option<Point>,
    pub pointer_delta: Point,
    buttons: [ButtonState; 3],
    pub wheel: Point,
    keys_down: [bool; 256],
    keys_pressed: Vec<(Key, bool)>,
    keys_released: Vec<Key>,
    pub text: String,
    pub focused: bool,
    // Whether focus was lost this frame, which cancels presses and drags.
    pub focus_lost: bool,
    pub gamepad: Gamepad,
    pub previous_gamepad: Gamepad,
    // Gamepad buttons and keys held when the window got focus, which count
    // only once they're let go of.
    ignored_buttons: u32,
    ignored_keys: [bool; 256],
    // Whether the next gamepad reading's held buttons are to be ignored.
    ignore_gamepad: bool,
    // Seconds since the program started, and since the last frame.
    pub time: f64,
    pub delta: f32,
    last_pointer: Option<Point>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            pointer: None,
            pointer_delta: Point::default(),
            buttons: [ButtonState::default(); 3],
            wheel: Point::default(),
            keys_down: [false; 256],
            keys_pressed: Vec::new(),
            keys_released: Vec::new(),
            text: String::new(),
            focused: false,
            focus_lost: false,
            gamepad: Gamepad::default(),
            previous_gamepad: Gamepad::default(),
            ignored_buttons: 0,
            ignored_keys: [false; 256],
            ignore_gamepad: false,
            time: 0.0,
            delta: 0.0,
            last_pointer: None,
        }
    }
}

impl Input {
    // Starts a frame at `time`: what happened last frame is forgotten, and
    // the events since then apply in order.
    pub fn begin(&mut self, time: f64, events: &[Event]) {
        self.delta = (time - self.time).max(0.0) as f32;
        self.time = time;
        for button in &mut self.buttons {
            button.pressed = false;
            button.released = false;
        }
        self.wheel = Point::default();
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.text.clear();
        self.focus_lost = false;
        self.previous_gamepad = self.gamepad;
        self.last_pointer = self.pointer;
        for event in events {
            self.apply(event);
        }
        self.pointer_delta = match (self.pointer, self.last_pointer) {
            (Some(now), Some(before)) => now - before,
            _ => Point::default(),
        };
    }

    fn apply(&mut self, event: &Event) {
        match event {
            Event::FocusGained(held) => {
                self.focused = true;
                for key in held {
                    self.ignored_keys[key.0 as usize] = true;
                }
                self.ignore_gamepad = true;
            }
            Event::FocusLost => {
                self.focused = false;
                self.focus_lost = true;
                for button in &mut self.buttons {
                    button.down = false;
                }
                self.keys_down = [false; 256];
                self.ignored_keys = [false; 256];
                self.pointer = None;
                self.gamepad = Gamepad {
                    connected: self.gamepad.connected,
                    ..Gamepad::default()
                };
            }
            _ if !self.focused => {}
            Event::PointerMoved(point) => self.pointer = Some(*point),
            Event::PointerLeft => {
                if !self.buttons.iter().any(|button| button.down) {
                    self.pointer = None;
                }
            }
            Event::ButtonPressed(button) => {
                let time = self.time;
                let position = self.pointer.unwrap_or_default();
                let state = &mut self.buttons[button.index()];
                let repeat = state.clicks > 0
                    && time - state.press_time <= DOUBLE_CLICK_SECONDS
                    && position.distance(state.press_position) <= DOUBLE_CLICK_DISTANCE;
                state.clicks = if repeat { state.clicks + 1 } else { 1 };
                state.down = true;
                state.pressed = true;
                state.press_time = time;
                state.press_position = position;
            }
            Event::ButtonReleased(button) => {
                let state = &mut self.buttons[button.index()];
                if state.down {
                    state.down = false;
                    state.released = true;
                }
            }
            Event::Wheel(notches) => self.wheel = self.wheel + *notches,
            Event::KeyPressed { key, repeat } => {
                if self.ignored_keys[key.0 as usize] {
                    return;
                }
                self.keys_down[key.0 as usize] = true;
                self.keys_pressed.push((*key, *repeat));
            }
            Event::KeyReleased(key) => {
                if self.ignored_keys[key.0 as usize] {
                    self.ignored_keys[key.0 as usize] = false;
                    return;
                }
                if self.keys_down[key.0 as usize] {
                    self.keys_down[key.0 as usize] = false;
                    self.keys_released.push(*key);
                }
            }
            Event::Text(character) => self.text.push(*character),
            Event::Gamepad(gamepad) => {
                // Buttons held since focus came count once they're let go of.
                if std::mem::replace(&mut self.ignore_gamepad, false) {
                    self.ignored_buttons = gamepad.buttons.0;
                }
                self.ignored_buttons &= gamepad.buttons.0;
                let mut gamepad = *gamepad;
                gamepad.buttons.0 &= !self.ignored_buttons;
                self.gamepad = gamepad;
            }
        }
    }

    pub fn button_down(&self, button: Button) -> bool {
        self.buttons[button.index()].down
    }

    pub fn button_pressed(&self, button: Button) -> bool {
        self.buttons[button.index()].pressed
    }

    pub fn button_released(&self, button: Button) -> bool {
        self.buttons[button.index()].released
    }

    // 1 for a click, 2 for a double-click, and so on, for this frame's press.
    pub fn clicks(&self, button: Button) -> u32 {
        self.buttons[button.index()].clicks
    }

    pub fn press_position(&self, button: Button) -> Point {
        self.buttons[button.index()].press_position
    }

    pub fn any_button_down(&self) -> bool {
        self.buttons.iter().any(|button| button.down)
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.keys_down[key.0 as usize]
    }

    // Pressed this frame, counting the repeats of a held key.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.iter().any(|(pressed, _)| *pressed == key)
    }

    // Pressed this frame, not counting repeats.
    pub fn key_pressed_once(&self, key: Key) -> bool {
        self.keys_pressed.iter().any(|(pressed, repeat)| *pressed == key && !repeat)
    }

    pub fn key_released(&self, key: Key) -> bool {
        self.keys_released.contains(&key)
    }

    pub fn keys_pressed(&self) -> &[(Key, bool)] {
        &self.keys_pressed
    }

    pub fn keys_held(&self) -> Vec<Key> {
        (0..=255u8).map(Key).filter(|key| self.key_down(*key)).collect()
    }

    pub fn modifiers(&self) -> Modifiers {
        Modifiers {
            shift: self.key_down(Key::SHIFT),
            control: self.key_down(Key::CONTROL),
            alt: self.key_down(Key::ALT),
        }
    }

    // Whether a gamepad button went down this frame.
    pub fn gamepad_pressed(&self, button: u32) -> bool {
        self.gamepad.buttons.contains(button) && !self.previous_gamepad.buttons.contains(button)
    }
}
