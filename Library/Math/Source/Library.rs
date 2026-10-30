// Vectors and matrices for 3D, in the conventions of DirectXMath:
//
// - Vectors are rows, and a matrix transforms one as v * M. So transforms
//   compose from left to right, like scaling * rotation * translation, and
//   the translation is in the last row.
// - Matrices are stored row by row, like XMFLOAT4X4.
// - Spaces are right-handed: in view space, +X is right, +Y is up, and the
//   camera looks along -Z.
// - Angles are in radians.

use std::ops::Mul;

pub type Vector3 = [f32; 3];
pub type Vector4 = [f32; 4];

pub fn add(a: Vector3, b: Vector3) -> Vector3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn subtract(a: Vector3, b: Vector3) -> Vector3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(v: Vector3, factor: f32) -> Vector3 {
    [v[0] * factor, v[1] * factor, v[2] * factor]
}

pub fn dot(a: Vector3, b: Vector3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: Vector3, b: Vector3) -> Vector3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn length(v: Vector3) -> f32 {
    dot(v, v).sqrt()
}

pub fn normalize(v: Vector3) -> Vector3 {
    scale(v, 1.0 / length(v))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix(pub [[f32; 4]; 4]);

impl Matrix {
    pub const IDENTITY: Matrix = Matrix([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    pub fn scaling(x: f32, y: f32, z: f32) -> Matrix {
        Matrix([
            [x, 0.0, 0.0, 0.0],
            [0.0, y, 0.0, 0.0],
            [0.0, 0.0, z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Matrix {
        Matrix([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x, y, z, 1.0],
        ])
    }

    // Counterclockwise when looking from +X toward the origin, like
    // XMMatrixRotationX. So are the other two about their axes.
    pub fn rotation_x(angle: f32) -> Matrix {
        let (sin, cos) = angle.sin_cos();
        Matrix([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, cos, sin, 0.0],
            [0.0, -sin, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_y(angle: f32) -> Matrix {
        let (sin, cos) = angle.sin_cos();
        Matrix([
            [cos, 0.0, -sin, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [sin, 0.0, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_z(angle: f32) -> Matrix {
        let (sin, cos) = angle.sin_cos();
        Matrix([
            [cos, sin, 0.0, 0.0],
            [-sin, cos, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    // The view matrix of a camera at `eye` that looks at `focus`, with `up`
    // pointing up on the screen, like XMMatrixLookAtRH. Its rows' first three
    // columns are the camera's right, up and backward directions, as columns.
    pub fn look_at_rh(eye: Vector3, focus: Vector3, up: Vector3) -> Matrix {
        let backward = normalize(subtract(eye, focus));
        let right = normalize(cross(up, backward));
        let up = cross(backward, right);
        Matrix([
            [right[0], up[0], backward[0], 0.0],
            [right[1], up[1], backward[1], 0.0],
            [right[2], up[2], backward[2], 0.0],
            [-dot(right, eye), -dot(up, eye), -dot(backward, eye), 1.0],
        ])
    }

    // An orthographic projection of a `width` x `height` view, from `near`
    // to `far` in front of the camera, to depths 0 to 1, like
    // XMMatrixOrthographicRH.
    pub fn orthographic_rh(width: f32, height: f32, near: f32, far: f32) -> Matrix {
        let range = 1.0 / (near - far);
        Matrix([
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, 2.0 / height, 0.0, 0.0],
            [0.0, 0.0, range, 0.0],
            [0.0, 0.0, range * near, 1.0],
        ])
    }

    pub fn transpose(&self) -> Matrix {
        let m = &self.0;
        Matrix(std::array::from_fn(|row| std::array::from_fn(|column| m[column][row])))
    }

    // v * M.
    pub fn transform(&self, v: Vector4) -> Vector4 {
        let m = &self.0;
        std::array::from_fn(|column| (0..4).map(|row| v[row] * m[row][column]).sum())
    }

    // A point, with w = 1, so the translation applies.
    pub fn transform_point(&self, point: Vector3) -> Vector3 {
        let [x, y, z, _] = self.transform([point[0], point[1], point[2], 1.0]);
        [x, y, z]
    }

    // A direction, with w = 0, so the translation doesn't apply, like
    // XMVector3TransformNormal.
    pub fn transform_direction(&self, direction: Vector3) -> Vector3 {
        let [x, y, z, _] = self.transform([direction[0], direction[1], direction[2], 0.0]);
        [x, y, z]
    }
}

// self * other transforms by self, then by other.
impl Mul for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        let (a, b) = (&self.0, &other.0);
        Matrix(std::array::from_fn(|row| {
            std::array::from_fn(|column| (0..4).map(|k| a[row][k] * b[k][column]).sum())
        }))
    }
}
