// Where GameInputDemo's select box for which gamepad to listen to is, and
// what its list offers, through the panels' real layout, which a canvas
// builds without a window or a GPU.

use std::sync::OnceLock;

use application::FrameCounter;
use canvas::{Canvas, Font};
use game_input_demo::FONT_SIZE;
use game_input_demo::input::{Input, Listen};
use game_input_demo::panels::{self, Click, Status, Targets};
use game_input_demo::scripted_source::ScriptedSource;
use json::Value;

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
        cursor: None,
        picker_open,
    };
    panels::draw(&mut canvas, input, &status, 2560.0, 1369.0)
}

// Each target's name and area, as scripted runs save them.
fn areas(targets: &Targets) -> Vec<(String, [f32; 4])> {
    let Value::Array(targets) = targets.state() else {
        panic!("targets aren't a list");
    };
    targets
        .iter()
        .map(|target| {
            let Some(Value::String(name)) = target.get("Name") else {
                panic!("a target has no name");
            };
            let Some(Value::Array(area)) = target.get("Area") else {
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
        })
        .collect()
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
