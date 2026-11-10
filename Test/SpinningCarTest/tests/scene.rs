// SpinningCarTest without a window: where the car is over time, the card out
// of its path, and frames drawn offscreen with the GPU. Test/Meshes tests the
// car mesh.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use spinning_car_test::scene::{BACKGROUND, SPHERE_CENTER, SPHERE_RADIUS, VIEW_HEIGHT, car_on_screen, car_placement, to_screen};
use spinning_car_test::settings::SceneSettings;
use spinning_car_test::spinning_car::SpinningCar;
use user_interface::graphics::{AdapterChoice, Gpu, write_png};
use user_interface::tokens::{FULLSCREEN_HEIGHT, MAXIMIZED_HEIGHT, PAGE_PADDING, PERF_BAR_HEIGHT, WINDOW_WIDTH};
use user_interface::{Headless, InfoCard, Rect, Theme};

fn car() -> Headless<SpinningCar> {
    Headless::new(SpinningCar::new(SceneSettings::default()))
}

fn near(actual: (f32, f32), expected: (f32, f32)) -> bool {
    (actual.0 - expected.0).abs() < 1e-5 && (actual.1 - expected.1).abs() < 1e-5
}

#[test]
fn the_car_goes_from_the_middle_to_7_then_to_minus_7_and_back() {
    for (seconds, expected) in [
        (0.0, (0.0, 0.0)),
        (0.5, (4.0, FRAC_PI_4)),
        (0.875, (7.0, 0.875 * FRAC_PI_2)),
        (1.75, (0.0, 1.75 * FRAC_PI_2)),
        (2.625, (-7.0, 2.625 * FRAC_PI_2)),
        // A cycle after 0.5 seconds, the car is back at 4, and after a whole
        // turn, it starts over.
        (4.0, (4.0, 0.0)),
    ] {
        let placement = car_placement(seconds, false);
        assert!(near(placement, expected), "at {seconds} s, {placement:?} isn't {expected:?}");
    }
    // 3.5 seconds is a whole cycle of 28 units at 8 units a second.
    for seconds in [0.3, 1.1, 2.9] {
        assert!((car_placement(seconds, false).0 - car_placement(seconds + 3.5, false).0).abs() < 1e-5);
    }
}

#[test]
fn a_stationary_car_stays_in_the_middle_facing_plus_x() {
    for seconds in [0.0, 1.0, 123.4] {
        assert_eq!(car_placement(seconds, true), (0.0, FRAC_PI_2));
    }
}

#[test]
fn the_car_keeps_the_frames_clock() {
    let mut app = car();
    app.wait(0.5);
    assert_eq!(app.program.placement, car_placement(app.time(), false));
    let state = app.state();
    assert_eq!(state.get("Stationary"), Some(&false.into()));
    assert_eq!(state.get("TextShown"), Some(&true.into()));
}

// How far apart the rectangles are: less than 0 if they overlap.
fn gap(a: Rect, b: Rect) -> f32 {
    (b.x - a.right()).max(a.x - b.right()).max(b.y - a.bottom()).max(a.y - b.bottom())
}

#[test]
fn the_card_is_at_the_bottom_left_out_of_the_car_s_path_at_both_heights() {
    let mut app = car();
    for fullscreen in [false, true] {
        app.set_fullscreen(fullscreen);
        let height = if fullscreen { FULLSCREEN_HEIGHT } else { MAXIMIZED_HEIGHT };
        let card = app.target(InfoCard::NAME);
        assert_eq!((card.x, card.bottom(), card.width), (PAGE_PADDING, height - PAGE_PADDING, InfoCard::WIDTH));
        // Every place and turn of the car comes in 28 seconds, 8 of its 3.5-second
        // trips and 7 of its 4-second turns.
        let mut closest = f32::MAX;
        for frame in 0..(28 * 60) {
            let placement = car_placement(frame as f64 / 60.0, false);
            closest = closest.min(gap(car_on_screen(placement, WINDOW_WIDTH, height), card));
        }
        closest = closest.min(gap(car_on_screen(car_placement(0.0, true), WINDOW_WIDTH, height), card));
        assert!(closest > 50.0, "the car comes within {closest} pixels of the card at {height} pixels high");
        // Nor does the card cover the sphere.
        let [x, y] = to_screen(SPHERE_CENTER, WINDOW_WIDTH, height);
        let radius = SPHERE_RADIUS * height / VIEW_HEIGHT;
        assert!(gap(Rect::new(x - radius, y - radius, radius * 2.0, radius * 2.0), card) > 50.0);
    }
}

fn close(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter().zip(b).take(3).all(|(a, b)| (*a as i32 - b as i32).abs() <= 2)
}

#[test]
fn a_frame_draws_the_scene_under_the_card_at_both_heights() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = car();
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/TestOutput");
    std::fs::create_dir_all(&folder).unwrap();
    let background = BACKGROUND.map(|channel| (channel * 255.0).round() as u8);
    // Back to maximized, after the depth buffer was made for fullscreen.
    for fullscreen in [false, true, false] {
        app.set_fullscreen(fullscreen);
        let image = app.render(&mut gpu).unwrap();
        let name = if fullscreen { "Fullscreen" } else { "Maximized" };
        write_png(&folder.join(format!("SpinningCarTest{name}.png")), &image).unwrap();
        let height = if fullscreen { FULLSCREEN_HEIGHT } else { MAXIMIZED_HEIGHT };
        assert_eq!((image.width, image.height), (WINDOW_WIDTH as u32, height as u32));

        // The background, at the top right corner under the PerfBar, which
        // nothing covers.
        let corner = (2550, PERF_BAR_HEIGHT as u32 + 10);
        assert!(close(image.pixel(corner.0, corner.1), background), "{:?}", image.pixel(corner.0, corner.1));
        // The sphere, in its dark blue, under the light at its middle.
        let [x, y] = to_screen(SPHERE_CENTER, WINDOW_WIDTH, height);
        let below = y + SPHERE_RADIUS * height / VIEW_HEIGHT * 0.6;
        let [red, green, blue, _] = image.pixel(x as u32, below as u32);
        assert!(blue > green && green > red, "{:?}", [red, green, blue]);
        // The cube, in red, flat on top.
        let [x, y] = to_screen([0.0, 1.0, -1.5], WINDOW_WIDTH, height);
        let [red, green, blue, _] = image.pixel(x as u32, y as u32);
        assert!(red > 150 && green < 80 && blue < 80, "{:?}", [red, green, blue]);
        // The car, in its blue, in front of the ground's green.
        let middle = car_on_screen(app.program.placement, WINDOW_WIDTH, height).center();
        let [red, green, blue, _] = image.pixel(middle.x as u32, middle.y as u32);
        assert!(blue > green && blue > red, "{:?}", [red, green, blue]);
        // The card, opaque, right of its title.
        let card = app.target(InfoCard::NAME);
        let fill = image.pixel(card.right() as u32 - 24, card.y as u32 + 24);
        assert!(close(fill, Theme::light().acrylic.to_bytes()), "{fill:?}");
    }
}
