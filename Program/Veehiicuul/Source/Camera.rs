// The cameras: orthographic, around from +Z toward +X by 45 degrees, so that
// the screen's up is toward -X and -Z. The game's camera looks down at 45
// degrees, at a fixed place on the track or at the car, with a view whose
// height, in units, the player zooms. The editors' cameras look straight
// down by default, so that the ground is as it is, or at the game's angle,
// and the editor pans and zooms them.

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

use math::Matrix;

pub const PITCH: f64 = FRAC_PI_4;
pub const YAW: f64 = FRAC_PI_4;
// How far the camera is from what it looks at, and the nearest and farthest
// that it draws, in units, which take in the whole view at any zoom.
const DISTANCE: f32 = 500.0;
pub const NEAR: f32 = 1.0;
pub const FAR: f32 = 1000.0;
// The view's heights that zooming stays between, in units.
pub const LOWEST: f64 = 2.0;
pub const HIGHEST: f64 = 300.0;
// How fast the gamepad zooms, in units of height a second, and how much
// each notch of the wheel does.
pub const ZOOM_SPEED: f64 = 50.0;
pub const NOTCH: f64 = 1.25;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    // Whether it follows the car, instead of looking at the track's place.
    pub follow: bool,
    // How tall its view is, in units.
    pub height: f64,
}

// A camera's view: what it looks at on the ground, how tall the view is,
// and how far it looks down, in radians, from 0 along the ground to a
// right angle straight down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eye {
    pub center: [f64; 2],
    pub height: f64,
    pub pitch: f64,
}

impl Eye {
    pub fn game(center: [f64; 2], height: f64) -> Eye {
        Eye {
            center,
            height,
            pitch: PITCH,
        }
    }

    pub fn top_down(center: [f64; 2], height: f64) -> Eye {
        Eye {
            center,
            height,
            pitch: FRAC_PI_2,
        }
    }

    // Toward where it looks, along the screen to the right, and along it
    // up, in the world.
    fn axes(&self) -> [[f64; 3]; 3] {
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();
        let (yaw_sin, yaw_cos) = YAW.sin_cos();
        let forward = [-pitch_cos * yaw_sin, -pitch_sin, -pitch_cos * yaw_cos];
        let right = [yaw_cos, 0.0, -yaw_sin];
        // Right, crossed with forward.
        let up = [
            right[1] * forward[2] - right[2] * forward[1],
            right[2] * forward[0] - right[0] * forward[2],
            right[0] * forward[1] - right[1] * forward[0],
        ];
        [forward, right, up]
    }

    // From the world to the camera's space.
    pub fn view(&self) -> Matrix {
        let [forward, _, _] = self.axes();
        let (yaw_sin, yaw_cos) = YAW.sin_cos();
        let target = [self.center[0] as f32, 0.0, self.center[1] as f32];
        let eye = [
            target[0] - DISTANCE * forward[0] as f32,
            target[1] - DISTANCE * forward[1] as f32,
            target[2] - DISTANCE * forward[2] as f32,
        ];
        // Up is away from the camera along the ground, which is the screen's
        // up at any pitch, even straight down.
        Matrix::look_at_rh(eye, target, [-yaw_sin as f32, 0.0, -yaw_cos as f32])
    }

    // Where a point in the world is on a screen of `size` pixels.
    pub fn to_screen(&self, [x, y, z]: [f64; 3], size: [f32; 2]) -> [f32; 2] {
        let [_, right, up] = self.axes();
        let offset = [x - self.center[0], y, z - self.center[1]];
        let dot = |axis: [f64; 3]| offset[0] * axis[0] + offset[1] * axis[1] + offset[2] * axis[2];
        let scale = size[1] as f64 / self.height;
        [
            (size[0] as f64 / 2.0 + dot(right) * scale) as f32,
            (size[1] as f64 / 2.0 - dot(up) * scale) as f32,
        ]
    }

    pub fn ground_to_screen(&self, [x, z]: [f64; 2], size: [f32; 2]) -> [f32; 2] {
        self.to_screen([x, 0.0, z], size)
    }

    // The point on the ground under a point on the screen.
    pub fn to_ground(&self, [sx, sy]: [f32; 2], size: [f32; 2]) -> [f64; 2] {
        let [forward, right, up] = self.axes();
        let scale = self.height / size[1] as f64;
        let across = (sx as f64 - size[0] as f64 / 2.0) * scale;
        let along = -(sy as f64 - size[1] as f64 / 2.0) * scale;
        let point = [
            self.center[0] + across * right[0] + along * up[0],
            across * right[1] + along * up[1],
            self.center[1] + across * right[2] + along * up[2],
        ];
        // Along the camera's line of sight to the ground.
        let t = -point[1] / forward[1];
        [point[0] + t * forward[0], point[2] + t * forward[2]]
    }

    // Zooms by `factor`, keeping the ground under `anchor` on the screen
    // where it is.
    pub fn zoom_at(&mut self, factor: f64, anchor: [f32; 2], size: [f32; 2]) {
        let before = self.to_ground(anchor, size);
        self.height = (self.height * factor).clamp(LOWEST / 4.0, HIGHEST * 2.0);
        let after = self.to_ground(anchor, size);
        self.center[0] += before[0] - after[0];
        self.center[1] += before[1] - after[1];
    }
}

// A direction on the screen, +X right and +Y up, as one on the ground, along
// X and Z, for a push toward where the player wants the car to go.
pub fn to_ground([right, up]: [f64; 2]) -> [f64; 2] {
    let (sin, cos) = YAW.sin_cos();
    [right * cos - up * sin, -right * sin - up * cos]
}
