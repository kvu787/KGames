// What input does to SimplePaintDemo's editor, through the cards' real
// layout, which user_interface::Headless builds without a window or a GPU,
// and where the cards go at both supported window heights.

use std::path::{Path, PathBuf};

use simple_paint::{Material, Parameters};
use simple_paint_demo::cards::PREVIEW_WIDTH;
use simple_paint_demo::demo::Demo;
use simple_paint_demo::editor::{self, Command, SETTINGS};
use simple_paint_demo::preview::{CopyPlace, IMAGE_HEIGHT, IMAGE_WIDTH, copy_place};
use user_interface::tokens::{FULLSCREEN_HEIGHT, MAXIMIZED_HEIGHT, WINDOW_WIDTH};
use user_interface::{Headless, Key, Modifiers, Point, Rect, Settings, Severity, Startup, slider_rail};
use windows_api::D3D12_BOX;

fn demo(parameters: Parameters, paint_path: PathBuf) -> Headless<Demo> {
    Headless::new(Demo::new(parameters, paint_path))
}

fn nowhere() -> PathBuf {
    PathBuf::from(r"C:\Nowhere\Paint.json")
}

// A fresh folder for one test.
fn folder(test: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("SimplePaintDemo").join(test);
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

// The point at `fraction` along a slider's rail.
fn along(app: &Headless<Demo>, name: &str, fraction: f32) -> Point {
    let area = app.target(name);
    let (left, right) = slider_rail(area);
    Point::new(left + (right - left) * fraction, area.center().y)
}

fn value(app: &Headless<Demo>, name: &str) -> f64 {
    let setting = SETTINGS.iter().find(|setting| setting.name == name).unwrap();
    setting.value(&app.program.editor.parameters)
}

fn selected(app: &Headless<Demo>) -> &'static str {
    SETTINGS[app.program.editor.selected].name
}

#[test]
fn a_press_on_a_slider_sets_it_and_drags_it() {
    let mut app = demo(Parameters::default(), nowhere());
    let half = along(&app, "R", 0.5);
    app.press_point(half);
    assert_eq!(value(&app, "R"), 0.5);
    assert_eq!(app.label("R").as_deref(), Some("0.500"));
    assert_eq!((selected(&app), app.program.editor.dragging), ("R", Some(0)));
    // Dragging follows the pointer along the rail, and holds the slider even
    // when the pointer leaves the row.
    let quarter = along(&app, "R", 0.25);
    app.move_point(Point::new(quarter.x, 900.0));
    assert_eq!(value(&app, "R"), 0.25);
    app.release();
    assert_eq!(app.program.editor.dragging, None);
    // After the release, moving the pointer changes nothing.
    let three_quarters = along(&app, "R", 0.75);
    app.move_point(three_quarters);
    assert_eq!(value(&app, "R"), 0.25);
}

#[test]
fn sliders_stop_at_the_last_step_simple_paint_accepts() {
    let mut app = demo(Parameters::default(), nowhere());
    for (name, low, high) in [
        ("R", 0.001, 0.999),
        ("Brightness", 0.001, 0.999),
        ("Shift", 0.0, 0.999),
        ("Rotation", 0.0, 359.9),
        ("Dark Point", 0.0, 0.999),
        ("Light Point", 0.001, 1.0),
    ] {
        for (x, expected) in [(-500.0f32, low), (5000.0, high)] {
            // Pressed on the rail, then dragged far past its end.
            let half = along(&app, name, 0.5);
            app.press_point(half).move_point(Point::new(x, 0.0)).release();
            assert_eq!(value(&app, name), expected, "{name}");
        }
    }
    Material::compile(&app.program.editor.parameters).unwrap();
}

#[test]
fn a_press_on_a_setting_s_row_selects_it() {
    let mut app = demo(Parameters::default(), nowhere());
    let row = app.target("Setting/Brightness");
    app.click_point(Point::new(row.x + 40.0, row.center().y));
    assert_eq!(selected(&app), "Brightness");
    assert_eq!(app.program.editor.parameters, Parameters::default(), "a press on the name changes nothing");
}

#[test]
fn losing_focus_ends_a_drag() {
    let mut app = demo(Parameters::default(), nowhere());
    let half = along(&app, "Shift", 0.5);
    app.press_point(half);
    assert_eq!(app.program.editor.dragging, Some(4));
    app.focus(false);
    assert_eq!(app.program.editor.dragging, None);
    app.focus(true);
    let far = along(&app, "Shift", 0.9);
    app.move_point(far);
    assert_eq!(app.program.editor.parameters.shift, 0.5);
}

#[test]
fn the_wheel_moves_a_slider_by_its_smallest_step() {
    let mut app = demo(Parameters::default(), nowhere());
    let rotation = along(&app, "Rotation", 0.0);
    app.scroll_point(rotation, 3.0);
    assert_eq!(app.program.editor.parameters.rotation_degrees, 0.3);
    assert_eq!(selected(&app), "Rotation");
    app.scroll_point(rotation, -5.0);
    assert_eq!(app.program.editor.parameters.rotation_degrees, 0.0);
    // Off the sliders, the wheel does nothing.
    app.scroll_point(Point::new(100.0, 100.0), 1.0);
    let label = app.target("Setting/Shift").origin() + Point::new(20.0, 10.0);
    app.scroll_point(label, 1.0);
    assert_eq!(app.program.editor.parameters, Parameters::default());
}

#[test]
fn parts_of_a_notch_add_up_over_the_same_slider() {
    // Like a touchpad, which turns the wheel by a quarter of a notch at a time.
    let mut app = demo(Parameters::default(), nowhere());
    let rotation = along(&app, "Rotation", 0.0);
    for _ in 0..3 {
        app.scroll_point(rotation, 0.25);
    }
    assert_eq!(app.program.editor.parameters.rotation_degrees, 0.0);
    app.scroll_point(rotation, 0.25);
    assert_eq!(app.program.editor.parameters.rotation_degrees, 0.1);
    for _ in 0..6 {
        app.scroll_point(rotation, 0.5);
    }
    assert_eq!(app.program.editor.parameters.rotation_degrees, 0.4);
    // What's left over one slider doesn't carry over to another.
    app.scroll_point(rotation, 0.5);
    assert_eq!(app.program.editor.parameters.rotation_degrees, 0.4);
    let shift = along(&app, "Shift", 0.0);
    app.scroll_point(shift, 0.5).scroll_point(shift, 0.25);
    assert_eq!(app.program.editor.parameters.shift, 0.0);
}

#[test]
fn arrow_keys_select_and_move_a_setting() {
    let mut app = demo(Parameters::default(), nowhere());
    app.key(Key::UP);
    assert_eq!(selected(&app), "R", "Up stops at the first setting");
    app.key(Key::RIGHT);
    // Values on a step are the double nearest their decimal, so they save as
    // written.
    assert_eq!(app.program.editor.parameters.base_color_srgb[0], 0.108);
    app.key_with(Modifiers::SHIFT, Key::RIGHT);
    assert_eq!(app.program.editor.parameters.base_color_srgb[0], 0.118);
    app.key(Key::LEFT);
    assert_eq!(app.program.editor.parameters.base_color_srgb[0], 0.117);
    for _ in 0..20 {
        app.key(Key::DOWN);
    }
    assert_eq!(selected(&app), "Light Point", "Down stops at the last setting");
    app.key(Key::RIGHT);
    assert_eq!(app.program.editor.parameters.light_point, 1.0);
    // With Ctrl, the arrows do nothing.
    app.key_with(Modifiers::CONTROL, Key::LEFT).key_with(Modifiers::CONTROL, Key::UP);
    assert_eq!((selected(&app), app.program.editor.parameters.light_point), ("Light Point", 1.0));
}

#[test]
fn the_sliders_take_no_keyboard_focus_so_the_arrows_stay_the_editor_s() {
    let mut app = demo(Parameters::default(), nowhere());
    let shift = along(&app, "Shift", 0.5);
    app.click_point(shift);
    assert_eq!(selected(&app), "Shift");
    assert_eq!(app.ui.focused_name(), None);
    app.key(Key::RIGHT);
    assert_eq!(app.program.editor.parameters.shift, 0.501);
}

#[test]
fn keys_and_their_modifiers_make_commands() {
    let command = editor::command;
    assert_eq!(command(Key::UP, false, Modifiers::NONE), Some(Command::Select(-1)));
    assert_eq!(command(Key::DOWN, true, Modifiers::SHIFT), Some(Command::Select(1)));
    // Holding an arrow repeats it.
    assert_eq!(command(Key::RIGHT, true, Modifiers::NONE), Some(Command::Nudge(1.0)));
    assert_eq!(command(Key::LEFT, false, Modifiers::SHIFT), Some(Command::Nudge(-10.0)));
    assert_eq!(command(Key::LEFT, false, Modifiers::CONTROL), None);
    // Ctrl+S saves, but holding it doesn't save again.
    assert_eq!(command(Key::S, false, Modifiers::CONTROL), Some(Command::Save));
    assert_eq!(command(Key::S, true, Modifiers::CONTROL), None);
    assert_eq!(command(Key::S, false, Modifiers::NONE), None);
    assert_eq!(command(Key::S, false, Modifiers::CONTROL_SHIFT), None);
}

#[test]
fn an_arrow_or_the_wheel_never_moves_a_value_the_other_way() {
    // SimplePaint accepts these, like MinimalSceneTest's sphere does, but
    // they're past the sliders' ranges, which end at 0.001 and 0.999.
    let parameters = Parameters {
        base_color_srgb: [0.0009765625, 0.5, 0.9990234375],
        ..Parameters::default()
    };
    Material::compile(&parameters).unwrap();
    let mut app = demo(parameters, nowhere());
    app.key(Key::LEFT);
    assert_eq!(app.program.editor.parameters.base_color_srgb[0], 0.0009765625, "Left left R as it was");
    app.key(Key::RIGHT);
    assert_eq!(app.program.editor.parameters.base_color_srgb[0], 0.002);
    let b = along(&app, "B", 0.5);
    app.scroll_point(b, 1.0);
    assert_eq!(app.program.editor.parameters.base_color_srgb[2], 0.9990234375, "the wheel left B as it was");
    app.key(Key::RIGHT);
    assert_eq!(app.program.editor.parameters.base_color_srgb[2], 0.9990234375, "Right left B as it was");
    app.key(Key::LEFT);
    assert_eq!(app.program.editor.parameters.base_color_srgb[2], 0.998);
}

#[test]
fn ctrl_s_and_save_write_paint_json_and_it_loads_back() {
    let path = folder("Save").join("Paint.json");
    let mut app = demo(Parameters::default(), path.clone());
    let shift = along(&app, "Shift", 0.6);
    app.click_point(shift);
    app.key_with(Modifiers::CONTROL, Key::S);
    assert_eq!(app.program.editor.status, Some((format!("Saved to {}", path.display()), false)));
    let mut notices = Vec::new();
    assert_eq!(editor::load(&path, &mut notices), app.program.editor.parameters);
    assert!(notices.is_empty(), "{notices:?}");

    // The Save button does the same, and says when it couldn't.
    app.program.paint_path = nowhere();
    app.click("Save");
    let status = &app.program.editor.status;
    assert!(
        matches!(status, Some((message, true)) if message.starts_with(r"Couldn't save to C:\Nowhere\Paint.json")),
        "{status:?}"
    );
}

#[test]
fn reset_goes_back_to_simple_paint_s_defaults() {
    let mut app = demo(
        Parameters {
            shift: 0.5,
            ..Parameters::default()
        },
        nowhere(),
    );
    app.program.editor.selected = 3;
    app.click("Reset");
    assert_eq!(app.program.editor.parameters, Parameters::default());
    assert_eq!(selected(&app), "Brightness", "the selection stays");
}

#[test]
fn a_bad_paint_json_falls_back_to_the_defaults_with_a_notice() {
    let missing = folder("Missing").join("Paint.json");
    let mut notices = Vec::new();
    assert_eq!(editor::load(&missing, &mut notices), Parameters::default());
    assert!(notices.is_empty(), "a missing file is fine: {notices:?}");
    for (text, problem) in [
        ("{ \"Shift\": }", "line 1, column 12: expected a value"),
        ("{ \"Shift\": 1.5 }", "SimplePaint Shift must be finite and in [0, 0.9990234375]."),
        ("{ \"Glow\": 1 }", "Glow isn't a SimplePaint setting."),
    ] {
        let path = folder("Bad").join("Paint.json");
        std::fs::write(&path, text).unwrap();
        let mut notices = Vec::new();
        assert_eq!(editor::load(&path, &mut notices), Parameters::default());
        assert_eq!(notices.len(), 1);
        assert!(
            notices[0].contains(problem) && notices[0].ends_with("The default material is used."),
            "{}",
            notices[0]
        );
    }
}

#[test]
fn the_demo_starts_with_paint_json_and_says_what_s_wrong_with_it() {
    let saved = folder("Start");
    let parameters = Parameters {
        shift: 0.25,
        ..Parameters::default()
    };
    std::fs::write(saved.join("Paint.json"), paint_settings::format(&parameters, 0)).unwrap();
    let app = Headless::start(Startup::new(Settings::from_text("{}"), &saved, false), |startup| {
        Ok(Demo::start(startup))
    });
    assert_eq!(app.program.editor.parameters, parameters);
    assert!(app.ui.notices().is_empty());

    let bad = folder("StartBad");
    std::fs::write(bad.join("Paint.json"), "{ \"Glow\": 1 }").unwrap();
    let app = Headless::start(Startup::new(Settings::from_text("{}"), &bad, false), |startup| Ok(Demo::start(startup)));
    assert_eq!(app.program.editor.parameters, Parameters::default());
    let notice = &app.ui.notices()[0];
    assert_eq!((notice.severity, notice.title.as_str()), (Severity::Warning, "Paint.json"));
    assert!(notice.message.contains("Glow isn't a SimplePaint setting."), "{}", notice.message);
    assert!(app.has_target("Notice/0"));

    // A scripted run starts from the default material, whatever's there.
    let app = Headless::start(Startup::for_tests("{}", &saved), |startup| Ok(Demo::start(startup)));
    assert_eq!(app.program.editor.parameters, Parameters::default());
}

#[test]
fn any_input_leaves_a_valid_material() {
    let mut app = demo(Parameters::default(), nowhere());
    // A fixed sequence of presses, drags, wheels and keys all over the sliders.
    let mut seed = 0x5EED_u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as f32 / (1u64 << 31) as f32
    };
    for _ in 0..2000 {
        let name = SETTINGS[(next() * 8.0) as usize % 8].name;
        let point = along(&app, name, next() * 1.4 - 0.2);
        let key = [Key::LEFT, Key::RIGHT, Key::UP, Key::DOWN][(next() * 4.0) as usize % 4];
        // The editor compiles every change, and would panic on an invalid one.
        match (next() * 6.0) as u32 {
            0 => app.press_point(point),
            1 => app.move_point(point),
            2 => app.release(),
            3 => app.scroll_point(point, next() * 20.0 - 10.0),
            4 => app.key(key),
            _ => app.key_with(Modifiers::SHIFT, key),
        };
        Material::compile(&app.program.editor.parameters).unwrap();
    }
}

#[test]
fn the_cards_fit_both_window_heights() {
    let maximized = demo(Parameters::default(), nowhere());
    let names = |app: &Headless<Demo>| -> Vec<String> {
        app.ui.targets().iter().filter(|target| target.listed).map(|target| target.name.clone()).collect()
    };
    // The buttons, then each setting's row and slider.
    let mut expected = vec!["Reset".to_string(), "Save".to_string()];
    for setting in &SETTINGS {
        expected.extend([format!("Setting/{}", setting.name), setting.name.to_string()]);
    }
    assert_eq!(names(&maximized), expected);
    for fullscreen in [false, true] {
        let mut app = demo(Parameters::default(), nowhere());
        app.set_fullscreen(fullscreen).frame();
        let height = if fullscreen { FULLSCREEN_HEIGHT } else { MAXIMIZED_HEIGHT };
        for name in names(&app) {
            let area = app.target(&name);
            assert!(area.x >= PREVIEW_WIDTH && area.right() <= WINDOW_WIDTH && area.bottom() <= height, "{name}: {area:?}");
            // The layout doesn't depend on the height above the status bar.
            assert_eq!(area, maximized.target(&name), "{name}");
        }
        // The view fills the preview card, and the sphere's image fills the
        // view: the whole image in fullscreen, and its middle maximized.
        let view = app.program.view.get().unwrap();
        assert_eq!(view.width, IMAGE_WIDTH as f32);
        assert_eq!(view.height, IMAGE_HEIGHT as f32 - if fullscreen { 0.0 } else { FULLSCREEN_HEIGHT - MAXIMIZED_HEIGHT });
        let place = copy_place(view, [WINDOW_WIDTH as u32, height as u32]).unwrap();
        let box_size = [place.source.right - place.source.left, place.source.bottom - place.source.top];
        assert_eq!(place.destination, [view.x as u32, view.y as u32]);
        assert_eq!(box_size, [view.width as u32, view.height as u32]);
    }
    // Every card's text fits, with a long message from Save.
    for fullscreen in [false, true] {
        let mut app = demo(Parameters::default(), folder("Fit").join("A folder with a long name to save in").join("Paint.json"));
        app.key_with(Modifiers::CONTROL, Key::S).set_fullscreen(fullscreen);
        assert!(app.program.editor.status.as_ref().is_some_and(|(_, failed)| *failed));
        assert_eq!(*app.program.overflow.borrow(), Vec::<&str>::new(), "cards whose content didn't fit, fullscreen {fullscreen}");
    }
}

#[test]
fn the_image_s_middle_goes_in_the_view_inside_the_frame() {
    let size = |place: CopyPlace| [place.source.right - place.source.left, place.source.bottom - place.source.top];
    // A view as big as the image takes all of it.
    let whole = copy_place(Rect::new(32.0, 120.0, 1152.0, 1239.0), [2560, 1440]).unwrap();
    assert_eq!(whole.source, D3D12_BOX { left: 0, top: 0, front: 0, right: 1152, bottom: 1239, back: 1 });
    // A shorter one takes its middle.
    let middle = copy_place(Rect::new(32.0, 120.0, 1152.0, 1139.0), [2560, 1369]).unwrap();
    assert_eq!((middle.destination, middle.source.top, size(middle)), ([32, 120], 50, [1152, 1139]));
    // A bigger one has the image in its middle.
    let centered = copy_place(Rect::new(0.0, 0.0, 1352.0, 1439.0), [2560, 1440]).unwrap();
    assert_eq!((centered.destination, size(centered)), ([100, 100], [1152, 1239]));
    // Past the frame's edges, it's cut there, and outside it, there's none.
    let cut = copy_place(Rect::new(-100.0, 1000.0, 1152.0, 1239.0), [2560, 1440]).unwrap();
    assert_eq!((cut.destination, cut.source.left, cut.source.top, size(cut)), ([0, 1000], 100, 0, [1052, 440]));
    assert_eq!(copy_place(Rect::new(3000.0, 0.0, 1152.0, 1239.0), [2560, 1440]), None);
}
