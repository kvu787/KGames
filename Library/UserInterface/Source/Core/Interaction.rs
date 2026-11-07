// What a widget is to the input: its identity, what it senses, its target,
// which the pointer can be on, and the response that says what the input did
// to it. These are the types between the Ui's frame machinery and the
// widgets; the machinery itself is in Ui.rs.

use crate::drawing::geometry::{Point, Rect};
use crate::drawing::painter::Layer;
use crate::input::devices::MouseButton;

pub type Id = u64;

// A widget's identity, from its name: FNV-1a, which is fast for short names.
pub fn id(name: &str) -> Id {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

// The keys that a widget takes while it has keyboard focus, which the
// program's shortcuts then don't see.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Keys {
    #[default]
    None,
    // Space and Enter, like a button's.
    Activate,
    // The arrows, Home, End, Page Up and Page Down, and Space and Enter,
    // like a slider's or a list's.
    Navigate,
    // Everything typed, and the keys that edit text.
    Text,
}

// What a widget does with the wheel turned over it, which decides whether
// what it's in may scroll instead.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Wheel {
    // Leaves the wheel to what it's in, like a list or a scroll area.
    #[default]
    Ignored,
    // Takes the wheel, like a slider or a 3D view.
    Taken,
    // Takes the wheel only while it has keyboard focus, as WinUI's
    // NumberBox does.
    WhileFocused,
}

// What a widget wants from the pointer and the keyboard.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sense {
    // Presses of each mouse button on it.
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    // Whether it takes keyboard focus when pressed, and with Tab.
    pub focusable: bool,
    pub keys: Keys,
    pub wheel: Wheel,
}

impl Sense {
    // Covers what's under it, and nothing else, like a card.
    pub const NONE: Sense = Sense::new(false, false, false, false, Keys::None);
    // Left presses, without taking focus.
    pub const CLICK: Sense = Sense::new(true, false, false, false, Keys::None);
    // Like a button: left presses, focus, and Space and Enter.
    pub const BUTTON: Sense = Sense::new(true, false, false, true, Keys::Activate);
    // Like a slider or a list: focus, and the keys that move.
    pub const NAVIGATE: Sense = Sense::new(true, false, false, true, Keys::Navigate);
    pub const TEXT: Sense = Sense::new(true, false, false, true, Keys::Text);
    // Every button and the wheel, without focus, like a 3D view's.
    pub const VIEW: Sense = Sense::new(true, true, true, false, Keys::None).takes_wheel();

    pub const fn new(left: bool, right: bool, middle: bool, focusable: bool, keys: Keys) -> Self {
        Self {
            left,
            right,
            middle,
            focusable,
            keys,
            wheel: Wheel::Ignored,
        }
    }

    // The widget takes the wheel, so what it's in won't scroll under it.
    pub const fn takes_wheel(mut self) -> Self {
        self.wheel = Wheel::Taken;
        self
    }

    pub const fn takes_wheel_while_focused(mut self) -> Self {
        self.wheel = Wheel::WhileFocused;
        self
    }

    pub(crate) fn takes(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.left,
            MouseButton::Right => self.right,
            MouseButton::Middle => self.middle,
        }
    }
}

// A widget's area, which the pointer can be on.
#[derive(Clone, Debug)]
pub struct Target {
    pub id: Id,
    pub name: String,
    // What it shows, like a row's text, for scripts and tests.
    pub label: Option<String>,
    // Its area, inside the clip it was in.
    pub rect: Rect,
    pub layer: Layer,
    pub enabled: bool,
    pub focusable: bool,
    pub keys: Keys,
    pub wheel: Wheel,
    // Like "Button" or "Slider", as WinUI names its controls.
    pub kind: &'static str,
    // Whether scripts and tests see it. Surfaces that only cover what's
    // under them, like cards, aren't listed.
    pub listed: bool,
}

// WinUI's visual states of a control, which pick its brushes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ControlState {
    #[default]
    Normal,
    PointerOver,
    Pressed,
    Disabled,
}

// What the input did to a widget this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct Response {
    pub id: Id,
    pub rect: Rect,
    pub enabled: bool,
    // The pointer is on it, and it's enabled.
    pub hovered: bool,
    // The left button went down on it this frame.
    pub pressed: bool,
    // It has a press: a button that went down on it is still down.
    pub held: bool,
    // Its left press ended this frame, wherever the pointer was.
    pub released: bool,
    // Its left press ended on it, or Space or Enter pressed it.
    pub clicked: bool,
    // Its press was the second of a double-click.
    pub double_clicked: bool,
    // The right button went down on it, or came up on it after going down
    // on it, which is when WinUI asks for a context menu.
    pub secondary_pressed: bool,
    pub secondary_clicked: bool,
    // The middle button went down on it.
    pub middle_pressed: bool,
    // The button that its press is of, while it's held.
    pub held_button: Option<MouseButton>,
    pub focused: bool,
    // Its value changed, for widgets that have one.
    pub changed: bool,
    // The pointer, while it's on the widget or the widget is held.
    pub pointer: Option<Point>,
    // How far the pointer moved this frame while the widget was held.
    pub drag_delta: Point,
    // Notches of the wheel turned over it.
    pub wheel: Point,
}

impl Response {
    // Whether the left button holds it down, as its look shows it: from the
    // frame that reads the press until the frame that reads the release,
    // which shows it let go. It's still held on that frame, for what the
    // release does, like a click.
    pub fn pressing(&self) -> bool {
        self.held && self.held_button == Some(MouseButton::Left) && !self.released
    }

    pub fn state(&self) -> ControlState {
        if !self.enabled {
            ControlState::Disabled
        } else if self.pressing() && self.hovered {
            ControlState::Pressed
        } else if self.hovered || self.pressing() {
            ControlState::PointerOver
        } else {
            ControlState::Normal
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Cursor {
    #[default]
    Arrow,
    Text,
    Hand,
    Move,
    ResizeHorizontal,
    ResizeVertical,
    ResizeDiagonal,
    ResizeAntidiagonal,
    Crosshair,
    NotAllowed,
}
