// Play through its frames, as the main loop runs them, with input that the
// tests make up, without a window, with Library/UserInterface's Headless:
// driving with the gamepad and the keys, braking, crashing, the vehicles,
// the scenes, the camera, the card, settings that name what isn't there,
// and a window without focus. And the 3D view, drawn offscreen with the GPU.

use user_interface::graphics::{AdapterChoice, Gpu, Image};
use user_interface::tokens::{APP_BAR_HEIGHT, FULLSCREEN_HEIGHT, MAXIMIZED_HEIGHT, PAGE_PADDING, STATS_BAR_HEIGHT, STATUS_BAR_HEIGHT, WINDOW_WIDTH};
use user_interface::{Gamepad, GamepadButtons, Headless, Key, Point, Startup};
use veehiicuul::camera::{self, to_ground};
use veehiicuul::controls::Controls;
use veehiicuul::game::PAUSE;
use veehiicuul::program::Veehiicuul;
use veehiicuul::track::Pose;

// The game as it starts, with Settings.json's text, after its first frames.
fn start_with(settings: &str) -> Headless<Veehiicuul> {
    // Where tests that don't save would save.
    let folder = std::env::temp_dir().join("VeehiicuulGameTests");
    Headless::start(Startup::for_tests(settings, &folder), Veehiicuul::start)
}

fn start() -> Headless<Veehiicuul> {
    let app = start_with("{}");
    assert!(app.ui.notices().is_empty(), "{:?}", app.ui.notices());
    app
}

fn start_of(app: &Headless<Veehiicuul>) -> Pose {
    app.program.game.current_scene().track.start
}

fn gamepad(gamepad: Gamepad) -> Gamepad {
    Gamepad {
        connected: true,
        ..gamepad
    }
}

fn buttons(buttons: u32) -> Gamepad {
    gamepad(Gamepad {
        buttons: GamepadButtons(buttons),
        ..Gamepad::default()
    })
}

fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn moved(app: &Headless<Veehiicuul>) -> [f64; 2] {
    let start = start_of(app).position;
    let position = app.program.game.car.position;
    [position[0] - start[0], position[1] - start[1]]
}

fn speed(app: &Headless<Veehiicuul>) -> f64 {
    app.program.game.car.speed()
}

// Where to push the right stick, on the screen, for the car to go the way
// that it faces.
fn ahead_on_the_screen(app: &Headless<Veehiicuul>) -> Point {
    let (sin, cos) = app.program.game.car.heading.sin_cos();
    let ahead = [sin, cos];
    Point::new(dot(ahead, to_ground([1.0, 0.0])) as f32, dot(ahead, to_ground([0.0, 1.0])) as f32)
}

#[test]
fn the_car_starts_on_the_start_line_at_rest() {
    let app = start();
    let game = &app.program.game;
    let scene = game.current_scene();
    assert_eq!((scene.name.as_str(), game.current_vehicle().name.as_str()), ("Ribeye", "Yellow"));
    assert_eq!(game.car.position, scene.track.start.position);
    assert_eq!(game.car.heading, scene.track.start.radians());
    assert_eq!(game.car.speed(), 0.0);
    assert!(!game.paused());
    assert_eq!(game.camera_center(), scene.camera_center);
    assert_eq!(app.state().get("Mode"), Some(&"Play".into()));
}

// The right stick, pushed the way the car faces on the screen, speeds it up
// at its vehicle's forward rate, 140 for Yellow, without turning it, and the
// left trigger brakes it to a stop, at its reverse rate, 280.
#[test]
fn the_stick_drives_the_car_and_the_trigger_stops_it() {
    let mut app = start();
    let heading = app.program.game.car.heading;
    // 30 frames, half a second, of pushing.
    let push = ahead_on_the_screen(&app);
    app.set_gamepad(gamepad(Gamepad {
        right_stick: push,
        ..Gamepad::default()
    }));
    app.frames(28);
    assert!((speed(&app) - 70.0).abs() < 1e-6, "{}", speed(&app));
    assert!((app.program.game.car.heading - heading).abs() < 1e-9);
    app.set_gamepad(gamepad(Gamepad {
        left_trigger: 1.0,
        right_stick: Point::new(-1.0, 0.0),
        ..Gamepad::default()
    }));
    app.frames(12);
    assert!(speed(&app) > 0.0);
    app.frames(2);
    assert_eq!(app.program.game.car.velocity, [0.0, 0.0]);
    // 0.5 seconds speeding up to 70, and 0.25 slowing down: about 26 units.
    let distance = moved(&app)[0].hypot(moved(&app)[1]);
    assert!((25.0..28.0).contains(&distance), "{distance}");
    assert_eq!(app.program.game.crashes, 0);
}

// The arrow keys push the car too, toward where they point on the screen,
// where the road goes up and to the left from the start, and Space brakes.
#[test]
fn the_keys_drive_the_car_up_the_road_and_space_stops_it() {
    let mut app = start();
    app.hold(Key::UP).hold(Key::LEFT).frames(16);
    assert!(speed(&app) > 10.0, "{}", speed(&app));
    let up_and_left = to_ground([-1.0, 1.0]);
    assert!(dot(moved(&app), up_and_left) > 1.0, "{:?}", moved(&app));
    app.hold(Key::SPACE).frames(60);
    assert_eq!(speed(&app), 0.0);
    assert_eq!(app.program.game.crashes, 0);
}

// Pushing the car into the road's edge puts it back at the start, at rest,
// and the push doesn't move it again for PAUSE seconds.
#[test]
fn touching_an_edge_puts_the_car_back_at_the_start() {
    let mut app = start();
    app.hold(Key::RIGHT);
    let mut frames = 0;
    while app.program.game.crashes == 0 {
        app.frame();
        frames += 1;
        assert!(frames < 120, "the car didn't reach the edge");
    }
    let start = start_of(&app);
    assert_eq!((app.program.game.car.position, app.program.game.car.heading), (start.position, start.radians()));
    assert!(app.program.game.paused());
    // Still pushing, it stays put until the pause is over.
    let paused_frames = (PAUSE * 60.0) as usize;
    app.frames(paused_frames - 1);
    assert_eq!(app.program.game.car.position, start.position);
    app.frames(2);
    assert!(!app.program.game.paused());
    assert_ne!(app.program.game.car.position, start.position);
    assert_eq!(app.state().get("Crashes"), Some(&1u32.into()));
}

// Q and E, and the d-pad's left and right, go through the vehicles, around
// from the last to the first, and each puts the car back at the start. The
// d-pad's up and down go through the scenes.
#[test]
fn the_vehicles_go_around_and_each_starts_over() {
    let mut app = start();
    app.hold(Key::UP).frames(8).release_key(Key::UP);
    let mut names = Vec::new();
    for key in [Key::E, Key::E, Key::E, Key::Q] {
        app.key(key);
        names.push(app.program.game.current_vehicle().name.clone());
    }
    assert_eq!(names, ["Cyan", "Magenta", "Green", "Magenta"]);
    assert_eq!(app.program.game.car.position, start_of(&app).position);
    assert!(app.program.game.paused());
    app.set_gamepad(buttons(GamepadButtons::DPAD_LEFT)).frames(3);
    assert_eq!(app.program.game.current_vehicle().name, "Cyan", "holding the button switches once");
    // With one scene, the next scene is Ribeye again, from the start.
    app.set_gamepad(buttons(0)).set_gamepad(buttons(GamepadButtons::DPAD_UP));
    assert_eq!(app.program.game.current_scene().name, "Ribeye");
}

// C, or Menu, switches between the fixed camera and one that follows the
// car, each at its scene's height. The wheel over the view and the gamepad
// zoom, and Home, or Y, zooms back.
#[test]
fn the_camera_follows_the_car_and_zooms() {
    let mut app = start();
    assert_eq!(app.program.game.camera.height, 150.0);
    app.key(Key::C);
    assert!(app.program.game.camera.follow);
    assert_eq!(app.program.game.camera.height, 45.0);
    assert_eq!(app.program.game.camera_center(), app.program.game.car.position);
    app.scroll_point(Point::new(1280.0, 500.0), 1.0);
    assert!((app.program.game.camera.height - 36.0).abs() < 1e-9);
    // The wheel over the app bar doesn't zoom.
    app.scroll_point(Point::new(1280.0, 20.0), 1.0);
    assert!((app.program.game.camera.height - 36.0).abs() < 1e-9);
    app.key(Key::HOME);
    assert_eq!(app.program.game.camera.height, 45.0);
    // RB and the left stick up zoom in at ZOOM_SPEED units a second.
    app.set_gamepad(gamepad(Gamepad {
        left_stick: Point::new(0.0, 1.0),
        buttons: GamepadButtons(GamepadButtons::RIGHT_SHOULDER),
        ..Gamepad::default()
    }));
    app.frames(28);
    assert!((app.program.game.camera.height - (45.0 - camera::ZOOM_SPEED * 0.5)).abs() < 1e-9);
    app.frames(60);
    assert_eq!(app.program.game.camera.height, camera::LOWEST);
    app.set_gamepad(buttons(GamepadButtons::MENU));
    assert!(!app.program.game.camera.follow);
    assert_eq!(app.program.game.camera.height, 150.0);
}

// F1 hides the card and shows it again, and the card fits in the page's
// bottom left corner, under the notices, at either height.
#[test]
fn f1_hides_the_card_which_fits_the_page() {
    let mut app = start();
    app.key(Key::F1);
    assert!(!app.program.game.show_card);
    assert!(!app.has_target("Play/Card"));
    app.key(Key::F1);
    assert!(app.program.game.show_card);
    let mut app = start_with(r#"{ "Vehicle": "Purple" }"#);
    for fullscreen in [false, true] {
        app.set_fullscreen(fullscreen);
        let height = if fullscreen { FULLSCREEN_HEIGHT } else { MAXIMIZED_HEIGHT };
        let card = app.target("Play/Card");
        assert!(card.width < WINDOW_WIDTH / 2.0 && card.height < height / 2.0, "{card:?}");
        assert_eq!((card.x, card.bottom()), (PAGE_PADDING, height - STATUS_BAR_HEIGHT - PAGE_PADDING));
        assert!(card.y > app.target("Notice/0").bottom(), "{card:?}");
    }
}

// Settings that name a scene or a vehicle that isn't there start the game
// in the first, with a notice under the app bar that says so.
#[test]
fn settings_that_name_what_is_not_there_make_notices() {
    let app = start_with(r#"{ "Scene": "Sirloin", "Vehicle": "Purple", "Camera": "Above" }"#);
    let game = &app.program.game;
    assert_eq!((game.current_scene().name.as_str(), game.current_vehicle().name.as_str()), ("Ribeye", "Green"));
    let messages: Vec<&str> = app.ui.notices().iter().map(|notice| notice.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "Camera must be \"Fixed\" or \"Follow\", not \"Above\", so it's the default.",
            "Scene is Sirloin, which isn't in Scenes, so the game starts in Ribeye.",
            "Vehicle is Purple, which isn't in Vehicles.json, so the game starts with Green.",
        ]
    );
    assert!(app.has_target("Notice/2"));
    assert!(!app.program.game.camera.follow);
}

// Without the window's focus, the keys and the gamepad do nothing.
#[test]
fn without_focus_nothing_moves_the_car() {
    let mut app = start();
    app.focus(false).hold(Key::UP);
    app.set_gamepad(gamepad(Gamepad {
        right_stick: Point::new(0.0, 1.0),
        buttons: GamepadButtons(GamepadButtons::MENU),
        ..Gamepad::default()
    }));
    app.frames(30);
    assert_eq!(speed(&app), 0.0);
    assert!(!app.program.game.camera.follow);
}

// How many of the image's pixels are near the color.
fn count(image: &Image, [red, green, blue]: [u8; 3]) -> usize {
    image
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|bgra| {
            let near = |a: u8, b: u8| a.abs_diff(b) <= 2;
            near(bgra[2], red) && near(bgra[1], green) && near(bgra[0], blue)
        })
        .count()
}

// The 3D view fills the window under the bars and the card: the scene's
// background, the road, and the car at the start; and in the Vehicles tab,
// the vehicle on the theme's background.
#[test]
fn the_view_draws_the_scene_under_the_interface() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = start();
    let image = app.render(&mut gpu).unwrap();
    let total = (image.width * image.height) as usize;
    let scene = app.program.game.current_scene();
    let background = count(&image, scene.background);
    let road = count(&image, scene.track.road_color);
    assert!(background > total / 5 && road > total / 20, "{background} and {road} of {total}");
    // The app bar, under the performance stats, is opaque over it.
    let bar = image.pixel(1000, (STATS_BAR_HEIGHT + APP_BAR_HEIGHT / 2.0) as u32);
    assert!(bar[0] > 240 && bar[1] > 240 && bar[2] > 240, "{bar:?}");
    // The Vehicles tab shows the yellow car, big, in the middle of the
    // window, on the theme's background.
    app.click("Mode/Vehicles");
    let image = app.render(&mut gpu).unwrap();
    let yellow = image
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|bgra| bgra[2] > 180 && bgra[1] > 150 && bgra[0] < 80)
        .count();
    assert!(yellow > total / 50, "{yellow} of {total}");
    assert_ne!(image.pixel(image.width / 2, image.height / 2), image.pixel(image.width / 2, 100));
    assert!(count(&image, [0xF3, 0xF3, 0xF3]) > total / 4);
}

// Ctrl's chords are the editors' shortcuts, like Ctrl+S, and don't push
// the car.
#[test]
fn keys_with_ctrl_do_not_drive_the_car() {
    let mut app = start();
    app.hold(Key::CONTROL).hold(Key::S).frames(30);
    assert_eq!(speed(&app), 0.0);
    assert_eq!(moved(&app), [0.0, 0.0]);
    app.release_key(Key::CONTROL).frames(16);
    assert!(speed(&app) > 0.0, "without Ctrl, S drives");
}

// A start that touches an edge counts one crash, and the car waits out the
// pause at the start, instead of a crash every frame.
#[test]
fn a_crash_counts_once() {
    let mut app = start();
    let start = start_of(&app);
    let right = to_ground([1.0, 0.0]);
    let touching = {
        let game = &mut app.program.game;
        let footprint = game.footprint();
        let shape = game.shape();
        (1..400)
            .map(|step| [start.position[0] + right[0] * step as f64 * 0.25, start.position[1] + right[1] * step as f64 * 0.25])
            .find(|&position| shape.edges.touch(&footprint, position, start.radians()))
            .expect("an edge to the right of the start")
    };
    let scene = app.program.game.scene;
    app.program.game.document.scenes[scene].track.start.position = touching;
    app.program.game.car.position = touching;
    app.frames(10);
    assert_eq!(app.program.game.crashes, 1);
    assert!(app.program.game.paused());
}

// The car drives in fixed steps, so the same push for the same time puts it
// in the same place at 60 frames a second as at 240.
#[test]
fn the_car_drives_the_same_at_any_frame_rate() {
    let mut slow = start();
    let mut fast = start();
    // Up the road, as the keys test drives, for half a second.
    let controls = Controls {
        push: [-0.7, 0.7],
        ..Controls::default()
    };
    slow.program.game.update(&controls, 0.0);
    fast.program.game.update(&controls, 0.0);
    for frame in 1..=30 {
        slow.program.game.update(&controls, frame as f64 / 60.0);
    }
    for frame in 1..=120 {
        fast.program.game.update(&controls, frame as f64 / 240.0);
    }
    assert_eq!(slow.program.game.car, fast.program.game.car);
    assert_eq!(slow.program.game.crashes, 0);
    assert!(slow.program.game.car.speed() > 1.0, "{}", slow.program.game.car.speed());
}
