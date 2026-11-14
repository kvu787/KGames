// Tests how far a mouse's motion moves the pointer, by Windows' pointer speed
// and acceleration, as SDL 3's WIN_ApplySystemScale moves it, which
// input/ballistics.rs is a port of. The expected values come from SDL's C
// code, worked through by hand, not from the port.

use user_interface::Point;
use user_interface::input::ballistics::{Ballistics, PointerSpeed, WINDOWS_CURVE};

fn close(moved: Point, expected: (f32, f32)) -> bool {
    (moved.x - expected.0).abs() < 1e-4 && (moved.y - expected.1).abs() < 1e-4
}

// Without acceleration, the speed scales the motion as Windows' slider does,
// a count a pixel at 10, its default, in 32nds: from 1/32 at 1 to 3.5 at 20.
#[test]
fn the_pointer_speed_scales_the_motion() {
    for (speed, scale) in [(1, 1.0 / 32.0), (2, 2.0 / 32.0), (6, 0.5), (10, 1.0), (14, 2.0), (20, 3.5)] {
        let mut ballistics = Ballistics::new(PointerSpeed { speed, acceleration: None });
        let moved = ballistics.apply(1, -3);
        assert!(close(moved, (scale, -3.0 * scale)), "speed {speed}: {moved:?}");
    }
}

// Tests and scripts move the pointer a count a pixel.
#[test]
fn the_default_moves_a_count_a_pixel() {
    let mut ballistics = Ballistics::default();
    for (x, y) in [(1, 0), (-7, 3), (1000, -2000)] {
        assert_eq!(ballistics.apply(x, y), Point::new(x as f32, y as f32));
    }
}

// With "Enhance pointer precision", the motion is scaled by how fast the
// mouse moves, on Windows' curve: less than a pixel a count slowly, and more
// quickly, and a motion that's faster than the last, on a later segment of
// the curve, is scaled by half of each.
#[test]
fn acceleration_scales_by_how_fast_the_mouse_moves() {
    let mut ballistics = Ballistics::new(PointerSpeed {
        speed: 10,
        acceleration: Some(WINDOWS_CURVE),
    });
    let expected = [
        ((1, 0), (0.568893, 0.0)),
        ((10, 0), (8.156845, 0.0)),
        ((10, 0), (10.624695, 0.0)),
        ((-3, 4), (-2.604507, 3.472672)),
        ((0, 0), (0.0, 0.0)),
        ((40, -25), (70.78308, -44.239426)),
        ((1, 1), (0.568893, 0.568893)),
    ];
    for ((x, y), position) in expected {
        let moved = ballistics.apply(x, y);
        assert!(close(moved, position), "({x}, {y}): {moved:?}, not {position:?}");
    }
}

// The speed scales the acceleration's curve too.
#[test]
fn the_pointer_speed_scales_the_acceleration() {
    let mut ballistics = Ballistics::new(PointerSpeed {
        speed: 15,
        acceleration: Some(WINDOWS_CURVE),
    });
    assert!(close(ballistics.apply(2, 1), (1.878006, 0.938995)));
    assert!(close(ballistics.apply(20, 5), (38.77588, 9.69397)));
}
