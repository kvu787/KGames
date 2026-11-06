// MinimalSceneTest without a window: the sphere's motion, the card out of its
// path, and frames drawn offscreen with the GPU.

use minimal_scene_test::moving_sphere::MovingSphere;
use minimal_scene_test::scene::{BACKGROUND, Motion, SPHERE_RADIUS, TRAVEL_DISTANCE, VIEW_HEIGHT, sphere_center, to_screen};
use minimal_scene_test::settings::SceneSettings;
use user_interface::graphics::{AdapterChoice, Gpu, write_png};
use user_interface::tokens::{FULLSCREEN_HEIGHT, MAXIMIZED_HEIGHT, PAGE_PADDING, PERF_BAR_HEIGHT, WINDOW_WIDTH};
use user_interface::{Headless, InfoCard, Rect, Theme};

fn sphere() -> Headless<MovingSphere> {
    Headless::new(MovingSphere::new(SceneSettings::default()))
}

#[test]
fn the_sphere_starts_in_the_middle_moving_at_8_units_a_second() {
    let mut motion = Motion::default();
    assert_eq!((motion.position, motion.direction), (0.0, 1.0));
    motion.advance(0.5);
    assert_eq!(motion.position, 4.0);
}

#[test]
fn the_sphere_bounces_back_from_7_by_as_far_as_it_went_past() {
    let mut motion = Motion::default();
    for _ in 0..7 {
        motion.advance(0.125);
    }
    assert_eq!((motion.position, motion.direction), (7.0, 1.0));
    motion.advance(0.125);
    assert_eq!((motion.position, motion.direction), (6.0, -1.0));
    for _ in 0..14 {
        motion.advance(0.125);
    }
    assert_eq!((motion.position, motion.direction), (-6.0, 1.0));
    motion.advance(0.125);
    assert_eq!(motion.position, -5.0);
}

#[test]
fn any_time_keeps_the_sphere_on_its_path() {
    // From 5, 16 units take it to 7 and back to -7.
    let mut motion = Motion {
        position: 5.0,
        direction: 1.0,
    };
    motion.advance(2.0);
    assert_eq!((motion.position, motion.direction), (-7.0, -1.0));
    // Many trips back and forth.
    motion.advance(1000.0);
    assert!(motion.position.abs() <= TRAVEL_DISTANCE, "{}", motion.position);
}

#[test]
fn small_frames_keep_the_sphere_on_its_path() {
    let mut motion = Motion::default();
    let frame = 1.0 / 3000.0;
    let mut turns = 0;
    for _ in 0..(3000 * 10) {
        let direction = motion.direction;
        motion.advance_frame(frame);
        turns += (motion.direction != direction) as u32;
        assert!(motion.position.abs() <= TRAVEL_DISTANCE, "{}", motion.position);
    }
    // 80 units in 10 seconds, 14 units between turns, after 7 to the first.
    assert_eq!(turns, 6);
}

#[test]
fn a_frame_counts_a_tenth_of_a_second_at_most() {
    // Like the first frame after the window was minimized for a minute.
    let mut motion = Motion::default();
    motion.advance_frame(60.0);
    assert_eq!((motion.position, motion.direction), (0.8, 1.0));
    motion.advance_frame(0.05);
    assert!((motion.position - 1.2).abs() < 1e-6, "{}", motion.position);
}

#[test]
fn the_sphere_moves_with_the_frames_clock() {
    let mut app = sphere();
    app.wait(0.5);
    let expected = 8.0 * app.time() as f32;
    assert!((app.program.motion.position - expected).abs() < 1e-4, "{} isn't {expected}", app.program.motion.position);
    let state = app.state();
    assert_eq!(state.get("SphereDirection"), Some(&1.0f32.into()));
    assert_eq!(state.get("TextShown"), Some(&true.into()));
}

// How far the circle is outside the rectangle: less than 0 if they overlap.
fn clearance(center: [f32; 2], radius: f32, rect: Rect) -> f32 {
    let dx = (rect.x - center[0]).max(center[0] - rect.right()).max(0.0);
    let dy = (rect.y - center[1]).max(center[1] - rect.bottom()).max(0.0);
    (dx * dx + dy * dy).sqrt() - radius
}

#[test]
fn the_card_is_at_the_bottom_left_out_of_the_sphere_s_path_at_both_heights() {
    let mut app = sphere();
    for fullscreen in [false, true] {
        app.set_fullscreen(fullscreen);
        let height = if fullscreen { FULLSCREEN_HEIGHT } else { MAXIMIZED_HEIGHT };
        let card = app.target(InfoCard::NAME);
        assert_eq!((card.x, card.bottom(), card.width), (PAGE_PADDING, height - PAGE_PADDING, InfoCard::WIDTH));
        let radius = SPHERE_RADIUS * height / VIEW_HEIGHT;
        let mut closest = f32::MAX;
        for step in -700..=700 {
            let center = to_screen(sphere_center(step as f32 / 100.0), WINDOW_WIDTH, height);
            closest = closest.min(clearance(center, radius, card));
        }
        assert!(closest > 50.0, "the sphere comes within {closest} pixels of the card at {height} pixels high");
    }
}

fn close(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter().zip(b).take(3).all(|(a, b)| (*a as i32 - b as i32).abs() <= 2)
}

#[test]
fn a_frame_draws_the_scene_under_the_card_at_both_heights() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = sphere();
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/TestOutput");
    std::fs::create_dir_all(&folder).unwrap();
    let background = BACKGROUND.map(|channel| (channel * 255.0).round() as u8);
    // Back to maximized, after the depth buffer was made for fullscreen.
    for fullscreen in [false, true, false] {
        app.set_fullscreen(fullscreen);
        let image = app.render(&mut gpu).unwrap();
        let name = if fullscreen { "Fullscreen" } else { "Maximized" };
        write_png(&folder.join(format!("MinimalSceneTest{name}.png")), &image).unwrap();
        let height = if fullscreen { FULLSCREEN_HEIGHT } else { MAXIMIZED_HEIGHT };
        assert_eq!((image.width, image.height), (WINDOW_WIDTH as u32, height as u32));

        // The background, at the top right corner under the performance
        // stats, which nothing covers.
        let corner = (2550, PERF_BAR_HEIGHT as u32 + 10);
        assert!(close(image.pixel(corner.0, corner.1), background), "{:?}", image.pixel(corner.0, corner.1));
        // The sphere, in front of the cube, in its blue.
        let [x, y] = to_screen(sphere_center(app.program.motion.position), WINDOW_WIDTH, height);
        let [red, green, blue, _] = image.pixel(x as u32, y as u32);
        assert!(blue > 200 && blue > green && green > red, "{:?}", [red, green, blue]);
        // The card, opaque, right of its title.
        let card = app.target(InfoCard::NAME);
        let fill = image.pixel(card.right() as u32 - 24, card.y as u32 + 24);
        assert!(close(fill, Theme::light().acrylic.to_bytes()), "{fill:?}");
    }
}
