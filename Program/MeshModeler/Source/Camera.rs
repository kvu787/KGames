// The view's camera: orthographic, as SimplePaint paints only orthographic
// views, orbiting a target point. It turns around the target, pans across
// the view, and zooms by how much of the world the view's height shows. It
// also maps between the world and the view's pixels, for drawing overlays and
// picking.

use math::{Matrix, Vector3, add, cross, dot, normalize, scale, subtract};
use simple_paint::orthographic::{Projection, make_projection};
use user_interface::{Point, Rect};

// How far the eye is from the target. Orthographic views don't change with
// it, but it puts the near and far planes around the scene.
const DISTANCE: f32 = 100.0;
const LIMIT_PITCH: f32 = 89.9f32 * std::f32::consts::PI / 180.0;
pub const MINIMUM_HEIGHT: f32 = 0.05;
pub const MAXIMUM_HEIGHT: f32 = 200.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub target: Vector3,
    // Around the up axis from looking down -Z, and up from the horizon, in
    // radians.
    pub yaw: f32,
    pub pitch: f32,
    // How much of the world the view's height shows.
    pub height: f32,
}

// The views that View's menu has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Viewpoint {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: [0.0, 0.0, 0.0],
            yaw: 35f32.to_radians(),
            pitch: 25f32.to_radians(),
            height: 6.0,
        }
    }
}

impl Camera {
    // From the target toward the eye.
    pub fn backward(&self) -> Vector3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        [cos_pitch * sin_yaw, sin_pitch, cos_pitch * cos_yaw]
    }

    pub fn right(&self) -> Vector3 {
        normalize(cross([0.0, 1.0, 0.0], self.backward()))
    }

    pub fn up(&self) -> Vector3 {
        cross(self.backward(), self.right())
    }

    pub fn eye(&self) -> Vector3 {
        add(self.target, scale(self.backward(), DISTANCE))
    }

    pub fn view(&self) -> Matrix {
        Matrix::look_at_rh(self.eye(), self.target, [0.0, 1.0, 0.0])
    }

    fn width(&self, view: Rect) -> f32 {
        self.height * view.width / view.height.max(1.0)
    }

    // SimplePaint's packed projection for the view's size.
    pub fn projection(&self, view: Rect) -> Projection {
        make_projection(self.width(view), self.height, 0.0, DISTANCE * 2.0).expect("the camera's projection is in range")
    }

    // From the world to the view's clip space, v * M.
    pub fn world_to_clip(&self, view: Rect) -> Matrix {
        self.view() * Matrix::orthographic_rh(self.width(view), self.height, 0.0, DISTANCE * 2.0)
    }

    // How much of the world a pixel is.
    pub fn pixel_size(&self, view: Rect) -> f32 {
        self.height / view.height.max(1.0)
    }

    // Where a point of the world is in the window, in pixels, and its depth
    // toward the camera: more is nearer.
    pub fn to_screen(&self, point: Vector3, view: Rect) -> (Point, f32) {
        let relative = subtract(point, self.target);
        let pixel = self.pixel_size(view);
        let center = view.center();
        let x = center.x + dot(relative, self.right()) / pixel;
        let y = center.y - dot(relative, self.up()) / pixel;
        (Point::new(x, y), dot(relative, self.backward()))
    }

    // The ray through a pixel of the window: where it starts, far toward the
    // camera, and its direction into the view.
    pub fn ray(&self, pixel: Point, view: Rect) -> (Vector3, Vector3) {
        let size = self.pixel_size(view);
        let center = view.center();
        let across = scale(self.right(), (pixel.x - center.x) * size);
        let up = scale(self.up(), (center.y - pixel.y) * size);
        let origin = add(add(self.eye(), across), up);
        (origin, scale(self.backward(), -1.0))
    }

    // Turns around the target by pixels of a drag.
    pub fn orbit(&mut self, delta: Point) {
        self.yaw -= delta.x * 0.008;
        self.pitch = (self.pitch + delta.y * 0.008).clamp(-LIMIT_PITCH, LIMIT_PITCH);
    }

    // Moves across the view so that the world follows the pointer.
    pub fn pan(&mut self, delta: Point, view: Rect) {
        let size = self.pixel_size(view);
        let across = scale(self.right(), -delta.x * size);
        let up = scale(self.up(), delta.y * size);
        self.target = add(add(self.target, across), up);
    }

    // Zooms in for notches up, keeping the point of the world under the
    // pointer where it is.
    pub fn zoom(&mut self, notches: f32, at: Point, view: Rect) {
        let before = self.ray(at, view).0;
        self.height = (self.height * 0.85f32.powf(notches)).clamp(MINIMUM_HEIGHT, MAXIMUM_HEIGHT);
        let after = self.ray(at, view).0;
        self.target = add(self.target, subtract(before, after));
    }

    // Shows the box from `minimum` to `maximum` whole, from where the camera
    // looks now.
    pub fn frame(&mut self, minimum: Vector3, maximum: Vector3) {
        self.target = scale(add(minimum, maximum), 0.5);
        let extent = math::length(subtract(maximum, minimum));
        self.height = (extent * 1.25).clamp(MINIMUM_HEIGHT * 4.0, MAXIMUM_HEIGHT);
    }

    pub fn look_from(&mut self, viewpoint: Viewpoint) {
        let (yaw, pitch) = match viewpoint {
            Viewpoint::Front => (0.0, 0.0),
            Viewpoint::Back => (180.0, 0.0),
            Viewpoint::Right => (90.0, 0.0),
            Viewpoint::Left => (-90.0, 0.0),
            Viewpoint::Top => (0.0, 89.9),
            Viewpoint::Bottom => (0.0, -89.9),
        };
        self.yaw = f32::to_radians(yaw);
        self.pitch = f32::to_radians(pitch);
    }
}
