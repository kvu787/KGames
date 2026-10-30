// Checks Library/Math against values that DirectXMath agrees on.

use math::{Matrix, Vector3, cross, dot, length};

fn near(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() <= 1.0e-5 * expected.abs().max(1.0)
}

fn assert_near(actual: &[f32], expected: &[f32]) {
    assert!(
        actual.iter().zip(expected).all(|(&a, &e)| near(a, e)),
        "{actual:?} isn't {expected:?}"
    );
}

// MinimalSceneTest's camera, 5 units along BACKWARD from the origin: 47.5
// degrees above the horizon, and 30 degrees from +Z toward +X.
const RIGHT: Vector3 = [0.8660254, 0.0, -0.5];
const UP: Vector3 = [-0.36863866, 0.6755902, -0.63850087];
const BACKWARD: Vector3 = [0.3377951, 0.7372773, 0.5850783];

#[test]
fn look_at_puts_the_camera_basis_in_the_columns() {
    let eye = BACKWARD.map(|component| component * 5.0);
    let view = Matrix::look_at_rh(eye, [0.0; 3], UP);
    for (column, axis) in [RIGHT, UP, BACKWARD].into_iter().enumerate() {
        let actual: Vec<f32> = (0..3).map(|row| view.0[row][column]).collect();
        assert_near(&actual, &axis);
    }
    // The eye goes to the origin, and the focus straight ahead of it.
    assert_near(&view.transform_point(eye), &[0.0; 3]);
    assert_near(&view.transform_point([0.0; 3]), &[0.0, 0.0, -5.0]);
    // A view matrix is a rotation: the basis stays right-handed and unit length.
    let rows: Vec<Vector3> = (0..3).map(|row| std::array::from_fn(|column| view.0[row][column])).collect();
    assert!(near(length(rows[0]), 1.0) && near(dot(rows[0], rows[1]), 0.0));
    assert_near(&cross(rows[0], rows[1]), &rows[2]);
}

#[test]
fn rotations_are_counterclockwise_about_their_axes() {
    let quarter = std::f32::consts::FRAC_PI_2;
    assert_near(&Matrix::rotation_x(quarter).transform_direction([0.0, 1.0, 0.0]), &[0.0, 0.0, 1.0]);
    assert_near(&Matrix::rotation_y(quarter).transform_direction([0.0, 0.0, 1.0]), &[1.0, 0.0, 0.0]);
    assert_near(&Matrix::rotation_z(quarter).transform_direction([1.0, 0.0, 0.0]), &[0.0, 1.0, 0.0]);
}

#[test]
fn transforms_compose_from_left_to_right() {
    let scale = Matrix::scaling(2.0, 2.0, 2.0);
    let rotate = Matrix::rotation_z(std::f32::consts::FRAC_PI_2);
    let translate = Matrix::translation(10.0, 0.0, 0.0);
    let combined = scale * rotate * translate;
    // Scaled to (2, 0, 0), rotated to (0, 2, 0), then moved.
    assert_near(&combined.transform_point([1.0, 0.0, 0.0]), &[10.0, 2.0, 0.0]);
    assert_eq!(Matrix::IDENTITY * combined, combined);
    assert_eq!(combined.transpose().transpose(), combined);
}

#[test]
fn orthographic_projection_maps_near_and_far_to_depths_0_and_1() {
    let projection = Matrix::orthographic_rh(8.0, 4.0, 1.0, 20.0);
    assert_near(&projection.transform([4.0, -2.0, -1.0, 1.0]), &[1.0, -1.0, 0.0, 1.0]);
    assert_near(&projection.transform([-4.0, 2.0, -20.0, 1.0]), &[-1.0, 1.0, 1.0, 1.0]);
}
