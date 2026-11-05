// What input does to UiShowcase's widgets, through its panels' real layout,
// which a canvas builds without a window or a GPU, and that its panels fit
// both supported window heights.

use application::Event;
use json::Value;
use ui_showcase::showcase::Showcase;
use user_interface::headless::{FULLSCREEN, Headless, MAXIMIZED, click, control, key, typed};
use user_interface::Program;
use windows_api::*;

// A showcase, drawn once, so that the next frame's input has targets.
fn start(size: [f32; 2]) -> (Showcase, Headless) {
    let mut showcase = Showcase::default();
    let mut headless = Headless::new(size);
    headless.program(&mut showcase, &[]).unwrap();
    (showcase, headless)
}

fn step(showcase: &mut Showcase, headless: &mut Headless, events: &[Event]) {
    headless.program(showcase, events).unwrap();
}

fn state(showcase: &Showcase, name: &str) -> Value {
    showcase.state().get(name).cloned().unwrap_or_else(|| panic!("no {name}"))
}

#[test]
fn the_panels_fit_both_window_heights() {
    for size in [MAXIMIZED, FULLSCREEN] {
        let (mut showcase, mut headless) = start(size);
        let outcome = headless.program(&mut showcase, &[]).unwrap();
        assert_eq!(outcome.dropped_lines, 0, "at {size:?}");
        // Every target is inside the window, and the list stretches to use
        // the 71 more pixels of fullscreen.
        for name in headless.interface.target_names() {
            let area = headless.target(&name);
            assert!(area.x >= 0.0 && area.right() <= size[0] && area.bottom() <= size[1], "{name}: {area:?}");
        }
    }
    let list = |size| start(size).1.target("Items");
    assert_eq!(list(FULLSCREEN).height - list(MAXIMIZED).height, 71.0);
}

#[test]
fn it_has_a_widget_of_every_kind() {
    let (_, headless) = start(MAXIMIZED);
    for name in [
        "Normal", "Primary", "Danger", "Disabled", "Enable", "Grid", "Animate", "Tooltips", "Size", "Size/Large",
        "Tabs", "Tabs/Notes", "Volume", "Locked", "Color", "Density", "Tags/Rust", "Name", "Greet", "Search", "Items",
        "Items/Scroll bar", "Sentence", "Columns", "Bottom",
    ] {
        headless.target(name);
    }
}

#[test]
fn buttons_count_their_presses_and_the_disabled_one_takes_none() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    for name in ["Primary", "Primary", "Danger", "Disabled"] {
        let point = headless.center(name);
        step(&mut showcase, &mut headless, &click(point));
    }
    assert_eq!(showcase.presses, [0, 2, 1]);
    assert_eq!(showcase.log.front().map(String::as_str), Some("Danger: pressed 1 time"));
}

#[test]
fn the_first_checkbox_disables_the_four_under_it() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let enable = headless.center("Enable");
    let grid = headless.center("Grid");
    step(&mut showcase, &mut headless, &click(enable));
    assert!(!showcase.enabled);
    step(&mut showcase, &mut headless, &click(grid));
    assert!(showcase.grid);
    step(&mut showcase, &mut headless, &click(enable));
    step(&mut showcase, &mut headless, &click(grid));
    assert!(!showcase.grid);
}

#[test]
fn a_slider_drags_and_the_locked_one_does_not() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let volume = headless.target("Volume");
    let [left, right] = user_interface::track(volume);
    let middle = volume.center()[1];
    step(&mut showcase, &mut headless, &[Event::Press([left + (right - left) * 0.75, middle])]);
    assert_eq!(showcase.volume, 75.0);
    headless.move_to([right + 500.0, middle + 300.0]);
    step(&mut showcase, &mut headless, &[]);
    step(&mut showcase, &mut headless, &[Event::Release([right + 500.0, middle + 300.0])]);
    assert_eq!(showcase.volume, 100.0);
    assert_eq!(showcase.log.front().map(String::as_str), Some("Volume: 100%"));
    let locked = headless.center("Locked");
    step(&mut showcase, &mut headless, &click([locked[0] + 100.0, locked[1]]));
    assert_eq!(showcase.locked, 0.5);
}

#[test]
fn the_color_select_box_picks_from_its_list() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let color = headless.center("Color");
    step(&mut showcase, &mut headless, &click(color));
    let purple = headless.center("Color/2");
    step(&mut showcase, &mut headless, &click(purple));
    assert_eq!(state(&showcase, "Color"), Value::from("Purple"));
    assert!(headless.interface.popup().is_none());
}

#[test]
fn enter_or_greet_says_hello_to_the_name() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let name = headless.center("Name");
    let greet = headless.center("Greet");
    step(&mut showcase, &mut headless, &click(greet));
    assert_eq!(showcase.greeting.as_deref(), Some("Type a name first"));
    step(&mut showcase, &mut headless, &click(name));
    step(&mut showcase, &mut headless, &typed("Grace"));
    step(&mut showcase, &mut headless, &[key(VK_RETURN)]);
    assert_eq!(showcase.greeting.as_deref(), Some("Hello, Grace!"));
}

#[test]
fn the_search_filters_the_list_and_a_press_selects_what_it_shows() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let search = headless.center("Search");
    step(&mut showcase, &mut headless, &click(search));
    step(&mut showcase, &mut headless, &typed("comet"));
    assert_eq!(showcase.matches().len(), 10);
    step(&mut showcase, &mut headless, &[]);
    let row = headless.center("Items/2");
    step(&mut showcase, &mut headless, &click(row));
    assert_eq!(state(&showcase, "Selected"), Value::from("023  Cobalt Comet"));
    // The selection stays when the search no longer shows it.
    let search = headless.center("Search");
    step(&mut showcase, &mut headless, &click(search));
    step(&mut showcase, &mut headless, &typed("zzz"));
    assert_eq!(showcase.matches().len(), 0);
    assert_eq!(state(&showcase, "Selected"), Value::from("023  Cobalt Comet"));
}

#[test]
fn the_list_rows_follow_the_density() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let height = |headless: &Headless| headless.target("Items/0").height;
    assert_eq!(height(&headless), 24.0);
    let density = headless.center("Density");
    step(&mut showcase, &mut headless, &click(density));
    let compact = headless.center("Density/2");
    step(&mut showcase, &mut headless, &click(compact));
    step(&mut showcase, &mut headless, &[]);
    assert_eq!(height(&headless), 20.0);
}

#[test]
fn hovering_shows_tooltips_until_f1_turns_them_off() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let bottom = headless.target("Bottom");
    headless.move_to(bottom.center());
    step(&mut showcase, &mut headless, &[]);
    let (name, area) = headless.interface.tooltip().expect("a tooltip");
    assert_eq!(name, "Bottom");
    // There's no room under the cursor, so it's above it.
    assert!(area.bottom() < bottom.center()[1]);
    step(&mut showcase, &mut headless, &[key(VK_F1)]);
    assert!(!showcase.tooltips);
    assert!(headless.interface.tooltip().is_none());
}

#[test]
fn tab_reaches_every_widget_that_takes_focus_in_order() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let mut order = Vec::new();
    for _ in 0..40 {
        step(&mut showcase, &mut headless, &[key(VK_TAB)]);
        let focus = headless.interface.focus().unwrap();
        if order.first() == Some(&focus) {
            break;
        }
        order.push(focus);
    }
    assert_eq!(
        order,
        [
            "Normal", "Primary", "Danger", "Enable", "Grid", "Snap", "Sound", "Animate", "Tooltips", "Size", "Tabs",
            "Volume", "Opacity", "Angle", "Color", "Density", "Tags/Rust", "Tags/Win32", "Tags/Direct3D 12",
            "Tags/DirectWrite", "Tags/Low latency", "Tags/Light theme", "Name", "Greet", "Search", "Items",
        ]
    );
}

#[test]
fn ctrl_r_resets_everything_and_f11_asks_for_fullscreen() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let primary = headless.center("Primary");
    step(&mut showcase, &mut headless, &click(primary));
    step(&mut showcase, &mut headless, &[control('R' as i32)]);
    assert_eq!(showcase.presses, [0, 0, 0]);
    assert_eq!(showcase.log.front().map(String::as_str), Some("Everything: reset"));
    let outcome = headless.program(&mut showcase, &[key(VK_F11)]).unwrap();
    assert!(outcome.fullscreen);
}

#[test]
fn without_focus_nothing_happens() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    headless.focused = false;
    let primary = headless.center("Primary");
    step(&mut showcase, &mut headless, &click(primary));
    step(&mut showcase, &mut headless, &[key(VK_F1)]);
    assert_eq!((showcase.presses, showcase.tooltips), ([0, 0, 0], true));
}
