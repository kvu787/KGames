// What input does to each widget: checkboxes, switches, radio buttons, tabs,
// chips, sliders, select boxes, text fields, lists and tooltips.

use application::Event;
use user_interface::headless::{self, Headless, MAXIMIZED, click, key, shifted, typed};
use user_interface::{Choice, Frame, Rect, SliderRange, track};
use windows_api::*;

const WIDTH: f32 = MAXIMIZED[0];
const HEIGHT: f32 = MAXIMIZED[1];

#[test]
fn a_checkbox_and_a_switch_flip_on_a_press_or_space() {
    let mut harness = Headless::default();
    let (mut checked, mut on) = (false, true);
    let check_area = Rect::new(100.0, 100.0, 200.0, 24.0);
    let switch_area = Rect::new(100.0, 140.0, 200.0, 24.0);
    let mut draw = |harness: &mut Headless, events: &[Event]| {
        harness.frame(events, |frame| {
            let check = frame.checkbox("Check", check_area, "Check", &mut checked);
            let switch = frame.switch("Switch", switch_area, "Switch", &mut on);
            (check.changed, switch.changed)
        })
    };
    draw(&mut harness, &[]);
    assert_eq!(draw(&mut harness, &click(check_area.center())), (true, false));
    assert_eq!(draw(&mut harness, &click(switch_area.center())), (false, true));
    assert_eq!(draw(&mut harness, &[key(VK_SPACE)]), (false, true));
    drop(draw);
    assert_eq!((checked, on), (true, true));
}

#[test]
fn radio_buttons_and_tabs_pick_on_a_press_and_with_the_arrow_keys() {
    let mut harness = Headless::default();
    let (mut size, mut tab) = (0, 0);
    let radio_area = Rect::new(100.0, 100.0, 200.0, 100.0);
    let tabs_area = Rect::new(400.0, 100.0, 300.0, 30.0);
    let mut draw = |harness: &mut Headless, events: &[Event]| {
        harness.frame(events, |frame| {
            frame.radio("Size", radio_area, &["Small", "Medium", "Large"], &mut size);
            frame.tabs("Tabs", tabs_area, &["One", "Two", "Three"], &mut tab);
        })
    };
    draw(&mut harness, &[]);
    let large = harness.center("Size/Large");
    draw(&mut harness, &click(large));
    // A press on a part gives the group focus.
    assert_eq!(harness.interface.focus().as_deref(), Some("Size"));
    draw(&mut harness, &[key(VK_UP)]);
    let two = harness.center("Tabs/Two");
    draw(&mut harness, &click(two));
    draw(&mut harness, &[key(VK_RIGHT), key(VK_RIGHT)]);
    drop(draw);
    assert_eq!((size, tab), (1, 2));
}

#[test]
fn chips_flip_on_a_press() {
    let mut harness = Headless::default();
    let mut values = [false, true, false];
    let labels = ["Red", "Green", "Blue"];
    let area = Rect::new(100.0, 100.0, 400.0, 100.0);
    let mut draw = |harness: &mut Headless, events: &[Event]| {
        harness.frame(events, |frame| frame.chip_switches("Colors", area, &labels, &mut values).0)
    };
    draw(&mut harness, &[]);
    let red = harness.center("Colors/Red");
    let green = harness.center("Colors/Green");
    assert!(draw(&mut harness, &click(red)));
    assert!(draw(&mut harness, &click(green)));
    drop(draw);
    assert_eq!(values, [true, false, false]);
}

const SLIDER: Rect = Rect { x: 100.0, y: 100.0, width: 432.0, height: 40.0 };

fn slide(harness: &mut Headless, value: &mut f64, range: &SliderRange, events: &[Event]) -> bool {
    harness.frame(events, |frame| frame.slider("Slider", SLIDER, value, range).changed)
}

#[test]
fn a_slider_sets_its_value_from_the_press_and_stays_on_its_steps() {
    let mut harness = Headless::default();
    let range = SliderRange {
        start: 0.0,
        end: 1.0,
        minimum: 0.001,
        maximum: 0.999,
        steps: 1000.0,
    };
    let mut value = 0.5;
    let [left, right] = track(SLIDER);
    slide(&mut harness, &mut value, &range, &[]);
    slide(&mut harness, &mut value, &range, &click([left + (right - left) * 0.25, 120.0]));
    assert_eq!(value, 0.25);
    // Past the ends, it stops at the first and last steps in its range.
    slide(&mut harness, &mut value, &range, &click([left - 10.0, 120.0]));
    assert_eq!(value, 0.001);
    slide(&mut harness, &mut value, &range, &click([right + 10.0, 120.0]));
    assert_eq!(value, 0.999);
}

#[test]
fn a_slider_steps_with_the_wheel_and_the_arrow_keys() {
    let mut harness = Headless::default();
    let range = SliderRange::new(0.0, 360.0, 10.0);
    let mut value = 45.0;
    let middle = SLIDER.center();
    slide(&mut harness, &mut value, &range, &[]);
    slide(&mut harness, &mut value, &range, &[Event::Wheel(middle, 2.0)]);
    assert_eq!(value, 45.2);
    // Parts of a notch add up.
    slide(&mut harness, &mut value, &range, &[Event::Wheel(middle, -0.5)]);
    assert_eq!(value, 45.2);
    slide(&mut harness, &mut value, &range, &[Event::Wheel(middle, -0.5)]);
    assert_eq!(value, 45.1);
    // The keys need focus, which Tab gives it.
    slide(&mut harness, &mut value, &range, &[key(VK_RIGHT)]);
    assert_eq!(value, 45.1);
    slide(&mut harness, &mut value, &range, &[key(VK_TAB), key(VK_RIGHT), shifted(VK_RIGHT)]);
    assert_eq!(value, 46.2);
    slide(&mut harness, &mut value, &range, &[key(VK_END)]);
    assert_eq!(value, 360.0);
    slide(&mut harness, &mut value, &range, &[key(VK_HOME), key(VK_LEFT)]);
    assert_eq!(value, 0.0);
}

#[test]
fn a_value_past_the_range_stays_put_until_it_can_move_into_it() {
    let range = SliderRange {
        start: 0.0,
        end: 1.0,
        minimum: 0.001,
        maximum: 0.999,
        steps: 1000.0,
    };
    assert_eq!(range.nudge(0.9990234375, 1.0), 0.9990234375);
    assert_eq!(range.nudge(0.9990234375, -1.0), 0.998);
    assert_eq!(range.nearest(0.1074), 0.107);
}

const BOX: Rect = Rect { x: 100.0, y: 100.0, width: 200.0, height: 23.0 };
const CHOICES: [Choice; 3] = [
    Choice { label: "Alpha", detail: "first" },
    Choice { label: "Beta", detail: "second" },
    Choice { label: "Gamma", detail: "third" },
];

fn pick(harness: &mut Headless, selected: &mut usize, events: &[Event]) -> bool {
    harness.frame(events, |frame| {
        let changed = frame.select("Pick", BOX, &CHOICES, selected).changed;
        let under = frame.button("Under", Rect::new(100.0, 150.0, 500.0, 30.0), "Under").clicked;
        changed || under
    })
}

#[test]
fn a_select_box_opens_its_list_and_a_press_on_a_choice_picks_it() {
    let mut harness = Headless::default();
    let mut selected = 0;
    pick(&mut harness, &mut selected, &[]);
    pick(&mut harness, &mut selected, &click(BOX.center()));
    assert!(harness.interface.popup().is_some());
    // The list covers the button under the box.
    let gamma = harness.center("Pick/2");
    let list = harness.target("Pick/List");
    assert!(list.y >= BOX.bottom() && list.width >= BOX.width);
    assert!(pick(&mut harness, &mut selected, &click(gamma)));
    assert_eq!(selected, 2);
    assert_eq!(harness.interface.popup(), None);
}

#[test]
fn a_press_off_the_open_list_only_closes_it() {
    let mut harness = Headless::default();
    let mut selected = 0;
    pick(&mut harness, &mut selected, &[]);
    pick(&mut harness, &mut selected, &click(BOX.center()));
    // The button is under the list, so the press is off it at its far end.
    let under_end = [550.0, 165.0];
    assert!(!harness.target("Pick/List").contains(under_end));
    assert!(!pick(&mut harness, &mut selected, &click(under_end)));
    assert_eq!(harness.interface.popup(), None);
    // Now the button takes it.
    assert!(pick(&mut harness, &mut selected, &click(under_end)));
}

#[test]
fn a_select_box_takes_keys_with_focus_and_esc_closes_it() {
    let mut harness = Headless::default();
    let mut selected = 0;
    pick(&mut harness, &mut selected, &[]);
    pick(&mut harness, &mut selected, &[key(VK_TAB), key(VK_DOWN), key(VK_DOWN), key(VK_DOWN)]);
    assert_eq!(selected, 2);
    pick(&mut harness, &mut selected, &[key(VK_SPACE)]);
    assert!(harness.interface.popup().is_some());
    pick(&mut harness, &mut selected, &[key(VK_HOME)]);
    assert_eq!(selected, 0);
    pick(&mut harness, &mut selected, &[key(VK_ESCAPE)]);
    assert_eq!(harness.interface.popup(), None);
    assert_eq!(harness.interface.focus().as_deref(), Some("Pick"));
}

#[test]
fn a_select_box_that_takes_no_focus_gets_none_from_its_choices() {
    let mut harness = Headless::default();
    let mut selected = 0;
    let mut draw = |harness: &mut Headless, events: &[Event]| {
        harness.frame(events, |frame| {
            frame.set_focusable(false);
            frame.select("Pick", BOX, &CHOICES, &mut selected);
            frame.shortcut(VK_DOWN)
        })
    };
    draw(&mut harness, &[]);
    draw(&mut harness, &click(BOX.center()));
    let beta = harness.center("Pick/1");
    draw(&mut harness, &click(beta));
    assert_eq!(harness.interface.focus(), None);
    // So Down is the program's, and leaves the choice alone.
    assert!(draw(&mut harness, &[key(VK_DOWN)]));
    drop(draw);
    assert_eq!(selected, 1);
}

#[test]
fn the_state_has_the_choice_of_the_frame_that_made_it() {
    let mut harness = Headless::default();
    let mut selected = 0;
    let label = |harness: &Headless| {
        let state = harness.interface.state();
        let Some(json::Value::Array(targets)) = state.get("Targets") else {
            panic!("no targets");
        };
        let pick = targets.iter().find(|target| target.get("Name") == Some(&json::Value::from("Pick"))).unwrap();
        pick.get("Label").cloned()
    };
    pick(&mut harness, &mut selected, &[]);
    pick(&mut harness, &mut selected, &[key(VK_TAB), key(VK_DOWN)]);
    assert_eq!(label(&harness), Some(json::Value::from("Beta")));
}

#[test]
fn a_select_list_opens_above_the_box_when_there_is_no_room_under_it() {
    let mut harness = Headless::default();
    let mut selected = 0;
    let low = Rect { y: HEIGHT - 40.0, ..BOX };
    let draw = |frame: &mut Frame, selected: &mut usize| {
        frame.select("Pick", low, &CHOICES, selected);
    };
    harness.frame(&[], |frame| draw(frame, &mut selected));
    harness.frame(&click(low.center()), |frame| draw(frame, &mut selected));
    let list = harness.target("Pick/List");
    assert!(list.bottom() <= low.y);
}

const FIELD: Rect = Rect { x: 100.0, y: 100.0, width: 200.0, height: 30.0 };

fn edit(harness: &mut Headless, text: &mut String, events: &[Event]) -> (bool, bool) {
    harness.frame(events, |frame| {
        let response = frame.text_field("Name", FIELD, text, "Your name", 32);
        (response.submitted, frame.shortcut('A' as i32) || frame.shortcut(VK_F11))
    })
}

#[test]
fn a_text_field_takes_typing_and_editing_keys_with_focus() {
    let mut harness = Headless::default();
    let mut text = String::new();
    edit(&mut harness, &mut text, &[]);
    // Without focus, typing is the program's.
    assert_eq!(edit(&mut harness, &mut text, &[key('A' as i32)]), (false, true));
    edit(&mut harness, &mut text, &click(FIELD.center()));
    let mut events = vec![key('A' as i32)];
    events.extend(typed("Ada Lovelace"));
    assert_eq!(edit(&mut harness, &mut text, &events), (false, false));
    assert_eq!(text, "Ada Lovelace");
    edit(&mut harness, &mut text, &[key(VK_HOME), key(VK_DELETE), key(VK_END), key(VK_BACK), key(VK_LEFT)]);
    assert_eq!(text, "da Lovelac");
    // The caret is before the c.
    edit(&mut harness, &mut text, &typed("\u{E9}"));
    assert_eq!(text, "da Lovela\u{E9}c");
    // The font has no glyph for these, so they aren't taken.
    edit(&mut harness, &mut text, &typed("\u{4E2D}\u{1F600}"));
    assert_eq!(text, "da Lovela\u{E9}c");
    // F11 is still the program's.
    assert_eq!(edit(&mut harness, &mut text, &[key(VK_F11)]), (false, true));
    assert_eq!(edit(&mut harness, &mut text, &[key(VK_RETURN)]), (true, false));
}

#[test]
fn a_press_puts_the_caret_at_the_nearest_character() {
    let mut harness = Headless::default();
    let mut text = "abcdef".to_string();
    edit(&mut harness, &mut text, &[]);
    let char_width = headless::font().cell_width as f32;
    let inset = (headless::font().cell_height as f32 * 0.4).round();
    // Between c and d.
    let x = FIELD.x + inset + char_width * 3.0 + 1.0;
    edit(&mut harness, &mut text, &click([x, FIELD.center()[1]]));
    edit(&mut harness, &mut text, &typed("-"));
    assert_eq!(text, "abc-def");
}

#[test]
fn a_text_field_takes_at_most_its_limit() {
    let mut harness = Headless::default();
    let mut text = String::new();
    edit(&mut harness, &mut text, &[]);
    edit(&mut harness, &mut text, &click(FIELD.center()));
    edit(&mut harness, &mut text, &typed(&"x".repeat(40)));
    assert_eq!(text.len(), 32);
}

const LIST: Rect = Rect { x: 100.0, y: 100.0, width: 300.0, height: 202.0 };

fn browse(harness: &mut Headless, selected: &mut Option<usize>, events: &[Event]) -> user_interface::ListView {
    harness.frame(events, |frame| frame.list("Items", LIST, 100, 20.0, selected))
}

#[test]
fn a_list_scrolls_with_the_wheel_three_rows_a_notch() {
    let mut harness = Headless::default();
    let mut selected = None;
    let view = browse(&mut harness, &mut selected, &[]);
    assert_eq!((view.first, view.visible, view.rows.len()), (0, 10, 10));
    let view = browse(&mut harness, &mut selected, &[Event::Wheel(LIST.center(), -2.0)]);
    assert_eq!(view.first, 6);
    let view = browse(&mut harness, &mut selected, &[Event::Wheel(LIST.center(), 5.0)]);
    assert_eq!(view.first, 0);
    let view = browse(&mut harness, &mut selected, &[Event::Wheel(LIST.center(), -100.0)]);
    assert_eq!((view.first, view.rows.last().map(|(index, _)| *index)), (90, Some(99)));
}

#[test]
fn a_press_on_a_row_selects_it_and_the_keys_move_the_selection_into_view() {
    let mut harness = Headless::default();
    let mut selected = None;
    browse(&mut harness, &mut selected, &[]);
    let row = harness.center("Items/3");
    let view = browse(&mut harness, &mut selected, &click(row));
    assert!(view.response.changed);
    assert_eq!(selected, Some(3));
    assert_eq!(harness.interface.focus().as_deref(), Some("Items"));
    let downs = vec![key(VK_DOWN); 10];
    let view = browse(&mut harness, &mut selected, &downs);
    assert_eq!((selected, view.first), (Some(13), 4));
    let view = browse(&mut harness, &mut selected, &[key(VK_HOME)]);
    assert_eq!((selected, view.first), (Some(0), 0));
}

#[test]
fn dragging_the_scroll_bar_scrolls_the_list() {
    let mut harness = Headless::default();
    let mut selected = None;
    browse(&mut harness, &mut selected, &[]);
    let bar = harness.target("Items/Scroll bar");
    // The thumb is at the top. Grab it, and drag it to the bottom.
    let grab = [bar.center()[0], bar.y + 5.0];
    browse(&mut harness, &mut selected, &[Event::Press(grab)]);
    harness.move_to([bar.center()[0], bar.bottom() + 100.0]);
    let view = browse(&mut harness, &mut selected, &[]);
    assert_eq!(view.first, 90);
    // A press on the track puts the thumb's middle there.
    browse(&mut harness, &mut selected, &[Event::Release([0.0, 0.0])]);
    let view = browse(&mut harness, &mut selected, &click([bar.center()[0], bar.center()[1]]));
    assert!((40..=50).contains(&view.first), "{}", view.first);
}

#[test]
fn a_tooltip_goes_under_the_cursor_or_above_it_inside_the_window() {
    let mut harness = Headless::default();
    let anchor = Rect::new(100.0, 100.0, 100.0, 20.0);
    let show = |harness: &mut Headless, cursor: [f32; 2]| {
        harness.move_to(cursor);
        harness.frame(&[], |frame| frame.tooltip_text("Tip", anchor, "Some help"))
    };
    let under = show(&mut harness, [150.0, 110.0]);
    assert!(under.y > 110.0 && under.x > 150.0);
    let low = show(&mut harness, [150.0, HEIGHT - 20.0]);
    assert!(low.bottom() < HEIGHT - 20.0);
    let right = show(&mut harness, [WIDTH - 5.0, 110.0]);
    assert!(right.right() <= WIDTH - user_interface::MARGIN);
    assert_eq!(harness.interface.tooltip().map(|(name, _)| name).as_deref(), Some("Tip"));
}
