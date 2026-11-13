// What the mouse and the window do to GameInputDemo's cards: the combo box
// that picks which gamepad to listen to, what its list offers, the Devices
// card's list, its rows and their tooltips, focus and F11, through the
// cards' real layout, which Library/UserInterface's Headless draws without a
// window. And one frame drawn with the GPU, offscreen.

use game_input_demo::cards::{DEVICE_ROWS, GAMEPAD_COLUMN, GAMEPAD_COMBO_BOX, STICK_GRAPH};
use game_input_demo::demo::Demo;
use game_input_demo::device_tree::{Node, Tree};
use game_input_demo::input::{Input, Listen};
use json::Value;
use user_interface::controls::TOOLTIP_OFFSET;
use user_interface::graphics::{AdapterChoice, Gpu, write_png};
use user_interface::tokens::{
    APP_BAR_HEIGHT, BODY, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, FULLSCREEN_HEIGHT, MAXIMIZED_HEIGHT, PAGE_PADDING,
    PERF_BAR_HEIGHT, SPACE_S, STATUS_BAR_HEIGHT, WINDOW_WIDTH,
};
use user_interface::{Headless, Key, Point, Program, Rect, Theme, ThemeKind};

// The demo, drawn once, with gamepads of these names connected, so that the
// next frame's input has targets.
fn demo(gamepads: &[&str]) -> Headless<Demo> {
    let mut demo = Demo::scripted(0.1);
    for (index, name) in gamepads.iter().enumerate() {
        let step = format!(r#"{{ "Connect": {{ "Id": {}, "Name": "{name}", "Input": ["Gamepad"] }} }}"#, index + 1);
        demo.script(&json::parse(&step).unwrap()).unwrap();
    }
    Headless::new(demo)
}

// Does what a script's Program step says, and runs a frame with it.
fn step(app: &mut Headless<Demo>, step: &str) {
    app.program.script(&json::parse(step).unwrap()).unwrap();
    app.frame();
}

fn input(app: &Headless<Demo>) -> &Input {
    app.program.input.as_ref().unwrap()
}

fn listen(app: &Headless<Demo>) -> String {
    input(app).listen.label().to_string()
}

// The names of the last frame's widgets.
fn names(app: &Headless<Demo>) -> Vec<String> {
    app.ui.targets().iter().filter(|target| target.listed).map(|target| target.name.clone()).collect()
}

// The labels of the combo box's choices, in the list's order.
fn choices(app: &Headless<Demo>) -> Vec<String> {
    (0..)
        .map(|index| format!("{GAMEPAD_COMBO_BOX}/{index}"))
        .take_while(|name| app.has_target(name))
        .map(|name| app.label(&name).unwrap())
        .collect()
}

// Opens the combo box's list.
fn open(app: &mut Headless<Demo>) {
    app.click(GAMEPAD_COMBO_BOX);
    assert_eq!(app.ui.popup_name(), Some(GAMEPAD_COMBO_BOX));
}

// The Devices card's rows, in its order.
fn rows(app: &Headless<Demo>) -> Vec<Rect> {
    (0..)
        .map(|index| format!("{DEVICE_ROWS}/{index}"))
        .take_while(|name| app.has_target(name))
        .map(|name| app.target(&name))
        .collect()
}

// The widget whose tooltip shows, and the tooltip's area.
fn tooltip(app: &Headless<Demo>) -> Option<(String, Rect)> {
    let state = app.ui.state();
    let tooltip = state.get("Tooltip")?;
    let Some(Value::String(name)) = tooltip.get("Name") else {
        return None;
    };
    let Some(Value::Array(area)) = tooltip.get("Area") else {
        panic!("a tooltip without an area");
    };
    let number = |index: usize| match area[index] {
        Value::Number(number) => number as f32,
        _ => panic!("an area that isn't numbers"),
    };
    Some((name.clone(), Rect::new(number(0), number(1), number(2), number(3))))
}

// The tooltip once the pointer has rested at the point.
fn rest(app: &mut Headless<Demo>, point: Point) -> Option<(String, Rect)> {
    app.move_point(point).wait(0.6);
    tooltip(app)
}

#[test]
fn the_combo_box_and_the_device_list_are_the_only_widgets() {
    let app = demo(&["Pad One", "Pad Two"]);
    assert_eq!(names(&app), [GAMEPAD_COMBO_BOX, DEVICE_ROWS, "Devices/0", "Devices/1"]);
    assert_eq!(app.label(GAMEPAD_COMBO_BOX).as_deref(), Some("Pad One"));
    // In the Gamepad card's header, after its title.
    let combo_box = app.target(GAMEPAD_COMBO_BOX);
    let header_y = PERF_BAR_HEIGHT + APP_BAR_HEIGHT + PAGE_PADDING + CARD_PADDING;
    assert_eq!(combo_box.y, header_y);
    assert!(combo_box.x > PAGE_PADDING + CARD_PADDING + 40.0, "{combo_box:?}");
    assert!(combo_box.right() < PAGE_PADDING + GAMEPAD_COLUMN, "{combo_box:?}");
    assert_eq!(app.label("Devices/1").as_deref(), Some("Pad Two"));
}

#[test]
fn the_open_list_offers_each_gamepad_and_none() {
    let mut app = demo(&["Pad One", "Pad Two"]);
    open(&mut app);
    assert_eq!(choices(&app), ["Pad One", "Pad Two", "None"]);

    // A click on a choice picks it, and closes the list.
    app.click("Gamepad/1");
    assert_eq!(listen(&app), "Pad Two");
    assert!(input(&app).listen_picked);
    assert_eq!(app.ui.popup_name(), None);
    assert_eq!(app.label(GAMEPAD_COMBO_BOX).as_deref(), Some("Pad Two"));
    open(&mut app);
    app.click("Gamepad/2");
    assert_eq!(input(&app).listen, Listen::NoGamepad);
    assert_eq!(app.label(GAMEPAD_COMBO_BOX).as_deref(), Some("None"));
}

#[test]
fn a_picked_gamepad_stays_in_the_list_while_it_is_disconnected() {
    let mut app = demo(&["Pad One", "Pad Two"]);
    open(&mut app);
    app.click("Gamepad/1");
    step(&mut app, r#"{ "Disconnect": 2 }"#);
    assert_eq!(listen(&app), "Pad Two");
    assert_eq!(rows(&app).len(), 1);
    open(&mut app);
    assert_eq!(choices(&app), ["Pad One", "Pad Two", "None"]);
}

#[test]
fn without_gamepads_the_list_offers_none() {
    let mut app = demo(&[]);
    assert_eq!(app.label(GAMEPAD_COMBO_BOX).as_deref(), Some("None"));
    open(&mut app);
    assert_eq!(choices(&app), ["None"]);
}

// Until one is picked, a gamepad that the demo chose for itself leaves the
// combo box and its list on the frame that it disconnects, if it was the last
// one connected.
#[test]
fn once_every_gamepad_disconnects_the_box_shows_none_unless_one_was_picked() {
    let mut app = demo(&["Pad One", "Pad Two"]);
    step(&mut app, r#"{ "Disconnect": 1 }"#);
    assert_eq!(app.label(GAMEPAD_COMBO_BOX).as_deref(), Some("Pad Two"));
    step(&mut app, r#"{ "Disconnect": 2 }"#);
    assert_eq!(app.label(GAMEPAD_COMBO_BOX).as_deref(), Some("None"));
    open(&mut app);
    assert_eq!(choices(&app), ["None"]);
    app.key(Key::ESCAPE);

    // The next one to connect is listened to.
    step(&mut app, r#"{ "Connect": { "Id": 3, "Name": "Pad Three", "Input": ["Gamepad"] } }"#);
    assert_eq!(app.label(GAMEPAD_COMBO_BOX).as_deref(), Some("Pad Three"));
}

#[test]
fn esc_or_a_press_off_the_list_closes_it_and_keys_leave_the_gamepad_alone() {
    let mut app = demo(&["Pad One", "Pad Two"]);
    open(&mut app);
    app.key(Key::ESCAPE);
    assert_eq!(app.ui.popup_name(), None);
    open(&mut app);
    app.click_point(Point::new(1300.0, 300.0));
    assert_eq!(app.ui.popup_name(), None);
    assert_eq!(listen(&app), "Pad One");

    // Neither a press on the box nor one on a choice gives it keyboard
    // focus, so once the list is closed, the keys that the user presses to
    // try the Keyboard card leave the gamepad alone.
    open(&mut app);
    app.click("Gamepad/0");
    for key in [Key::DOWN, Key::UP, Key::END, Key::TAB, Key::SPACE, Key::DOWN] {
        app.key(key);
        assert_eq!(app.ui.focused_name(), None);
        assert_eq!(app.ui.popup_name(), None);
    }
    assert_eq!(listen(&app), "Pad One");
}

#[test]
fn while_the_list_is_open_the_arrows_move_to_a_choice_and_enter_picks_it() {
    let mut app = demo(&["Pad One", "Pad Two"]);
    open(&mut app);
    assert_eq!(app.ui.popup_name(), Some(GAMEPAD_COMBO_BOX));
    app.key(Key::DOWN);
    assert_eq!(listen(&app), "Pad One");
    app.key(Key::END).key(Key::ENTER);
    assert_eq!(input(&app).listen, Listen::NoGamepad);
    assert_eq!(app.ui.popup_name(), None);
}

#[test]
fn without_game_input_there_is_nothing_to_point_at() {
    let app = Headless::new(Demo::with_input(Err("GameInput isn't installed".to_string())));
    assert!(names(&app).is_empty());
    assert_eq!(app.state(), Value::Object(vec![("Interface".to_string(), app.ui.state())]));
}

#[test]
fn f11_switches_fullscreen_from_the_window_and_not_through_game_input() {
    let mut app = demo(&[]);
    step(&mut app, r#"{ "Connect": { "Id": 9, "Name": "Keys", "Input": ["Keyboard"] } }"#);
    app.key(Key::F11);
    assert!(app.fullscreen());
    // GameInput reads the same key, which mustn't switch it back.
    step(&mut app, r#"{ "Keyboard": { "Id": 9, "Down": ["F11"] } }"#);
    step(&mut app, r#"{ "Keyboard": { "Id": 9, "Down": [] } }"#);
    assert!(app.fullscreen());
    assert_eq!(input(&app).log[1].text, "Keys: F11 down");
}

#[test]
fn input_without_window_focus_is_dropped_and_stops_the_motors() {
    let mut app = demo(&[]);
    step(&mut app, r#"{ "Connect": { "Id": 1, "Name": "Pad", "Input": ["Gamepad"], "Motors": ["LowFrequency"] } }"#);
    step(&mut app, r#"{ "Connect": { "Id": 2, "Name": "Keys", "Input": ["Keyboard"] } }"#);
    step(&mut app, r#"{ "Keyboard": { "Id": 2, "Down": ["F1"] } }"#);
    step(&mut app, r#"{ "Gamepad": { "Id": 1, "LeftStick": [1, 0] } }"#);
    assert!(input(&app).rumble_enabled);
    let rumble = |app: &mut Headless<Demo>| app.program.input.as_mut().unwrap().scripted().unwrap().rumble(1);
    assert_eq!(rumble(&mut app).unwrap().lowFrequency, 1.0);

    app.focus(false);
    assert!(rumble(&mut app).is_none());
    let count = input(&app).reading_count;
    step(&mut app, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0, 1], "Buttons": ["A"] } }"#);
    assert_eq!(input(&app).reading_count, count);
    assert_eq!(input(&app).gamepad.state.buttons, 0);

    // With focus again, the button held from before doesn't count until
    // it's let go of, but the stick does.
    app.focus(true);
    step(&mut app, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0, 0.5], "Buttons": ["A"] } }"#);
    assert_eq!(input(&app).gamepad.state.leftThumbstickY, 0.5);
    assert_eq!(input(&app).gamepad.state.buttons, 0);
}

#[test]
fn each_device_has_a_row_under_the_one_before() {
    let app = demo(&["Pad One", "Pad Two", "Pad Three"]);
    let rows = rows(&app);
    assert_eq!(rows.len(), 3);
    for pair in rows.windows(2) {
        assert_eq!((pair[1].x, pair[1].width), (pair[0].x, pair[0].width));
        assert_eq!(pair[1].y, pair[0].bottom());
    }
    // In the second column, under the Latest input event card.
    assert!(rows[0].x > PAGE_PADDING + GAMEPAD_COLUMN, "{:?}", rows[0]);
}

#[test]
fn rows_that_do_not_fit_scroll() {
    let mut app = demo(&[]);
    for id in 1..=40 {
        step(&mut app, &format!(r#"{{ "Connect": {{ "Id": {id}, "Name": "Keys {id}", "Input": ["Keyboard"] }} }}"#));
    }
    for fullscreen in [false, true] {
        app.set_fullscreen(fullscreen);
        // The list ends above the card's padding and the status bar, and so
        // do the rows in sight, with a scroll bar for the rest.
        let list = app.target(DEVICE_ROWS);
        assert!(list.bottom() <= app.size().height - STATUS_BAR_HEIGHT - PAGE_PADDING - CARD_PADDING, "{list:?}");
        let rows = rows(&app);
        assert!(rows.len() > 5 && rows.len() < 40, "{}", rows.len());
        assert!(rows.iter().all(|row| row.bottom() <= list.bottom()), "{rows:?}");
        assert!(app.has_target("Devices/ScrollBar"));
        // The wheel scrolls to the last row, and back.
        app.scroll(DEVICE_ROWS, -40.0);
        assert!(app.has_target("Devices/39") && !app.has_target("Devices/0"));
        assert_eq!(app.label("Devices/39").as_deref(), Some("Keys 40"));
        app.scroll(DEVICE_ROWS, 40.0);
        assert!(app.has_target("Devices/0"));
    }
}

#[test]
fn keys_and_details_that_do_not_fit_scroll() {
    let mut app = demo(&["Pad"]);
    step(&mut app, r#"{ "Connect": { "Id": 9, "Name": "Keys", "Input": ["Keyboard"] } }"#);
    assert!(!app.has_target("Keys/ScrollBar") && !app.has_target("Latest/ScrollBar"));
    let keys: Vec<String> = ('A'..='Z').chain('0'..='9').map(|key| format!("\"{key}\"")).collect();
    step(&mut app, &format!(r#"{{ "Keyboard": {{ "Id": 9, "Down": [{}] }} }}"#, keys.join(", ")));
    assert_eq!(input(&app).keyboard.keys.len(), 36);
    assert!(app.has_target("Keys/ScrollBar"));
    let buttons: Vec<String> = game_input_demo::input::GAMEPAD_BUTTONS.iter().map(|(_, name)| format!("\"{name}\"")).collect();
    step(&mut app, &format!(r#"{{ "Gamepad": {{ "Id": 1, "LeftStick": [1, 0], "Buttons": [{}] }} }}"#, buttons.join(", ")));
    assert!(input(&app).latest.as_ref().unwrap().changes.len() > 20);
    assert!(app.has_target("Latest/ScrollBar"));
}

#[test]
fn the_pointer_resting_on_a_row_shows_its_devices_tooltip() {
    let mut app = demo(&["Pad One", "Pad Two"]);
    let rows = rows(&app);
    for (index, row) in rows.iter().enumerate() {
        let (name, _) = rest(&mut app, row.center()).unwrap();
        assert_eq!(name, format!("Devices/{index}"));
    }
    assert_eq!(app.label("Devices/0").as_deref(), Some("Pad One"));
    // Not before half a second.
    app.move_point(rows[0].center()).wait(0.3);
    assert_eq!(tooltip(&app), None);
    // Not under the last row.
    let last = rows[1];
    assert_eq!(rest(&mut app, Point::new(last.center().x, last.bottom() + last.height)), None);
}

// A device with as much in its tooltip as a real one, whose node in
// Windows' device tree has `nodes` - 1 nodes above it.
fn real_device(nodes: usize) -> Headless<Demo> {
    let mut app = demo(&["Pad One", "Pad Two"]);
    let node = Node {
        instance_id: r"USB\VID_045E&PID_0B12\3032434D30303237303135323037".to_string(),
        name: "Xbox Controller".to_string(),
        bus_name: "Controller".to_string(),
        class: "XboxComposite".to_string(),
        location: "Port_#0003.Hub_#0004".to_string(),
        hardware_ids: vec![r"USB\VID_045E&PID_0B12&REV_0509".to_string(); 6],
        ..Default::default()
    };
    let input = app.program.input.as_mut().unwrap();
    input.devices[1].tree = Tree::known(Ok(vec![node; nodes]));
    input.devices[1].pnp_path =
        r"\\?\HID#VID_045E&PID_0B12&IG_00#3&2A8D2A0C&0&0000#{4D1E55B2-F16F-11CF-88CB-001111000030}\KBD".repeat(2);
    app.frame();
    app
}

#[test]
fn a_tooltip_stays_in_the_window_off_the_pointer() {
    for nodes in [1, 5, 9] {
        for fullscreen in [false, true] {
            let mut app = real_device(nodes);
            app.set_fullscreen(fullscreen);
            let size = app.size();
            for row in rows(&app) {
                for pointer in [Point::new(row.x + 1.0, row.center().y), Point::new(row.right() - 1.0, row.center().y)] {
                    let (_, area) = rest(&mut app, pointer).unwrap();
                    // The program's part of the window, under the
                    // PerfBar.
                    let top = PERF_BAR_HEIGHT;
                    assert!(area.x >= 0.0 && area.right() <= size.width && area.y >= top, "{nodes} nodes, {pointer:?}: {area:?}");
                    // A deep device's tooltip, though wider than WinUI's, can
                    // be taller than the maximized window. Then it starts at
                    // the top of the program's part of it.
                    if area.height <= size.height - top - SPACE_S * 2.0 {
                        assert!(area.bottom() <= size.height, "{nodes} nodes, {pointer:?}: {area:?}");
                    } else {
                        assert_eq!(area.y, top + SPACE_S, "{nodes} nodes, {pointer:?}: {area:?}");
                    }
                    // Above the pointer, or under it if there's no room
                    // above. A tooltip with room on neither side covers it.
                    let room = |space: f32| area.height + TOOLTIP_OFFSET + SPACE_S <= space;
                    if room(pointer.y - top) || room(size.height - pointer.y) {
                        assert!(pointer.y < area.y || pointer.y > area.bottom(), "{nodes} nodes, {pointer:?}: {area:?}");
                    }
                    if room(pointer.y - top) {
                        assert!(area.bottom() < pointer.y, "{nodes} nodes, {pointer:?}: {area:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn a_long_value_wraps_in_its_tooltip() {
    let mut app = real_device(5);
    let row = rows(&app)[1];
    let (_, area) = rest(&mut app, row.center()).unwrap();
    // The device's path, twice as long as a real one, wraps instead of
    // making the tooltip as wide, which is wider than WinUI's, so that every
    // detail fits in the window's height.
    assert!(area.width > user_interface::controls::TOOLTIP_MAX_WIDTH && area.width <= game_input_demo::cards::DEVICE_TOOLTIP_WIDTH, "{area:?}");
    assert!(area.bottom() <= user_interface::tokens::MAXIMIZED_HEIGHT, "{area:?}");
}

#[test]
fn every_widget_fits_in_the_window_at_both_heights_and_in_both_themes() {
    for theme in [ThemeKind::Light, ThemeKind::Dark] {
        for fullscreen in [false, true] {
            let mut app = Headless::with_theme(Demo::scripted(0.1), theme);
            for id in 1..=3 {
                let step_text = format!(r#"{{ "Connect": {{ "Id": {id}, "Name": "Pad {id}", "Input": ["Gamepad"] }} }}"#);
                step(&mut app, &step_text);
            }
            app.set_fullscreen(fullscreen);
            open(&mut app);
            let height = if fullscreen { FULLSCREEN_HEIGHT } else { MAXIMIZED_HEIGHT };
            for target in app.ui.targets().iter().filter(|target| target.listed) {
                let rect = target.rect;
                assert!(
                    rect.x >= 0.0 && rect.y >= 0.0 && rect.right() <= WINDOW_WIDTH && rect.bottom() <= height,
                    "{} is at {rect:?}, outside the window",
                    target.name
                );
            }
        }
    }
}

fn close(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter().zip(b).take(3).all(|(a, b)| (*a as i32 - b as i32).abs() <= 2)
}

#[test]
fn the_stick_graphs_draw_where_the_sticks_are() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = demo(&["Pad"]);
    step(&mut app, r#"{ "Gamepad": { "Id": 1, "LeftStick": [1, 0], "RightStick": [0, 1], "Buttons": ["A"] } }"#);
    let image = app.render(&mut gpu).unwrap();
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/TestOutput");
    std::fs::create_dir_all(&folder).unwrap();
    write_png(&folder.join("GameInputDemo.png"), &image).unwrap();

    // The graphs are under the Gamepad card's header and the captions over
    // them, with a ring at the stick's edge, SPACE_S inside the graph.
    let theme = Theme::new(ThemeKind::Light);
    let top = PERF_BAR_HEIGHT + APP_BAR_HEIGHT + PAGE_PADDING + CARD_PADDING + CARD_HEADER_HEIGHT + CARD_HEADER_GAP + BODY.line_height + SPACE_S;
    let left = PAGE_PADDING + CARD_PADDING;
    let radius = STICK_GRAPH / 2.0 - SPACE_S;
    let center = Point::new(left + STICK_GRAPH / 2.0, top + STICK_GRAPH / 2.0);
    // Left stick right, in the left stick's color.
    let dot = image.pixel((center.x + radius) as u32, center.y as u32);
    assert!(close(dot, theme.data.blue.to_bytes()), "{dot:?}");
    // Right stick up: stick y is up, and screen y down.
    let right = Point::new(center.x + STICK_GRAPH + 24.0, center.y - radius);
    let dot = image.pixel(right.x as u32, right.y as u32);
    assert!(close(dot, theme.data.orange.to_bytes()), "{dot:?}");
    // Nothing at the left graph's center but its axes and its dead zone.
    let middle = image.pixel(center.x as u32 + 3, center.y as u32 + 3);
    assert!(!close(middle, theme.data.blue.to_bytes()), "{middle:?}");
}
