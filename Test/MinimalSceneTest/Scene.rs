// MinimalSceneTest without a window: the sphere's motion, the keys, and the
// settings file.

use std::path::Path;

use application::{Event, ProgramSettings, Settings, WindowMode};
use minimal_scene_test::controls::{Action, action};
use minimal_scene_test::scene::Motion;
use minimal_scene_test::settings::SceneSettings;
use simple_paint::{Material, Parameters};

fn key(key: u32, control: bool) -> Event {
    Event::Key {
        key,
        control,
        shift: false,
        repeat: false,
    }
}

#[test]
fn the_sphere_starts_in_the_middle_moving_at_8_units_a_second() {
    let mut motion = Motion::default();
    assert_eq!((motion.position, motion.direction), (0.0, 1.0));
    motion.advance(0.5);
    assert_eq!(motion.position, 4.0);
}

#[test]
fn the_sphere_turns_back_once_it_is_past_7() {
    let mut motion = Motion::default();
    // It moves by the frame's time, and only turns back after a frame that
    // took it past the end.
    for _ in 0..7 {
        motion.advance(0.125);
    }
    assert_eq!((motion.position, motion.direction), (7.0, 1.0));
    motion.advance(0.125);
    assert_eq!((motion.position, motion.direction), (8.0, -1.0));
    for _ in 0..16 {
        motion.advance(0.125);
    }
    assert_eq!((motion.position, motion.direction), (-8.0, 1.0));
    motion.advance(0.125);
    assert_eq!(motion.position, -7.0);
}

#[test]
fn small_frames_keep_the_sphere_near_its_path() {
    let mut motion = Motion::default();
    let frame = 1.0 / 3000.0;
    let mut turns = 0;
    for _ in 0..(3000 * 10) {
        let direction = motion.direction;
        motion.advance(frame);
        turns += (motion.direction != direction) as u32;
        assert!(motion.position.abs() <= 7.0 + 8.0 * frame + 1e-4, "{}", motion.position);
    }
    // 80 units in 10 seconds, 14 units between turns, after 7 to the first.
    assert_eq!(turns, 6);
}

#[test]
fn f1_toggles_the_text_and_f11_fullscreen() {
    assert_eq!(action(&key(0x70, false)), Some(Action::ToggleText));
    assert_eq!(action(&key(0x70, true)), Some(Action::ToggleText));
    assert_eq!(action(&key(0x7A, false)), Some(Action::ToggleFullscreen));
    assert_eq!(action(&key('S' as u32, false)), None);
    assert_eq!(action(&Event::Press([10.0, 10.0])), None);
    // Holding a key down doesn't flip it back and forth.
    for code in [0x70, 0x7A] {
        let held = Event::Key {
            key: code,
            control: false,
            shift: false,
            repeat: true,
        };
        assert_eq!(action(&held), None);
    }
}

#[test]
fn the_defaults_open_fullscreen_with_a_constant_color_sphere() {
    let (settings, scene) = SceneSettings::defaults();
    assert_eq!(settings.window_mode, WindowMode::BorderlessFullscreen);
    // Equal Dark and Light Points paint a constant color, which is the base
    // color at tone 1 - Brightness.
    assert_eq!(scene.sphere.dark_point, scene.sphere.light_point);
    assert_eq!(scene.sphere.dark_point, 1.0 - scene.sphere.brightness);
    for material in [&scene.sphere, &scene.ground, &scene.cube] {
        Material::compile(material).unwrap();
    }
}

#[test]
fn the_settings_file_takes_each_object_s_material() {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("MinimalSceneSettings");
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("Settings.json");

    // The file it writes reads back as the defaults.
    let mut notices = Vec::new();
    let (_, written) = Settings::load::<SceneSettings>(&path, &mut notices);
    let (_, read) = Settings::load::<SceneSettings>(&path, &mut notices);
    assert!(notices.is_empty(), "{notices:?}");
    assert_eq!(
        [read.sphere, read.ground, read.cube],
        [written.sphere, written.ground, written.cube]
    );

    std::fs::write(
        &path,
        r#"{ "Ground": { "Shift": 0.5, "RotationDegrees": 90 }, "Cube": { "Shift": 2 } }"#,
    )
    .unwrap();
    let (_, read) = Settings::load::<SceneSettings>(&path, &mut notices);
    assert_eq!(
        read.ground,
        Parameters {
            shift: 0.5,
            rotation_degrees: 90.0,
            ..written.ground
        }
    );
    assert_eq!(read.cube, written.cube, "a material SimplePaint rejects keeps its default");
    assert_eq!(
        notices,
        [format!(
            "{}: Cube SimplePaint Shift must be finite and in [0, 0.9990234375].",
            path.display()
        )]
    );
}
