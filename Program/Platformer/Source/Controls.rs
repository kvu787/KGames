// What the player does in a frame, from the keyboard and the gamepad, which
// both do everything:
//
// | Gamepad               | Keyboard              | Does                        |
// | --------------------- | --------------------- | --------------------------- |
// | Left stick, D-pad     | A and D, or arrows    | Runs left and right         |
// | A                     | Space, W, or up       | Jumps; held jumps higher    |
// | X                     | R                     | Starts the run over         |
// | LB and RB             | PageDown and PageUp   | The previous and next level |
// | View                  | F1                    | Shows or hides the cards    |
//
// Without the window's focus, the interface has no input at all, so the
// controls are at rest.

use user_interface::{GamepadButtons, Key, Ui};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Controls {
    // The run, from -1, left, to 1, right.
    pub push: f64,
    // The jump's press this frame, and whether it's still held.
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub restart: bool,
    // To the previous level, -1, or the next, 1.
    pub level: i32,
    pub toggle_cards: bool,
}

impl Controls {
    pub fn read(ui: &Ui) -> Controls {
        let gamepad = *ui.gamepad();
        let input = ui.input();
        // Not with Ctrl, which is for shortcuts like Ctrl+S elsewhere.
        let control = input.modifiers().control;
        let held = |keys: &[Key]| !control && keys.iter().any(|&key| ui.key_down(key));
        // A key's press, not its repeats.
        let key = |key: Key| ui.key_pressed(key) && input.key_pressed_once(key) && !control;
        let pressed = |button: u32, other: Key| input.gamepad_pressed(button) || key(other);
        let keys = held(&[Key::RIGHT, Key::D]) as i32 - held(&[Key::LEFT, Key::A]) as i32;
        let push = if keys != 0 {
            f64::from(keys)
        } else {
            let stick = f64::from(gamepad.left_stick.x);
            let dpad = gamepad.buttons.contains(GamepadButtons::DPAD_RIGHT) as i32 - gamepad.buttons.contains(GamepadButtons::DPAD_LEFT) as i32;
            if dpad != 0 { f64::from(dpad) } else if stick.abs() > 0.25 { stick.clamp(-1.0, 1.0) } else { 0.0 }
        };
        let jump_keys = [Key::SPACE, Key::W, Key::UP];
        let jump_pressed = jump_keys.into_iter().any(key) || input.gamepad_pressed(GamepadButtons::A);
        let jump_held = held(&jump_keys) || gamepad.buttons.contains(GamepadButtons::A);
        let step = |previous: bool, next: bool| next as i32 - previous as i32;
        Controls {
            push,
            jump_pressed,
            jump_held,
            restart: pressed(GamepadButtons::X, Key::R),
            level: step(
                pressed(GamepadButtons::LEFT_SHOULDER, Key::PAGE_DOWN),
                pressed(GamepadButtons::RIGHT_SHOULDER, Key::PAGE_UP),
            ),
            toggle_cards: pressed(GamepadButtons::VIEW, Key::F1),
        }
    }
}
