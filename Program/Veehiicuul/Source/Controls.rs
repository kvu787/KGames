// What the player does in a frame, from the gamepad and the keyboard, which
// both do everything:
//
// | Gamepad               | Keyboard              | Does                                 |
// | --------------------- | --------------------- | ------------------------------------ |
// | Right stick           | Arrows or WASD        | Pushes the car toward there          |
// | Left trigger          | Space                 | Brakes                               |
// | D-pad left and right  | Q and E               | The previous and next vehicle        |
// | D-pad down and up     | PageDown and PageUp   | The previous and next scene          |
// | X                     | R                     | Puts the car back at the start       |
// | Menu                  | C                     | The fixed or the following camera    |
// | RB and left stick     | The wheel             | Zooms                                |
// | Y                     | Home                  | Zooms back to the camera's height    |
// | View                  | F1                    | Shows or hides the card              |
//
// Without the window's focus, the interface has no input at all, so the
// controls are at rest.

use user_interface::{GamepadButtons, Key, Ui};

use crate::driving::DeadZone;

// The left stick zooms only past this, so that resting a thumb on it while
// holding RB doesn't.
const ZOOM_DEAD_ZONE: DeadZone = DeadZone {
    inner: 0.05,
    outer: 0.95,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Controls {
    // Where the player pushes the car, on the screen, +X right and +Y up,
    // about 1 long at most. The dead zones apply later, along the car's
    // own directions.
    pub push: [f64; 2],
    // How hard they brake, from 0 to 1.
    pub brake: f64,
    // To the previous vehicle or scene, -1, or the next, 1.
    pub vehicle: i32,
    pub scene: i32,
    pub restart: bool,
    pub toggle_camera: bool,
    // How fast to zoom in, from -1, out, to 1, and by how many notches of the
    // wheel, positive in.
    pub zoom: f64,
    pub notches: f64,
    pub default_zoom: bool,
    pub toggle_card: bool,
}

impl Controls {
    // The frame's controls, with the wheel's notches over the game's view.
    pub fn read(ui: &Ui, notches: f64) -> Controls {
        let gamepad = *ui.gamepad();
        let input = ui.input();
        let held = |keys: [Key; 2]| keys.iter().any(|&key| ui.key_down(key));
        // A key's press, not its repeats, and not with Ctrl, which is for
        // the editors' shortcuts, like Ctrl+S.
        let key = |key: Key| ui.key_pressed(key) && input.key_pressed_once(key) && !input.modifiers().control;
        let pressed = |button: u32, other: Key| input.gamepad_pressed(button) || key(other);
        // Keys push all the way, and a diagonal as far as a straight push.
        let keys = [
            held([Key::RIGHT, Key::D]) as i32 - held([Key::LEFT, Key::A]) as i32,
            held([Key::UP, Key::W]) as i32 - held([Key::DOWN, Key::S]) as i32,
        ];
        let push = if keys == [0, 0] {
            [f64::from(gamepad.right_stick.x), f64::from(gamepad.right_stick.y)]
        } else {
            let length = f64::from(keys[0] * keys[0] + keys[1] * keys[1]).sqrt();
            keys.map(|key| f64::from(key) / length)
        };
        let brake = if ui.key_down(Key::SPACE) {
            1.0
        } else {
            f64::from(gamepad.left_trigger)
        };
        let zoom = if gamepad.buttons.contains(GamepadButtons::RIGHT_SHOULDER) {
            ZOOM_DEAD_ZONE.apply(f64::from(gamepad.left_stick.y))
        } else {
            0.0
        };
        let step = |previous: bool, next: bool| next as i32 - previous as i32;
        Controls {
            push,
            brake,
            vehicle: step(
                pressed(GamepadButtons::DPAD_LEFT, Key::Q),
                pressed(GamepadButtons::DPAD_RIGHT, Key::E),
            ),
            scene: step(
                pressed(GamepadButtons::DPAD_DOWN, Key::PAGE_DOWN),
                pressed(GamepadButtons::DPAD_UP, Key::PAGE_UP),
            ),
            restart: pressed(GamepadButtons::X, Key::R),
            toggle_camera: pressed(GamepadButtons::MENU, Key::C),
            zoom,
            notches,
            default_zoom: pressed(GamepadButtons::Y, Key::HOME),
            toggle_card: pressed(GamepadButtons::VIEW, Key::F1),
        }
    }
}
