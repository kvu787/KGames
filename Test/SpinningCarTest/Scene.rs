// SpinningCarTest without a window: where the car is over time, the keys, the
// settings file, and the car mesh.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};
use std::path::Path;

use application::{Event, ProgramSettings, Settings};
use simple_paint::{Material, validate_mesh};
use spinning_car_test::car_mesh::{INDICES, MATERIAL_COUNT, VERTICES};
use spinning_car_test::controls::{Action, action};
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
fn f1_toggles_the_text_f11_fullscreen_and_s_the_car() {
    let key = |key: u32| Event::Key {
        key,
        control: false,
        shift: false,
        repeat: false,
    };
    assert_eq!(action(&key(0x70)), Some(Action::ToggleText));
    assert_eq!(action(&key(0x7A)), Some(Action::ToggleFullscreen));
    assert_eq!(action(&key('S' as u32)), Some(Action::ToggleStationary));
    assert_eq!(action(&key('V' as u32)), None, "VSync isn't a setting here");
    assert_eq!(action(&key(0x1B)), None, "Esc doesn't quit");
    // Holding a key down doesn't flip it back and forth.
    for code in [0x70, 0x7A, 'S' as u32] {
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
fn the_car_mesh_is_what_simple_paint_accepts() {
    assert_eq!((VERTICES.len(), INDICES.len(), MATERIAL_COUNT), (1938, 5652, 5));
    // The car's five materials and the sphere's.
    validate_mesh(&VERTICES, &INDICES, MATERIAL_COUNT + 1).unwrap();
    for material in 0..MATERIAL_COUNT {
        assert!(VERTICES.iter().any(|vertex| vertex.material_index == material), "no vertex of material {material}");
    }
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
        (r#"{ "UResolution": 3, "VResolution": 512 }"#, [3, 512], None),
        (r#"{ "UResolution": 2 }"#, [64, 32], Some("UResolution must be a whole number from 3 to 512, not 2")),
        (r#"{ "VResolution": 513 }"#, [64, 32], Some("VResolution must be a whole number from 2 to 512, not 513")),
        (r#"{ "UResolution": 64.5 }"#, [64, 32], Some("UResolution must be a whole number from 3 to 512, not 64.5")),
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
