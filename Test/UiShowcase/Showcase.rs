// What input does to UiShowcase's widgets, through its cards' real layout,
// which a canvas builds without a window or a GPU, and that its cards fit
// both supported window heights.

use application::Event;
use json::Value;
use ui_showcase::showcase::Showcase;
use user_interface::headless::{FULLSCREEN, Headless, MAXIMIZED, click, control, key, typed};
use user_interface::{Program, slider_rail};
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

// A press and release on the widget of this name.
fn press(showcase: &mut Showcase, headless: &mut Headless, name: &str) {
    let point = headless.center(name);
    step(showcase, headless, &click(point));
}

fn state(showcase: &Showcase, name: &str) -> Value {
    showcase.state().get(name).cloned().unwrap_or_else(|| panic!("no {name}"))
}

#[test]
fn the_cards_fit_both_window_heights() {
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
fn the_dark_theme_fits_too() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    step(&mut showcase, &mut headless, &[key(VK_F2)]);
    let outcome = headless.program(&mut showcase, &[]).unwrap();
    assert_eq!(outcome.dropped_lines, 0);
    assert!(showcase.dark);
    assert_eq!(headless.interface.theme().name, "Dark");
}

#[test]
fn it_has_a_widget_of_every_kind() {
    let (_, headless) = start(MAXIMIZED);
    for name in [
        "Standard", "Accent", "Danger", "Subtle", "Disabled", "Reset count", "Add", "Remove", "Enable", "Grid",
        "Animate", "Tooltips", "Dark", "Size", "Size/Large", "View", "View/Month", "Tabs", "Tabs/Notes", "Volume",
        "Locked", "Color", "Density", "Tags/Rust", "Name", "Greet", "Search", "Close caution", "Items",
        "Items/Scroll bar", "Sentence", "Columns", "Bottom",
    ] {
        headless.target(name);
    }
}

#[test]
fn buttons_count_their_presses_and_the_disabled_one_takes_none() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    for name in ["Accent", "Accent", "Danger", "Subtle", "Disabled"] {
        press(&mut showcase, &mut headless, name);
    }
    assert_eq!(showcase.presses, [0, 2, 1, 1]);
    assert_eq!(showcase.log.front().map(String::as_str), Some("Subtle: pressed 1 time"));
}

#[test]
fn the_icon_buttons_add_to_and_take_from_the_count() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    for name in ["Add", "Add", "Remove", "Add"] {
        press(&mut showcase, &mut headless, name);
    }
    assert_eq!(showcase.count, 5);
    press(&mut showcase, &mut headless, "Reset count");
    assert_eq!(showcase.count, 0);
}

#[test]
fn the_first_checkbox_disables_the_four_under_it() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    press(&mut showcase, &mut headless, "Enable");
    assert!(!showcase.enabled);
    press(&mut showcase, &mut headless, "Grid");
    assert!(showcase.grid);
    press(&mut showcase, &mut headless, "Enable");
    press(&mut showcase, &mut headless, "Grid");
    assert!(!showcase.grid);
}

#[test]
fn a_slider_drags_and_the_locked_one_does_not() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let volume = headless.target("Volume");
    let [left, right] = slider_rail(volume);
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
fn the_color_combo_box_picks_from_its_list() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    press(&mut showcase, &mut headless, "Color");
    press(&mut showcase, &mut headless, "Color/2");
    assert_eq!(state(&showcase, "Color"), Value::from("Caution"));
    assert!(headless.interface.popup().is_none());
}

#[test]
fn the_segmented_control_and_radio_buttons_pick() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    press(&mut showcase, &mut headless, "View/Month");
    press(&mut showcase, &mut headless, "Size/Small");
    step(&mut showcase, &mut headless, &[key(VK_DOWN), key(VK_DOWN)]);
    assert_eq!((state(&showcase, "View"), state(&showcase, "Size")), (Value::from("Month"), Value::from("Large")));
}

#[test]
fn enter_or_greet_says_hello_to_the_name() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    press(&mut showcase, &mut headless, "Greet");
    assert_eq!(showcase.greeting.as_deref(), Some("Type a name first"));
    press(&mut showcase, &mut headless, "Name");
    step(&mut showcase, &mut headless, &typed("Grace"));
    step(&mut showcase, &mut headless, &[key(VK_RETURN)]);
    assert_eq!(showcase.greeting.as_deref(), Some("Hello, Grace!"));
}

#[test]
fn the_caution_info_bar_closes_and_comes_back() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    press(&mut showcase, &mut headless, "Close caution");
    assert!(showcase.caution_closed);
    // The button that shows it again is there from the next frame.
    step(&mut showcase, &mut headless, &[]);
    press(&mut showcase, &mut headless, "Show caution");
    assert!(!showcase.caution_closed);
    step(&mut showcase, &mut headless, &[]);
    headless.target("Close caution");
}

#[test]
fn the_search_filters_the_list_and_a_press_selects_what_it_shows() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    press(&mut showcase, &mut headless, "Search");
    step(&mut showcase, &mut headless, &typed("comet"));
    assert_eq!(showcase.matches().len(), 10);
    step(&mut showcase, &mut headless, &[]);
    press(&mut showcase, &mut headless, "Items/2");
    assert_eq!(state(&showcase, "Selected"), Value::from("023  Cobalt Comet"));
    // The selection stays when the search no longer shows it, and the
    // search's button clears it.
    press(&mut showcase, &mut headless, "Search");
    step(&mut showcase, &mut headless, &typed("zzz"));
    assert_eq!(showcase.matches().len(), 0);
    assert_eq!(state(&showcase, "Selected"), Value::from("023  Cobalt Comet"));
    press(&mut showcase, &mut headless, "Search/Clear");
    assert_eq!(showcase.search, "");
}

#[test]
fn a_double_click_on_a_row_opens_it() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let row = headless.center("Items/4");
    headless.move_to(row);
    step(&mut showcase, &mut headless, &click(row));
    step(&mut showcase, &mut headless, &click(row));
    assert_eq!(state(&showcase, "Opened"), Value::from("005  Eager Anchor"));
}

#[test]
fn the_list_rows_follow_the_density() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    let height = |headless: &Headless| headless.target("Items/0").height;
    assert_eq!(height(&headless), 36.0);
    press(&mut showcase, &mut headless, "Density");
    press(&mut showcase, &mut headless, "Density/2");
    step(&mut showcase, &mut headless, &[]);
    assert_eq!(height(&headless), 28.0);
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
    for _ in 0..60 {
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
            "Standard", "Accent", "Danger", "Subtle", "Reset count", "Remove", "Add", "Enable", "Grid", "Snap",
            "Sound", "Animate", "Tooltips", "Dark", "Size", "View", "Tabs", "Volume", "Opacity", "Angle", "Color",
            "Density", "Tags/Rust", "Tags/Win32", "Tags/Direct3D 12", "Tags/DirectWrite", "Tags/Low latency",
            "Tags/Fluent", "Name", "Greet", "Search", "Close caution", "Items",
        ]
    );
}

#[test]
fn ctrl_r_resets_everything_and_f11_asks_for_fullscreen() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    press(&mut showcase, &mut headless, "Accent");
    step(&mut showcase, &mut headless, &[control('R' as i32)]);
    assert_eq!(showcase.presses, [0, 0, 0, 0]);
    assert_eq!(showcase.log.front().map(String::as_str), Some("Everything: reset"));
    let outcome = headless.program(&mut showcase, &[key(VK_F11)]).unwrap();
    assert!(outcome.fullscreen);
}

#[test]
fn without_focus_nothing_happens() {
    let (mut showcase, mut headless) = start(MAXIMIZED);
    headless.focused = false;
    press(&mut showcase, &mut headless, "Accent");
    step(&mut showcase, &mut headless, &[key(VK_F1)]);
    assert_eq!((showcase.presses, showcase.tooltips), ([0, 0, 0, 0], true));
}
