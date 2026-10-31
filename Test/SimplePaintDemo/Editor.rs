// What input does to SimplePaintDemo's editor, through the panels' real
// layout, which a canvas builds without a window or a GPU, and where the
// panels go at both supported window heights.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use application::{Event, FrameCounter};
use canvas::{Canvas, Font};
use simple_paint::{Material, Parameters};
use simple_paint_demo::editor::{self, Action, Click, Editor, SLIDERS, Target};
use simple_paint_demo::panels::{self, Layout, Status};
use simple_paint_demo::preview::IMAGE_SIZE;
use simple_paint_demo::FONT_SIZE;

const WIDTH: f32 = 2560.0;
const MAXIMIZED_HEIGHT: f32 = 1369.0;
const FULLSCREEN_HEIGHT: f32 = 1440.0;

fn font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| Font::new(FONT_SIZE).unwrap())
}

// Draws a frame of the editor, and returns where things went.
fn layout(editor: &Editor, height: f32) -> Layout {
    let frames = FrameCounter::default();
    let mut vertices = Vec::new();
    let mut canvas = Canvas::new(font(), &mut vertices);
    let status = Status {
        focused: true,
        frames: &frames,
        display: "TEST01 (Test display)",
        adapter: "Test GPU",
        notices: &[],
        supported: "",
        window_problems: &[],
        cursor: None,
    };
    panels::draw(&mut canvas, editor, &status, WIDTH, height)
}

fn targets(editor: &Editor) -> Vec<Target> {
    layout(editor, MAXIMIZED_HEIGHT).targets
}

// The slider's target, by its name.
fn slider(targets: &[Target], name: &str) -> Target {
    *targets
        .iter()
        .find(|target| matches!(target.click, Click::Slider(index) if SLIDERS[index].name == name))
        .unwrap_or_else(|| panic!("no slider {name}"))
}

// The point at `fraction` along a slider's track.
fn along(target: &Target, fraction: f32) -> [f32; 2] {
    let [left, right] = target.track;
    [left + (right - left) * fraction, target.area.y + target.area.height * 0.5]
}

fn key(key: u32) -> Event {
    Event::Key {
        key,
        control: false,
        shift: false,
        repeat: false,
    }
}

// The same key, as Windows repeats it while it's held.
fn repeated(key: u32) -> Event {
    Event::Key {
        key,
        control: false,
        shift: false,
        repeat: true,
    }
}

const LEFT: u32 = 0x25;
const UP: u32 = 0x26;
const RIGHT: u32 = 0x27;
const DOWN: u32 = 0x28;
const F11: u32 = 0x7A;

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
fn a_click_on_a_slider_sets_it_and_drags_it() {
    let mut editor = Editor::new(Parameters::default());
    let targets = targets(&editor);
    let r = slider(&targets, "R");
    editor.handle(&Event::Press(along(&r, 0.5)), &targets, &nowhere());
    assert_eq!(editor.parameters.base_color_srgb[0], 0.5);
    assert_eq!((editor.selected, editor.dragging), (0, Some(0)));
    // Dragging follows the cursor along the track, and holds the slider
    // even when the cursor leaves the row.
    editor.follow(Some([along(&r, 0.25)[0], 900.0]), &targets);
    assert_eq!(editor.parameters.base_color_srgb[0], 0.25);
    editor.handle(&Event::Release(along(&r, 0.25)), &targets, &nowhere());
    assert_eq!(editor.dragging, None);
    // After the release, moving the cursor changes nothing.
    editor.follow(Some(along(&r, 0.75)), &targets);
    assert_eq!(editor.parameters.base_color_srgb[0], 0.25);
}

#[test]
fn sliders_stop_at_the_last_step_simple_paint_accepts() {
    let mut editor = Editor::new(Parameters::default());
    let targets = targets(&editor);
    for (name, low, high) in [
        ("R", 0.001, 0.999),
        ("Brightness", 0.001, 0.999),
        ("Shift", 0.0, 0.999),
        ("Rotation", 0.0, 359.9),
        ("Dark Point", 0.0, 0.999),
        ("Light Point", 0.001, 1.0),
    ] {
        let target = slider(&targets, name);
        let index = SLIDERS.iter().position(|slider| slider.name == name).unwrap();
        for (x, expected) in [(-500.0f32, low), (5000.0, high)] {
            // Pressed on the track, then dragged far past its end.
            editor.handle(&Event::Press(along(&target, 0.5)), &targets, &nowhere());
            editor.follow(Some([x, 0.0]), &targets);
            editor.handle(&Event::Release([x, 0.0]), &targets, &nowhere());
            assert_eq!(SLIDERS[index].value(&editor.parameters), expected, "{name}");
        }
    }
    Material::compile(&editor.parameters).unwrap();
}

#[test]
fn losing_focus_ends_a_drag() {
    let mut editor = Editor::new(Parameters::default());
    let targets = targets(&editor);
    let shift = slider(&targets, "Shift");
    editor.handle(&Event::Press(along(&shift, 0.5)), &targets, &nowhere());
    // Without focus, a frame has no cursor, and the release never comes.
    editor.follow(None, &targets);
    assert_eq!(editor.dragging, None);
    editor.follow(Some(along(&shift, 0.9)), &targets);
    assert_eq!(editor.parameters.shift, 0.5);
}

#[test]
fn the_wheel_moves_a_slider_by_its_smallest_step() {
    let mut editor = Editor::new(Parameters::default());
    let targets = targets(&editor);
    let rotation = slider(&targets, "Rotation");
    editor.handle(&Event::Wheel(along(&rotation, 0.0), 3.0), &targets, &nowhere());
    assert_eq!(editor.parameters.rotation_degrees, 0.3);
    assert_eq!(SLIDERS[editor.selected].name, "Rotation");
    editor.handle(&Event::Wheel(along(&rotation, 0.0), -5.0), &targets, &nowhere());
    assert_eq!(editor.parameters.rotation_degrees, 0.0);
    // Off the sliders, the wheel does nothing.
    editor.handle(&Event::Wheel([100.0, 100.0], 1.0), &targets, &nowhere());
    assert_eq!(editor.parameters, Parameters { rotation_degrees: 0.0, ..Parameters::default() });
}

#[test]
fn parts_of_a_notch_add_up_over_the_same_slider() {
    // Like a touchpad, which turns the wheel by a quarter of a notch at a time.
    let mut editor = Editor::new(Parameters::default());
    let targets = targets(&editor);
    let rotation = along(&slider(&targets, "Rotation"), 0.0);
    for _ in 0..3 {
        editor.handle(&Event::Wheel(rotation, 0.25), &targets, &nowhere());
    }
    assert_eq!(editor.parameters.rotation_degrees, 0.0);
    editor.handle(&Event::Wheel(rotation, 0.25), &targets, &nowhere());
    assert_eq!(editor.parameters.rotation_degrees, 0.1);
    for _ in 0..6 {
        editor.handle(&Event::Wheel(rotation, -0.5), &targets, &nowhere());
        editor.handle(&Event::Wheel(rotation, 1.0), &targets, &nowhere());
    }
    assert_eq!(editor.parameters.rotation_degrees, 0.4);
    // What's left over one slider doesn't carry over to another.
    editor.handle(&Event::Wheel(rotation, 0.5), &targets, &nowhere());
    assert_eq!(editor.parameters.rotation_degrees, 0.4);
    let shift = along(&slider(&targets, "Shift"), 0.0);
    editor.handle(&Event::Wheel(shift, 0.5), &targets, &nowhere());
    editor.handle(&Event::Wheel(shift, 0.25), &targets, &nowhere());
    assert_eq!(editor.parameters.shift, 0.0);
}

#[test]
fn arrow_keys_select_and_move_a_slider() {
    let mut editor = Editor::new(Parameters::default());
    let targets = targets(&editor);
    editor.handle(&key(UP), &targets, &nowhere());
    assert_eq!(editor.selected, 0, "Up stops at the first slider");
    editor.handle(&key(RIGHT), &targets, &nowhere());
    // Values on a step are the double nearest their decimal, so they save
    // as written.
    assert_eq!(editor.parameters.base_color_srgb[0], 0.108);
    editor.handle(
        &Event::Key {
            key: RIGHT,
            control: false,
            shift: true,
            repeat: false,
        },
        &targets,
        &nowhere(),
    );
    assert_eq!(editor.parameters.base_color_srgb[0], 0.118);
    editor.handle(&key(LEFT), &targets, &nowhere());
    assert_eq!(editor.parameters.base_color_srgb[0], 0.117);
    for _ in 0..20 {
        editor.handle(&key(DOWN), &targets, &nowhere());
    }
    assert_eq!(SLIDERS[editor.selected].name, "Light Point", "Down stops at the last slider");
    editor.handle(&key(RIGHT), &targets, &nowhere());
    assert_eq!(editor.parameters.light_point, 1.0);
    assert!(matches!(editor.handle(&key(F11), &targets, &nowhere()), Action::ToggleFullscreen));
}

#[test]
fn holding_an_arrow_repeats_it_but_holding_f11_or_ctrl_s_does_not() {
    let path = file("Repeat", "Paint.json");
    let mut editor = Editor::new(Parameters::default());
    let targets = targets(&editor);
    editor.handle(&key(RIGHT), &targets, &path);
    editor.handle(&repeated(RIGHT), &targets, &path);
    assert_eq!(editor.parameters.base_color_srgb[0], 0.109);
    assert!(matches!(editor.handle(&repeated(F11), &targets, &path), Action::None));
    let held_save = Event::Key {
        key: 'S' as u32,
        control: true,
        shift: false,
        repeat: true,
    };
    editor.handle(&held_save, &targets, &path);
    assert_eq!(editor.status, None);
    assert!(!path.exists());
}

#[test]
fn ctrl_s_and_save_write_paint_json_and_it_loads_back() {
    let path = file("Save", "Paint.json");
    let mut editor = Editor::new(Parameters::default());
    let targets = targets(&editor);
    editor.handle(&Event::Press(along(&slider(&targets, "Shift"), 0.6)), &targets, &path);
    editor.handle(&Event::Release([0.0, 0.0]), &targets, &path);
    let save = Event::Key {
        key: 'S' as u32,
        control: true,
        shift: false,
        repeat: false,
    };
    editor.handle(&save, &targets, &path);
    assert_eq!(editor.status, Some((format!("Saved to {}", path.display()), false)));
    let mut notices = Vec::new();
    assert_eq!(editor::load(&path, &mut notices), editor.parameters);
    assert!(notices.is_empty(), "{notices:?}");

    // The Save button does the same, and says when it couldn't.
    let button = targets.iter().find(|target| matches!(target.click, Click::Save)).unwrap();
    let center = [button.area.x + 5.0, button.area.y + 5.0];
    editor.handle(&Event::Press(center), &targets, &nowhere());
    assert!(matches!(&editor.status, Some((message, true)) if message.starts_with(r"Couldn't save to C:\Nowhere\Paint.json")));
}

#[test]
fn reset_goes_back_to_simple_paint_s_defaults() {
    let mut editor = Editor::new(Parameters {
        shift: 0.5,
        ..Parameters::default()
    });
    editor.selected = 3;
    let targets = targets(&editor);
    let button = targets.iter().find(|target| matches!(target.click, Click::Reset)).unwrap();
    editor.handle(&Event::Press([button.area.x + 5.0, button.area.y + 5.0]), &targets, &nowhere());
    assert_eq!(editor.parameters, Parameters::default());
    assert_eq!(editor.selected, 3, "the selection stays");
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
    let mut editor = Editor::new(Parameters::default());
    let targets = targets(&editor);
    // A fixed sequence of presses, drags, wheels and keys all over the sliders.
    let mut seed = 0x5EED_u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as f32 / (1u64 << 31) as f32
    };
    for _ in 0..2000 {
        let target = slider(&targets, SLIDERS[(next() * 8.0) as usize % 8].name);
        let point = along(&target, next() * 1.4 - 0.2);
        let event = match (next() * 5.0) as u32 {
            0 => Event::Press(point),
            1 => Event::Release(point),
            2 => Event::Wheel(point, next() * 20.0 - 10.0),
            3 => key([LEFT, RIGHT, UP, DOWN][(next() * 4.0) as usize % 4]),
            _ => Event::Key {
                key: [LEFT, RIGHT][(next() * 2.0) as usize % 2],
                control: false,
                shift: true,
                repeat: next() < 0.5,
            },
        };
        editor.handle(&event, &targets, &nowhere());
        editor.follow(Some(point), &targets);
        // The editor compiles every change, and would panic on an invalid one.
        Material::compile(&editor.parameters).unwrap();
    }
}

#[test]
fn the_panels_fit_both_window_heights() {
    let editor = Editor::new(Parameters::default());
    for height in [MAXIMIZED_HEIGHT, FULLSCREEN_HEIGHT] {
        let layout = layout(&editor, height);
        let [x, y] = layout.image.unwrap();
        assert!(y + IMAGE_SIZE <= height as u32 - 40, "the sphere's image runs into the footer at {height}");
        assert!(x + IMAGE_SIZE <= 1200, "the sphere's image runs into the editor");
        // The sliders, then the two buttons, all in the editor's column.
        assert_eq!(layout.targets.len(), SLIDERS.len() + 2);
        for target in &layout.targets {
            assert!(target.area.x >= 1200.0 && target.area.right() <= WIDTH, "{:?}", target.area.x);
        }
        // The layout doesn't depend on the height above the footer.
        assert_eq!(layout.targets[0].area.y, targets(&editor)[0].area.y);
    }
}
