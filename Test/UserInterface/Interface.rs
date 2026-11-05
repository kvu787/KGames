// How input reaches widgets: clicks hit what the frame before drew, the
// topmost target takes a press, the widget with the press gets the drag and
// the release, keyboard focus and Tab, popups over the base layer, disabled
// widgets, the window without focus, double-clicks, easing, the theme,
// notices, and the state that scripts save.

use application::Event;
use json::Value;
use user_interface::headless::{Headless, click, key, shifted};
use user_interface::{Button, Choice, Frame, Gamepad, GamepadButton, Id, Rect, Sense, SliderRange, slider_rail, theme};
use windows_api::*;

const SAVE: Rect = Rect { x: 100.0, y: 100.0, width: 80.0, height: 32.0 };
const OPEN: Rect = Rect { x: 200.0, y: 100.0, width: 80.0, height: 32.0 };

fn save(frame: &mut Frame, area: Rect) -> user_interface::Response {
    frame.button("Save", area, Button::new("Save"))
}

#[test]
fn a_press_hits_what_the_frame_before_drew() {
    let mut harness = Headless::default();
    // Nothing was drawn before the first frame, so its press hits nothing.
    let first = harness.frame(&click(SAVE.center()), |frame| save(frame, SAVE));
    assert!(!first.pressed);
    let second = harness.frame(&click(SAVE.center()), |frame| save(frame, SAVE));
    assert!(second.pressed && second.clicked && second.released);
    // A button that moved acts where it was.
    let moved = Rect { x: 600.0, ..SAVE };
    let third = harness.frame(&click(SAVE.center()), |frame| save(frame, moved));
    assert!(third.clicked);
    let fourth = harness.frame(&click(SAVE.center()), |frame| save(frame, moved));
    assert!(!fourth.clicked);
}

#[test]
fn the_press_goes_to_the_topmost_target() {
    let mut harness = Headless::default();
    let draw = |frame: &mut Frame| {
        let under = frame.button("Under", SAVE, Button::new("Under"));
        let over = frame.button("Over", Rect { x: 150.0, ..SAVE }, Button::new("Over"));
        (under.clicked, over.clicked)
    };
    harness.frame(&[], draw);
    assert_eq!(harness.frame(&click([160.0, 110.0]), draw), (false, true));
    assert_eq!(harness.frame(&click([110.0, 110.0]), draw), (true, false));
    // A target that only shows a tooltip keeps presses from what's under it.
    let covered = |frame: &mut Frame| {
        let button = save(frame, SAVE);
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
    let range = SliderRange::new(0.0, 1.0, 1000.0);
    let area = Rect::new(100.0, 100.0, 420.0, 32.0);
    let [left, right] = slider_rail(area);
    let mut draw = |harness: &mut Headless, events: &[Event]| {
        harness.frame(events, |frame| frame.slider("Slider", area, &mut value, &range))
    };
    draw(&mut harness, &[]);
    let start = [left + (right - left) * 0.25, 115.0];
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
    let draw = |frame: &mut Frame| {
        let save = save(frame, SAVE);
        let open = frame.button("Open", OPEN, Button::new("Open").accent());
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
    let draw = |frame: &mut Frame| {
        save(frame, SAVE);
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
    harness.frame(&[], |frame| save(frame, SAVE));
    let disabled = |frame: &mut Frame| {
        frame.set_enabled(false);
        save(frame, SAVE)
    };
    let response = harness.frame(&click(SAVE.center()), disabled);
    assert!(!response.clicked);
    let response = harness.frame(&click(SAVE.center()), disabled);
    assert!(!response.clicked);
    // It still shows that the cursor is over it, for its tooltip.
    assert!(response.hovered);
}

#[test]
fn widgets_that_are_not_focusable_leave_keys_to_the_program() {
    let mut harness = Headless::default();
    let mut value = 0.5;
    let range = SliderRange::new(0.0, 1.0, 1000.0);
    let area = Rect::new(100.0, 100.0, 420.0, 32.0);
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
    harness.frame(&[], |frame| frame.button("Save", SAVE, Button::new("Save")));
    harness.frame(&click(SAVE.center()), |frame| frame.button("Save", SAVE, Button::new("Save")));
    assert_eq!(harness.interface.focus().as_deref(), Some("Save"));
    harness.held_keys = vec![VK_LEFT as u32, VK_SPACE as u32];
    let held = harness.frame(&[], |frame| {
        frame.button("Save", SAVE, Button::new("Save"));
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
    let mut harness = Headless {
        gamepad: Gamepad {
            connected: true,
            right_stick: [0.0, 1.0],
            buttons: GamepadButton::A.bit(),
            ..Gamepad::default()
        },
        ..Headless::default()
    };
    let first = harness.frame(&[], |frame| *frame.gamepad());
    assert!(first.pressed(GamepadButton::A) && first.held(GamepadButton::A));
    assert_eq!(first.right_stick, [0.0, 1.0]);
    let second = harness.frame(&[], |frame| *frame.gamepad());
    assert!(second.held(GamepadButton::A) && !second.pressed(GamepadButton::A));
    harness.focused = false;
    let unfocused = harness.frame(&[], |frame| *frame.gamepad());
    assert!(unfocused.connected && unfocused.buttons == 0 && unfocused.right_stick == [0.0, 0.0]);
}

#[test]
fn a_widget_that_is_not_drawn_loses_focus_and_its_press() {
    let mut harness = Headless::default();
    harness.frame(&[], |frame| save(frame, SAVE));
    harness.frame(&[Event::Press(SAVE.center())], |frame| save(frame, SAVE));
    assert_eq!(harness.interface.focus().as_deref(), Some("Save"));
    harness.frame(&[], |_| {});
    assert_eq!(harness.interface.focus(), None);
    let held = harness.interface.state().get("Held").cloned();
    assert_eq!(held, Some(Value::Null));
}

#[test]
fn without_focus_popups_close_and_nothing_is_hovered() {
    let mut harness = Headless::default();
    let choices = [Choice { label: "One", detail: "" }, Choice { label: "Two", detail: "" }];
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
fn a_second_press_soon_on_the_same_widget_is_a_double_click() {
    let mut harness = Headless::default();
    let draw = |frame: &mut Frame| save(frame, SAVE).double_clicked;
    harness.frame(&[], draw);
    assert!(!harness.frame(&click(SAVE.center()), draw));
    assert!(harness.frame(&click(SAVE.center()), draw));
    // A third press starts over.
    assert!(!harness.frame(&click(SAVE.center()), draw));
    // Half a second later is too late: 31 frames at 60 a second.
    for _ in 0..31 {
        harness.frame(&[], draw);
    }
    assert!(!harness.frame(&click(SAVE.center()), draw));
}

#[test]
fn eased_values_start_where_they_go_and_then_follow_quickly() {
    let mut harness = Headless::default();
    let id = Id::new("Knob");
    assert_eq!(harness.frame(&[], |frame| frame.ease(id, 1.0)), 1.0);
    let next = harness.frame(&[], |frame| frame.ease(id, 0.0));
    assert!(next > 0.0 && next < 0.6, "{next}");
    // A frame is 1/60 of a second, so in 6, 100 ms, it's there.
    let mut value = next;
    for _ in 0..6 {
        value = harness.frame(&[], |frame| frame.ease(id, 0.0));
    }
    assert_eq!(value, 0.0);
    // A value that a frame doesn't ask for goes, and starts over.
    harness.frame(&[], |_| {});
    assert_eq!(harness.frame(&[], |frame| frame.ease(id, 1.0)), 1.0);
}

#[test]
fn the_theme_is_light_until_a_program_sets_another() {
    let mut harness = Headless::default();
    let name = harness.frame(&[], |frame| frame.theme().name);
    assert_eq!(name, "Light");
    harness.frame(&[], |frame| frame.set_theme(&theme::DARK));
    assert_eq!(harness.interface.theme().name, "Dark");
    assert_eq!(harness.interface.state().get("Theme"), Some(&Value::from("Dark")));
}

#[test]
fn a_press_on_a_notices_button_closes_it() {
    let mut harness = Headless {
        notices: vec!["First".to_string(), "Second".to_string()],
        ..Headless::default()
    };
    let draw = |frame: &mut Frame| frame.header("Test", "");
    harness.frame(&[], draw);
    let first = harness.target("Notice/0");
    let second = harness.target("Notice/1");
    assert!(second.y > first.y);
    harness.frame(&click(first.center()), draw);
    harness.frame(&[], draw);
    assert!(harness.interface.target("Notice/0").is_none());
    // The second moves up to where the first was.
    assert_eq!(harness.target("Notice/1").y, first.y);
}

#[test]
fn the_state_names_the_targets_and_what_the_input_is_on() {
    let mut harness = Headless::default();
    let draw = |frame: &mut Frame| {
        save(frame, SAVE);
        let open = frame.button("Open", OPEN, Button::new("Open"));
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
    let dropped = harness.frame(&[], |frame| {
        save(frame, SAVE);
        frame.show_window_problems("Supported: two setups", &["The window isn't maximized".to_string()]);
        frame.dropped_lines()
    });
    assert!(harness.interface.target_names().is_empty());
    assert_eq!(dropped, 0);
    let clicked = harness.frame(&click(SAVE.center()), |frame| save(frame, SAVE).clicked);
    assert!(!clicked);
}
