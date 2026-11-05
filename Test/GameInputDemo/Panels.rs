// What the mouse does to GameInputDemo's cards: its combo box for which
// gamepad to listen to, what its list offers, and the Devices card's rows
// and their tooltips, through the cards' real layout, which
// Library/UserInterface's Headless draws without a window or a GPU.

use application::Event;
use game_input_demo::demo::Demo;
use game_input_demo::device_tree::{Node, Tree};
use game_input_demo::input::{Input, Listen};
use game_input_demo::panels::{DEVICE_ROWS, GAMEPAD_SELECT};
use game_input_demo::scripted_source::ScriptedSource;
use json::Value;
use user_interface::headless::{FULLSCREEN, Headless, MAXIMIZED, click, key};
use user_interface::{CARD_PADDING, MARGIN, PANELS_TOP, Program, Rect};
use windows_api::*;

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

// The demo with this input, drawn once in a window of this size, so that the
// next frame's input has targets.
fn start(input: Result<Input, String>, size: [f32; 2]) -> (Demo, Headless) {
    let mut demo = Demo { input };
    let mut headless = Headless::new(size);
    headless.program(&mut demo, &[]).unwrap();
    (demo, headless)
}

fn step(demo: &mut Demo, headless: &mut Headless, events: &[Event]) {
    headless.program(demo, events).unwrap();
}

// Does what a script's "Program" step says, for the next frame.
fn program_step(demo: &mut Demo, step: &str) {
    let input = demo.input.as_mut().unwrap();
    input.scripted().unwrap().apply(&json::parse(step).unwrap()).unwrap();
}

fn listen(demo: &Demo) -> &Listen {
    &demo.input.as_ref().unwrap().listen
}

// The names of the last frame's targets that start with `prefix`.
fn names(headless: &Headless, prefix: &str) -> Vec<String> {
    let mut names = headless.interface.target_names();
    names.retain(|name| name.starts_with(prefix));
    names
}

// What the last frame's target of this name shows, as the interface's state
// says, like a choice's label.
fn label(headless: &Headless, name: &str) -> String {
    let Some(Value::Array(targets)) = headless.interface.state().get("Targets").cloned() else {
        panic!("no targets");
    };
    let target = targets
        .iter()
        .find(|target| target.get("Name") == Some(&Value::from(name)))
        .unwrap_or_else(|| panic!("no {name}"));
    match target.get("Label") {
        Some(Value::String(label)) => label.clone(),
        _ => panic!("{name} has no label"),
    }
}

// The labels of the combo box's choices, in the list's order.
fn choices(headless: &Headless) -> Vec<String> {
    let prefix = format!("{GAMEPAD_SELECT}/");
    names(headless, &prefix)
        .into_iter()
        .filter(|name| name[prefix.len()..].parse::<usize>().is_ok())
        .map(|name| label(headless, &name))
        .collect()
}

// Opens the combo box's list.
fn open(demo: &mut Demo, headless: &mut Headless) {
    let select = headless.center(GAMEPAD_SELECT);
    step(demo, headless, &click(select));
    assert_eq!(headless.interface.popup().as_deref(), Some(GAMEPAD_SELECT));
}

// The Devices card's rows, in its order.
fn rows(headless: &Headless) -> Vec<Rect> {
    names(headless, &format!("{DEVICE_ROWS}/")).iter().map(|name| headless.target(name)).collect()
}

// The device whose tooltip shows with the cursor here, and the tooltip's
// area.
fn hover(demo: &mut Demo, headless: &mut Headless, cursor: [f32; 2]) -> Option<(String, Rect)> {
    headless.move_to(cursor);
    step(demo, headless, &[]);
    headless.interface.tooltip()
}

#[test]
fn the_closed_combo_box_is_the_only_widget_but_the_device_rows() {
    let (_, headless) = start(Ok(input(&["Pad One", "Pad Two"])), MAXIMIZED);
    assert_eq!(headless.interface.target_names(), [GAMEPAD_SELECT, "Devices/0", "Devices/1"]);
    assert_eq!(label(&headless, GAMEPAD_SELECT), "Pad One");
    // After the card's title, around its line.
    let select = headless.target(GAMEPAD_SELECT);
    let title_y = PANELS_TOP + CARD_PADDING;
    assert!(select.x > MARGIN + CARD_PADDING, "{select:?}");
    assert!(select.y < title_y && select.bottom() > title_y + 20.0, "{select:?}");
}

#[test]
fn the_open_list_offers_each_gamepad_and_none_under_the_box() {
    let (mut demo, mut headless) = start(Ok(input(&["Pad One", "Pad Two"])), MAXIMIZED);
    open(&mut demo, &mut headless);
    assert_eq!(choices(&headless), ["Pad One", "Pad Two", "None"]);
    // Rows of one size, one under the other, from under the box.
    let select = headless.target(GAMEPAD_SELECT);
    let rows: Vec<Rect> = (0..3).map(|index| headless.target(&format!("{GAMEPAD_SELECT}/{index}"))).collect();
    assert!(rows[0].y > select.bottom());
    for pair in rows.windows(2) {
        assert_eq!(pair[1].y, pair[0].bottom());
        assert_eq!((pair[1].x, pair[1].width), (pair[0].x, pair[0].width));
    }
    // Inside the list's padding.
    assert_eq!(rows[0].x, select.x + 4.0);

    // A press on a choice picks it, and closes the list.
    step(&mut demo, &mut headless, &click(rows[1].center()));
    assert_eq!(listen(&demo).label(), "Pad Two");
    assert!(demo.input.as_ref().unwrap().listen_picked);
    assert!(headless.interface.popup().is_none());
    // The box shows it from the next frame on: its target's label is what
    // it showed before the frame's input.
    step(&mut demo, &mut headless, &[]);
    assert_eq!(label(&headless, GAMEPAD_SELECT), "Pad Two");
    open(&mut demo, &mut headless);
    let none = headless.center(&format!("{GAMEPAD_SELECT}/2"));
    step(&mut demo, &mut headless, &click(none));
    assert_eq!(*listen(&demo), Listen::NoGamepad);
}

#[test]
fn a_picked_gamepad_stays_in_the_list_while_it_is_disconnected() {
    let (mut demo, mut headless) = start(Ok(input(&["Pad One", "Pad Two"])), MAXIMIZED);
    open(&mut demo, &mut headless);
    let pad_two = headless.center(&format!("{GAMEPAD_SELECT}/1"));
    step(&mut demo, &mut headless, &click(pad_two));
    program_step(&mut demo, r#"{ "Disconnect": 2 }"#);
    step(&mut demo, &mut headless, &[]);
    assert_eq!(listen(&demo).label(), "Pad Two");
    open(&mut demo, &mut headless);
    assert_eq!(choices(&headless), ["Pad One", "Pad Two", "None"]);
}

#[test]
fn without_gamepads_the_list_offers_none() {
    let (mut demo, mut headless) = start(Ok(input(&[])), MAXIMIZED);
    assert_eq!(label(&headless, GAMEPAD_SELECT), "None");
    open(&mut demo, &mut headless);
    assert_eq!(choices(&headless), ["None"]);
}

#[test]
fn esc_or_a_press_off_the_list_closes_it_and_keys_do_not_change_the_gamepad() {
    let (mut demo, mut headless) = start(Ok(input(&["Pad One", "Pad Two"])), MAXIMIZED);
    open(&mut demo, &mut headless);
    step(&mut demo, &mut headless, &[key(VK_ESCAPE)]);
    assert!(headless.interface.popup().is_none());
    open(&mut demo, &mut headless);
    step(&mut demo, &mut headless, &click([5.0, 5.0]));
    assert!(headless.interface.popup().is_none());
    assert_eq!(listen(&demo).label(), "Pad One");

    // Neither a press on the box nor one on a choice gives it keyboard
    // focus, so once the list is closed, the arrow keys, which the user
    // presses to try the keyboard panel, leave the gamepad alone.
    open(&mut demo, &mut headless);
    let pad_one = headless.center(&format!("{GAMEPAD_SELECT}/0"));
    step(&mut demo, &mut headless, &click(pad_one));
    for arrow in [VK_DOWN, VK_UP, VK_END, VK_TAB, VK_DOWN] {
        step(&mut demo, &mut headless, &[key(arrow)]);
        assert_eq!(headless.interface.focus(), None);
    }
    assert_eq!(listen(&demo).label(), "Pad One");
}

#[test]
fn while_the_list_is_open_the_arrow_keys_pick_another_choice() {
    let (mut demo, mut headless) = start(Ok(input(&["Pad One", "Pad Two"])), MAXIMIZED);
    open(&mut demo, &mut headless);
    step(&mut demo, &mut headless, &[key(VK_DOWN)]);
    assert_eq!(listen(&demo).label(), "Pad Two");
    step(&mut demo, &mut headless, &[key(VK_END)]);
    assert_eq!(*listen(&demo), Listen::NoGamepad);
    assert_eq!(headless.interface.popup().as_deref(), Some(GAMEPAD_SELECT));
    step(&mut demo, &mut headless, &[key(VK_RETURN)]);
    assert!(headless.interface.popup().is_none());
}

#[test]
fn nothing_takes_input_without_game_input_or_in_an_unsupported_window() {
    let (_, headless) = start(Err("GameInput isn't installed".to_string()), MAXIMIZED);
    assert!(headless.interface.target_names().is_empty());

    // The main loop shows what's wrong instead of the cards.
    let (mut demo, mut headless) = start(Ok(input(&["Pad One"])), MAXIMIZED);
    let problems = ["The window is too small".to_string()];
    headless.frame(&[], |frame| {
        demo.frame(frame).unwrap();
        frame.show_window_problems("", &problems);
    });
    assert!(headless.interface.target_names().is_empty());
}

#[test]
fn f11_switches_fullscreen_only_through_game_input() {
    let (mut demo, mut headless) = start(Ok(input(&[])), MAXIMIZED);
    program_step(&mut demo, r#"{ "Connect": { "Id": 9, "Name": "Keys", "Input": ["Keyboard"] } }"#);
    step(&mut demo, &mut headless, &[]);
    // The window's own F11 is the same key, which GameInput reads too.
    let outcome = headless.program(&mut demo, &[key(VK_F11)]).unwrap();
    assert!(!outcome.fullscreen);
    program_step(&mut demo, r#"{ "Keyboard": { "Id": 9, "Down": ["F11"] } }"#);
    let outcome = headless.program(&mut demo, &[]).unwrap();
    assert!(outcome.fullscreen);
}

#[test]
fn each_device_has_a_row_under_the_one_before() {
    let (_, headless) = start(Ok(input(&["Pad One", "Pad Two", "Pad Three"])), MAXIMIZED);
    let rows = rows(&headless);
    assert_eq!(rows.len(), 3);
    for pair in rows.windows(2) {
        assert_eq!((pair[1].x, pair[1].width), (pair[0].x, pair[0].width));
        assert_eq!(pair[1].y, pair[0].bottom());
    }
}

#[test]
fn the_cursor_on_a_row_shows_its_devices_tooltip() {
    let (mut demo, mut headless) = start(Ok(input(&["Pad One", "Pad Two"])), MAXIMIZED);
    let rows = rows(&headless);
    for (row, name) in rows.iter().zip(["Pad One", "Pad Two"]) {
        let tooltip = hover(&mut demo, &mut headless, row.center());
        assert_eq!(tooltip.map(|(device, _)| device).as_deref(), Some(name));
    }
    // Not under the last row, or without the cursor in the window.
    let last = rows[1];
    assert!(hover(&mut demo, &mut headless, [last.center()[0], last.bottom() + last.height * 0.5]).is_none());
    headless.cursor = None;
    step(&mut demo, &mut headless, &[]);
    assert!(headless.interface.tooltip().is_none());
}

// A device with as much in its tooltip as a real one, with five nodes in
// Windows' device tree.
fn real_input() -> Input {
    input_with_tree(5)
}

// The second device has `nodes` nodes in Windows' device tree.
fn input_with_tree(nodes: usize) -> Input {
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
    input.devices[1].tree = Tree::known(Ok(vec![node; nodes]));
    input
}

// Where the cursor goes on a row: at both of its ends.
fn ends(row: Rect) -> [[f32; 2]; 2] {
    let middle = row.center()[1];
    [[row.x + 1.0, middle], [row.right() - 1.0, middle]]
}

fn inside(area: Rect, [width, height]: [f32; 2]) -> bool {
    area.x >= MARGIN && area.right() <= width - MARGIN && area.y >= MARGIN && area.bottom() <= height - MARGIN
}

#[test]
fn a_tooltip_stays_in_the_window_off_the_cursor() {
    for size in [MAXIMIZED, FULLSCREEN] {
        let (mut demo, mut headless) = start(Ok(real_input()), size);
        for row in rows(&headless) {
            for cursor in ends(row) {
                let (_, area) = hover(&mut demo, &mut headless, cursor).unwrap();
                assert!(inside(area, size), "{cursor:?}: {area:?}");
                assert!(cursor[1] < area.y || cursor[1] > area.bottom(), "{cursor:?}: {area:?}");
            }
        }
    }
}

#[test]
fn a_tooltip_that_fits_neither_under_nor_above_goes_beside_the_row() {
    // With 9 nodes in the device tree, it's taller than the room either way.
    for size in [MAXIMIZED, FULLSCREEN] {
        let (mut demo, mut headless) = start(Ok(input_with_tree(9)), size);
        let row = rows(&headless)[1];
        for cursor in ends(row) {
            let (_, area) = hover(&mut demo, &mut headless, cursor).unwrap();
            assert!(inside(area, size), "{cursor:?}: {area:?}");
            // Beside the row, so that it covers neither the row nor the cursor.
            assert!(area.y < cursor[1] && cursor[1] < area.bottom(), "{cursor:?}: {area:?}");
            assert!(area.x >= row.right() || area.right() <= row.x, "{cursor:?}: {area:?}");
        }
    }
}

#[test]
fn a_tall_tooltip_goes_above_the_cursor_when_it_does_not_fit_under() {
    let (mut demo, mut headless) = start(Ok(real_input()), MAXIMIZED);
    let rows = rows(&headless);
    let cursor = rows[1].center();
    let (_, area) = hover(&mut demo, &mut headless, cursor).unwrap();
    assert!(area.bottom() < cursor[1]);
    // A short one fits under it.
    let cursor = rows[0].center();
    let (_, area) = hover(&mut demo, &mut headless, cursor).unwrap();
    assert!(area.y > cursor[1]);
}

#[test]
fn the_cards_fit_both_window_heights() {
    for size in [MAXIMIZED, FULLSCREEN] {
        let (mut demo, mut headless) = start(Ok(real_input()), size);
        let outcome = headless.program(&mut demo, &[]).unwrap();
        assert_eq!(outcome.dropped_lines, 0, "at {size:?}");
        for name in headless.interface.target_names() {
            let area = headless.target(&name);
            assert!(area.x >= 0.0 && area.right() <= size[0] && area.bottom() <= size[1], "{name}: {area:?}");
        }
    }
}
