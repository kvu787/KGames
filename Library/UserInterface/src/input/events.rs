// A frame's input, FrameInput: the time since the frame before, and the
// events since then, from GameInput's readings, or made up by a script or a
// test, in the order they happened, which is all that the interface and
// programs get of the devices and the clock. The interface takes the events
// in steps (see `Input::steps`), each of which it
// handles on the state that the steps before it left, and gathers each
// step's events into what happened in the step and what's held. Programs
// and widgets read it the same way whatever made it.
//
// The window's focus isn't input: the reader (input/reader.rs) gives no
// events without it, lets go of everything held as it goes, with releases
// and the gamepad at rest, and holds back what's held as it comes, until
// it's let go of. So a program never hears of focus, and nothing it holds
// is ever stuck.
//
// The pointer is the interface's: the mice's motion moves it, in GameInput's
// counts times MOUSE_SENSITIVITY, inside the frame, as SDL's relative mode
// keeps one. It starts in the frame's middle with the first of the mice's
// events, and carries on from where it is after that, except where it's
// placed: a window places it where Windows' cursor is, or in the middle if
// that's outside the window, as it starts to keep the cursor, like as it
// gets focus (see platform/cursor.rs), which isn't motion. It moves in the events' order, so a press happens where the
// pointer was as the button went down. It's worked out in f64, so that a
// test's motion, worked out from where the pointer is to a point, lands on
// that point exactly (see testing/gestures.rs).

use crate::drawing::geometry::{Point, Size};
use crate::input::devices::{Gamepad, Key, Modifiers, MouseButton, Shortcut};
use crate::input::typing::typed;

// How far the pointer moves for each count of the mice's motion, in pixels.
pub const MOUSE_SENSITIVITY: f64 = 0.8;

// A frame's input.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrameInput {
    // Seconds since the frame before, which the frame's last step has as
    // Ui::delta, and which Ui::time adds up.
    pub delta: f64,
    // What happened since the frame before, in the order it happened.
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    // The mice's motion, in GameInput's counts, before MOUSE_SENSITIVITY. A
    // real mouse moves by whole counts; a test may move by parts of one, to
    // land the pointer on a point.
    MouseMoved { x: f64, y: f64 },
    // The pointer put at a point in the frame, inside it, as the mice's input
    // starts: where Windows' cursor was as the window started to keep it, or
    // the middle if that was outside the window.
    // Unlike motion, it's no part of `Input::pointer_delta`, so a drag or a
    // camera that the pointer turns doesn't jump with it.
    PointerPlaced(Point),
    ButtonPressed(MouseButton),
    ButtonReleased(MouseButton),
    // In notches of the wheel, +y away from the user, +x to the right.
    Wheel(Point),
    KeyPressed(Key),
    KeyReleased(Key),
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
    // Seconds since the program started, the frame's in each of its steps,
    // and since the last frame, in a frame's last step, and 0 in the steps
    // before it, so that a simulation moves once a frame, after its events.
    pub time: f64,
    pub delta: f32,
    // The frame's size, which the pointer stays inside, and starts in the
    // middle of.
    pub size: Size,
    pub pointer: Option<Point>,
    // How far the mice's motion moved the pointer in this step, without
    // where it was placed.
    pub pointer_delta: Point,
    // Notches of the wheel in this step.
    pub wheel: Point,
    // What was typed in this step: the ASCII that its key types (see
    // input/typing.rs).
    pub text: String,
    pub gamepad: Gamepad,
    // The gamepad as the step before left it.
    pub previous_gamepad: Gamepad,
    buttons: [ButtonState; 3],
    keys_down: [bool; 256],
    // This step's presses.
    keys_pressed: Vec<Key>,
    keys_released: Vec<Key>,
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
            size: Size::default(),
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
            step_gamepad_buttons: 0,
        }
    }
}

impl Input {
    // Splits a frame's events into its steps, in order, which the interface
    // handles one at a time, each on the state that the steps before it
    // left, so that each event sees what the events before it did. Each
    // event that does something is the end of a step: a press or a release
    // of a button or a key, the wheel, or a change of the gamepad's
    // buttons. A key's press and the text it types are one
    // step, as the text is made from the key. The events before it that only
    // move or place something, the
    // pointer or the gamepad's sticks, are its step's too, so that it
    // happens where they left the pointer. Those after the frame's last such
    // event are a step of their own if they move the pointer, so that the
    // event before them happens where the pointer was then, and are the last
    // step's otherwise. A frame with no events is one step with none.
    pub fn steps<'a>(&mut self, events: &'a [Event]) -> Vec<&'a [Event]> {
        let mut steps: Vec<&'a [Event]> = Vec::new();
        // Where the step being gathered starts, and where the last one did.
        let mut start = 0;
        let mut last_start = 0;
        let mut index = 0;
        while index < events.len() {
            let acts = match &events[index] {
                Event::MouseMoved { .. } | Event::PointerPlaced(_) => false,
                Event::Gamepad(gamepad) => std::mem::replace(&mut self.step_gamepad_buttons, gamepad.buttons.0) != gamepad.buttons.0,
                _ => true,
            };
            index += 1;
            if !acts {
                continue;
            }
            steps.push(&events[start..index]);
            last_start = start;
            start = index;
        }
        let rest = &events[start..];
        let moves_pointer = rest.iter().any(|event| matches!(event, Event::MouseMoved { .. } | Event::PointerPlaced(_)));
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
        self.previous_gamepad = self.gamepad;
        let before = self.pointer;
        // How far the step's placements took the pointer, which isn't motion.
        let mut placed = Point::ZERO;
        for event in events {
            let from = self.pointer;
            self.apply(event);
            if let (Event::PointerPlaced(_), Some(from), Some(to)) = (event, from, self.pointer) {
                placed += to - from;
            }
        }
        self.pointer_delta = match (self.pointer, before) {
            (Some(now), Some(before)) => now - before - placed,
            _ => Point::ZERO,
        };
    }

    fn apply(&mut self, event: &Event) {
        // The pointer starts with the mice's first event.
        if matches!(event, Event::MouseMoved { .. } | Event::ButtonPressed(_) | Event::ButtonReleased(_) | Event::Wheel(_)) && self.pointer.is_none() {
            self.pointer = Some(moved_pointer(None, 0.0, 0.0, self.size));
        }
        match event {
            Event::MouseMoved { x, y } => self.pointer = Some(moved_pointer(self.pointer, *x, *y, self.size)),
            Event::PointerPlaced(at) => self.pointer = Some(moved_pointer(Some(*at), 0.0, 0.0, self.size)),
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
            Event::KeyPressed(key) => {
                self.keys_down[key.0 as usize] = true;
                self.keys_pressed.push(*key);
                let modifiers = self.modifiers();
                if !modifiers.control
                    && !modifiers.alt
                    && let Some(character) = typed(*key, modifiers.shift)
                {
                    self.text.push(character);
                }
            }
            Event::KeyReleased(key) => {
                let index = key.0 as usize;
                if self.keys_down[index] {
                    self.keys_down[index] = false;
                    self.keys_released.push(*key);
                }
            }
            Event::Gamepad(reading) => self.gamepad = *reading,
        }
    }

    // The frame's size, which the pointer stays inside: a smaller one takes it
    // in at once.
    pub fn resize(&mut self, size: Size) {
        self.size = size;
        if let Some(pointer) = self.pointer {
            self.pointer = Some(moved_pointer(Some(pointer), 0.0, 0.0, size));
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

    // Pressed in this step.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn key_released(&self, key: Key) -> bool {
        self.keys_released.contains(&key)
    }

    // This step's releases, in order.
    pub fn keys_released(&self) -> &[Key] {
        &self.keys_released
    }

    // This step's presses, in order.
    pub fn keys_pressed(&self) -> &[Key] {
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

// Where the mice's motion of x and y counts moves the pointer from `from`, or
// from the middle of a frame of the size if it's nowhere yet: by the counts
// times MOUSE_SENSITIVITY, worked out in f64, inside the frame, from 0 to its
// width or height less 1, as SDL keeps its own.
pub fn moved_pointer(from: Option<Point>, x: f64, y: f64, size: Size) -> Point {
    let from = from.unwrap_or(Point::new(size.width / 2.0, size.height / 2.0));
    let inside = |at: f64, extent: f32| at.clamp(0.0, (extent as f64 - 1.0).max(0.0)) as f32;
    Point::new(inside(from.x as f64 + x * MOUSE_SENSITIVITY, size.width), inside(from.y as f64 + y * MOUSE_SENSITIVITY, size.height))
}

// The mice's motion that moves the pointer from `from` to `to`, or from the
// middle of a frame of the size if it's nowhere yet, for tests and scripts,
// which point at places: moved_pointer then lands on `to` exactly, if it's
// inside the frame.
pub fn motion_to(from: Option<Point>, to: Point, size: Size) -> Event {
    let from = from.unwrap_or(Point::new(size.width / 2.0, size.height / 2.0));
    Event::MouseMoved {
        x: (to.x as f64 - from.x as f64) / MOUSE_SENSITIVITY,
        y: (to.y as f64 - from.y as f64) / MOUSE_SENSITIVITY,
    }
}
