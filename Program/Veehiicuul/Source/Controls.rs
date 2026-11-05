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
// | Menu                  | C                     | The fixed camera or the following one |
// | RB and left stick     | The wheel             | Zooms                                |
// | Y                     | Home                  | Zooms back to the camera's height    |
// | View                  | F1                    | Shows or hides the text              |

use application::Event;
use user_interface::{Button, Frame};

use crate::driving::DeadZone;

const SPACE: i32 = 0x20;
const PAGE_UP: i32 = 0x21;
const PAGE_DOWN: i32 = 0x22;
const HOME: i32 = 0x24;
const LEFT: i32 = 0x25;
const UP: i32 = 0x26;
const RIGHT: i32 = 0x27;
const DOWN: i32 = 0x28;
const F1: i32 = 0x70;

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
    pub toggle_text: bool,
}

impl Controls {
    // Without the window's focus, a frame has no input, so the controls are
    // at rest.
    pub fn read(frame: &Frame) -> Controls {
        let gamepad = frame.gamepad();
        let held = |keys: [i32; 2]| keys.iter().any(|&key| frame.held(key));
        let pressed = |button: Button, key: i32| gamepad.pressed(button) || frame.shortcut(key);
        // Keys push all the way, and a diagonal as far as a straight push.
        let keys = [
            held([RIGHT, 'D' as i32]) as i32 - held([LEFT, 'A' as i32]) as i32,
            held([UP, 'W' as i32]) as i32 - held([DOWN, 'S' as i32]) as i32,
        ];
        let push = if keys == [0, 0] {
            gamepad.right_stick.map(f64::from)
        } else {
            let length = f64::from(keys[0] * keys[0] + keys[1] * keys[1]).sqrt();
            keys.map(|key| f64::from(key) / length)
        };
        let brake = if frame.held(SPACE) {
            1.0
        } else {
            f64::from(gamepad.left_trigger)
        };
        let zoom = if gamepad.held(Button::RightShoulder) {
            ZOOM_DEAD_ZONE.apply(f64::from(gamepad.left_stick[1]))
        } else {
            0.0
        };
        let notches = frame
            .program_events()
            .map(|event| match event {
                Event::Wheel(_, notches) => f64::from(*notches),
                _ => 0.0,
            })
            .sum();
        let step = |previous: bool, next: bool| next as i32 - previous as i32;
        Controls {
            push,
            brake,
            vehicle: step(pressed(Button::Left, 'Q' as i32), pressed(Button::Right, 'E' as i32)),
            scene: step(pressed(Button::Down, PAGE_DOWN), pressed(Button::Up, PAGE_UP)),
            restart: pressed(Button::X, 'R' as i32),
            toggle_camera: pressed(Button::Menu, 'C' as i32),
            zoom,
            notches,
            default_zoom: pressed(Button::Y, HOME),
            toggle_text: pressed(Button::View, F1),
        }
    }
}
