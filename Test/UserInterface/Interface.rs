// How input reaches widgets: clicks hit what the frame before drew, the
// topmost target takes a press, the widget with the press gets the drag and
// the release, keyboard focus and Tab, popups over the base layer, disabled
// widgets, the window without focus, and the state that scripts save.

use application::Event;
use user_interface::headless::{Headless, click, key, shifted};
use json::Value;
use user_interface::{Button, Gamepad, Rect, Sense, Tone};
use windows_api::*;

const SAVE: Rect = Rect { x: 100.0, y: 100.0, width: 80.0, height: 30.0 };
const OPEN: Rect = Rect { x: 200.0, y: 100.0, width: 80.0, height: 30.0 };

#[test]
fn a_press_hits_what_the_frame_before_drew() {
    let mut harness = Headless::default();
    // Nothing was drawn before the first frame, so its press hits nothing.
    let first = harness.frame(&click(SAVE.center()), |frame| frame.button("Save", SAVE, "Save"));
    assert!(!first.pressed);
    let second = harness.frame(&click(SAVE.center()), |frame| frame.button("Save", SAVE, "Save"));
    assert!(second.pressed && second.clicked && second.released);
    // A button that moved acts where it was.
    let moved = Rect { x: 600.0, ..SAVE };
    let third = harness.frame(&click(SAVE.center()), |frame| frame.button("Save", moved, "Save"));
    assert!(third.clicked);
    let fourth = harness.frame(&click(SAVE.center()), |frame| frame.button("Save", moved, "Save"));
    assert!(!fourth.clicked);
}

#[test]
fn the_press_goes_to_the_topmost_target() {
    let mut harness = Headless::default();
    let draw = |frame: &mut user_interface::Frame| {
        let under = frame.button("Under", SAVE, "Under");
        let over = frame.button("Over", Rect { x: 150.0, ..SAVE }, "Over");
        (under.clicked, over.clicked)
    };
    harness.frame(&[], draw);
    assert_eq!(harness.frame(&click([160.0, 110.0]), draw), (false, true));
    assert_eq!(harness.frame(&click([110.0, 110.0]), draw), (true, false));
    // A target that only shows a tooltip keeps presses from what's under it.
    let covered = |frame: &mut user_interface::Frame| {
        let button = frame.button("Under", SAVE, "Under");
        frame.interact("Cover", "Cover", SAVE, Sense::HOVER);
        button.clicked
    };
    harness.frame(&[], covered);
    assert!(!harness.frame(&click(SAVE.center()), covered));
}

#[test]
fn the_widget_with_the_press_gets_the_drag_and_release_anywhere() {
    let mut harness = Headless::default();
    let mut value = 0.5;
    let range = user_interface::SliderRange::new(0.0, 1.0, 1000.0);
    let area = Rect::new(100.0, 100.0, 432.0, 30.0);
    let track = user_interface::track(area);
    let mut draw = |harness: &mut Headless, events: &[Event]| {
        harness.frame(events, |frame| frame.slider("Slider", area, &mut value, &range))
    };
    draw(&mut harness, &[]);
    let start = [track[0] + (track[1] - track[0]) * 0.25, 115.0];
    let response = draw(&mut harness, &[Event::Press(start)]);
    assert!(response.pressed && response.held && response.changed);
    // Far below the slider, and past its right end.
    harness.move_to([3000.0, 900.0]);
    let response = draw(&mut harness, &[]);
    assert!(response.held);
    let response = draw(&mut harness, &[Event::Release([3000.0, 900.0])]);
    assert!(response.released && !response.held);
    drop(draw);
    assert_eq!(value, 1.0);
}

#[test]
fn space_and_enter_press_the_focused_button_and_tab_moves_focus() {
    let mut harness = Headless::default();
    let draw = |frame: &mut user_interface::Frame| {
        let save = frame.button("Save", SAVE, "Save");
        let open = frame.button_with("Open", OPEN, "Open", Tone::Primary);
        (save.clicked, open.clicked, frame.shortcut(VK_SPACE))
    };
    harness.frame(&[], draw);
    // Without focus, Space is the program's.
    assert_eq!(harness.frame(&[key(VK_SPACE)], draw), (false, false, true));
    harness.frame(&[key(VK_TAB)], draw);
    assert_eq!(harness.interface.focus().as_deref(), Some("Save"));
    assert_eq!(harness.frame(&[key(VK_SPACE)], draw), (true, false, false));
    harness.frame(&[key(VK_TAB)], draw);
    assert_eq!(harness.frame(&[key(VK_RETURN)], draw), (false, true, false));
    // Tab goes around, and Shift+Tab goes back.
    harness.frame(&[key(VK_TAB)], draw);
    assert_eq!(harness.interface.focus().as_deref(), Some("Save"));
    harness.frame(&[shifted(VK_TAB)], draw);
    assert_eq!(harness.interface.focus().as_deref(), Some("Open"));
    // Esc takes focus away, and then it's the program's.
    harness.frame(&[key(VK_ESCAPE)], draw);
    assert_eq!(harness.interface.focus(), None);
    let escape = harness.frame(&[key(VK_ESCAPE)], |frame| frame.shortcut(VK_ESCAPE));
    assert!(escape);
}

#[test]
fn a_press_on_nothing_takes_focus_away_and_is_the_programs() {
    let mut harness = Headless::default();
    let draw = |frame: &mut user_interface::Frame| {
        frame.button("Save", SAVE, "Save");
        frame.program_events().count()
    };
    harness.frame(&[], draw);
    harness.frame(&click(SAVE.center()), draw);
    assert_eq!(harness.interface.focus().as_deref(), Some("Save"));
    assert_eq!(harness.frame(&click([900.0, 900.0]), draw), 2);
    assert_eq!(harness.interface.focus(), None);
}

#[test]
fn a_disabled_widget_takes_no_input_even_from_the_frame_before() {
    let mut harness = Headless::default();
    harness.frame(&[], |frame| frame.button("Save", SAVE, "Save"));
    let response = harness.frame(&click(SAVE.center()), |frame| {
        frame.set_enabled(false);
        frame.button("Save", SAVE, "Save")
    });
    assert!(!response.clicked);
    let response = harness.frame(&click(SAVE.center()), |frame| {
        frame.set_enabled(false);
        frame.button("Save", SAVE, "Save")
    });
    assert!(!response.clicked);
    // It still shows that the cursor is over it, for its tooltip.
    assert!(response.hovered);
}

#[test]
fn widgets_that_are_not_focusable_leave_keys_to_the_program() {
    let mut harness = Headless::default();
    let mut value = 0.5;
    let range = user_interface::SliderRange::new(0.0, 1.0, 1000.0);
    let area = Rect::new(100.0, 100.0, 432.0, 30.0);
    let mut draw = |harness: &mut Headless, events: &[Event]| {
        harness.frame(events, |frame| {
            frame.set_focusable(false);
            frame.slider("Slider", area, &mut value, &range);
            frame.shortcut(VK_RIGHT)
        })
    };
    draw(&mut harness, &[]);
    draw(&mut harness, &click(area.center()));
    assert!(draw(&mut harness, &[key(VK_RIGHT)]));
    drop(draw);
    assert_eq!(value, 0.5);
    assert_eq!(harness.interface.focus(), None);
}

// A held key is the program's whatever has keyboard focus, as it's for what
// goes on while it's held, and no key is held without the window's focus.
#[test]
fn held_keys_are_held_whatever_has_focus_and_none_without_it() {
    let mut harness = Headless::default();
    harness.frame(&[], |frame| frame.button("Save", SAVE, "Save"));
    harness.frame(&click(SAVE.center()), |frame| frame.button("Save", SAVE, "Save"));
    assert_eq!(harness.interface.focus().as_deref(), Some("Save"));
    harness.held_keys = vec![VK_LEFT as u32, VK_SPACE as u32];
    let held = harness.frame(&[], |frame| {
        frame.button("Save", SAVE, "Save");
        [frame.held(VK_LEFT), frame.held(VK_SPACE), frame.held(VK_RIGHT)]
    });
    assert_eq!(held, [true, true, false]);
    harness.focused = false;
    assert!(!harness.frame(&[], |frame| frame.held(VK_LEFT)));
}

// The gamepad is the program's too: a button is pressed in the frame that it
// goes down, and the gamepad is at rest without the window's focus.
#[test]
fn the_gamepad_is_the_programs_and_at_rest_without_focus() {
    let mut harness = Headless::default();
    harness.gamepad = Gamepad {
        connected: true,
        right_stick: [0.0, 1.0],
        buttons: Button::A.bit(),
        ..Gamepad::default()
    };
    let first = harness.frame(&[], |frame| *frame.gamepad());
    assert!(first.pressed(Button::A) && first.held(Button::A));
    assert_eq!(first.right_stick, [0.0, 1.0]);
    let second = harness.frame(&[], |frame| *frame.gamepad());
    assert!(second.held(Button::A) && !second.pressed(Button::A));
    harness.focused = false;
    let unfocused = harness.frame(&[], |frame| *frame.gamepad());
    assert!(unfocused.connected && unfocused.buttons == 0 && unfocused.right_stick == [0.0, 0.0]);
}

#[test]
fn a_widget_that_is_not_drawn_loses_focus_and_its_press() {
    let mut harness = Headless::default();
    harness.frame(&[], |frame| frame.button("Save", SAVE, "Save"));
    harness.frame(&[Event::Press(SAVE.center())], |frame| frame.button("Save", SAVE, "Save"));
    assert_eq!(harness.interface.focus().as_deref(), Some("Save"));
    harness.frame(&[], |_| {});
    assert_eq!(harness.interface.focus(), None);
    let held = harness.interface.state().get("Held").cloned();
    assert_eq!(held, Some(Value::Null));
}

#[test]
fn without_focus_popups_close_and_nothing_is_hovered() {
    let mut harness = Headless::default();
    let choices = [
        user_interface::Choice { label: "One", detail: "" },
        user_interface::Choice { label: "Two", detail: "" },
    ];
    let mut selected = 0;
    let mut draw = |harness: &mut Headless, events: &[Event]| {
        harness.frame(events, |frame| frame.select("Pick", SAVE, &choices, &mut selected))
    };
    draw(&mut harness, &[]);
    draw(&mut harness, &click(SAVE.center()));
    assert_eq!(harness.interface.popup().as_deref(), Some("Pick"));
    harness.focused = false;
    let response = draw(&mut harness, &[]);
    assert!(!response.hovered);
    assert_eq!(harness.interface.popup(), None);
}

#[test]
fn the_state_names_the_targets_and_what_the_input_is_on() {
    let mut harness = Headless::default();
    let draw = |frame: &mut user_interface::Frame| {
        frame.button("Save", SAVE, "Save");
        let open = frame.button("Open", OPEN, "Open");
        if open.hovered {
            frame.tooltip_text("Open", open.area, "Opens a file");
        }
    };
    harness.frame(&[], draw);
    harness.move_to(OPEN.center());
    harness.frame(&[Event::Press(OPEN.center())], draw);
    let state = harness.interface.state();
    let names: Vec<String> = match state.get("Targets") {
        Some(Value::Array(targets)) => targets
            .iter()
            .map(|target| match target.get("Name") {
                Some(Value::String(name)) => name.clone(),
                other => panic!("{other:?}"),
            })
            .collect(),
        other => panic!("{other:?}"),
    };
    assert_eq!(names, ["Save", "Open"]);
    let get = |name: &str| state.get(name).cloned().unwrap();
    assert_eq!(get("Hovered"), Value::from("Open"));
    assert_eq!(get("Held"), Value::from("Open"));
    assert_eq!(get("Focus"), Value::from("Open"));
    assert_eq!(get("Popup"), Value::Null);
    assert_eq!(get("Tooltip").get("Name"), Some(&Value::from("Open")));
}

#[test]
fn the_window_problems_page_takes_away_every_target() {
    let mut harness = Headless::default();
    harness.frame(&[], |frame| {
        frame.button("Save", SAVE, "Save");
        frame.show_window_problems("Supported: two setups", &["The window isn't maximized".to_string()]);
    });
    assert!(harness.interface.target_names().is_empty());
    let clicked = harness.frame(&click(SAVE.center()), |frame| frame.button("Save", SAVE, "Save").clicked);
    assert!(!clicked);
}
