// The camera: orthographic, looking down at the ground from 45 degrees above
// it, and from 45 degrees around from +Z toward +X, so that the screen's up
// is toward -X and -Z. It looks at a fixed place on the track, or follows
// the car, with a view whose height, in units, the player zooms.

use std::f64::consts::FRAC_PI_4;

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

// A direction on the screen, +X right and +Y up, as one on the ground, along
// X and Z, for a push toward where the player wants the car to go.
pub fn to_ground([right, up]: [f64; 2]) -> [f64; 2] {
    let (sin, cos) = YAW.sin_cos();
    [right * cos - up * sin, -right * sin - up * cos]
}

// From the world to the camera's space, for a camera that looks at `center`
// on the ground.
pub fn view(center: [f64; 2]) -> Matrix {
    let (pitch_sin, pitch_cos) = (PITCH as f32).sin_cos();
    let (yaw_sin, yaw_cos) = (YAW as f32).sin_cos();
    let target = [center[0] as f32, 0.0, center[1] as f32];
    let eye = [
        target[0] + DISTANCE * pitch_cos * yaw_sin,
        DISTANCE * pitch_sin,
        target[2] + DISTANCE * pitch_cos * yaw_cos,
    ];
    Matrix::look_at_rh(eye, target, [0.0, 1.0, 0.0])
}
