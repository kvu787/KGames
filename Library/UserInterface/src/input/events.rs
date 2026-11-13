// A frame's input: the events since the last frame, from the window, or
// made up by a script or a test, in the order they happened. The interface
// takes them in steps (see `Input::steps`), each of which it handles on the
// state that the steps before it left, and gathers each step's events into
// what happened in the step and what's held. Programs and widgets read it
// the same way whatever made it.
//
// Without window focus there's no input at all. As focus goes, everything
// held is let go of on that step: keys and mouse buttons are released, the
// gamepad goes to rest, and `focus_lost` says why. Keys and gamepad buttons
// held when focus comes count only once they're let go of, so that the Alt
// of an Alt+Tab, or a button pressed in another program, does nothing here.

use crate::drawing::geometry::Point;
use crate::input::devices::{Gamepad, Key, Modifiers, MouseButton, Shortcut};

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    PointerMoved(Point),
    // The pointer left the window with no button held.
    PointerLeft,
    ButtonPressed(MouseButton),
    ButtonReleased(MouseButton),
    // In notches of the wheel, +y away from the user, +x to the right.
    Wheel(Point),
    KeyPressed { key: Key, repeat: bool },
    KeyReleased(Key),
    Text(char),
    // With the keys held as focus came, which count once they're let go of.
    FocusGained(Vec<Key>),
    FocusLost,
    Gamepad(Gamepad),
}

// How close in time and place a press must be to the last one to make a
// double-click: Windows' defaults.
pub const DOUBLE_CLICK_TIME: f64 = 0.5;
pub const DOUBLE_CLICK_DISTANCE: f32 = 4.0;

#[derive(Clone, Copy, Debug, Default)]
struct ButtonState {
    down: bool,
    pressed: bool,
    released: bool,
    // Presses in a row, close in time and place: 2 for a double-click.
    clicks: u32,
    press_time: f64,
    press_position: Point,
}

#[derive(Clone, Debug)]
pub struct Input {
    // Seconds since the program started, and since the step before: the
    // time since the last frame in a frame's last step, and 0 in the steps
    // before it, so that time passes once a frame, after its events.
    pub time: f64,
    pub delta: f32,
    pub focused: bool,
    // Focus went away in this step, which ends presses and drags.
    pub focus_lost: bool,
    pub pointer: Option<Point>,
    // How far the pointer moved in this step.
    pub pointer_delta: Point,
    // Notches of the wheel in this step.
    pub wheel: Point,
    // What was typed in this step.
    pub text: String,
    pub gamepad: Gamepad,
    // The gamepad as the step before left it.
    pub previous_gamepad: Gamepad,
    buttons: [ButtonState; 3],
    keys_down: [bool; 256],
    // This step's presses, and whether each is a held key's repeat.
    keys_pressed: Vec<(Key, bool)>,
    keys_released: Vec<Key>,
    ignored_keys: [bool; 256],
    ignored_gamepad_buttons: u32,
    ignore_next_gamepad: bool,
    // The gamepad's buttons as the last Gamepad event that `steps` saw had
    // them, which tells a change of them, which is a step, from a reading
    // of the sticks, which isn't.
    step_gamepad_buttons: u32,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            time: 0.0,
            delta: 0.0,
            focused: false,
            focus_lost: false,
            pointer: None,
            pointer_delta: Point::ZERO,
            wheel: Point::ZERO,
            text: String::new(),
            gamepad: Gamepad::default(),
            previous_gamepad: Gamepad::default(),
            buttons: [ButtonState::default(); 3],
            keys_down: [false; 256],
            keys_pressed: Vec::new(),
            keys_released: Vec::new(),
            ignored_keys: [false; 256],
            ignored_gamepad_buttons: 0,
            ignore_next_gamepad: false,
            step_gamepad_buttons: 0,
        }
    }
}

impl Input {
    // Splits a frame's events into its steps, in order, which the interface
    // handles one at a time, each on the state that the steps before it
    // left, so that each event sees what the events before it did. Each
    // event that does something is the end of a step: a press or a release
    // of a button or a key, with the text that a key's press types, the
    // wheel, text typed, focus coming or going, or a change of the gamepad's
    // buttons. The events before it that only move something, the pointer
    // or the gamepad's sticks, are its step's too, so that it happens where
    // they left the pointer. Those after the frame's last such event are a
    // step of their own if they move the pointer, so that the event before
    // them happens where the pointer was then, and are the last step's
    // otherwise. A frame with no events is one step with none.
    pub fn steps<'a>(&mut self, events: &'a [Event]) -> Vec<&'a [Event]> {
        let mut steps: Vec<&'a [Event]> = Vec::new();
        // Where the step being gathered starts, and where the last one did.
        let mut start = 0;
        let mut last_start = 0;
        let mut index = 0;
        while index < events.len() {
            let acts = match &events[index] {
                Event::PointerMoved(_) | Event::PointerLeft => false,
                Event::Gamepad(gamepad) => std::mem::replace(&mut self.step_gamepad_buttons, gamepad.buttons.0) != gamepad.buttons.0,
                _ => true,
            };
            index += 1;
            if !acts {
                continue;
            }
            if matches!(events[index - 1], Event::KeyPressed { .. }) {
                while index < events.len() && matches!(events[index], Event::Text(_)) {
                    index += 1;
                }
            }
            steps.push(&events[start..index]);
            last_start = start;
            start = index;
        }
        let rest = &events[start..];
        let moves_pointer = rest.iter().any(|event| matches!(event, Event::PointerMoved(_) | Event::PointerLeft));
        if steps.is_empty() || moves_pointer {
            steps.push(rest);
        } else if !rest.is_empty() {
            *steps.last_mut().unwrap() = &events[last_start..];
        }
        steps
    }

    // Starts a step at the time: the last step's presses are forgotten, and
    // its events apply in order.
    pub fn begin(&mut self, time: f64, events: &[Event]) {
        self.delta = (time - self.time).max(0.0) as f32;
        self.time = time;
        for button in &mut self.buttons {
            button.pressed = false;
            button.released = false;
        }
        self.wheel = Point::ZERO;
        self.text.clear();
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.focus_lost = false;
        self.previous_gamepad = self.gamepad;
        let before = self.pointer;
        for event in events {
            self.apply(event);
        }
        self.pointer_delta = match (self.pointer, before) {
            (Some(now), Some(before)) => now - before,
            _ => Point::ZERO,
        };
    }

    fn apply(&mut self, event: &Event) {
        match event {
            Event::FocusGained(held) => {
                self.focused = true;
                for key in held {
                    self.ignored_keys[key.0 as usize] = true;
                }
                self.ignore_next_gamepad = true;
            }
            Event::FocusLost => {
                self.focused = false;
                self.focus_lost = true;
                for button in &mut self.buttons {
                    if button.down {
                        button.down = false;
                        button.released = true;
                    }
                }
                for code in 0..=255u8 {
                    if std::mem::replace(&mut self.keys_down[code as usize], false) {
                        self.keys_released.push(Key(code));
                    }
                }
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
                if !self.any_button_down() {
                    self.pointer = None;
                }
            }
            Event::ButtonPressed(button) => {
                let time = self.time;
                let position = self.pointer.unwrap_or_default();
                let state = &mut self.buttons[button.index()];
                let again = state.clicks > 0
                    && time - state.press_time <= DOUBLE_CLICK_TIME
                    && position.distance(state.press_position) <= DOUBLE_CLICK_DISTANCE;
                state.clicks = if again { state.clicks + 1 } else { 1 };
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
            Event::Wheel(notches) => self.wheel += *notches,
            Event::KeyPressed { key, repeat } => {
                if self.ignored_keys[key.0 as usize] {
                    return;
                }
                self.keys_down[key.0 as usize] = true;
                self.keys_pressed.push((*key, *repeat));
            }
            Event::KeyReleased(key) => {
                let index = key.0 as usize;
                if std::mem::replace(&mut self.ignored_keys[index], false) {
                    return;
                }
                if self.keys_down[index] {
                    self.keys_down[index] = false;
                    self.keys_released.push(*key);
                }
            }
            Event::Text(character) => self.text.push(*character),
            Event::Gamepad(reading) => {
                if std::mem::replace(&mut self.ignore_next_gamepad, false) {
                    self.ignored_gamepad_buttons = reading.buttons.0;
                }
                // A button that's let go of counts again from then on.
                self.ignored_gamepad_buttons &= reading.buttons.0;
                let mut reading = *reading;
                reading.buttons.0 &= !self.ignored_gamepad_buttons;
                self.gamepad = reading;
            }
        }
    }

    pub fn button_down(&self, button: MouseButton) -> bool {
        self.buttons[button.index()].down
    }

    pub fn button_pressed(&self, button: MouseButton) -> bool {
        self.buttons[button.index()].pressed
    }

    pub fn button_released(&self, button: MouseButton) -> bool {
        self.buttons[button.index()].released
    }

    // 1 for this step's press if it's a click, 2 for a double-click, and on.
    pub fn clicks(&self, button: MouseButton) -> u32 {
        self.buttons[button.index()].clicks
    }

    // Where the button last went down.
    pub fn press_position(&self, button: MouseButton) -> Point {
        self.buttons[button.index()].press_position
    }

    pub fn any_button_down(&self) -> bool {
        self.buttons.iter().any(|button| button.down)
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.keys_down[key.0 as usize]
    }

    // Pressed in this step, counting a held key's repeats.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.iter().any(|(pressed, _)| *pressed == key)
    }

    // Pressed in this step, not counting repeats.
    pub fn key_pressed_once(&self, key: Key) -> bool {
        self.keys_pressed.iter().any(|(pressed, repeat)| *pressed == key && !repeat)
    }

    pub fn key_released(&self, key: Key) -> bool {
        self.keys_released.contains(&key)
    }

    // This step's releases, in order.
    pub fn keys_released(&self) -> &[Key] {
        &self.keys_released
    }

    // This step's presses, with whether each is a repeat.
    pub fn keys_pressed(&self) -> &[(Key, bool)] {
        &self.keys_pressed
    }

    pub fn keys_held(&self) -> Vec<Key> {
        (0..=255u8).map(Key).filter(|key| self.key_down(*key)).collect()
    }

    pub fn modifiers(&self) -> Modifiers {
        Modifiers::new(self.key_down(Key::CONTROL), self.key_down(Key::SHIFT), self.key_down(Key::ALT))
    }

    // Whether the shortcut's key was pressed in this step, with exactly its
    // modifiers held.
    pub fn shortcut_pressed(&self, shortcut: Shortcut) -> bool {
        self.modifiers() == shortcut.modifiers && self.key_pressed(shortcut.key)
    }

    // Whether a gamepad button went down in this step.
    pub fn gamepad_pressed(&self, button: u32) -> bool {
        self.gamepad.buttons.contains(button) && !self.previous_gamepad.buttons.contains(button)
    }

    pub fn gamepad_released(&self, button: u32) -> bool {
        !self.gamepad.buttons.contains(button) && self.previous_gamepad.buttons.contains(button)
    }
}
