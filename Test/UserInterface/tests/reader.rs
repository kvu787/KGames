// Tests how GameInput's readings become the interface's events: keys, the
// mouse's buttons, wheel and motion, which moves the pointer, the gamepad,
// what the window's focus does to them, which the interface never hears of,
// F1 and F12, and the time of a frame's newest input, which its input
// latency starts at. The readings, the window's size and the focus changes
// are made up, with times on the performance counter.

use user_interface::game_input::*;
use user_interface::input::reader::{Connections, InputReader, WindowState, connections};
use user_interface::FocusChange;
use user_interface::{Event, GamepadButtons, Input, Key, MouseButton, Point, Reading};

const KEYBOARD: usize = 1;
const MOUSE: usize = 2;
const GAMEPAD: usize = 3;
const A: (u8, u32) = (0x41, 0x1E);
const LEFT_SHIFT: (u8, u32) = (0xA0, 0x2A);
const RIGHT_SHIFT: (u8, u32) = (0xA1, 0x36);
const LEFT_CONTROL: (u8, u32) = (0xA2, 0x1D);
const BACKSPACE: (u8, u32) = (0x08, 0x0E);
const F1: (u8, u32) = (0x70, 0x3B);
const F12: (u8, u32) = (0x7B, 0x58);

fn keyboard(time: i64, keys: &[(u8, u32)]) -> (i64, Reading) {
    let keys = keys
        .iter()
        .map(|(virtual_key, scan_code)| GameInputKeyState {
            scanCode: *scan_code,
            codePoint: 0,
            virtualKey: *virtual_key,
            isDeadKey: false,
        })
        .collect();
    let reading = Reading {
        device: Some(KEYBOARD),
        kinds: GameInputKindKeyboard,
        keys,
        ..Default::default()
    };
    (time, reading)
}

fn mouse(time: i64, buttons: GameInputMouseButtons, wheel: i64) -> (i64, Reading) {
    let reading = Reading {
        device: Some(MOUSE),
        kinds: GameInputKindMouse,
        mouse: GameInputMouseState {
            buttons,
            positions: GameInputMouseRelativePosition,
            wheelY: wheel,
            ..Default::default()
        },
        ..Default::default()
    };
    (time, reading)
}

// A reading of a mouse whose motion adds up to the position, in counts.
fn mouse_at(time: i64, buttons: GameInputMouseButtons, x: i64, y: i64) -> (i64, Reading) {
    let (time, mut reading) = mouse(time, buttons, 0);
    reading.mouse.positionX = x;
    reading.mouse.positionY = y;
    (time, reading)
}

// A reading of a mouse with its motion's and its wheel's totals.
fn mouse_with(time: i64, x: i64, y: i64, wheel: i64) -> (i64, Reading) {
    let (time, mut reading) = mouse_at(time, GameInputMouseNone, x, y);
    reading.mouse.wheelY = wheel;
    (time, reading)
}

fn gamepad(time: i64, buttons: GameInputGamepadButtons) -> (i64, Reading) {
    let reading = Reading {
        device: Some(GAMEPAD),
        kinds: GameInputKindGamepad,
        gamepad: GameInputGamepadState {
            buttons,
            ..Default::default()
        },
        ..Default::default()
    };
    (time, reading)
}

// The window, which may get or lose focus at a time, and keeps Windows'
// cursor, as the window says it does at the frame's end: not once it lost
// focus, as the window lets the cursor go as it finds that, before the
// frame's readings are read.
fn window(focus: Option<(i64, bool)>) -> WindowState {
    WindowState {
        focus_changes: focus.map(|(time, gained)| FocusChange { time, gained }).into_iter().collect(),
        keeps_cursor: focus.is_none_or(|(_, gained)| gained),
    }
}

// The window, which doesn't keep Windows' cursor, like while the click that
// gave it focus is held, or once F1 let it go.
fn letting_go(focus: Option<(i64, bool)>) -> WindowState {
    WindowState {
        keeps_cursor: false,
        ..window(focus)
    }
}

fn buttons(events: &[Event]) -> Vec<Event> {
    events.iter().filter(|event| matches!(event, Event::ButtonPressed(_) | Event::ButtonReleased(_))).cloned().collect()
}

fn mouse_events(events: &[Event]) -> Vec<Event> {
    events.iter().filter(|event| matches!(event, Event::MouseMoved { .. } | Event::ButtonPressed(_) | Event::ButtonReleased(_) | Event::Wheel(_))).cloned().collect()
}

fn reader() -> InputReader {
    InputReader::new(true)
}

// A reader whose window got focus at time 1.
fn focused() -> InputReader {
    let mut reader = reader();
    reader.read(&[], &[], &window(Some((1, true))));
    reader
}

// A frame of readings, which may come with a focus change, through the
// reader and into the interface's input, at a time that's both the counter's
// ticks and milliseconds.
fn step(reader: &mut InputReader, input: &mut Input, readings: &[(i64, Reading)], focus: Option<(i64, bool)>, now: i64) {
    let frame = reader.read(readings, &[], &window(focus));
    input.begin(now as f64 / 1000.0, &frame.events);
}

fn keys(events: &[Event]) -> Vec<Event> {
    events
        .iter()
        .filter(|event| matches!(event, Event::KeyPressed { .. } | Event::KeyReleased(_)))
        .cloned()
        .collect()
}

// The reader gives keys, and no text: the interface makes the text from
// them, with Shift as it's held, whatever Windows' keyboard layout is.
#[test]
fn a_key_press_is_its_key_and_the_interface_types_its_text() {
    let mut reader = focused();
    let mut input = Input::default();
    let mut typed = String::new();
    let mut type_frame = |events: &[Event]| {
        for step in input.steps(events) {
            input.begin(0.0, step);
            typed.push_str(&input.text);
        }
    };
    let frame = reader.read(&[keyboard(10, &[A])], &[], &window(None));
    assert_eq!(keys(&frame.events), vec![Event::KeyPressed(Key::A)]);
    type_frame(&frame.events);
    let frame = reader.read(&[keyboard(20, &[]), keyboard(30, &[LEFT_SHIFT]), keyboard(40, &[LEFT_SHIFT, A])], &[], &window(None));
    assert_eq!(keys(&frame.events), vec![Event::KeyReleased(Key::A), Event::KeyPressed(Key::SHIFT), Event::KeyPressed(Key::A)]);
    type_frame(&frame.events);
    assert_eq!(typed, "aA");
}

// Keys that one reading has go down after Shift, Ctrl and Alt, and come up
// before them, whatever their codes, as each step sees what's held as its
// key goes: a Ctrl+Backspace in one reading is that, and not a Backspace,
// which would delete a character in a text field.
#[test]
fn a_reading_s_keys_go_down_after_its_modifiers_and_up_before_them() {
    let mut reader = focused();
    let frame = reader.read(&[keyboard(10, &[LEFT_CONTROL, BACKSPACE])], &[], &window(None));
    let position = |events: &[Event], wanted: Event| events.iter().position(|event| *event == wanted);
    let control = position(&frame.events, Event::KeyPressed(Key::CONTROL));
    let backspace = position(&frame.events, Event::KeyPressed(Key::BACKSPACE));
    assert!(control.is_some() && backspace.is_some() && control < backspace, "{:?}", frame.events);
    // In the interface's steps, the Backspace's has Ctrl held.
    let mut input = Input::default();
    let mut seen = false;
    for step in input.steps(&frame.events) {
        input.begin(0.0, step);
        if input.key_pressed(Key::BACKSPACE) {
            assert!(input.modifiers().control);
            seen = true;
        }
    }
    assert!(seen);
    let frame = reader.read(&[keyboard(20, &[])], &[], &window(None));
    assert_eq!(keys(&frame.events), vec![Event::KeyReleased(Key::BACKSPACE), Event::KeyReleased(Key::CONTROL)]);
}

#[test]
fn shift_is_one_key_on_the_left_and_the_right() {
    let mut reader = focused();
    reader.read(&[keyboard(10, &[LEFT_SHIFT]), keyboard(20, &[LEFT_SHIFT, RIGHT_SHIFT])], &[], &window(None));
    let frame = reader.read(&[keyboard(30, &[RIGHT_SHIFT])], &[], &window(None));
    assert_eq!(keys(&frame.events), Vec::new());
    let frame = reader.read(&[keyboard(40, &[])], &[], &window(None));
    assert_eq!(keys(&frame.events), vec![Event::KeyReleased(Key::SHIFT)]);
}

// GameInput's keyboard readings are the keys held down, and come only when
// they change, so a held key doesn't repeat, however long it's held.
#[test]
fn a_held_key_does_not_repeat() {
    let mut reader = focused();
    let frame = reader.read(&[keyboard(10, &[A])], &[], &window(None));
    assert_eq!(keys(&frame.events), vec![Event::KeyPressed(Key::A)]);
    let frame = reader.read(&[], &[], &window(None));
    assert_eq!(keys(&frame.events), Vec::new());
    let frame = reader.read(&[keyboard(10 + user_interface::clock::frequency() * 2, &[])], &[], &window(None));
    assert_eq!(keys(&frame.events), vec![Event::KeyReleased(Key::A)]);
}

// A gamepad's button pressed and let go of within one frame is a change of
// its buttons, then another, each an event at its time, and the frame's last
// event is the gamepad as it is.
#[test]
fn a_gamepad_button_pressed_and_let_go_of_within_a_frame_is_two_changes() {
    let mut reader = focused();
    let frame = reader.read(&[gamepad(10, GameInputGamepadA), gamepad(20, GameInputGamepadNone), gamepad(30, GameInputGamepadNone)], &[], &window(None));
    let buttons: Vec<Vec<&str>> = frame.events.iter().filter_map(|event| if let Event::Gamepad(state) = event { Some(state.buttons.names()) } else { None }).collect();
    assert_eq!(buttons, [vec!["A"], vec![], vec![]]);
    // Into the interface's input, a step at a time: one step has the press.
    let mut input = Input::default();
    input.begin(0.0, &[Event::Gamepad(Default::default())]);
    let pressed = input.steps(&frame.events).into_iter().filter(|step| {
        input.begin(0.0, step);
        input.gamepad_pressed(GamepadButtons::A)
    });
    assert_eq!(pressed.count(), 1);
}

// GameInput's device events have no times, so a device that's unplugged lets
// go of what it held after the frame's readings. A reading of its in the
// same frame, from before it went, doesn't bring it back with a key or a
// button stuck down.
#[test]
fn a_device_s_reading_in_the_frame_it_is_unplugged_doesn_t_bring_it_back() {
    let mut reader = focused();
    reader.read(&[keyboard(10, &[A]), mouse(11, GameInputMouseLeftButton, 0)], &[], &window(None));
    let frame = reader.read(&[keyboard(15, &[A]), mouse(16, GameInputMouseLeftButton, 0)], &[KEYBOARD, MOUSE], &window(None));
    assert_eq!(keys(&frame.events), vec![Event::KeyReleased(Key::A)]);
    assert!(frame.events.contains(&Event::ButtonReleased(MouseButton::Left)));
    assert!(!frame.events.contains(&Event::ButtonPressed(MouseButton::Left)));
    // Nothing is held any more: the next frame lets go of nothing.
    let frame = reader.read(&[], &[], &window(None));
    assert!(!frame.events.iter().any(|event| matches!(event, Event::KeyReleased(_) | Event::ButtonReleased(_))));
}

// Without focus, keys count for nothing, and one held as focus comes, whose
// press the interface never got, gives no release as it's let go of, and
// counts once it's pressed again. The interface hears nothing of focus.
#[test]
fn without_focus_keys_count_for_nothing_and_ones_held_when_it_comes_count_once_let_go() {
    let mut reader = reader();
    let frame = reader.read(&[keyboard(10, &[A])], &[], &window(None));
    assert_eq!(keys(&frame.events), Vec::new());
    assert_eq!(frame.input_time, None);
    let frame = reader.read(&[], &[], &window(Some((20, true))));
    assert_eq!(keys(&frame.events), Vec::new());
    let frame = reader.read(&[keyboard(30, &[])], &[], &window(None));
    assert_eq!(keys(&frame.events), Vec::new());
    let frame = reader.read(&[keyboard(40, &[A])], &[], &window(None));
    assert_eq!(keys(&frame.events), vec![Event::KeyPressed(Key::A)]);
}

// A key pressed before focus came and let go of after it, in one frame,
// counts for nothing, and one pressed after it counts.
#[test]
fn readings_and_focus_changes_happen_in_the_order_of_their_times() {
    let mut reader = reader();
    let frame = reader.read(&[keyboard(10, &[A]), keyboard(30, &[]), keyboard(40, &[A])], &[], &window(Some((20, true))));
    assert_eq!(keys(&frame.events), vec![Event::KeyPressed(Key::A)]);
}

// Without background input, which the library leaves off, GameInput gives no
// readings while the window doesn't have focus. As focus goes, it gives each
// device a reading at rest, so nothing stays held. On the development
// machine, with GameInput 4.0.306, those readings came a few milliseconds
// before the frame that saw the window lose focus, but either order works. A
// key held as focus went, like the Alt of an Alt+Tab away, then counts again
// once it's pressed with focus back, like for an Alt+F4.
#[test]
fn a_key_held_as_focus_goes_counts_again_once_pressed_with_focus_back() {
    const ALT: (u8, u32) = (0xA4, 0x38);
    const F4: (u8, u32) = (0x73, 0x3E);
    for at_rest in [19, 21] {
        let mut reader = focused();
        let frame = reader.read(&[keyboard(10, &[ALT])], &[], &window(None));
        assert_eq!(keys(&frame.events), vec![Event::KeyPressed(Key::ALT)]);
        // GameInput's reading at rest, and the window losing focus.
        reader.read(&[keyboard(at_rest, &[])], &[], &window(Some((20, false))));
        // GameInput doesn't report keys that are held as focus comes back.
        let frame = reader.read(&[], &[], &window(Some((40, true))));
        assert_eq!(keys(&frame.events), Vec::new());
        let frame = reader.read(&[keyboard(50, &[ALT, F4])], &[], &window(None));
        assert_eq!(
            keys(&frame.events),
            vec![Event::KeyPressed(Key::ALT), Event::KeyPressed(Key::F4)],
            "with GameInput's reading at rest at {at_rest}"
        );
    }
}

// As focus goes, a key held is released on that frame, once, like a mouse
// button held, whether GameInput's reading at rest comes before the window
// loses focus or after. A program that acts on a release, like one that ends
// a drag, sees it either way: on the step of the reading at rest, if it came
// first, or on the step that focus goes in, which the reader lets go of
// everything in.
#[test]
fn a_key_held_as_focus_goes_is_released_once_on_that_frame() {
    const ALT: (u8, u32) = (0xA4, 0x38);
    for at_rest in [19, 21] {
        let mut reader = reader();
        let mut input = Input::default();
        step(&mut reader, &mut input, &[keyboard(10, &[ALT])], Some((1, true)), 10);
        assert!(input.key_down(Key::ALT));
        let frame = reader.read(&[keyboard(at_rest, &[])], &[], &window(Some((20, false))));
        let mut released = Vec::new();
        for events in input.steps(&frame.events) {
            input.begin(0.03, events);
            released.extend(input.keys_released().iter().copied());
        }
        assert_eq!(released, [Key::ALT], "with GameInput's reading at rest at {at_rest}");
        assert!(!input.key_down(Key::ALT), "with GameInput's reading at rest at {at_rest}");
        step(&mut reader, &mut input, &[], None, 40);
        assert_eq!(input.keys_released(), [], "with GameInput's reading at rest at {at_rest}");
    }
}

// As focus comes, GameInput reports a gamepad's whole state, buttons held
// too, unlike keyboards' keys and mice's buttons held from before focus came,
// which it doesn't report at all, held or let go of. On the development
// machine, with GameInput 4.0.306, that reading came before the frame that
// saw the window get focus. The interface ignores a button held then until
// it's let go of, as the library promises.
#[test]
fn a_gamepad_button_held_as_focus_comes_counts_once_let_go_of() {
    let mut reader = focused();
    let mut input = Input::default();
    step(&mut reader, &mut input, &[], None, 5);
    // GameInput's reading at rest as focus goes, then its reading with A
    // held as focus comes back.
    step(&mut reader, &mut input, &[gamepad(19, GameInputGamepadNone)], Some((20, false)), 20);
    step(&mut reader, &mut input, &[gamepad(39, GameInputGamepadA)], Some((40, true)), 40);
    assert!(!input.gamepad.buttons.contains(GamepadButtons::A));
    // Let go of, then pressed: it counts.
    step(&mut reader, &mut input, &[gamepad(50, GameInputGamepadNone)], None, 50);
    assert!(!input.gamepad_released(GamepadButtons::A));
    step(&mut reader, &mut input, &[gamepad(60, GameInputGamepadA)], None, 60);
    assert!(input.gamepad_pressed(GamepadButtons::A));
}

// If GameInput's reading of a gamepad as focus comes is late, and comes in a
// frame after the one that saw the window get focus, its time still says
// that it's from before focus came: a button held in it counts once it's let
// go of, and never as a press.
#[test]
fn a_gamepad_reading_from_before_focus_counts_as_without_it_even_when_it_comes_late() {
    let mut reader = focused();
    let mut input = Input::default();
    step(&mut reader, &mut input, &[gamepad(19, GameInputGamepadNone)], Some((20, false)), 20);
    step(&mut reader, &mut input, &[], Some((40, true)), 40);
    step(&mut reader, &mut input, &[gamepad(39, GameInputGamepadA)], None, 48);
    assert!(!input.gamepad.buttons.contains(GamepadButtons::A) && !input.gamepad_pressed(GamepadButtons::A));
    step(&mut reader, &mut input, &[gamepad(50, GameInputGamepadNone)], None, 50);
    assert!(!input.gamepad_released(GamepadButtons::A));
    step(&mut reader, &mut input, &[gamepad(60, GameInputGamepadA)], None, 60);
    assert!(input.gamepad_pressed(GamepadButtons::A));
}

#[test]
fn the_click_that_gives_the_window_focus_does_only_that() {
    let mut reader = reader();
    let frame = reader.read(
        &[mouse(10, GameInputMouseLeftButton, 0), mouse(30, GameInputMouseNone, 0)],
        &[],
        &window(Some((20, true))),
    );
    assert!(!frame.events.iter().any(|event| matches!(event, Event::ButtonPressed(_) | Event::ButtonReleased(_))));
}

// F1 and F12 are the platform's: the reader gives neither their presses nor
// their releases. F1 lets Windows' cursor go, and keeps it again, and F12
// quits.
#[test]
fn f1_and_f12_are_the_platform_s() {
    let mut reader = focused();
    assert!(reader.captured());
    let frame = reader.read(&[keyboard(10, &[F1]), keyboard(20, &[])], &[], &window(None));
    assert_eq!(keys(&frame.events), Vec::new());
    assert!(!reader.captured() && !frame.quit);
    let frame = reader.read(&[keyboard(30, &[F1]), keyboard(40, &[F1, A]), keyboard(50, &[A])], &[], &window(None));
    assert_eq!(keys(&frame.events), vec![Event::KeyPressed(Key::A)]);
    assert!(reader.captured() && !frame.quit);
    let frame = reader.read(&[keyboard(60, &[F12])], &[], &window(None));
    assert_eq!(keys(&frame.events), vec![Event::KeyReleased(Key::A)]);
    assert!(frame.quit);
    let frame = reader.read(&[keyboard(70, &[])], &[], &window(None));
    assert_eq!(keys(&frame.events), Vec::new());
    assert!(!frame.quit);
}

// Without focus, F1 and F12 do nothing: their presses aren't input.
#[test]
fn f1_and_f12_without_focus_do_nothing() {
    let mut reader = reader();
    let frame = reader.read(&[keyboard(10, &[F1, F12])], &[], &window(None));
    assert!(reader.captured() && !frame.quit);
}

// As F1 lets Windows' cursor go, the mice's input stops: the button that it
// pressed is let go of, on the step of the F1, and what the mice do after it
// doesn't reach the interface, until F1 keeps the cursor again. A button
// held as that happens counts once it's let go of.
#[test]
fn f1_stops_the_mice_s_input_and_lets_go_of_their_buttons() {
    let mut reader = focused();
    reader.read(&[mouse_at(5, GameInputMouseNone, 0, 0)], &[], &window(None));
    let frame = reader.read(&[mouse_at(10, GameInputMouseLeftButton, 0, 0), keyboard(20, &[F1]), mouse_at(30, GameInputMouseLeftButton, 10, 0)], &[], &window(None));
    assert_eq!(mouse_events(&frame.events), [Event::ButtonPressed(MouseButton::Left), Event::ButtonReleased(MouseButton::Left)]);
    assert!(!reader.captured());
    // The window lets the cursor go as the frame ends.
    let frame = reader.read(&[mouse_at(40, GameInputMouseNone, 20, 0), mouse_at(50, GameInputMouseRightButton, 20, 0), mouse(60, GameInputMouseRightButton, 120)], &[], &letting_go(None));
    assert_eq!(mouse_events(&frame.events), Vec::new());
    // F1 again, with the right button held: the window keeps the cursor from
    // the next frame, and the held button counts only once it's let go of.
    let frame = reader.read(&[keyboard(70, &[]), keyboard(80, &[F1])], &[], &letting_go(None));
    assert!(reader.captured());
    assert_eq!(mouse_events(&frame.events), Vec::new());
    let frame = reader.read(&[mouse_at(90, GameInputMouseNone, 20, 0), mouse_at(100, GameInputMouseLeftButton, 20, 0)], &[], &window(None));
    assert_eq!(buttons(&frame.events), [Event::ButtonPressed(MouseButton::Left)]);
}

// The mice's input reaches the interface only while the window keeps
// Windows' cursor: not while the click that gave the window focus is held,
// and the window doesn't keep it yet, though the keys do. And as the window
// lets the cursor go, the buttons the mice pressed are let go of.
#[test]
fn the_mice_reach_the_interface_only_while_the_window_keeps_the_cursor() {
    let mut reader = reader();
    reader.read(&[], &[], &letting_go(Some((1, true))));
    let frame = reader.read(&[mouse_at(10, GameInputMouseNone, 0, 0), mouse_at(20, GameInputMouseNone, 30, 0), keyboard(30, &[A])], &[], &letting_go(None));
    assert_eq!(mouse_events(&frame.events), Vec::new());
    assert_eq!(keys(&frame.events), vec![Event::KeyPressed(Key::A)]);
    let frame = reader.read(&[mouse_at(40, GameInputMouseLeftButton, 30, 0)], &[], &window(None));
    assert_eq!(buttons(&frame.events), [Event::ButtonPressed(MouseButton::Left)]);
    let frame = reader.read(&[], &[], &letting_go(None));
    assert_eq!(buttons(&frame.events), [Event::ButtonReleased(MouseButton::Left)]);
}

// A frame that loses focus keeps the input from before the loss, in the
// stream's order, though the window let the cursor go before the frame's
// readings were read: a click before the loss counts, and what was held is
// let go of after it, buttons first, then keys.
#[test]
fn a_frame_that_loses_focus_keeps_what_came_before_in_order() {
    let mut reader = focused();
    reader.read(&[mouse_at(5, GameInputMouseNone, 0, 0), mouse_at(6, GameInputMouseLeftButton, 0, 0)], &[], &window(None));
    let frame = reader.read(
        &[keyboard(10, &[A]), mouse_at(12, GameInputMouseLeftButton | GameInputMouseRightButton, 10, 0)],
        &[],
        &window(Some((20, false))),
    );
    let kept: Vec<Event> = frame.events.iter().filter(|event| !matches!(event, Event::Gamepad(_))).cloned().collect();
    assert_eq!(
        kept,
        [
            Event::KeyPressed(Key::A),
            Event::MouseMoved { x: 10.0, y: 0.0 },
            Event::ButtonPressed(MouseButton::Right),
            Event::ButtonReleased(MouseButton::Left),
            Event::ButtonReleased(MouseButton::Right),
            Event::KeyReleased(Key::A),
        ]
    );
}

// F12 quits once the frame's input before it has happened: a Ctrl+S just
// before it saves. Nothing after it happens.
#[test]
fn f12_quits_after_what_came_before_it_and_nothing_after_it() {
    const LEFT_CONTROL_S: (u8, u32) = (0x53, 0x1F);
    let mut reader = focused();
    let frame = reader.read(
        &[keyboard(10, &[LEFT_CONTROL]), keyboard(20, &[LEFT_CONTROL, LEFT_CONTROL_S]), keyboard(30, &[LEFT_CONTROL, LEFT_CONTROL_S, F12]), keyboard(40, &[A])],
        &[],
        &window(None),
    );
    assert!(frame.quit);
    assert_eq!(keys(&frame.events), [Event::KeyPressed(Key::CONTROL), Event::KeyPressed(Key::S)]);
}

// The input latency starts at input that reaches the interface: not at the
// mice's while their input is stopped, as while the click that gave the
// window focus is held, or F1 let the cursor go.
#[test]
fn the_mice_s_stopped_input_doesn_t_start_the_input_latency() {
    let mut reader = reader();
    reader.read(&[], &[], &letting_go(Some((1, true))));
    let frame = reader.read(&[mouse_at(10, GameInputMouseNone, 0, 0), mouse_at(20, GameInputMouseNone, 30, 0)], &[], &letting_go(None));
    assert_eq!(frame.input_time, None);
    let frame = reader.read(&[mouse_at(30, GameInputMouseNone, 40, 0), keyboard(40, &[A])], &[], &letting_go(None));
    assert_eq!(frame.input_time, Some(40));
    let frame = reader.read(&[mouse_at(50, GameInputMouseNone, 50, 0)], &[], &window(None));
    assert_eq!(frame.input_time, Some(50));
}

// Focus going lets Windows' cursor go without changing what F1 last chose,
// so focus coming back keeps it again only if F1 chose that.
#[test]
fn focus_doesn_t_change_what_f1_chose() {
    let mut reader = focused();
    reader.read(&[keyboard(10, &[F1]), keyboard(20, &[])], &[], &window(None));
    reader.read(&[], &[], &window(Some((30, false))));
    reader.read(&[], &[], &window(Some((40, true))));
    assert!(!reader.captured());
    reader.read(&[keyboard(50, &[F1]), keyboard(60, &[])], &[], &window(None));
    reader.read(&[], &[], &window(Some((70, false))));
    reader.read(&[], &[], &window(Some((80, true))));
    assert!(reader.captured());
}

// The mice's motion in a frame's events, in counts.
fn motions(events: &[Event]) -> Vec<(f64, f64)> {
    events.iter().filter_map(|event| if let Event::MouseMoved { x, y } = event { Some((*x, *y)) } else { None }).collect()
}

// The reader gives each mouse's motion as GameInput counts it, which the
// interface moves its pointer by (see Test/UserInterface/tests/pointer.rs).
// A mouse's first reading has no motion: its position adds up from wherever
// GameInput started it.
#[test]
fn the_reader_gives_each_mouse_s_motion_in_counts() {
    let mut reader = focused();
    let frame = reader.read(&[mouse_at(10, GameInputMouseNone, 0, 0)], &[], &window(None));
    assert_eq!(motions(&frame.events), []);
    let frame = reader.read(&[mouse_at(20, GameInputMouseNone, 2, -1), mouse_at(21, GameInputMouseNone, 42, -1)], &[], &window(None));
    assert_eq!(motions(&frame.events), [(2.0, -1.0), (40.0, 0.0)]);
}

// GameInput's reading of each mouse as focus goes has no positions, and its
// totals are 0, even if its time puts it before the focus change: it neither
// moves nor turns the wheel. GameInput goes on adding a mouse's motion and
// turns up without focus, so each mouse starts over as focus goes: its first
// reading after focus comes back only says where its totals are, and what
// moved in another program doesn't count. As measured with GameInput 4.0.306:
// totals (200, 150) and wheel 240, a reading at rest with 0s as focus went,
// (30, 0) and 120 without focus, then (-100, 0) with it, read as (130, 150)
// and 360.
#[test]
fn a_mouse_starts_over_across_focus_changes() {
    let mut reader = focused();
    reader.read(&[mouse_with(10, 200, 150, 240)], &[], &window(None));
    let at_rest = (19, Reading {
        device: Some(MOUSE),
        kinds: GameInputKindMouse,
        ..Default::default()
    });
    let frame = reader.read(&[at_rest], &[], &window(Some((20, false))));
    assert!(!frame.events.iter().any(|event| matches!(event, Event::MouseMoved { .. } | Event::Wheel(_))), "{:?}", frame.events);
    let frame = reader.read(&[mouse_with(50, 130, 150, 360)], &[], &window(Some((40, true))));
    assert!(!frame.events.iter().any(|event| matches!(event, Event::MouseMoved { .. } | Event::Wheel(_))), "{:?}", frame.events);
    let frame = reader.read(&[mouse_with(60, 137, 157, 240)], &[], &window(None));
    assert_eq!(motions(&frame.events), [(7.0, 7.0)]);
    assert!(frame.events.contains(&Event::Wheel(Point::new(0.0, -1.0))), "{:?}", frame.events);
}

// A mouse starts over as focus goes even when GameInput's reading at rest is
// lost, as when a frame loses its place in GameInput's stream: what moved and
// turned in another program, which GameInput's totals still have, doesn't
// count.
#[test]
fn a_mouse_starts_over_as_focus_goes_without_its_reading_at_rest() {
    let mut reader = focused();
    reader.read(&[mouse_with(10, 200, 150, 240)], &[], &window(None));
    reader.read(&[], &[], &window(Some((20, false))));
    let frame = reader.read(&[mouse_with(50, 230, 150, 360)], &[], &window(Some((40, true))));
    assert!(!frame.events.iter().any(|event| matches!(event, Event::MouseMoved { .. } | Event::Wheel(_))), "{:?}", frame.events);
}

// Only relative positions are motion: a reading with only absolute ones,
// like a pen's, isn't, and the mouse's next reading with relative ones starts
// over.
#[test]
fn a_reading_with_only_absolute_positions_moves_nothing() {
    let mut reader = focused();
    reader.read(&[mouse_at(10, GameInputMouseNone, 0, 0)], &[], &window(None));
    let (time, mut absolute) = mouse_at(20, GameInputMouseNone, 500, 500);
    absolute.mouse.positions = GameInputMouseAbsolutePosition;
    let frame = reader.read(&[(time, absolute), mouse_at(30, GameInputMouseNone, 510, 500)], &[], &window(None));
    assert_eq!(motions(&frame.events), []);
    let frame = reader.read(&[mouse_at(40, GameInputMouseNone, 520, 500)], &[], &window(None));
    assert_eq!(motions(&frame.events), [(10.0, 0.0)]);
}

// Each mouse's motion comes in the stream's order, so that a press and a
// release each happen where the pointer was as the button went down or up.
#[test]
fn the_mouse_moves_in_the_stream_s_order() {
    let mut reader = focused();
    let frame = reader.read(
        &[
            mouse_at(10, GameInputMouseNone, 5000, 7000),
            mouse_at(11, GameInputMouseLeftButton, 5010, 6995),
            mouse_at(12, GameInputMouseLeftButton, 5030, 6995),
            mouse_at(13, GameInputMouseNone, 5030, 6995),
        ],
        &[],
        &window(None),
    );
    assert_eq!(
        mouse_events(&frame.events),
        vec![
            Event::MouseMoved { x: 10.0, y: -5.0 },
            Event::ButtonPressed(MouseButton::Left),
            Event::MouseMoved { x: 20.0, y: 0.0 },
            Event::ButtonReleased(MouseButton::Left),
        ]
    );
}

// What comes after a move in the stream comes after its motion, like a key
// that acts on what the pointer is over.
#[test]
fn a_key_comes_after_the_motion_before_it() {
    let mut reader = focused();
    let frame = reader.read(
        &[
            mouse_at(10, GameInputMouseNone, 0, 0),
            mouse_at(11, GameInputMouseNone, 10, 0),
            keyboard(12, &[A]),
            mouse_at(13, GameInputMouseNone, 20, 0),
        ],
        &[],
        &window(None),
    );
    let kinds: Vec<&str> = frame
        .events
        .iter()
        .filter_map(|event| match event {
            Event::MouseMoved { .. } => Some("Moved"),
            Event::KeyPressed(_) => Some("Key"),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, ["Moved", "Key", "Moved"]);
}

// Two mice each give their own motion.
#[test]
fn each_mouse_gives_its_own_motion() {
    let mut reader = focused();
    let other = |time: i64, x: i64, y: i64| {
        let (time, mut reading) = mouse_at(time, GameInputMouseNone, x, y);
        reading.device = Some(MOUSE + 10);
        (time, reading)
    };
    reader.read(&[mouse_at(10, GameInputMouseNone, 0, 0), other(11, 900, 900)], &[], &window(None));
    let frame = reader.read(&[mouse_at(20, GameInputMouseNone, 10, 0), other(21, 900, 920)], &[], &window(None));
    assert_eq!(motions(&frame.events), [(10.0, 0.0), (0.0, 20.0)]);
}

// A mouse's motion is a running total, which goes around an i64's ends.
#[test]
fn the_motion_goes_around_an_i64_s_ends() {
    let mut reader = focused();
    reader.read(&[mouse_at(10, GameInputMouseNone, i64::MAX - 1, i64::MIN + 1)], &[], &window(None));
    let frame = reader.read(&[mouse_at(20, GameInputMouseNone, i64::MIN + 1, i64::MAX - 1)], &[], &window(None));
    assert_eq!(motions(&frame.events), [(3.0, -3.0)]);
}

#[test]
fn the_wheel_turns_in_notches_of_120() {
    let mut reader = focused();
    reader.read(&[mouse(10, GameInputMouseNone, 0)], &[], &window(None));
    let frame = reader.read(&[mouse(20, GameInputMouseNone, 240)], &[], &window(None));
    assert!(frame.events.contains(&Event::Wheel(Point::new(0.0, 2.0))));
}

#[test]
fn a_frame_s_input_time_is_its_newest_reading_s() {
    let mut reader = focused();
    let frame = reader.read(&[keyboard(10, &[A]), mouse(30, GameInputMouseNone, 0), keyboard(20, &[])], &[], &window(None));
    assert_eq!(frame.input_time, Some(30));
    let frame = reader.read(&[], &[], &window(None));
    assert_eq!(frame.input_time, None);
}

#[test]
fn the_gamepad_counts_as_input_only_for_a_program_that_reads_it() {
    let mut reader = InputReader::new(false);
    reader.read(&[], &[], &window(Some((1, true))));
    let frame = reader.read(&[gamepad(10, GameInputGamepadA)], &[], &window(None));
    assert_eq!(frame.input_time, None);
    assert!(!frame.events.iter().any(|event| matches!(event, Event::Gamepad(_))));
    let mut reader = focused();
    let frame = reader.read(&[gamepad(10, GameInputGamepadA)], &[], &window(None));
    assert_eq!(frame.input_time, Some(10));
}

#[test]
fn the_gamepad_is_its_newest_reading_until_it_disconnects() {
    let mut reader = focused();
    let frame = reader.read(&[gamepad(10, GameInputGamepadA), gamepad(20, GameInputGamepadB)], &[], &window(None));
    let Some(Event::Gamepad(state)) = frame.events.iter().rev().find(|event| matches!(event, Event::Gamepad(_))) else {
        panic!("no gamepad");
    };
    assert!(state.connected);
    assert_eq!(state.buttons.names(), vec!["B"]);
    let frame = reader.read(&[], &[GAMEPAD], &window(None));
    assert_eq!(frame.events.last(), Some(&Event::Gamepad(Default::default())));
}

#[test]
fn keys_and_buttons_held_on_a_device_that_is_unplugged_are_let_go_of() {
    let mut reader = focused();
    reader.read(&[keyboard(10, &[A]), mouse(11, GameInputMouseLeftButton, 0)], &[], &window(None));
    let frame = reader.read(&[], &[KEYBOARD, MOUSE], &window(None));
    assert_eq!(keys(&frame.events), vec![Event::KeyReleased(Key::A)]);
    assert!(frame.events.contains(&Event::ButtonReleased(MouseButton::Left)));
    // Nothing is held any more, so the next press is a press.
    let frame = reader.read(&[keyboard(30, &[A])], &[], &window(None));
    assert!(frame.events.contains(&Event::KeyPressed(Key::A)));
}

// GameInput's stream, made up, for walking: each reading is a device's and a
// value, in the stream's order, which keeps the newest `kept` readings, as
// GameInput's history does, and each device's newest reading however old,
// as GameInput does. Two readings can have the same value, as two of
// GameInput's readings can have the same timestamp, and are still two
// readings: each is known by where it is, as GameInput's are by their
// objects.
struct Stream {
    readings: Vec<(u8, u32)>,
    kept: usize,
}

impl Stream {
    fn kept(&self, index: usize) -> bool {
        index + self.kept >= self.readings.len()
    }
}

impl user_interface::Stream for Stream {
    type Reading = usize;
    type Device = u8;

    fn current(&self, device: Option<&u8>) -> Result<usize, windows_core::HRESULT> {
        (0..self.readings.len())
            .rev()
            .find(|index| device.is_none_or(|device| self.readings[*index].0 == *device))
            .ok_or(GAMEINPUT_E_READING_NOT_FOUND)
    }

    fn next(&self, after: &usize, device: Option<&u8>) -> Result<usize, windows_core::HRESULT> {
        if !self.kept(*after) {
            return Err(GAMEINPUT_E_REFERENCE_READING_TOO_OLD);
        }
        (*after + 1..self.readings.len())
            .find(|index| device.is_none_or(|device| self.readings[*index].0 == *device))
            .ok_or(GAMEINPUT_E_READING_NOT_FOUND)
    }

    fn previous(&self, before: &usize) -> Result<usize, windows_core::HRESULT> {
        if !self.kept(*before) {
            return Err(GAMEINPUT_E_REFERENCE_READING_TOO_OLD);
        }
        before.checked_sub(1).filter(|index| self.kept(*index)).ok_or(GAMEINPUT_E_READING_NOT_FOUND)
    }

    fn device(&self, reading: &usize) -> Option<u8> {
        Some(self.readings[*reading].0)
    }
}

// A frame's walk, as each reading's device and value, and whether it lost
// its place.
fn walk(walker: &mut user_interface::Walker<Stream>, stream: &Stream, devices: &[u8]) -> (Vec<(u8, u32)>, bool) {
    let walked = walker.walk(stream, devices);
    (walked.readings.iter().map(|index| stream.readings[*index]).collect(), walked.restarted)
}

const PAD: u8 = 1;
const KEYS: u8 = 2;
const MICE: u8 = 3;

// Each frame walks GameInput's stream from the last reading it handed out,
// as Microsoft's GameInputSequential sample does, and hands out every reading
// after it once, in the stream's order, whatever their values: nothing is
// sorted, merged or told apart by its time.
#[test]
fn a_frame_hands_out_every_reading_after_the_last_once_in_the_stream_s_order() {
    let mut stream = Stream { readings: vec![(PAD, 5)], kept: 100 };
    let mut walker = user_interface::Walker::default();
    assert_eq!(walk(&mut walker, &stream, &[PAD, KEYS]), (vec![(PAD, 5)], false));
    // Readings whose times go back, and two the same, keep their order.
    stream.readings.extend([(KEYS, 30), (PAD, 20), (PAD, 20), (KEYS, 40)]);
    assert_eq!(walk(&mut walker, &stream, &[PAD, KEYS]), (vec![(KEYS, 30), (PAD, 20), (PAD, 20), (KEYS, 40)], false));
    // Nothing new: nothing again.
    assert_eq!(walk(&mut walker, &stream, &[PAD, KEYS]), (vec![], false));
}

// The first walk hands out what the history has, in the stream's order,
// after the newest reading of each device that has none in it, which left
// the history, whose order the history no longer has, so that every device's
// state is known from the start.
#[test]
fn the_first_walk_hands_out_the_history() {
    let stream = Stream {
        readings: vec![(MICE, 1), (KEYS, 2), (PAD, 3), (KEYS, 4), (PAD, 5)],
        kept: 4,
    };
    let mut walker = user_interface::Walker::default();
    assert_eq!(
        walk(&mut walker, &stream, &[PAD, KEYS, MICE]),
        (vec![(MICE, 1), (KEYS, 2), (PAD, 3), (KEYS, 4), (PAD, 5)], false)
    );
}

// A frame that took longer than GameInput's history lasts loses its place in
// the stream. It starts over from the history's start: a key let go of in
// the readings that left the history is let go of, rather than held until the
// keyboard reads again, while a gamepad that reads thousands of times a
// second filled the history. Only the readings that left the history are
// lost, and nothing is handed out twice.
#[test]
fn a_frame_that_loses_its_place_starts_over_from_the_history_s_start() {
    let mut stream = Stream { readings: vec![(KEYS, 1)], kept: 3 };
    let mut walker = user_interface::Walker::default();
    assert_eq!(walk(&mut walker, &stream, &[PAD, KEYS]), (vec![(KEYS, 1)], false));
    // The key let go of, then the gamepad's readings, which push it out.
    stream.readings.extend([(KEYS, 0), (PAD, 10), (PAD, 11), (PAD, 12), (PAD, 13), (PAD, 14)]);
    assert_eq!(walk(&mut walker, &stream, &[PAD, KEYS]), (vec![(KEYS, 0), (PAD, 12), (PAD, 13), (PAD, 14)], true));
    stream.readings.push((PAD, 15));
    assert_eq!(walk(&mut walker, &stream, &[PAD, KEYS]), (vec![(PAD, 15)], false));
    // Losing its place again with nothing new from the keyboard: its newest
    // reading went out already, and isn't handed out again.
    stream.readings.extend([(PAD, 16), (PAD, 17), (PAD, 18), (PAD, 19)]);
    assert_eq!(walk(&mut walker, &stream, &[PAD, KEYS]), (vec![(PAD, 17), (PAD, 18), (PAD, 19)], true));
}

// A key tapped while a frame took longer than the history lasts isn't lost
// if the history still has it, though the last reading handed out left it.
#[test]
fn a_tap_still_in_the_history_counts_after_losing_the_place() {
    let mut stream = Stream { readings: vec![(KEYS, 1)], kept: 4 };
    let mut walker = user_interface::Walker::default();
    assert_eq!(walk(&mut walker, &stream, &[PAD, KEYS]), (vec![(KEYS, 1)], false));
    stream.readings.extend([(PAD, 10), (KEYS, 2), (KEYS, 3), (PAD, 11)]);
    assert_eq!(walk(&mut walker, &stream, &[PAD, KEYS]), (vec![(PAD, 10), (KEYS, 2), (KEYS, 3), (PAD, 11)], true));
}

// Until the process has its first reading, GameInput's newest reading isn't
// in the history: it's a device's state at rest, from which GetNextReading
// and GetPreviousReading fail, as measured with GameInput 4.0.306. Each walk
// starts over until the first reading, and the walk that finds the first
// readings hands out all of them, a key's tap among them.
#[test]
fn the_walk_starts_from_the_state_at_rest_that_gameinput_gives_before_any_reading() {
    // GameInput as it starts: a device's state at rest, outside the history,
    // until there are readings.
    struct Starting(Stream);
    const AT_REST: usize = 1000;
    impl user_interface::Stream for Starting {
        type Reading = usize;
        type Device = u8;
        fn current(&self, device: Option<&u8>) -> Result<usize, windows_core::HRESULT> {
            self.0.current(device).or(Ok(AT_REST + device.copied().unwrap_or(MICE) as usize))
        }
        fn next(&self, after: &usize, device: Option<&u8>) -> Result<usize, windows_core::HRESULT> {
            if *after >= AT_REST { Err(GAMEINPUT_E_REFERENCE_READING_TOO_OLD) } else { self.0.next(after, device) }
        }
        fn previous(&self, before: &usize) -> Result<usize, windows_core::HRESULT> {
            if *before >= AT_REST { Err(GAMEINPUT_E_REFERENCE_READING_TOO_OLD) } else { self.0.previous(before) }
        }
        fn device(&self, reading: &usize) -> Option<u8> {
            if *reading >= AT_REST { Some((*reading - AT_REST) as u8) } else { self.0.device(reading) }
        }
    }
    let mut stream = Starting(Stream { readings: Vec::new(), kept: 100 });
    let mut walker = user_interface::Walker::default();
    assert_eq!(walker.walk(&stream, &[KEYS, MICE]).readings, [AT_REST + MICE as usize]);
    assert_eq!(walker.walk(&stream, &[KEYS, MICE]).readings, Vec::<usize>::new());
    // A key's tap and the mouse's motion, in one frame.
    stream.0.readings.extend([(KEYS, 1), (KEYS, 0), (MICE, 5)]);
    assert_eq!(walker.walk(&stream, &[KEYS, MICE]).readings, [0, 1, 2]);
    stream.0.readings.push((KEYS, 7));
    let walked = walker.walk(&stream, &[KEYS, MICE]);
    assert_eq!((walked.readings, walked.restarted), (vec![3], false));
}

// A walk that loses its place with readings after the last one handed out
// still in the history hands out all of those, in the stream's order, and not
// the ones before the last reading handed out.
#[test]
fn a_restart_in_the_history_hands_out_only_what_came_after_the_last_reading() {
    // A stream whose next readings fail, as GetNextReading may for other
    // reasons than age, while the readings are still in the history.
    struct Failing(Stream);
    impl user_interface::Stream for Failing {
        type Reading = usize;
        type Device = u8;
        fn current(&self, device: Option<&u8>) -> Result<usize, windows_core::HRESULT> {
            self.0.current(device)
        }
        fn next(&self, after: &usize, device: Option<&u8>) -> Result<usize, windows_core::HRESULT> {
            if device.is_none() { Err(GAMEINPUT_E_DEVICE_DISCONNECTED) } else { self.0.next(after, device) }
        }
        fn previous(&self, before: &usize) -> Result<usize, windows_core::HRESULT> {
            self.0.previous(before)
        }
        fn device(&self, reading: &usize) -> Option<u8> {
            self.0.device(reading)
        }
    }
    let mut stream = Failing(Stream { readings: vec![(KEYS, 1), (PAD, 2)], kept: 100 });
    let mut walker = user_interface::Walker::default();
    let walked = walker.walk(&stream, &[PAD, KEYS]);
    assert_eq!(walked.readings, [0, 1]);
    stream.0.readings.extend([(PAD, 3), (KEYS, 4), (PAD, 5)]);
    let walked = walker.walk(&stream, &[PAD, KEYS]);
    assert_eq!((walked.readings, walked.restarted), (vec![2, 3, 4], true));
    // Without a newest reading at all, there's nothing to start from.
    let empty = Stream { readings: Vec::new(), kept: 100 };
    let mut walker = user_interface::Walker::default();
    assert_eq!(walk(&mut walker, &empty, &[PAD]), (vec![], false));
}

// A frame's readings keep the stream's order, whatever their times: a key
// pressed, and a button pressed after it in the stream, with an earlier
// time, as two devices' readings may have, are a key and then a button.
#[test]
fn a_frame_s_events_are_in_the_stream_s_order_not_its_times() {
    let mut reader = focused();
    let frame = reader.read(&[keyboard(30, &[A]), mouse(20, GameInputMouseLeftButton, 0)], &[], &window(None));
    let key = frame.events.iter().position(|event| *event == Event::KeyPressed(Key::A));
    let button = frame.events.iter().position(|event| *event == Event::ButtonPressed(MouseButton::Left));
    assert!(key.is_some() && button.is_some() && key < button, "{:?}", frame.events);
}

// A focus change goes among the readings before the first one that's later,
// and the readings keep their order around it: A, pressed before focus went,
// is let go of as it goes, and B, pressed after, is without focus.
#[test]
fn a_focus_change_goes_before_the_first_later_reading() {
    const B: (u8, u32) = (0x42, 0x30);
    let mut reader = focused();
    let frame = reader.read(&[keyboard(10, &[A]), keyboard(30, &[A, B])], &[], &window(Some((20, false))));
    assert_eq!(keys(&frame.events), vec![Event::KeyPressed(Key::A), Event::KeyReleased(Key::A)]);
}

// As focus goes, the gamepad goes to rest, once, and without focus it says
// nothing, however its sticks and buttons move.
#[test]
fn the_gamepad_goes_to_rest_as_focus_goes_and_says_nothing_without_it() {
    let mut reader = focused();
    reader.read(&[gamepad(10, GameInputGamepadA)], &[], &window(None));
    let frame = reader.read(&[], &[], &window(Some((20, false))));
    let gamepads: Vec<&Event> = frame.events.iter().filter(|event| matches!(event, Event::Gamepad(_))).collect();
    let at_rest = user_interface::Gamepad {
        connected: true,
        ..Default::default()
    };
    assert_eq!(gamepads, [&Event::Gamepad(at_rest)]);
    let frame = reader.read(&[gamepad(30, GameInputGamepadB), gamepad(40, GameInputGamepadNone)], &[], &window(None));
    assert!(!frame.events.iter().any(|event| matches!(event, Event::Gamepad(_))), "{:?}", frame.events);
}

// A reading of a device with more than one kind of input, like a keyboard
// with a trackball, is each of them, once.
#[test]
fn a_reading_with_a_keyboard_and_a_mouse_is_both() {
    let mut reader = focused();
    let (time, mut reading) = keyboard(10, &[A]);
    reading.kinds |= GameInputKindMouse;
    reading.mouse.buttons = GameInputMouseLeftButton;
    let frame = reader.read(&[(time, reading)], &[], &window(None));
    assert_eq!(frame.events.iter().filter(|event| **event == Event::KeyPressed(Key::A)).count(), 1);
    assert_eq!(frame.events.iter().filter(|event| **event == Event::ButtonPressed(MouseButton::Left)).count(), 1);
}

// Each gamepad keeps its own state, and the interface's gamepad is all of
// them at once. A button held on one gamepad stays held while another, at
// rest, reads thousands of times a second, as one whose sticks have no dead
// zone does: no reading of one covers the other.
#[test]
fn a_gamepad_at_rest_never_covers_another_in_use() {
    const OTHER: usize = 4;
    let other = |time: i64, buttons: GameInputGamepadButtons, stick: f32| {
        let (time, mut reading) = gamepad(time, buttons);
        reading.device = Some(OTHER);
        reading.gamepad.leftThumbstickX = stick;
        (time, reading)
    };
    let mut reader = focused();
    let mut readings = vec![gamepad(10, GameInputGamepadA)];
    for time in 11..40 {
        readings.push(other(time, GameInputGamepadNone, 0.01 * (time % 3) as f32));
    }
    let frame = reader.read(&readings, &[], &window(None));
    let states: Vec<_> = frame.events.iter().filter_map(|event| if let Event::Gamepad(state) = event { Some(*state) } else { None }).collect();
    // One change, the press, then the frame's last event: no release.
    assert_eq!(states.iter().map(|state| state.buttons.names()).collect::<Vec<_>>(), [vec!["A"], vec!["A"]]);
    // Each stick is the one pushed furthest, and a trigger the one pulled
    // furthest.
    let mut pushed = other(50, GameInputGamepadB, 0.8);
    pushed.1.gamepad.rightTrigger = 0.5;
    let frame = reader.read(&[gamepad(45, GameInputGamepadA), pushed], &[], &window(None));
    let Some(Event::Gamepad(state)) = frame.events.last() else { panic!("no gamepad") };
    assert_eq!((state.buttons.names(), state.left_stick.x, state.right_trigger), (vec!["A", "B"], 0.8, 0.5));
    // One that's unplugged lets go of what only it held.
    let frame = reader.read(&[], &[OTHER], &window(None));
    let Some(Event::Gamepad(state)) = frame.events.last() else { panic!("no gamepad") };
    assert_eq!((state.buttons.names(), state.left_stick.x, state.connected), (vec!["A"], 0.0, true));
    let frame = reader.read(&[], &[GAMEPAD], &window(None));
    assert_eq!(frame.events.last(), Some(&Event::Gamepad(Default::default())));
}

// The interface takes the gamepad event right after focus comes as what's
// held then. The reader sends it as focus comes, so a button pressed after
// focus came, in the same frame, is a press.
#[test]
fn a_gamepad_press_right_after_focus_comes_is_a_press() {
    let mut reader = focused();
    let mut input = Input::default();
    step(&mut reader, &mut input, &[], None, 5);
    step(&mut reader, &mut input, &[gamepad(19, GameInputGamepadNone)], Some((20, false)), 20);
    let frame = reader.read(&[gamepad(45, GameInputGamepadA)], &[], &window(Some((40, true))));
    let pressed = input.steps(&frame.events).into_iter().filter(|events| {
        input.begin(0.05, events);
        input.gamepad_pressed(GamepadButtons::A)
    });
    assert_eq!(pressed.count(), 1);
}

// Keys and mouse buttons go with focus, even when GameInput's reading at rest
// as focus goes is lost, as it is when a frame loses its place in GameInput's
// stream: with focus back, a key that was held is pressed anew, and counts.
#[test]
fn keys_and_buttons_go_with_focus_without_a_reading_at_rest() {
    let mut reader = focused();
    reader.read(&[keyboard(10, &[A]), mouse(11, GameInputMouseLeftButton, 0)], &[], &window(None));
    let frame = reader.read(&[], &[], &window(Some((20, false))));
    assert_eq!(keys(&frame.events), vec![Event::KeyReleased(Key::A)]);
    assert_eq!(buttons(&frame.events), vec![Event::ButtonReleased(MouseButton::Left)]);
    let frame = reader.read(&[keyboard(50, &[A]), mouse(51, GameInputMouseLeftButton, 0)], &[], &window(Some((40, true))));
    assert!(frame.events.contains(&Event::KeyPressed(Key::A)));
    assert!(frame.events.contains(&Event::ButtonPressed(MouseButton::Left)));
}

// A gamepad is connected from the moment it connects, before it reads, and
// one that disconnects and connects again in a frame stays.
#[test]
fn a_gamepad_is_connected_from_when_it_connects() {
    let mut reader = focused();
    reader.gamepad_connected(GAMEPAD);
    let frame = reader.read(&[], &[], &window(None));
    let Some(Event::Gamepad(state)) = frame.events.last() else { panic!("no gamepad") };
    assert!(state.connected);
    let frame = reader.read(&[], &[GAMEPAD], &window(None));
    assert_eq!(frame.events.last(), Some(&Event::Gamepad(Default::default())));
}

// A device's last connection or disconnection in a frame says whether it's
// connected: a gamepad that connects and then changes, like its XInput
// index, in the same frame, is connected, one that disconnects and connects
// again stays, and one that connects and disconnects is gone. A keyboard
// that connects isn't a gamepad.
#[test]
fn a_device_s_last_connection_in_a_frame_counts() {
    let device = |id: usize, kind: GameInputKind| user_interface::Device {
        id,
        supported_input: kind,
        ..Default::default()
    };
    let events = [
        (GameInputDeviceEventConnected, device(5, GameInputKindGamepad)),
        (GameInputDeviceEventXInputIndexUpdated, device(5, GameInputKindGamepad)),
        (GameInputDeviceEventDisconnected, device(6, GameInputKindGamepad)),
        (GameInputDeviceEventConnected, device(6, GameInputKindGamepad)),
        (GameInputDeviceEventConnected, device(7, GameInputKindGamepad)),
        (GameInputDeviceEventDisconnected, device(7, GameInputKindGamepad)),
        (GameInputDeviceEventConnected, device(8, GameInputKindKeyboard)),
        (GameInputDeviceEventKeyboardLayoutUpdated, device(8, GameInputKindKeyboard)),
    ];
    assert_eq!(
        connections(&events),
        Connections {
            gamepads: vec![5, 6],
            disconnected: vec![7],
        }
    );
}

// GameInput starts with its device callback, reads frames, and stops,
// unregistering it. A test doesn't have focus, so with background input off
// it may get no readings.

#[test]
fn the_devices_start_read_frames_and_stop() {
    let mut devices = user_interface::Devices::new().unwrap();
    for _ in 0..3 {
        devices.read(&[]);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(devices.frame(), 3);
}
