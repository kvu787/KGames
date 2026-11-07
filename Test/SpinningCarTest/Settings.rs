// SpinningCarTest's settings: the materials, the sphere's resolution, the
// file that it writes, and what the card says about a setting that's wrong.

use std::path::Path;

use simple_paint::Material;
use spinning_car_test::settings::SceneSettings;
use spinning_car_test::spinning_car::SpinningCar;
use user_interface::{Headless, InfoCard, Settings, Startup};

#[test]
fn the_defaults_are_materials_that_simple_paint_accepts() {
    let scene = SceneSettings::default();
    assert_eq!(scene.sphere_resolution, [64, 32]);
    // The body's.
    assert_eq!(scene.materials[1].base_color_srgb, [0.103999995, 0.3592001, 0.8]);
    assert_eq!(scene.materials[1].rotation_degrees, 35.600586);
    for material in &scene.materials {
        Material::compile(material).unwrap();
    }
}

#[test]
fn the_file_it_writes_reads_back_as_the_defaults() {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("SpinningCarSettings");
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Program/SpinningCarTest/Settings.json");
    let mut settings = Settings::load(&path);
    assert_eq!(SceneSettings::read(&mut settings), SceneSettings::default());
    assert!(settings.problems().is_empty(), "{:?}", settings.problems());
}

#[test]
fn the_settings_check_the_sphere_s_resolution() {
    for (sphere, resolution, problem) in [
        (r#"{ "UResolution": 4, "VResolution": 512 }"#, [4, 512], None),
        (r#"{ "VResolution": 2 }"#, [64, 2], None),
        (r#"{ "UResolution": 3 }"#, [64, 32], Some("UResolution must be a whole number from 4 to 512, not 3")),
        (r#"{ "VResolution": 513 }"#, [64, 32], Some("VResolution must be a whole number from 2 to 512, not 513")),
        (r#"{ "UResolution": 64.5 }"#, [64, 32], Some("UResolution must be a whole number from 4 to 512, not 64.5")),
        (r#"{ "Radius": 1 }"#, [64, 32], Some("has Radius, which isn't one of UResolution and VResolution")),
        (r#"{ "UResolution": 128, "VResolution": 600 }"#, [64, 32], Some("VResolution must be a whole number from 2 to 512, not 600")),
        ("64", [64, 32], Some("must be an object with UResolution and VResolution, not 64")),
    ] {
        let mut settings = Settings::from_text(&format!(r#"{{ "Sphere": {sphere} }}"#));
        let read = SceneSettings::read(&mut settings);
        assert_eq!(read.sphere_resolution, resolution, "for {sphere}");
        let expected: Vec<String> = problem.map(|problem| format!("Sphere {problem}, so it's the default.")).into_iter().collect();
        assert_eq!(settings.problems(), expected, "for {sphere}");
    }
}

#[test]
fn a_material_that_is_wrong_keeps_its_default() {
    let mut settings = Settings::from_text(r#"{ "SimplePaintShader_Body": { "Brightness": 0.25 }, "SimplePaintShader_Wheels": { "Shift": 2 } }"#);
    let read = SceneSettings::read(&mut settings);
    let defaults = SceneSettings::default();
    assert_eq!(read.materials[1].brightness, 0.25);
    assert_eq!(read.materials[4], defaults.materials[4], "a material SimplePaint rejects keeps its default");
    assert_eq!(
        settings.problems(),
        ["SimplePaintShader_Wheels SimplePaint Shift must be finite and in [0, 0.9990234375], so it's the default."]
    );
}

#[test]
fn the_card_says_what_is_wrong_with_the_settings() {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("SpinningCarNotices");
    std::fs::create_dir_all(&folder).unwrap();
    let startup = Startup::for_tests(r#"{ "Sphere": { "UResolution": 1000 } }"#, &folder);
    let app = Headless::start(startup, |startup| Ok(SpinningCar::start(startup)));
    assert_eq!(app.program.settings, SceneSettings::default());
    let label = app.label("Notice/0").unwrap();
    assert!(label.starts_with("Warning: Settings.json Sphere "), "{label}");
    let card = app.target(InfoCard::NAME);
    assert!(card.contains(app.target("Notice/0").center()));
}
