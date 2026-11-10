// What input comes from: keys and their names, shortcuts like Ctrl+S, mouse
// buttons, and the gamepad. Scripts name keys and buttons the way these do.

use crate::drawing::geometry::Point;

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
    pub const DIGIT_0: Key = Key(b'0');
    pub const DIGIT_1: Key = Key(b'1');
    pub const DIGIT_2: Key = Key(b'2');
    pub const DIGIT_3: Key = Key(b'3');
    pub const DIGIT_4: Key = Key(b'4');
    pub const DIGIT_5: Key = Key(b'5');
    pub const DIGIT_6: Key = Key(b'6');
    pub const DIGIT_7: Key = Key(b'7');
    pub const DIGIT_8: Key = Key(b'8');
    pub const DIGIT_9: Key = Key(b'9');
    pub const A: Key = Key(b'A');
    pub const B: Key = Key(b'B');
    pub const C: Key = Key(b'C');
    pub const D: Key = Key(b'D');
    pub const E: Key = Key(b'E');
    pub const F: Key = Key(b'F');
    pub const G: Key = Key(b'G');
    pub const H: Key = Key(b'H');
    pub const I: Key = Key(b'I');
    pub const J: Key = Key(b'J');
    pub const K: Key = Key(b'K');
    pub const L: Key = Key(b'L');
    pub const M: Key = Key(b'M');
    pub const N: Key = Key(b'N');
    pub const O: Key = Key(b'O');
    pub const P: Key = Key(b'P');
    pub const Q: Key = Key(b'Q');
    pub const R: Key = Key(b'R');
    pub const S: Key = Key(b'S');
    pub const T: Key = Key(b'T');
    pub const U: Key = Key(b'U');
    pub const V: Key = Key(b'V');
    pub const W: Key = Key(b'W');
    pub const X: Key = Key(b'X');
    pub const Y: Key = Key(b'Y');
    pub const Z: Key = Key(b'Z');
    pub const LEFT_WINDOWS: Key = Key(0x5B);
    pub const RIGHT_WINDOWS: Key = Key(0x5C);
    pub const NUMPAD_0: Key = Key(0x60);
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
    pub const LEFT_SHIFT: Key = Key(0xA0);
    pub const RIGHT_SHIFT: Key = Key(0xA1);
    pub const LEFT_CONTROL: Key = Key(0xA2);
    pub const RIGHT_CONTROL: Key = Key(0xA3);
    pub const LEFT_ALT: Key = Key(0xA4);
    pub const RIGHT_ALT: Key = Key(0xA5);

    // Names that aren't the key's character or F-number.
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
        (Key::LEFT_SHIFT, "LeftShift"),
        (Key::RIGHT_SHIFT, "RightShift"),
        (Key::LEFT_CONTROL, "LeftCtrl"),
        (Key::RIGHT_CONTROL, "RightCtrl"),
        (Key::LEFT_ALT, "LeftAlt"),
        (Key::RIGHT_ALT, "RightAlt"),
        (Key(0xBA), "Semicolon"),
        (Key(0xBB), "Plus"),
        (Key(0xBC), "Comma"),
        (Key(0xBD), "Minus"),
        (Key(0xBE), "Period"),
        (Key(0xBF), "Slash"),
        (Key(0xC0), "Backtick"),
        (Key(0xDE), "Quote"),
    ];

    // The key of a letter or a digit.
    pub fn character(character: char) -> Option<Key> {
        let upper = character.to_ascii_uppercase();
        upper.is_ascii_alphanumeric().then_some(Key(upper as u8))
    }

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
        match upper.as_str() {
            "CONTROL" => return Some(Key::CONTROL),
            "ESCAPE" => return Some(Key::ESCAPE),
            "RETURN" => return Some(Key::ENTER),
            _ => {}
        }
        if upper.chars().count() == 1 {
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
        if let Some(code) = upper.strip_prefix("KEY").and_then(|digits| u8::from_str_radix(digits, 16).ok()) {
            return Some(Key(code));
        }
        None
    }

    pub fn is_modifier(self) -> bool {
        matches!(self.0, 0x10..=0x12 | 0x5B | 0x5C | 0xA0..=0xA5)
    }

    pub fn is_function(self) -> bool {
        (0x70..=0x87).contains(&self.0)
    }

    // The arrows, Home, End, Page Up and Page Down.
    pub fn is_navigation(self) -> bool {
        matches!(self, Key::LEFT | Key::RIGHT | Key::UP | Key::DOWN | Key::HOME | Key::END | Key::PAGE_UP | Key::PAGE_DOWN)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub control: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Modifiers {
    pub const NONE: Modifiers = Modifiers::new(false, false, false);
    pub const CONTROL: Modifiers = Modifiers::new(true, false, false);
    pub const SHIFT: Modifiers = Modifiers::new(false, true, false);
    pub const ALT: Modifiers = Modifiers::new(false, false, true);
    pub const CONTROL_SHIFT: Modifiers = Modifiers::new(true, true, false);

    pub const fn new(control: bool, shift: bool, alt: bool) -> Self {
        Self { control, shift, alt }
    }

    pub fn any(&self) -> bool {
        self.control || self.shift || self.alt
    }
}

// A key with the modifiers held for it, like Ctrl+S.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: Key,
}

impl Shortcut {
    pub const fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    pub const fn key(key: Key) -> Self {
        Self::new(Modifiers::NONE, key)
    }

    pub const fn control(key: Key) -> Self {
        Self::new(Modifiers::CONTROL, key)
    }

    // From its name, like "Ctrl+Shift+S" or "F11".
    pub fn parse(text: &str) -> Option<Shortcut> {
        let parts: Vec<&str> = text.split('+').map(str::trim).collect();
        let (last, held) = parts.split_last()?;
        let mut modifiers = Modifiers::NONE;
        for part in held {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers.control = true,
                "shift" => modifiers.shift = true,
                "alt" => modifiers.alt = true,
                _ => return None,
            }
        }
        Some(Shortcut::new(modifiers, Key::parse(last)?))
    }

    // As Windows writes it in menus, like "Ctrl+Shift+S".
    pub fn name(&self) -> String {
        let mut name = String::new();
        for (held, word) in [(self.modifiers.control, "Ctrl+"), (self.modifiers.shift, "Shift+"), (self.modifiers.alt, "Alt+")] {
            if held {
                name.push_str(word);
            }
        }
        name.push_str(&self.key.name());
        name
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    pub const ALL: [MouseButton; 3] = [MouseButton::Left, MouseButton::Right, MouseButton::Middle];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            MouseButton::Left => "Left",
            MouseButton::Right => "Right",
            MouseButton::Middle => "Middle",
        }
    }

    pub fn parse(name: &str) -> Option<MouseButton> {
        MouseButton::ALL.into_iter().find(|button| button.name().eq_ignore_ascii_case(name))
    }
}

// The gamepad's buttons, as bits, which are GameInput's own.
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
    pub const ALL: u32 = (1 << 14) - 1;

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
        GamepadButtons::NAMES.iter().filter(|(bit, _)| self.contains(*bit)).map(|(_, name)| *name).collect()
    }

    pub fn parse(name: &str) -> Option<u32> {
        GamepadButtons::NAMES.iter().find(|(_, known)| known.eq_ignore_ascii_case(name)).map(|(bit, _)| *bit)
    }
}

// A gamepad's reading: sticks from -1 to 1 with +y up, and triggers from 0
// to 1, raw, without dead zones.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gamepad {
    pub connected: bool,
    pub left_stick: Point,
    pub right_stick: Point,
    pub left_trigger: f32,
    pub right_trigger: f32,
    pub buttons: GamepadButtons,
}
