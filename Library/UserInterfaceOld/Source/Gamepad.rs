// Reads the gamepad with GameInput, for programs that a gamepad controls.
// GameInput reads gamepads whether or not the window has focus; the
// interface drops what it reads without focus.

use game_input::{GameInputKindGamepad, IGameInput};

use crate::geometry::Point;
use crate::input::{Gamepad, GamepadButtons};

pub struct GamepadReader {
    game_input: Option<IGameInput>,
}

impl GamepadReader {
    // Without a GameInput runtime, there's never a gamepad.
    pub fn new() -> Self {
        Self {
            game_input: game_input::create().ok().map(|(game_input, _)| game_input),
        }
    }

    // The newest reading of any gamepad.
    pub fn read(&self) -> Gamepad {
        let Some(game_input) = &self.game_input else {
            return Gamepad::default();
        };
        unsafe {
            let Ok(reading) = game_input.GetCurrentReading(GameInputKindGamepad, None) else {
                return Gamepad::default();
            };
            let mut state = Default::default();
            if !reading.GetGamepadState(&mut state) {
                return Gamepad::default();
            }
            let state: game_input::GameInputGamepadState = state;
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
    }
}

impl Default for GamepadReader {
    fn default() -> Self {
        Self::new()
    }
}
