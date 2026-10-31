// Where GameInputDemo's select box for which gamepad to listen to is, and
// what its list offers, and the Devices panel's rows and their tooltips,
// through the panels' real layout, which a canvas builds without a window
// or a GPU.

use std::sync::OnceLock;

use application::FrameCounter;
use canvas::{Canvas, Font, MARGIN};
use game_input_demo::FONT_SIZE;
use game_input_demo::device_tree::{Node, Tree};
use game_input_demo::input::{Input, Listen};
use game_input_demo::panels::{self, Click, Status, Targets};
use game_input_demo::scripted_source::ScriptedSource;
use json::Value;

// The client areas' heights when maximized and in borderless fullscreen.
const HEIGHTS: [f32; 2] = [1369.0, 1440.0];

fn font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| Font::new(FONT_SIZE).unwrap())
}

// Input from gamepads with these names.
fn input(gamepads: &[&str]) -> Input {
    let mut input = Input::new(Box::new(ScriptedSource::default()), 0.1);
    for (index, name) in gamepads.iter().enumerate() {
        let step = json::object([(
            "Connect",
            json::object([
                ("Id", (index + 1).into()),
                ("Name", (*name).into()),
                ("Input", ["Gamepad"].into()),
            ]),
        )]);
        input.scripted().unwrap().apply(&step).unwrap();
    }
    input.update(true);
    input
}

// Draws a frame, and returns where clicks do something.
fn targets(input: &Result<Input, String>, picker_open: bool, window_problems: &[String]) -> Targets {
    frame(input, picker_open, window_problems, None, HEIGHTS[0])
}

// Draws a frame with the cursor at a place, in a window this tall.
fn hover(input: &Result<Input, String>, cursor: [f32; 2], height: f32) -> Targets {
    frame(input, false, &[], Some(cursor), height)
}

fn frame(
    input: &Result<Input, String>,
    picker_open: bool,
    window_problems: &[String],
    cursor: Option<[f32; 2]>,
    height: f32,
) -> Targets {
    let frames = FrameCounter::default();
    let mut vertices = Vec::new();
    let mut canvas = Canvas::new(font(), &mut vertices);
    let status = Status {
        focused: true,
        frames: &frames,
        display: "TEST01 (Test display)",
        notices: &[],
        supported: "",
        window_problems,
        cursor,
        picker_open,
    };
    panels::draw(&mut canvas, input, &status, 2560.0, height)
}

// A name and an area, as scripted runs save them.
fn named_area(value: &Value) -> (String, [f32; 4]) {
    let Some(Value::String(name)) = value.get("Name") else {
        panic!("{} has no name", value.to_json());
    };
    let Some(Value::Array(area)) = value.get("Area") else {
        panic!("{name} has no area");
    };
    let area: Vec<f32> = area
        .iter()
        .map(|number| match number {
            Value::Number(number) => *number as f32,
            _ => panic!("{name}'s area isn't numbers"),
        })
        .collect();
    (name.clone(), area.try_into().unwrap())
}

fn named_areas(list: Value) -> Vec<(String, [f32; 4])> {
    let Value::Array(items) = list else {
        panic!("{} isn't a list", list.to_json());
    };
    items.iter().map(named_area).collect()
}

// Each target's name and area.
fn areas(targets: &Targets) -> Vec<(String, [f32; 4])> {
    named_areas(targets.state())
}

// The device whose tooltip shows, and the tooltip's area.
fn tooltip(targets: &Targets) -> Option<(String, [f32; 4])> {
    match targets.tooltip_state() {
        Value::Null => None,
        tooltip => Some(named_area(&tooltip)),
    }
}

fn names(targets: &Targets) -> Vec<String> {
    areas(targets).into_iter().map(|(name, _)| name).collect()
}

fn center([x, y, width, height]: [f32; 4]) -> [f32; 2] {
    [x + width * 0.5, y + height * 0.5]
}

#[test]
fn the_closed_select_box_is_the_only_target() {
    let input = Ok(input(&["Pad One", "Pad Two"]));
    let targets = targets(&input, false, &[]);
    assert_eq!(names(&targets), ["Select box"]);
    let (_, select_box) = &areas(&targets)[0];
    assert!(matches!(targets.at(center(*select_box)), Some(Click::Picker)));
    assert!(targets.at([5.0, 5.0]).is_none());
}

#[test]
fn the_open_list_offers_each_gamepad_and_none_under_the_box() {
    let input = Ok(input(&["Pad One", "Pad Two"]));
    let targets = targets(&input, true, &[]);
    let areas = areas(&targets);
    assert_eq!(names(&targets), ["Select box", "Choose Pad One", "Choose Pad Two", "Choose None"]);
    // Rows of one size, one under the other, from under the box.
    let [x, y, _, height] = areas[0].1;
    let rows: Vec<[f32; 4]> = areas[1..].iter().map(|(_, area)| *area).collect();
    assert!(rows[0][1] > y + height);
    for pair in rows.windows(2) {
        assert_eq!(pair[1][1], pair[0][1] + pair[0][3]);
        assert_eq!((pair[1][0], pair[1][2]), (pair[0][0], pair[0][2]));
    }
    assert_eq!(rows[0][0], x + 1.0);

    match targets.at(center(rows[1])) {
        Some(Click::Choose(listen)) => assert_eq!(listen.label(), "Pad Two"),
        _ => panic!("Pad Two's row doesn't choose it"),
    }
    assert!(matches!(targets.at(center(rows[2])), Some(Click::Choose(Listen::NoGamepad))));
}

#[test]
fn a_picked_gamepad_stays_in_the_list_while_it_is_disconnected() {
    let mut input = input(&["Pad One", "Pad Two"]);
    let pad_two = input.gamepads().find(|device| device.label == "Pad Two").unwrap().clone();
    input.pick_listen(Listen::Gamepad(pad_two));
    input.scripted().unwrap().apply(&json::parse(r#"{ "Disconnect": 2 }"#).unwrap()).unwrap();
    input.update(true);
    let targets = targets(&Ok(input), true, &[]);
    assert_eq!(names(&targets), ["Select box", "Choose Pad One", "Choose Pad Two", "Choose None"]);
}

#[test]
fn without_gamepads_the_list_offers_none() {
    let targets = targets(&Ok(input(&[])), true, &[]);
    assert_eq!(names(&targets), ["Select box", "Choose None"]);
}

#[test]
fn nothing_is_clickable_without_input_or_in_an_unsupported_window() {
    let missing = Err("GameInput isn't installed".to_string());
    assert!(names(&targets(&missing, true, &[])).is_empty());
    let problems = ["The window is too small".to_string()];
    assert!(names(&targets(&Ok(input(&["Pad One"])), true, &problems)).is_empty());
}

#[test]
fn each_device_has_a_row_under_the_one_before() {
    let input = Ok(input(&["Pad One", "Pad Two", "Pad Three"]));
    let rows = named_areas(targets(&input, false, &[]).device_rows_state());
    let names: Vec<&str> = rows.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["Pad One", "Pad Two", "Pad Three"]);
    for pair in rows.windows(2) {
        let ([x, y, width, height], [next_x, next_y, next_width, _]) = (pair[0].1, pair[1].1);
        assert_eq!((next_x, next_width), (x, width));
        assert_eq!(next_y, y + height);
    }
}

#[test]
fn the_cursor_on_a_row_shows_its_devices_tooltip() {
    let input = Ok(input(&["Pad One", "Pad Two"]));
    let rows = named_areas(targets(&input, false, &[]).device_rows_state());
    for (name, row) in &rows {
        let targets = hover(&input, center(*row), HEIGHTS[0]);
        assert_eq!(tooltip(&targets).map(|(device, _)| device), Some(name.clone()));
    }
    // Not under the last row, or without the cursor in the window.
    let [x, y, width, height] = rows[1].1;
    assert!(tooltip(&hover(&input, [x + width * 0.5, y + height * 1.5], HEIGHTS[0])).is_none());
    assert!(tooltip(&targets(&input, false, &[])).is_none());
}

// A device with as much in its tooltip as a real one, with five nodes in
// Windows' device tree.
fn real_input() -> Input {
    let mut input = input(&["Pad One", "Pad Two"]);
    let node = Node {
        instance_id: r"USB\VID_045E&PID_0B12\3032434D30303237303135323037".to_string(),
        name: "Xbox Controller".to_string(),
        bus_name: "Controller".to_string(),
        class: "XboxComposite".to_string(),
        location: "Port_#0003.Hub_#0004".to_string(),
        hardware_ids: vec![r"USB\VID_045E&PID_0B12&REV_0509".to_string(); 6],
        ..Default::default()
    };
    input.devices[1].tree = Tree::known(Ok(vec![node; 5]));
    input
}

#[test]
fn a_tooltip_stays_in_the_window_off_the_cursor() {
    let input = Ok(real_input());
    for height in HEIGHTS {
        let rows = named_areas(frame(&input, false, &[], None, height).device_rows_state());
        for (_, row) in rows {
            // At both ends of the row.
            let [x, y, width, row_height] = row;
            for cursor in [[x + 1.0, y + row_height * 0.5], [x + width - 1.0, y + row_height * 0.5]] {
                let (_, [left, top, box_width, box_height]) =
                    tooltip(&hover(&input, cursor, height)).unwrap();
                assert!(left >= MARGIN && left + box_width <= 2560.0 - MARGIN, "{cursor:?}");
                assert!(top >= MARGIN && top + box_height <= height - MARGIN, "{cursor:?}");
                assert!(cursor[1] < top || cursor[1] > top + box_height, "{cursor:?}");
            }
        }
    }
}

#[test]
fn a_tall_tooltip_goes_above_the_cursor_when_it_does_not_fit_under() {
    let input = Ok(real_input());
    let rows = named_areas(targets(&input, false, &[]).device_rows_state());
    let cursor = center(rows[1].1);
    let (_, [_, top, _, height]) = tooltip(&hover(&input, cursor, HEIGHTS[0])).unwrap();
    assert!(top + height < cursor[1]);
    // A short one fits under it.
    let cursor = center(rows[0].1);
    let (_, [_, top, _, _]) = tooltip(&hover(&input, cursor, HEIGHTS[0])).unwrap();
    assert!(top > cursor[1]);
}
