// MinimalSceneTest's settings: each object's material, the file that it
// writes, and what the card says about a setting that's wrong.

use std::path::Path;

use minimal_scene_test::moving_sphere::MovingSphere;
use minimal_scene_test::options;
use minimal_scene_test::settings::SceneSettings;
use simple_paint::{Material, Parameters};
use user_interface::graphics::{AdapterChoice, Gpu, write_png};
use user_interface::{Headless, Point, Settings, Startup, Theme, WindowMode};

#[test]
fn the_defaults_open_fullscreen_with_a_constant_color_sphere() {
    assert_eq!(options().window_mode, WindowMode::BorderlessFullscreen);
    let scene = SceneSettings::default();
    // Equal Dark and Light Points paint a constant color, which is the base
    // color at tone 1 - Brightness.
    assert_eq!(scene.sphere.dark_point, scene.sphere.light_point);
    assert_eq!(scene.sphere.dark_point, 1.0 - scene.sphere.brightness);
    for material in [&scene.sphere, &scene.ground, &scene.cube] {
        Material::compile(material).unwrap();
    }
}

#[test]
fn the_file_it_writes_reads_back_as_the_defaults() {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("MinimalSceneSettings");
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("Settings.json");
    let mut settings = Settings::load(&path);
    assert_eq!(SceneSettings::read(&mut settings), SceneSettings::default());
    settings.write_if_missing().unwrap();
    let mut settings = Settings::load(&path);
    assert_eq!(SceneSettings::read(&mut settings), SceneSettings::default());
    assert!(settings.problems().is_empty(), "{:?}", settings.problems());
}

#[test]
fn the_program_s_settings_file_has_the_defaults() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Program/MinimalSceneTest/Settings.json");
    let mut settings = Settings::load(&path);
    assert_eq!(settings.choice("WindowMode", "Maximized", &["Maximized", "BorderlessFullscreen"]), "BorderlessFullscreen");
    assert_eq!(SceneSettings::read(&mut settings), SceneSettings::default());
    assert!(settings.problems().is_empty(), "{:?}", settings.problems());
}

#[test]
fn a_material_that_is_wrong_keeps_its_default() {
    let mut settings = Settings::from_text(r#"{ "Ground": { "Shift": 0.5, "RotationDegrees": 90 }, "Cube": { "Shift": 2 } }"#);
    let read = SceneSettings::read(&mut settings);
    let defaults = SceneSettings::default();
    assert_eq!(
        read.ground,
        Parameters {
            shift: 0.5,
            rotation_degrees: 90.0,
            ..defaults.ground
        }
    );
    assert_eq!(read.cube, defaults.cube, "a material SimplePaint rejects keeps its default");
    assert_eq!(read.sphere, defaults.sphere);
    assert_eq!(settings.problems(), ["Cube SimplePaint Shift must be finite and in [0, 0.9990234375], so it's the default."]);
}

#[test]
fn the_card_says_what_is_wrong_with_the_settings() {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("MinimalSceneNotices");
    std::fs::create_dir_all(&folder).unwrap();
    let startup = Startup::for_tests(r#"{ "Sphere": { "Brightness": "Bright" } }"#, &folder);
    let mut app = Headless::start(startup, |startup| Ok(MovingSphere::start(startup)));
    assert_eq!(app.program.settings, SceneSettings::default());
    let label = app.label("Notice/0").unwrap();
    assert!(label.starts_with("Warning: Settings.json Sphere "), "{label}");
    let card = app.target(minimal_scene_test::info_card::NAME);
    let notice = app.target("Notice/0");
    assert!(card.contains(notice.origin()) && card.contains(Point::new(notice.right(), notice.bottom())));

    // It's a warning, in its colors.
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let image = app.render(&mut gpu).unwrap();
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/TestOutput");
    std::fs::create_dir_all(&output).unwrap();
    write_png(&output.join("MinimalSceneTestNotice.png"), &image).unwrap();
    let theme = Theme::light();
    let fill = image.pixel(notice.right() as u32 - 8, notice.bottom() as u32 - 8);
    let expected = theme.caution_background.over(theme.acrylic).to_bytes();
    assert!(fill.iter().zip(expected).take(3).all(|(a, b)| (*a as i32 - b as i32).abs() <= 2), "{fill:?} isn't {expected:?}");
}
