// Play through its frames, as the main loop runs it, with input that the
// tests make up, without a window or a GPU: driving with the gamepad and the
// keys, braking, crashing, the vehicles, the scenes, the camera, the card,
// and a window without focus.

use std::path::{Path, PathBuf};

use application::{ProgramSettings, key_name};
use user_interface::headless::{FULLSCREEN, Headless, MAXIMIZED, key};
use user_interface::{GamepadButton, Gamepad, MARGIN, PANELS_TOP, Program};
use veehiicuul::camera::{self, to_ground};
use veehiicuul::game::{Game, PAUSE};
use veehiicuul::program::{Veehiicuul, play_card};
use veehiicuul::scene::Scene;
use veehiicuul::settings::GameSettings;

const LEFT: u32 = 0x25;
const UP: u32 = 0x26;
const RIGHT: u32 = 0x27;
const SPACE: u32 = 0x20;

// The game as it starts, drawn once, so that the next frame moves it on by
// a frame, a 60th of a second.
fn start() -> (Veehiicuul, Headless) {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Program/Veehiicuul");
    let (_, settings) = GameSettings::defaults();
    let mut notices = Vec::new();
    let game = Game::load(&folder, &settings, &mut notices).unwrap_or_else(|problem| panic!("{problem}"));
    assert!(notices.is_empty(), "{notices:?}");
    let mut program = Veehiicuul::without_drawing(game, nowhere());
    let mut headless = Headless::default();
    headless.program(&mut program, &[]).unwrap();
    (program, headless)
}

// Where tests that don't save would save.
fn nowhere() -> PathBuf {
    std::env::temp_dir().join("VeehiicuulGameTests")
}

fn start_of(program: &Veehiicuul) -> veehiicuul::track::Pose {
    program.game.current_scene().track.start
}

fn run(program: &mut Veehiicuul, headless: &mut Headless, frames: usize) {
    for _ in 0..frames {
        headless.program(program, &[]).unwrap();
    }
}

fn press(program: &mut Veehiicuul, headless: &mut Headless, code: u32) {
    headless.program(program, &[key(code as i32)]).unwrap();
}

fn connected(gamepad: Gamepad) -> Gamepad {
    Gamepad {
        connected: true,
        ..gamepad
    }
}

fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn moved(program: &Veehiicuul) -> [f64; 2] {
    let start = start_of(program).position;
    let position = program.game.car.position;
    [position[0] - start[0], position[1] - start[1]]
}

// Where to push the right stick, on the screen, for the car to go the way
// that it faces.
fn ahead_on_the_screen(program: &Veehiicuul) -> [f32; 2] {
    let (sin, cos) = program.game.car.heading.sin_cos();
    let ahead = [sin, cos];
    [dot(ahead, to_ground([1.0, 0.0])) as f32, dot(ahead, to_ground([0.0, 1.0])) as f32]
}

#[test]
fn the_car_starts_on_the_start_line_at_rest() {
    let (program, _) = start();
    let game = &program.game;
    let scene: &Scene = game.current_scene();
    assert_eq!((scene.name.as_str(), game.current_vehicle().name.as_str()), ("Ribeye", "Yellow"));
    assert_eq!(game.car.position, scene.track.start.position);
    assert_eq!(game.car.heading, scene.track.start.radians());
    assert_eq!(game.car.speed(), 0.0);
    assert!(!game.paused());
    assert_eq!(game.camera_center(), scene.camera_center);
}

// The right stick, pushed the way the car faces on the screen, speeds it up
// at its vehicle's forward rate, 140 for Yellow, without turning it, and the
// left trigger brakes it to a stop, at its reverse rate, 280.
#[test]
fn the_stick_drives_the_car_and_the_trigger_stops_it() {
    let (mut program, mut headless) = start();
    let heading = program.game.car.heading;
    headless.gamepad = connected(Gamepad {
        right_stick: ahead_on_the_screen(&program),
        ..Gamepad::default()
    });
    run(&mut program, &mut headless, 30);
    assert!((program.game.car.speed() - 70.0).abs() < 1e-6, "{}", program.game.car.speed());
    assert!((program.game.car.heading - heading).abs() < 1e-9);
    headless.gamepad = connected(Gamepad {
        left_trigger: 1.0,
        right_stick: [-1.0, 0.0],
        ..Gamepad::default()
    });
    run(&mut program, &mut headless, 14);
    assert!(program.game.car.speed() > 0.0);
    run(&mut program, &mut headless, 2);
    assert_eq!(program.game.car.velocity, [0.0, 0.0]);
    // 0.5 seconds speeding up to 70, and 0.25 slowing down: about 26 units.
    let distance = moved(&program)[0].hypot(moved(&program)[1]);
    assert!((25.0..28.0).contains(&distance), "{distance}");
    assert_eq!(program.game.crashes, 0);
}

// The arrow keys push the car too, toward where they point on the screen,
// where the road goes up and to the left from the start, and Space brakes.
#[test]
fn the_keys_drive_the_car_up_the_road_and_space_stops_it() {
    let (mut program, mut headless) = start();
    headless.held_keys = vec![UP, LEFT];
    run(&mut program, &mut headless, 20);
    assert!(program.game.car.speed() > 10.0);
    let up_and_left = to_ground([-1.0, 1.0]);
    assert!(dot(moved(&program), up_and_left) > 1.0, "{:?}", moved(&program));
    headless.held_keys = vec![UP, LEFT, SPACE];
    run(&mut program, &mut headless, 60);
    assert_eq!(program.game.car.speed(), 0.0);
    assert_eq!(program.game.crashes, 0);
}

// Pushing the car into the road's edge puts it back at the start, at rest,
// and the push doesn't move it again for PAUSE seconds.
#[test]
fn touching_an_edge_puts_the_car_back_at_the_start() {
    let (mut program, mut headless) = start();
    headless.held_keys = vec![RIGHT];
    let mut frames = 0;
    while program.game.crashes == 0 {
        run(&mut program, &mut headless, 1);
        frames += 1;
        assert!(frames < 120, "the car didn't reach the edge");
    }
    let start = start_of(&program);
    assert_eq!((program.game.car.position, program.game.car.heading), (start.position, start.radians()));
    assert!(program.game.paused());
    // Still pushing, it stays put until the pause is over.
    let paused_frames = (PAUSE * 60.0) as usize;
    run(&mut program, &mut headless, paused_frames - 1);
    assert_eq!(program.game.car.position, start.position);
    run(&mut program, &mut headless, 2);
    assert!(!program.game.paused());
    assert_ne!(program.game.car.position, start.position);
    let state = program.state();
    assert_eq!(state.get("Crashes"), Some(&1u32.into()));
}

// Q and E, and the d-pad's left and right, go through the vehicles, around
// from the last to the first, and each puts the car back at the start. The
// d-pad's up and down go through the scenes.
#[test]
fn the_vehicles_go_around_and_each_starts_over() {
    let (mut program, mut headless) = start();
    headless.held_keys = vec![UP];
    run(&mut program, &mut headless, 10);
    headless.held_keys.clear();
    let mut names = Vec::new();
    for code in ['E', 'E', 'E', 'Q'] {
        press(&mut program, &mut headless, code as u32);
        names.push(program.game.current_vehicle().name.clone());
    }
    assert_eq!(names, ["Cyan", "Magenta", "Green", "Magenta"]);
    assert_eq!(program.game.car.position, start_of(&program).position);
    assert!(program.game.paused());
    headless.gamepad = connected(Gamepad {
        buttons: GamepadButton::Left.bit(),
        ..Gamepad::default()
    });
    run(&mut program, &mut headless, 3);
    assert_eq!(program.game.current_vehicle().name, "Cyan", "holding the button switches once");
    // With one scene, the next scene is Ribeye again, from the start.
    headless.gamepad = connected(Gamepad::default());
    run(&mut program, &mut headless, 1);
    headless.gamepad.buttons = GamepadButton::Up.bit();
    run(&mut program, &mut headless, 1);
    assert_eq!(program.game.current_scene().name, "Ribeye");
}

// C, or Menu, switches between the fixed camera and one that follows the
// car, each at its track's height. The wheel and the gamepad zoom, and Home,
// or Y, zooms back.
#[test]
fn the_camera_follows_the_car_and_zooms() {
    let (mut program, mut headless) = start();
    assert_eq!(program.game.camera.height, 150.0);
    press(&mut program, &mut headless, 'C' as u32);
    assert!(program.game.camera.follow);
    assert_eq!(program.game.camera.height, 45.0);
    assert_eq!(program.game.camera_center(), program.game.car.position);
    headless.program(&mut program, &[application::Event::Wheel([1000.0, 500.0], 1.0)]).unwrap();
    assert!((program.game.camera.height - 36.0).abs() < 1e-9);
    press(&mut program, &mut headless, 0x24);
    assert_eq!(program.game.camera.height, 45.0);
    // RB and the left stick up zoom in at ZOOM_SPEED units a second.
    headless.gamepad = connected(Gamepad {
        left_stick: [0.0, 1.0],
        buttons: GamepadButton::RightShoulder.bit(),
        ..Gamepad::default()
    });
    run(&mut program, &mut headless, 30);
    assert!((program.game.camera.height - (45.0 - camera::ZOOM_SPEED * 0.5)).abs() < 1e-9);
    run(&mut program, &mut headless, 60);
    assert_eq!(program.game.camera.height, camera::LOWEST);
    headless.gamepad = connected(Gamepad {
        buttons: GamepadButton::Menu.bit(),
        ..Gamepad::default()
    });
    run(&mut program, &mut headless, 1);
    assert!(!program.game.camera.follow);
    assert_eq!(program.game.camera.height, 150.0);
}

// F1 hides the card and shows it again, and the card fits under the app bar,
// over the left of the window, at either height.
#[test]
fn f1_hides_the_card_which_fits_the_window() {
    let (mut program, mut headless) = start();
    press(&mut program, &mut headless, 0x70);
    assert!(!program.game.show_card);
    press(&mut program, &mut headless, 0x70);
    assert!(program.game.show_card);
    for size in [MAXIMIZED, FULLSCREEN] {
        let mut headless = Headless::new(size);
        headless.notices = vec!["A notice about the settings".to_string()];
        let (area, bottom) = headless.frame(&[], |frame| (play_card(&program.game, frame), frame.content_bottom()));
        assert!(area.width < size[0] / 2.0 && area.height < size[1] / 2.0, "{area:?}");
        assert_eq!((area.x, area.bottom()), (MARGIN, bottom - MARGIN));
        assert!(area.y > PANELS_TOP, "{area:?}");
    }
}

// Without the window's focus, the keys and the gamepad do nothing.
#[test]
fn without_focus_nothing_moves_the_car() {
    let (mut program, mut headless) = start();
    headless.focused = false;
    headless.held_keys = vec![UP];
    headless.gamepad = connected(Gamepad {
        right_stick: [0.0, 1.0],
        buttons: GamepadButton::Menu.bit(),
        ..Gamepad::default()
    });
    run(&mut program, &mut headless, 30);
    assert_eq!(program.game.car.speed(), 0.0);
    assert!(!program.game.camera.follow);
    assert_eq!(key_name(UP), "Up");
}
