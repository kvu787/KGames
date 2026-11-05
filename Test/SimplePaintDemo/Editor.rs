// What input does to SimplePaintDemo's editor, through the cards' real
// layout, which a canvas builds without a window or a GPU, and where the
// cards go at both supported window heights.

use std::path::{Path, PathBuf};

use application::Event;
use simple_paint::{Material, Parameters};
use simple_paint_demo::editor::{self, Editor, SLIDERS};
use simple_paint_demo::panels::{self, Cache, Layout};
use simple_paint_demo::preview::IMAGE_SIZE;
use user_interface::headless::{FULLSCREEN, Headless, MAXIMIZED, click, control, key, shifted};
use user_interface::{Rect, slider_rail};
use windows_api::*;

// The editor in a window without a GPU, drawn once, so that the next
// frame's input has targets.
struct Demo {
    editor: Editor,
    headless: Headless,
    cache: Cache,
    paint_path: PathBuf,
}

impl Demo {
    fn new(parameters: Parameters, paint_path: PathBuf) -> Self {
        Self::sized(parameters, paint_path, MAXIMIZED)
    }

    fn sized(parameters: Parameters, paint_path: PathBuf, size: [f32; 2]) -> Self {
        let mut demo = Demo {
            editor: Editor::new(parameters),
            headless: Headless::new(size),
            cache: Cache::default(),
            paint_path,
        };
        demo.step(&[]);
        demo
    }

    // Draws a frame with these events, and returns where the image went,
    // and how many lines of text didn't fit.
    fn step(&mut self, events: &[Event]) -> (Layout, usize) {
        let Demo {
            editor,
            headless,
            cache,
            paint_path,
        } = self;
        headless.frame(events, |frame| {
            let layout = panels::draw(frame, cache, editor, paint_path);
            (layout, frame.dropped_lines())
        })
    }

    fn target(&self, name: &str) -> Rect {
        self.headless.target(name)
    }

    // The point at `fraction` along a slider's rail.
    fn along(&self, name: &str, fraction: f32) -> [f32; 2] {
        let area = self.target(name);
        let [left, right] = slider_rail(area);
        [left + (right - left) * fraction, area.center()[1]]
    }

    fn value(&self, name: &str) -> f64 {
        let slider = SLIDERS.iter().find(|slider| slider.name == name).unwrap();
        slider.value(&self.editor.parameters)
    }
}

fn repeated(key: i32) -> Event {
    Event::Key {
        key: key as u32,
        control: false,
        shift: false,
        repeat: true,
    }
}

// A file for one test, in a fresh folder.
fn file(test: &str, name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder.join(name)
}

fn nowhere() -> PathBuf {
    PathBuf::from(r"C:\Nowhere\Paint.json")
}

#[test]
fn a_press_on_a_slider_sets_it_and_drags_it() {
    let mut demo = Demo::new(Parameters::default(), nowhere());
    let half = demo.along("R", 0.5);
    demo.step(&[Event::Press(half)]);
    assert_eq!(demo.value("R"), 0.5);
    assert_eq!((demo.editor.selected, demo.editor.dragging), (0, Some(0)));
    // Dragging follows the cursor along the track, and holds the slider
    // even when the cursor leaves the row.
    let quarter = demo.along("R", 0.25);
    demo.headless.move_to([quarter[0], 900.0]);
    demo.step(&[]);
    assert_eq!(demo.value("R"), 0.25);
    demo.step(&[Event::Release([quarter[0], 900.0])]);
    assert_eq!(demo.editor.dragging, None);
    // After the release, moving the cursor changes nothing.
    let three_quarters = demo.along("R", 0.75);
    demo.headless.move_to(three_quarters);
    demo.step(&[]);
    assert_eq!(demo.value("R"), 0.25);
}

#[test]
fn sliders_stop_at_the_last_step_simple_paint_accepts() {
    let mut demo = Demo::new(Parameters::default(), nowhere());
    for (name, low, high) in [
        ("R", 0.001, 0.999),
        ("Brightness", 0.001, 0.999),
        ("Shift", 0.0, 0.999),
        ("Rotation", 0.0, 359.9),
        ("Dark Point", 0.0, 0.999),
        ("Light Point", 0.001, 1.0),
    ] {
        for (x, expected) in [(-500.0f32, low), (5000.0, high)] {
            // Pressed on the track, then dragged far past its end.
            let half = demo.along(name, 0.5);
            demo.step(&[Event::Press(half)]);
            demo.headless.move_to([x, 0.0]);
            demo.step(&[]);
            demo.step(&[Event::Release([x, 0.0])]);
            assert_eq!(demo.value(name), expected, "{name}");
        }
    }
    Material::compile(&demo.editor.parameters).unwrap();
}

#[test]
fn losing_focus_ends_a_drag() {
    let mut demo = Demo::new(Parameters::default(), nowhere());
    let half = demo.along("Shift", 0.5);
    demo.step(&[Event::Press(half)]);
    // Without focus, a frame has no cursor, and the window ends the press
    // with a release.
    demo.headless.focused = false;
    demo.step(&[Event::Release([-1.0, -1.0])]);
    assert_eq!(demo.editor.dragging, None);
    demo.headless.focused = true;
    let far = demo.along("Shift", 0.9);
    demo.headless.move_to(far);
    demo.step(&[]);
    assert_eq!(demo.editor.parameters.shift, 0.5);
}

#[test]
fn the_wheel_moves_a_slider_by_its_smallest_step() {
    let mut demo = Demo::new(Parameters::default(), nowhere());
    let rotation = demo.along("Rotation", 0.0);
    demo.step(&[Event::Wheel(rotation, 3.0)]);
    assert_eq!(demo.editor.parameters.rotation_degrees, 0.3);
    assert_eq!(SLIDERS[demo.editor.selected].name, "Rotation");
    demo.step(&[Event::Wheel(rotation, -5.0)]);
    assert_eq!(demo.editor.parameters.rotation_degrees, 0.0);
    // Off the sliders, the wheel does nothing.
    demo.step(&[Event::Wheel([100.0, 100.0], 1.0)]);
    assert_eq!(demo.editor.parameters, Parameters { rotation_degrees: 0.0, ..Parameters::default() });
}

#[test]
fn parts_of_a_notch_add_up_over_the_same_slider() {
    // Like a touchpad, which turns the wheel by a quarter of a notch at a time.
    let mut demo = Demo::new(Parameters::default(), nowhere());
    let rotation = demo.along("Rotation", 0.0);
    for _ in 0..3 {
        demo.step(&[Event::Wheel(rotation, 0.25)]);
    }
    assert_eq!(demo.editor.parameters.rotation_degrees, 0.0);
    demo.step(&[Event::Wheel(rotation, 0.25)]);
    assert_eq!(demo.editor.parameters.rotation_degrees, 0.1);
    for _ in 0..6 {
        demo.step(&[Event::Wheel(rotation, -0.5), Event::Wheel(rotation, 1.0)]);
    }
    assert_eq!(demo.editor.parameters.rotation_degrees, 0.4);
    // What's left over one slider doesn't carry over to another.
    demo.step(&[Event::Wheel(rotation, 0.5)]);
    assert_eq!(demo.editor.parameters.rotation_degrees, 0.4);
    let shift = demo.along("Shift", 0.0);
    demo.step(&[Event::Wheel(shift, 0.5), Event::Wheel(shift, 0.25)]);
    assert_eq!(demo.editor.parameters.shift, 0.0);
}

#[test]
fn arrow_keys_select_and_move_a_slider() {
    let mut demo = Demo::new(Parameters::default(), nowhere());
    demo.step(&[key(VK_UP)]);
    assert_eq!(demo.editor.selected, 0, "Up stops at the first slider");
    demo.step(&[key(VK_RIGHT)]);
    // Values on a step are the double nearest their decimal, so they save
    // as written.
    assert_eq!(demo.editor.parameters.base_color_srgb[0], 0.108);
    demo.step(&[shifted(VK_RIGHT)]);
    assert_eq!(demo.editor.parameters.base_color_srgb[0], 0.118);
    demo.step(&[key(VK_LEFT)]);
    assert_eq!(demo.editor.parameters.base_color_srgb[0], 0.117);
    demo.step(&[key(VK_DOWN); 20]);
    assert_eq!(SLIDERS[demo.editor.selected].name, "Light Point", "Down stops at the last slider");
    demo.step(&[key(VK_RIGHT)]);
    assert_eq!(demo.editor.parameters.light_point, 1.0);
}

#[test]
fn the_sliders_take_no_keyboard_focus_so_the_arrows_stay_the_editors() {
    let mut demo = Demo::new(Parameters::default(), nowhere());
    let shift = demo.along("Shift", 0.5);
    demo.step(&click(shift));
    assert_eq!(SLIDERS[demo.editor.selected].name, "Shift");
    assert_eq!(demo.headless.interface.focus(), None);
    demo.step(&[key(VK_TAB), key(VK_RIGHT)]);
    assert_eq!(demo.editor.parameters.shift, 0.501);
}

#[test]
fn an_arrow_never_moves_a_value_the_other_way() {
    // SimplePaint accepts these, like MinimalSceneTest's sphere does, but
    // they're past the sliders' ranges, which end at 0.001 and 0.999.
    let parameters = Parameters {
        base_color_srgb: [0.0009765625, 0.5, 0.9990234375],
        ..Parameters::default()
    };
    Material::compile(&parameters).unwrap();
    let mut demo = Demo::new(parameters, nowhere());
    demo.step(&[key(VK_LEFT)]);
    assert_eq!(demo.editor.parameters.base_color_srgb[0], 0.0009765625, "Left left R as it was");
    demo.step(&[key(VK_RIGHT)]);
    assert_eq!(demo.editor.parameters.base_color_srgb[0], 0.002);
    demo.step(&[key(VK_DOWN), key(VK_DOWN), key(VK_RIGHT)]);
    assert_eq!(demo.editor.parameters.base_color_srgb[2], 0.9990234375, "Right left B as it was");
    demo.step(&[key(VK_LEFT)]);
    assert_eq!(demo.editor.parameters.base_color_srgb[2], 0.998);
}

#[test]
fn holding_an_arrow_repeats_it_but_holding_ctrl_s_does_not() {
    let path = file("Repeat", "Paint.json");
    let mut demo = Demo::new(Parameters::default(), path.clone());
    demo.step(&[key(VK_RIGHT), repeated(VK_RIGHT)]);
    assert_eq!(demo.editor.parameters.base_color_srgb[0], 0.109);
    let held_save = Event::Key {
        key: 'S' as u32,
        control: true,
        shift: false,
        repeat: true,
    };
    demo.step(&[held_save]);
    assert_eq!(demo.editor.status, None);
    assert!(!path.exists());
}

#[test]
fn ctrl_s_and_save_write_paint_json_and_it_loads_back() {
    let path = file("Save", "Paint.json");
    let mut demo = Demo::new(Parameters::default(), path.clone());
    let shift = demo.along("Shift", 0.6);
    demo.step(&click(shift));
    demo.step(&[control('S' as i32)]);
    assert_eq!(demo.editor.status, Some((format!("Saved to {}", path.display()), false)));
    let mut notices = Vec::new();
    assert_eq!(editor::load(&path, &mut notices), demo.editor.parameters);
    assert!(notices.is_empty(), "{notices:?}");

    // The Save button does the same, and says when it couldn't.
    demo.paint_path = nowhere();
    let save = demo.target("Save");
    demo.step(&click([save.x + 5.0, save.y + 5.0]));
    assert!(
        matches!(&demo.editor.status, Some((message, true)) if message.starts_with(r"Couldn't save to C:\Nowhere\Paint.json"))
    );
}

#[test]
fn reset_goes_back_to_simple_paint_s_defaults() {
    let mut demo = Demo::new(
        Parameters {
            shift: 0.5,
            ..Parameters::default()
        },
        nowhere(),
    );
    demo.editor.selected = 3;
    let reset = demo.target("Reset");
    demo.step(&click([reset.x + 5.0, reset.y + 5.0]));
    assert_eq!(demo.editor.parameters, Parameters::default());
    assert_eq!(demo.editor.selected, 3, "the selection stays");
}

#[test]
fn a_bad_paint_json_falls_back_to_the_defaults_with_a_notice() {
    let missing = file("Missing", "Paint.json");
    let mut notices = Vec::new();
    assert_eq!(editor::load(&missing, &mut notices), Parameters::default());
    assert!(notices.is_empty(), "a missing file is fine: {notices:?}");
    for (text, problem) in [
        ("{ \"Shift\": }", "line 1, column 12: expected a value"),
        ("{ \"Shift\": 1.5 }", "SimplePaint Shift must be finite and in [0, 0.9990234375]."),
        ("{ \"Glow\": 1 }", "Glow isn't a SimplePaint setting."),
    ] {
        let path = file("Bad", "Paint.json");
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
fn any_input_leaves_a_valid_material() {
    let mut demo = Demo::new(Parameters::default(), nowhere());
    // A fixed sequence of presses, drags, wheels and keys all over the sliders.
    let mut seed = 0x5EED_u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as f32 / (1u64 << 31) as f32
    };
    for _ in 0..2000 {
        let name = SLIDERS[(next() * 8.0) as usize % 8].name;
        let point = demo.along(name, next() * 1.4 - 0.2);
        let event = match (next() * 5.0) as u32 {
            0 => Event::Press(point),
            1 => Event::Release(point),
            2 => Event::Wheel(point, next() * 20.0 - 10.0),
            3 => key([VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN][(next() * 4.0) as usize % 4]),
            _ => Event::Key {
                key: [VK_LEFT, VK_RIGHT][(next() * 2.0) as usize % 2] as u32,
                control: false,
                shift: true,
                repeat: next() < 0.5,
            },
        };
        demo.headless.move_to(point);
        // The editor compiles every change, and would panic on an invalid one.
        demo.step(&[event]);
        Material::compile(&demo.editor.parameters).unwrap();
    }
}

#[test]
fn the_cards_fit_both_window_heights() {
    let names = |demo: &Demo| demo.headless.interface.target_names();
    let maximized = Demo::new(Parameters::default(), nowhere());
    for size in [MAXIMIZED, FULLSCREEN] {
        let mut demo = Demo::sized(Parameters::default(), nowhere(), size);
        let (layout, _) = demo.step(&[]);
        let [x, y] = layout.image.unwrap();
        assert!(y + IMAGE_SIZE <= size[1] as u32 - 40, "the sphere's image runs into the status bar at {size:?}");
        assert!(x + IMAGE_SIZE <= 1200, "the sphere's image runs into the editor");
        // The sliders, then the two buttons, all in the editor's column.
        let mut expected: Vec<String> = SLIDERS.iter().map(|slider| slider.name.to_string()).collect();
        expected.extend(["Reset".to_string(), "Save".to_string()]);
        assert_eq!(names(&demo), expected);
        for name in names(&demo) {
            let area = demo.target(&name);
            assert!(area.x >= 1200.0 && area.right() <= size[0], "{name}: {area:?}");
            // The layout doesn't depend on the height above the status bar.
            assert_eq!(area, maximized.target(&name));
        }
    }
    // Every card's text fits, with the tone curve's labels all on one tone,
    // where each takes a line of its own.
    let crowded = Parameters {
        brightness: 0.5,
        dark_point: 0.5,
        light_point: 0.5,
        ..Parameters::default()
    };
    for parameters in [Parameters::default(), crowded] {
        for size in [MAXIMIZED, FULLSCREEN] {
            let (_, dropped) = Demo::sized(parameters, nowhere(), size).step(&[]);
            assert_eq!(dropped, 0, "lines of text that didn't fit at {size:?}");
        }
    }
}

#[test]
fn the_tone_curve_s_labels_go_on_lines_where_they_don_t_overlap() {
    let gap = 16.0;
    // Apart, they share the first line.
    assert_eq!(panels::label_lines(&[[0.0, 100.0], [200.0, 300.0], [400.0, 500.0]], gap), [0, 0, 0]);
    // At one tone, each gets a line.
    assert_eq!(panels::label_lines(&[[0.0, 100.0], [4.0, 96.0], [2.0, 98.0]], gap), [0, 1, 2]);
    // Too close to the first, the second goes on the next line, and the
    // third on the first line where it's clear of the others.
    assert_eq!(panels::label_lines(&[[0.0, 100.0], [110.0, 210.0], [220.0, 320.0]], gap), [0, 1, 0]);
    assert_eq!(panels::label_lines(&[[0.0, 100.0], [110.0, 210.0], [90.0, 190.0]], gap), [0, 1, 2]);
}
