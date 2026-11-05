// What input does to each widget: checkboxes, switches, radio buttons,
// segmented controls, tabs, filter chips, sliders, combo boxes, text fields,
// lists, info bars and tooltips.

use application::Event;
use user_interface::headless::{self, Headless, MAXIMIZED, click, control, control_shifted, key, shifted, typed};
use user_interface::{Button, Choice, Frame, ListView, Rect, Severity, SliderRange, TextField, TextStyle, slider_rail};
use windows_api::*;

const WIDTH: f32 = MAXIMIZED[0];
const HEIGHT: f32 = MAXIMIZED[1];

#[test]
fn a_checkbox_and_a_switch_flip_on_a_press_or_space() {
    let mut harness = Headless::default();
    let (mut checked, mut on) = (false, true);
    let check_area = Rect::new(100.0, 100.0, 200.0, 32.0);
    let switch_area = Rect::new(100.0, 140.0, 200.0, 32.0);
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
fn radio_buttons_segments_and_tabs_pick_on_a_press_and_with_the_arrow_keys() {
    let mut harness = Headless::default();
    let (mut size, mut view, mut tab) = (0, 0, 0);
    let radio_area = Rect::new(100.0, 100.0, 200.0, 96.0);
    let segmented_area = Rect::new(400.0, 100.0, 300.0, 32.0);
    let tabs_area = Rect::new(800.0, 100.0, 400.0, 40.0);
    let mut draw = |harness: &mut Headless, events: &[Event]| {
        harness.frame(events, |frame| {
            frame.radio("Size", radio_area, &["Small", "Medium", "Large"], &mut size);
            frame.segmented("View", segmented_area, &["Day", "Week", "Month"], &mut view);
            frame.tabs("Tabs", tabs_area, &["One", "Two", "Three"], &mut tab);
        })
    };
    draw(&mut harness, &[]);
    let large = harness.center("Size/Large");
    draw(&mut harness, &click(large));
    // A press on a part gives the group focus.
    assert_eq!(harness.interface.focus().as_deref(), Some("Size"));
    draw(&mut harness, &[key(VK_UP)]);
    let month = harness.center("View/Month");
    draw(&mut harness, &click(month));
    draw(&mut harness, &[key(VK_LEFT), key(VK_HOME), key(VK_RIGHT)]);
    let two = harness.center("Tabs/Two");
    draw(&mut harness, &click(two));
    draw(&mut harness, &[key(VK_RIGHT), key(VK_RIGHT)]);
    drop(draw);
    assert_eq!((size, view, tab), (1, 1, 2));
    // Tabs are as wide as their labels, from the left.
    let one = harness.target("Tabs/One");
    let three = harness.target("Tabs/Three");
    assert_eq!(one.x, tabs_area.x);
    assert!(three.width > one.width);
}

#[test]
fn filter_chips_flip_on_a_press_and_grow_a_check_while_on() {
    let mut harness = Headless::default();
    let mut values = [false, true, false];
    let labels = ["Red", "Green", "Blue"];
    let area = Rect::new(100.0, 100.0, 400.0, 100.0);
    let mut draw = |harness: &mut Headless, events: &[Event]| {
        harness.frame(events, |frame| frame.filter_chips("Colors", area, &labels, &mut values).0)
    };
    draw(&mut harness, &[]);
    let red = harness.target("Colors/Red");
    let green = harness.center("Colors/Green");
    assert!(draw(&mut harness, &click(red.center())));
    assert!(harness.target("Colors/Red").width > red.width);
    assert!(draw(&mut harness, &click(green)));
    drop(draw);
    assert_eq!(values, [true, false, false]);
}

const SLIDER: Rect = Rect { x: 100.0, y: 100.0, width: 420.0, height: 32.0 };

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
    let [left, right] = slider_rail(SLIDER);
    slide(&mut harness, &mut value, &range, &[]);
    slide(&mut harness, &mut value, &range, &click([left + (right - left) * 0.25, 116.0]));
    assert_eq!(value, 0.25);
    // Past the ends, it stops at the first and last steps in its range.
    slide(&mut harness, &mut value, &range, &click([SLIDER.x + 1.0, 116.0]));
    assert_eq!(value, 0.001);
    slide(&mut harness, &mut value, &range, &click([SLIDER.right() - 1.0, 116.0]));
    assert_eq!(value, 0.999);
}

#[test]
fn a_slider_steps_with_the_wheel_and_the_keys() {
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
    slide(&mut harness, &mut value, &range, &[key(VK_TAB), key(VK_RIGHT), shifted(VK_RIGHT), key(VK_UP)]);
    assert_eq!(value, 46.3);
    slide(&mut harness, &mut value, &range, &[key(VK_PRIOR)]);
    assert_eq!(value, 47.3);
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

const BOX: Rect = Rect { x: 100.0, y: 100.0, width: 200.0, height: 32.0 };
const CHOICES: [Choice; 3] = [
    Choice { label: "Alpha", detail: "first" },
    Choice { label: "Beta", detail: "second" },
    Choice { label: "Gamma", detail: "third" },
];

fn pick(harness: &mut Headless, selected: &mut usize, events: &[Event]) -> bool {
    harness.frame(events, |frame| {
        let changed = frame.select("Pick", BOX, &CHOICES, selected).changed;
        let under = frame.button("Under", Rect::new(100.0, 150.0, 500.0, 32.0), Button::new("Under")).clicked;
        changed || under
    })
}

#[test]
fn a_combo_box_opens_its_list_and_a_press_on_a_choice_picks_it() {
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
    let under_end = [580.0, 166.0];
    assert!(!harness.target("Pick/List").contains(under_end));
    assert!(!pick(&mut harness, &mut selected, &click(under_end)));
    assert_eq!(harness.interface.popup(), None);
    // Now the button takes it.
    assert!(pick(&mut harness, &mut selected, &click(under_end)));
}

#[test]
fn a_combo_box_takes_keys_with_focus_and_esc_closes_it() {
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
fn a_combo_box_that_takes_no_focus_gets_none_from_its_choices() {
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
fn a_combo_box_list_opens_above_the_box_when_there_is_no_room_under_it() {
    let mut harness = Headless::default();
    let mut selected = 0;
    let low = Rect { y: HEIGHT - 48.0, ..BOX };
    let draw = |frame: &mut Frame, selected: &mut usize| {
        frame.select("Pick", low, &CHOICES, selected);
    };
    harness.frame(&[], |frame| draw(frame, &mut selected));
    harness.frame(&click(low.center()), |frame| draw(frame, &mut selected));
    let list = harness.target("Pick/List");
    assert!(list.bottom() <= low.y);
}

const FIELD: Rect = Rect { x: 100.0, y: 100.0, width: 300.0, height: 32.0 };

fn edit(harness: &mut Headless, text: &mut String, events: &[Event]) -> (bool, bool) {
    harness.frame(events, |frame| {
        let response = frame.text_field("Name", FIELD, text, TextField::new("Your name").limit(32));
        (response.submitted, frame.shortcut('A' as i32) || frame.shortcut(VK_F11))
    })
}

// A text field with focus, holding `text`.
fn focused_field(text: &str) -> (Headless, String) {
    let mut harness = Headless::default();
    let mut text = text.to_string();
    edit(&mut harness, &mut text, &[]);
    edit(&mut harness, &mut text, &click(FIELD.center()));
    (harness, text)
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
    // The fonts have no glyph for these, so they aren't taken.
    edit(&mut harness, &mut text, &typed("\u{4E2D}\u{1F600}"));
    assert_eq!(text, "da Lovela\u{E9}c");
    // F11 is still the program's.
    assert_eq!(edit(&mut harness, &mut text, &[key(VK_F11)]), (false, true));
    assert_eq!(edit(&mut harness, &mut text, &[key(VK_RETURN)]), (true, false));
}

#[test]
fn a_press_puts_the_caret_at_the_nearest_place_between_characters() {
    let mut harness = Headless::default();
    let mut text = "abcdef".to_string();
    edit(&mut harness, &mut text, &[]);
    let fonts = headless::fonts();
    // Just past the middle of d, which is nearer its end.
    let left = FIELD.x + user_interface::TEXT_INSET - 1.0;
    let x = left + fonts.measure("abc", TextStyle::Body) + fonts.measure("d", TextStyle::Body) * 0.6;
    edit(&mut harness, &mut text, &click([x, FIELD.center()[1]]));
    edit(&mut harness, &mut text, &typed("-"));
    assert_eq!(text, "abcd-ef");
}

#[test]
fn a_text_field_takes_at_most_its_limit() {
    let (mut harness, mut text) = focused_field("");
    edit(&mut harness, &mut text, &typed(&"x".repeat(40)));
    assert_eq!(text.len(), 32);
}

#[test]
fn shift_selects_and_typing_replaces_the_selection() {
    let (mut harness, mut text) = focused_field("one two three");
    edit(&mut harness, &mut text, &[key(VK_END), shifted(VK_LEFT), shifted(VK_LEFT)]);
    edit(&mut harness, &mut text, &typed("EE"));
    assert_eq!(text, "one two thrEE");
    // Left with a selection goes to its start, and Right to its end.
    edit(&mut harness, &mut text, &[key(VK_HOME), shifted(VK_RIGHT), shifted(VK_RIGHT), key(VK_RIGHT)]);
    edit(&mut harness, &mut text, &typed("_"));
    assert_eq!(text, "on_e two thrEE");
    edit(&mut harness, &mut text, &[control('A' as i32)]);
    edit(&mut harness, &mut text, &[key(VK_DELETE)]);
    assert_eq!(text, "");
}

#[test]
fn control_moves_and_deletes_by_words() {
    let (mut harness, mut text) = focused_field("alpha beta  gamma");
    edit(&mut harness, &mut text, &[key(VK_END), control(VK_LEFT), control(VK_LEFT)]);
    edit(&mut harness, &mut text, &typed("<"));
    assert_eq!(text, "alpha <beta  gamma");
    edit(&mut harness, &mut text, &[control(VK_RIGHT)]);
    edit(&mut harness, &mut text, &typed(">"));
    assert_eq!(text, "alpha <beta  >gamma");
    // Ctrl+Backspace deletes the word before the caret, and then the
    // spaces and the word before them.
    edit(&mut harness, &mut text, &[control(VK_BACK)]);
    assert_eq!(text, "alpha <beta  gamma");
    edit(&mut harness, &mut text, &[control(VK_BACK)]);
    assert_eq!(text, "alpha gamma");
    // Ctrl+Delete deletes the word after the caret, and the spaces after it.
    edit(&mut harness, &mut text, &[key(VK_HOME), control(VK_DELETE)]);
    assert_eq!(text, "gamma");
    edit(&mut harness, &mut text, &[key(VK_HOME), control_shifted(VK_RIGHT)]);
    edit(&mut harness, &mut text, &typed("!"));
    assert_eq!(text, "!");
}

#[test]
fn copy_cut_and_paste_use_the_interfaces_clipboard() {
    let (mut harness, mut text) = focused_field("copy me");
    // A test's interface has its own clipboard, never Windows'.
    edit(&mut harness, &mut text, &[control('A' as i32), control('C' as i32), key(VK_END)]);
    edit(&mut harness, &mut text, &[control('V' as i32)]);
    assert_eq!(text, "copy mecopy me");
    edit(&mut harness, &mut text, &[key(VK_HOME), control_shifted(VK_RIGHT), control('X' as i32)]);
    assert_eq!(text, "mecopy me");
    assert_eq!(harness.interface.clipboard().get().as_deref(), Some("copy "));
    // Pasted lines become one, and what the fonts lack is left out.
    harness.interface.clipboard().set("a\nb\u{4E2D}c");
    edit(&mut harness, &mut text, &[key(VK_END), control('V' as i32)]);
    assert_eq!(text, "mecopy mea bc");
}

#[test]
fn a_double_click_selects_a_word_and_dragging_selects_from_the_press() {
    let (mut harness, mut text) = focused_field("hello wide world");
    let fonts = headless::fonts();
    let left = FIELD.x + user_interface::TEXT_INSET - 1.0;
    let x_of = |prefix: &str| left + fonts.measure(prefix, TextStyle::Body);
    let middle_y = FIELD.center()[1];
    let in_wide = [x_of("hello wi"), middle_y];
    edit(&mut harness, &mut text, &click(in_wide));
    edit(&mut harness, &mut text, &click(in_wide));
    edit(&mut harness, &mut text, &typed("narrow"));
    assert_eq!(text, "hello narrow world");
    // A drag from the start of "world" to the end selects it. Half a second
    // later, so that the press isn't a third click.
    for _ in 0..40 {
        edit(&mut harness, &mut text, &[]);
    }
    let start = [x_of("hello narrow "), middle_y];
    edit(&mut harness, &mut text, &[Event::Press(start)]);
    harness.move_to([x_of("hello narrow world") + 30.0, middle_y]);
    edit(&mut harness, &mut text, &[]);
    edit(&mut harness, &mut text, &[Event::Release([0.0, 0.0])]);
    edit(&mut harness, &mut text, &typed("there"));
    assert_eq!(text, "hello narrow there");
}

#[test]
fn the_clear_button_empties_the_field_and_keeps_its_focus() {
    let (mut harness, mut text) = focused_field("something");
    // It shows once the field has focus and text.
    edit(&mut harness, &mut text, &[]);
    let clear = harness.center("Name/Clear");
    edit(&mut harness, &mut text, &click(clear));
    assert_eq!(text, "");
    assert_eq!(harness.interface.focus().as_deref(), Some("Name"));
    edit(&mut harness, &mut text, &typed("new"));
    assert_eq!(text, "new");
}

const LIST: Rect = Rect { x: 100.0, y: 100.0, width: 300.0, height: 200.0 };

// Draws a list of 100 rows, 20 pixels tall, and returns what it showed, and
// the rows that it drew.
fn browse(harness: &mut Headless, selected: &mut Option<usize>, events: &[Event]) -> (ListView, Vec<usize>) {
    harness.frame(events, |frame| {
        let mut rows = Vec::new();
        let view = frame.list("Items", LIST, 100, 20.0, selected, |_, row| rows.push(row.index));
        (view, rows)
    })
}

#[test]
fn a_list_scrolls_by_pixels_three_rows_a_notch() {
    let mut harness = Headless::default();
    let mut selected = None;
    let (view, rows) = browse(&mut harness, &mut selected, &[]);
    assert_eq!((view.scroll, view.visible, rows.len()), (0.0, 10, 10));
    let (view, _) = browse(&mut harness, &mut selected, &[Event::Wheel(LIST.center(), -2.0)]);
    assert_eq!(view.scroll, 120.0);
    // Half a notch scrolls half as far, and shows parts of rows at both
    // ends.
    let (view, rows) = browse(&mut harness, &mut selected, &[Event::Wheel(LIST.center(), 0.5)]);
    assert_eq!(view.scroll, 90.0);
    assert_eq!((rows[0], rows.len()), (4, 11));
    let (view, _) = browse(&mut harness, &mut selected, &[Event::Wheel(LIST.center(), 5.0)]);
    assert_eq!(view.scroll, 0.0);
    let (view, rows) = browse(&mut harness, &mut selected, &[Event::Wheel(LIST.center(), -100.0)]);
    assert_eq!((view.scroll, rows.last().copied()), (1800.0, Some(99)));
}

#[test]
fn a_press_on_a_row_selects_it_and_the_keys_move_the_selection_into_view() {
    let mut harness = Headless::default();
    let mut selected = None;
    browse(&mut harness, &mut selected, &[]);
    let row = harness.center("Items/3");
    let (view, _) = browse(&mut harness, &mut selected, &click(row));
    assert!(view.response.changed);
    assert_eq!(selected, Some(3));
    assert_eq!(harness.interface.focus().as_deref(), Some("Items"));
    let downs = vec![key(VK_DOWN); 10];
    let (view, _) = browse(&mut harness, &mut selected, &downs);
    assert_eq!((selected, view.scroll), (Some(13), 80.0));
    let (view, _) = browse(&mut harness, &mut selected, &[key(VK_NEXT)]);
    assert_eq!((selected, view.scroll), (Some(22), 260.0));
    let (view, _) = browse(&mut harness, &mut selected, &[key(VK_HOME)]);
    assert_eq!((selected, view.scroll), (Some(0), 0.0));
}

#[test]
fn a_row_partly_scrolled_out_only_takes_presses_where_it_shows() {
    let mut harness = Headless::default();
    let mut selected = None;
    browse(&mut harness, &mut selected, &[]);
    browse(&mut harness, &mut selected, &[Event::Wheel(LIST.center(), -0.5)]);
    browse(&mut harness, &mut selected, &[]);
    let first = harness.target("Items/1");
    assert_eq!((first.y, first.height), (LIST.y, 10.0));
}

#[test]
fn dragging_the_scroll_bar_scrolls_the_list() {
    let mut harness = Headless::default();
    let mut selected = None;
    browse(&mut harness, &mut selected, &[]);
    let bar = harness.target("Items/Scroll bar");
    // The thumb is at the top. Grab it, and drag it to the bottom.
    let grab = [bar.center()[0], bar.y + 8.0];
    browse(&mut harness, &mut selected, &[Event::Press(grab)]);
    harness.move_to([bar.center()[0], bar.bottom() + 100.0]);
    let (view, _) = browse(&mut harness, &mut selected, &[]);
    assert_eq!(view.scroll, 1800.0);
    // A press on the track puts the thumb's middle there.
    browse(&mut harness, &mut selected, &[Event::Release([0.0, 0.0])]);
    let (view, _) = browse(&mut harness, &mut selected, &click(bar.center()));
    assert!((800.0..=1000.0).contains(&view.scroll), "{}", view.scroll);
}

#[test]
fn an_info_bars_button_closes_it() {
    let mut harness = Headless::default();
    let area = Rect::new(100.0, 100.0, 600.0, 48.0);
    let draw = |frame: &mut Frame| frame.info_bar(Some("Close"), area, Severity::Caution, "Title", "Message").clicked;
    harness.frame(&[], draw);
    let close = harness.target("Close");
    assert!(close.right() <= area.right() && close.x > area.center()[0]);
    assert!(harness.frame(&click(close.center()), draw));
    // Without a name, it has no button.
    harness.frame(&[], |frame| frame.info_bar(None, area, Severity::Info, "", "Message"));
    assert!(harness.interface.target_names().is_empty());
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
    assert!(under.y > 110.0 && under.x >= 140.0);
    let low = show(&mut harness, [150.0, HEIGHT - 20.0]);
    assert!(low.bottom() < HEIGHT - 20.0);
    let right = show(&mut harness, [WIDTH - 5.0, 110.0]);
    assert!(right.right() <= WIDTH - user_interface::MARGIN);
    assert_eq!(harness.interface.tooltip().map(|(name, _)| name).as_deref(), Some("Tip"));
}
