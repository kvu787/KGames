// What a widget is to the input: its identity, what it senses, its targets,
// which the pointer can be on, and the interaction that says what the input
// did to it in a frame. These are the types between the interface's frame
// machinery, in Interface.rs, and the widgets.

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

// A part of a widget that's a target of its own, like a list's row or a
// text field's clear button. The whole widget is `Whole`. A part's target
// is named "<widget>/<suffix>", and keeps the part, so that the widget knows
// which of its targets the input was on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    Whole,
    // By its index, like a row, an item or a swatch.
    Index(usize),
    // By a name, like "Clear" or "Up", or a menu item's path.
    Named(String),
}

impl Part {
    pub fn index(&self) -> Option<usize> {
        match self {
            Part::Index(index) => Some(*index),
            _ => None,
        }
    }

    pub fn is_named(&self, name: &str) -> bool {
        matches!(self, Part::Named(own) if own == name)
    }
}

// A widget's area, which the pointer can be on: the whole widget, or one
// of its parts.
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
    pub sense: Sense,
    // Like "Button" or "Slider", as WinUI names its controls.
    pub kind: &'static str,
    // Whether scripts and tests see it. Surfaces that only cover what's
    // under them, like cards, aren't listed.
    pub listed: bool,
    // The widget it belongs to, and which part of it.
    pub owner: Id,
    pub part: Part,
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

// What the input did to a target this frame. Widgets act on presses, on the
// frame that the button goes down, and never wait for its release: then a
// button sends its message, and a menu or a context menu opens.
#[derive(Clone, Debug, Default)]
pub struct Hit {
    // The pointer is on it, and it's enabled.
    pub hovered: bool,
    // The left button went down on it this frame.
    pub pressed: bool,
    // It has a press: a button that went down on it is still down.
    pub held: bool,
    // Its left press ended this frame, wherever the pointer was, which ends
    // a drag, like a slider's.
    pub released: bool,
    // The left button went down on it, or Space or Enter went down while it
    // had focus: what a button acts on.
    pub activated: bool,
    // Its press was the second of a double-click.
    pub double_clicked: bool,
    // The right button went down on it, which opens its context menu.
    pub secondary_pressed: bool,
    // The middle button went down on it.
    pub middle_pressed: bool,
    // The button that its press is of, while it's held.
    pub held_button: Option<MouseButton>,
    // Notches of the wheel turned over it.
    pub wheel: Point,
}

impl Hit {
    // Whether the left button holds it down.
    pub fn pressing(&self) -> bool {
        self.held && self.held_button == Some(MouseButton::Left)
    }

    // WinUI's visual state, from what the input did to it.
    pub fn state(&self, enabled: bool) -> ControlState {
        if !enabled {
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

// What the input did to a widget and its parts this frame, which the widget
// turns into messages and changes of its own state.
#[derive(Clone, Debug, Default)]
pub struct Interaction {
    pub enabled: bool,
    // What happened to each of its targets that something happened to, by
    // part. A part that nothing happened to isn't here.
    pub hits: Vec<(Part, Hit)>,
    // It has keyboard focus, and the keys reach it: no popup or dialog over
    // it takes them. The part with focus, if the widget has focus at all,
    // whether or not the keys reach it.
    pub focused: bool,
    pub focused_part: Option<Part>,
    // The pointer, while it's on the widget or the widget is held.
    pub pointer: Option<Point>,
    // How far the pointer moved this frame while the widget was held.
    pub drag_delta: Point,
    // Notches of the wheel turned over the widget's area while what's under
    // the pointer doesn't take the wheel itself, for what scrolls.
    pub wheel_over: Point,
}

impl Interaction {
    pub fn hit(&self, part: &Part) -> Option<&Hit> {
        self.hits.iter().find(|(at, _)| at == part).map(|(_, hit)| hit)
    }

    // The whole widget's hit, or nothing's.
    pub fn whole(&self) -> Hit {
        self.hit(&Part::Whole).cloned().unwrap_or_default()
    }

    // The part that the pointer is on.
    pub fn hovered_part(&self) -> Option<&Part> {
        self.hits.iter().find(|(_, hit)| hit.hovered).map(|(part, _)| part)
    }

    // The part with a press.
    pub fn held_part(&self) -> Option<&Part> {
        self.hits.iter().find(|(_, hit)| hit.held).map(|(part, _)| part)
    }

    pub fn pressed_part(&self) -> Option<&Part> {
        self.hits.iter().find(|(_, hit)| hit.pressed).map(|(part, _)| part)
    }

    pub fn activated_part(&self) -> Option<&Part> {
        self.hits.iter().find(|(_, hit)| hit.activated).map(|(part, _)| part)
    }

    pub fn activated(&self, part: &Part) -> bool {
        self.hit(part).is_some_and(|hit| hit.activated)
    }

    pub fn pressed(&self, part: &Part) -> bool {
        self.hit(part).is_some_and(|hit| hit.pressed)
    }

    pub fn held(&self, part: &Part) -> bool {
        self.hit(part).is_some_and(|hit| hit.held)
    }

    pub fn hovered(&self, part: &Part) -> bool {
        self.hit(part).is_some_and(|hit| hit.hovered)
    }

    pub fn any_held(&self) -> bool {
        self.hits.iter().any(|(_, hit)| hit.held)
    }

    // The wheel over any of its targets.
    pub fn wheel(&self) -> Point {
        self.hits.iter().map(|(_, hit)| hit.wheel).fold(Point::ZERO, |sum, wheel| sum + wheel)
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
