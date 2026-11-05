// SpinningCarTest without a window: where the car is over time, its key, and
// the settings file. Test/SceneTest tests F1 and F11, and Test/Meshes the
// car mesh.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};
use std::path::Path;

use application::{Event, ProgramSettings, Settings};
use simple_paint::Material;
use spinning_car_test::controls::toggles_stationary;
use spinning_car_test::scene::car_placement;
use spinning_car_test::settings::SceneSettings;

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
fn s_toggles_the_car() {
    let key = |key: u32| Event::Key {
        key,
        control: false,
        shift: false,
        repeat: false,
    };
    assert!(toggles_stationary(&key('S' as u32)));
    // Library/SceneTest handles F1 and F11.
    for other in [0x70, 0x7A, 'V' as u32, 0x1B] {
        assert!(!toggles_stationary(&key(other)), "{other:#X}");
    }
    // Holding it down doesn't flip it back and forth.
    let held = Event::Key {
        key: 'S' as u32,
        control: false,
        shift: false,
        repeat: true,
    };
    assert!(!toggles_stationary(&held));
}

#[test]
fn the_defaults_are_materials_that_simple_paint_accepts() {
    let (settings, scene) = SceneSettings::defaults();
    assert_eq!(settings.display, None);
    assert_eq!(scene.sphere_resolution, [64, 32]);
    // The body's.
    assert_eq!(scene.materials[1].base_color_srgb, [0.103999995, 0.3592001, 0.8]);
    assert_eq!(scene.materials[1].rotation_degrees, 35.600586);
    for material in &scene.materials {
        Material::compile(material).unwrap();
    }
}

#[test]
fn the_settings_file_checks_the_sphere_s_resolution() {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("SpinningCarSettings");
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("Settings.json");
    let mut notices = Vec::new();
    let (_, written) = Settings::load::<SceneSettings>(&path, &mut notices);
    let (_, read) = Settings::load::<SceneSettings>(&path, &mut notices);
    assert!(notices.is_empty(), "{notices:?}");
    assert_eq!((read.materials, read.sphere_resolution), (written.materials, written.sphere_resolution));

    for (sphere, resolution, problem) in [
        (r#"{ "UResolution": 4, "VResolution": 512 }"#, [4, 512], None),
        (r#"{ "UResolution": 3 }"#, [64, 32], Some("UResolution must be a whole number from 4 to 512, not 3")),
        (r#"{ "VResolution": 513 }"#, [64, 32], Some("VResolution must be a whole number from 2 to 512, not 513")),
        (r#"{ "UResolution": 64.5 }"#, [64, 32], Some("UResolution must be a whole number from 4 to 512, not 64.5")),
        (r#"{ "Radius": 1 }"#, [64, 32], Some("Radius isn't a sphere setting. They're UResolution and VResolution.")),
        (r#"{ "UResolution": 128, "VResolution": 600 }"#, [64, 32], Some("VResolution must be a whole number from 2 to 512, not 600")),
    ] {
        std::fs::write(&path, format!(r#"{{ "Sphere": {sphere} }}"#)).unwrap();
        let mut notices = Vec::new();
        let (_, read) = Settings::load::<SceneSettings>(&path, &mut notices);
        assert_eq!(read.sphere_resolution, resolution, "for {sphere}");
        let expected: Vec<String> = problem
            .map(|problem| format!("{}: Sphere {problem}", path.display()))
            .into_iter()
            .collect();
        assert_eq!(notices, expected, "for {sphere}");
    }
}
