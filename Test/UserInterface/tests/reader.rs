// Tests how GameInput's readings become the interface's events: keys and
// their text and repeats, the mouse's buttons and wheel, the gamepad, focus,
// and the time of a frame's newest input, which its input latency starts at.
// The readings, the cursor and the focus changes are made up, with times on
// the performance counter.

use user_interface::game_input::*;
use user_interface::input::reader::{InputReader, WindowState};
use user_interface::FocusChange;
use user_interface::{Event, GamepadButtons, Input, Key, MouseButton, Point, Reading};

const KEYBOARD: usize = 1;
const MOUSE: usize = 2;
const GAMEPAD: usize = 3;
const A: (u8, u32) = (0x41, 0x1E);
const LEFT_SHIFT: (u8, u32) = (0xA0, 0x2A);
const RIGHT_SHIFT: (u8, u32) = (0xA1, 0x36);

fn keyboard(time: i64, keys: &[(u8, u32)]) -> (i64, GameInputKind, Reading) {
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
    (time, GameInputKindKeyboard, reading)
}

fn mouse(time: i64, buttons: GameInputMouseButtons, wheel: i64) -> (i64, GameInputKind, Reading) {
    let reading = Reading {
        device: Some(MOUSE),
        kinds: GameInputKindMouse,
        mouse: GameInputMouseState {
            buttons,
            wheelY: wheel,
            ..Default::default()
        },
        ..Default::default()
    };
    (time, GameInputKindMouse, reading)
}

fn gamepad(time: i64, buttons: GameInputGamepadButtons) -> (i64, GameInputKind, Reading) {
    let reading = Reading {
        device: Some(GAMEPAD),
        kinds: GameInputKindGamepad,
        gamepad: GameInputGamepadState {
            buttons,
            ..Default::default()
        },
        ..Default::default()
    };
    (time, GameInputKindGamepad, reading)
}

// The window with the cursor on it, which may get or lose focus at a time.
fn window(focus: Option<(i64, bool)>) -> WindowState {
    WindowState {
        cursor: Point::new(100.0, 50.0),
        inside: true,
        focus_changes: focus.map(|(time, gained)| FocusChange { time, gained }).into_iter().collect(),
    }
}

fn reader() -> InputReader {
    InputReader::with_repeat(true, 0.5, 10.0)
}

// A reader whose window got focus at time 1.
fn focused() -> InputReader {
    let mut reader = reader();
    reader.read(&[], &[], &window(Some((1, true))), 1);
    reader
}

// A frame of readings, which may come with a focus change, through the
// reader and into the interface's input, at a time that's both the counter's
// ticks and milliseconds.
fn step(reader: &mut InputReader, input: &mut Input, readings: &[(i64, GameInputKind, Reading)], focus: Option<(i64, bool)>, now: i64) {
    let frame = reader.read(readings, &[], &window(focus), now);
    input.begin(now as f64 / 1000.0, &frame.events);
}

fn keys(events: &[Event]) -> Vec<Event> {
    events
        .iter()
        .filter(|event| matches!(event, Event::KeyPressed { .. } | Event::KeyReleased(_) | Event::Text(_)))
        .cloned()
        .collect()
}

#[test]
fn a_key_press_types_the_character_that_the_layout_gives() {
    let mut reader = focused();
    let frame = reader.read(&[keyboard(10, &[A])], &[], &window(None), 10);
    assert_eq!(keys(&frame.events), vec![Event::KeyPressed { key: Key::A, repeat: false }, Event::Text('a')]);
    let frame = reader.read(&[keyboard(20, &[]), keyboard(30, &[LEFT_SHIFT]), keyboard(40, &[LEFT_SHIFT, A])], &[], &window(None), 40);
    assert_eq!(
        keys(&frame.events),
        vec![
            Event::KeyReleased(Key::A),
            Event::KeyPressed { key: Key::SHIFT, repeat: false },
            Event::KeyPressed { key: Key::A, repeat: false },
            Event::Text('A'),
        ]
    );
}

#[test]
fn shift_is_one_key_on_the_left_and_the_right() {
    let mut reader = focused();
    reader.read(&[keyboard(10, &[LEFT_SHIFT]), keyboard(20, &[LEFT_SHIFT, RIGHT_SHIFT])], &[], &window(None), 20);
    let frame = reader.read(&[keyboard(30, &[RIGHT_SHIFT])], &[], &window(None), 30);
    assert_eq!(keys(&frame.events), Vec::new());
    let frame = reader.read(&[keyboard(40, &[])], &[], &window(None), 40);
    assert_eq!(keys(&frame.events), vec![Event::KeyReleased(Key::SHIFT)]);
}

#[test]
fn a_held_key_repeats_after_the_delay_at_the_rate() {
    let second = user_interface::clock::frequency();
    let mut reader = focused();
    reader.read(&[keyboard(10, &[A])], &[], &window(None), 10);
    let frame = reader.read(&[], &[], &window(None), 10 + second * 4 / 10);
    assert_eq!(keys(&frame.events), Vec::new());
    // At half a second, then a tenth of a second apart.
    let frame = reader.read(&[], &[], &window(None), 10 + second * 75 / 100);
    let repeats = frame.events.iter().filter(|event| **event == Event::KeyPressed { key: Key::A, repeat: true }).count();
    assert_eq!(repeats, 3);
    let frame = reader.read(&[keyboard(10 + second, &[])], &[], &window(None), 10 + second * 2);
    assert_eq!(keys(&frame.events), vec![Event::KeyReleased(Key::A)]);
}

#[test]
fn without_focus_keys_count_for_nothing_and_ones_held_when_it_comes_count_once_let_go() {
    let mut reader = reader();
    let frame = reader.read(&[keyboard(10, &[A])], &[], &window(None), 10);
    assert_eq!(keys(&frame.events), Vec::new());
    assert_eq!(frame.input_time, None);
    let frame = reader.read(&[], &[], &window(Some((20, true))), 20);
    assert!(frame.events.contains(&Event::FocusGained(vec![Key::A])));
}

#[test]
fn readings_and_focus_changes_happen_in_the_order_of_their_times() {
    let mut reader = reader();
    // Pressed before focus came, and let go after, in one frame.
    let frame = reader.read(&[keyboard(10, &[A]), keyboard(30, &[])], &[], &window(Some((20, true))), 30);
    assert_eq!(frame.events.iter().filter(|event| matches!(event, Event::FocusGained(_))).count(), 1);
    assert!(frame.events.contains(&Event::FocusGained(vec![Key::A])));
    assert!(!frame.events.contains(&Event::KeyPressed { key: Key::A, repeat: false }));
    assert!(frame.events.contains(&Event::KeyReleased(Key::A)));
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
        let frame = reader.read(&[keyboard(10, &[ALT])], &[], &window(None), 10);
        assert_eq!(keys(&frame.events), vec![Event::KeyPressed { key: Key::ALT, repeat: false }]);
        // GameInput's reading at rest, and the window losing focus.
        reader.read(&[keyboard(at_rest, &[])], &[], &window(Some((20, false))), 30);
        // GameInput doesn't report keys that are held as focus comes back.
        let frame = reader.read(&[], &[], &window(Some((40, true))), 40);
        assert!(frame.events.contains(&Event::FocusGained(Vec::new())));
        let frame = reader.read(&[keyboard(50, &[ALT, F4])], &[], &window(None), 50);
        assert_eq!(
            keys(&frame.events),
            vec![Event::KeyPressed { key: Key::ALT, repeat: false }, Event::KeyPressed { key: Key::F4, repeat: false }],
            "with GameInput's reading at rest at {at_rest}"
        );
    }
}

// As focus goes, a key held is released on that frame, once, like a mouse
// button held, whether GameInput's reading at rest comes before the window
// loses focus or after, and `focus_lost` says why. A program that acts on a
// release, like one that ends a drag, sees it either way.
#[test]
fn a_key_held_as_focus_goes_is_released_once_on_that_frame() {
    const ALT: (u8, u32) = (0xA4, 0x38);
    for at_rest in [19, 21] {
        let mut reader = reader();
        let mut input = Input::default();
        step(&mut reader, &mut input, &[keyboard(10, &[ALT])], Some((1, true)), 10);
        assert!(input.key_down(Key::ALT));
        step(&mut reader, &mut input, &[keyboard(at_rest, &[])], Some((20, false)), 30);
        assert!(input.focus_lost && !input.key_down(Key::ALT), "with GameInput's reading at rest at {at_rest}");
        assert_eq!(input.keys_released(), [Key::ALT], "with GameInput's reading at rest at {at_rest}");
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
    assert!(input.focused);
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
    assert!(input.focused);
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
        30,
    );
    assert!(!frame.events.iter().any(|event| matches!(event, Event::ButtonPressed(_) | Event::ButtonReleased(_))));
}

#[test]
fn a_press_while_the_cursor_is_off_the_window_is_another_window_s() {
    let mut reader = focused();
    let off = WindowState {
        inside: false,
        ..window(None)
    };
    let frame = reader.read(&[mouse(10, GameInputMouseLeftButton, 0)], &[], &off, 10);
    assert!(!frame.events.iter().any(|event| matches!(event, Event::ButtonPressed(_))));
    assert!(frame.events.contains(&Event::PointerLeft));
    let frame = reader.read(&[mouse(20, GameInputMouseNone, 0)], &[], &window(None), 20);
    assert!(!frame.events.iter().any(|event| matches!(event, Event::ButtonReleased(_))));
}

#[test]
fn a_press_on_the_window_is_where_the_cursor_is_and_holds_the_pointer_past_its_edge() {
    let mut reader = reader();
    let moved = WindowState {
        cursor: Point::new(300.0, 200.0),
        ..window(Some((1, true)))
    };
    let frame = reader.read(&[mouse(10, GameInputMouseLeftButton, 0)], &[], &moved, 10);
    let moved = frame.events.iter().position(|event| *event == Event::PointerMoved(Point::new(300.0, 200.0)));
    let pressed = frame.events.iter().position(|event| *event == Event::ButtonPressed(MouseButton::Left));
    assert!(moved.is_some() && pressed.is_some() && moved < pressed);
    assert!(reader.capturing());
    let past = WindowState {
        cursor: Point::new(-20.0, 50.0),
        inside: false,
        focus_changes: Vec::new(),
    };
    let frame = reader.read(&[], &[], &past, 20);
    assert!(frame.events.contains(&Event::PointerMoved(Point::new(-20.0, 50.0))));
    let frame = reader.read(&[mouse(30, GameInputMouseNone, 0)], &[], &past, 30);
    assert!(frame.events.contains(&Event::ButtonReleased(MouseButton::Left)));
    assert!(!reader.capturing());
}

#[test]
fn the_wheel_turns_in_notches_of_120() {
    let mut reader = focused();
    reader.read(&[mouse(10, GameInputMouseNone, 0)], &[], &window(None), 10);
    let frame = reader.read(&[mouse(20, GameInputMouseNone, 240)], &[], &window(None), 20);
    assert!(frame.events.contains(&Event::Wheel(Point::new(0.0, 2.0))));
}

#[test]
fn a_frame_s_input_time_is_its_newest_reading_s() {
    let mut reader = focused();
    let frame = reader.read(&[keyboard(10, &[A]), mouse(30, GameInputMouseNone, 0), keyboard(20, &[])], &[], &window(None), 40);
    assert_eq!(frame.input_time, Some(30));
    let frame = reader.read(&[], &[], &window(None), 50);
    assert_eq!(frame.input_time, None);
}

#[test]
fn the_gamepad_counts_as_input_only_for_a_program_that_reads_it() {
    let mut reader = InputReader::with_repeat(false, 0.5, 10.0);
    reader.read(&[], &[], &window(Some((1, true))), 1);
    let frame = reader.read(&[gamepad(10, GameInputGamepadA)], &[], &window(None), 10);
    assert_eq!(frame.input_time, None);
    assert!(!frame.events.iter().any(|event| matches!(event, Event::Gamepad(_))));
    let mut reader = focused();
    let frame = reader.read(&[gamepad(10, GameInputGamepadA)], &[], &window(None), 10);
    assert_eq!(frame.input_time, Some(10));
}

#[test]
fn the_gamepad_is_its_newest_reading_until_it_disconnects() {
    let mut reader = focused();
    let frame = reader.read(&[gamepad(10, GameInputGamepadA), gamepad(20, GameInputGamepadB)], &[], &window(None), 20);
    let Some(Event::Gamepad(state)) = frame.events.iter().rev().find(|event| matches!(event, Event::Gamepad(_))) else {
        panic!("no gamepad");
    };
    assert!(state.connected);
    assert_eq!(state.buttons.names(), vec!["B"]);
    let frame = reader.read(&[], &[GAMEPAD], &window(None), 30);
    assert_eq!(frame.events.last(), Some(&Event::Gamepad(Default::default())));
}

#[test]
fn keys_and_buttons_held_on_a_device_that_is_unplugged_are_let_go_of() {
    let mut reader = focused();
    reader.read(&[keyboard(10, &[A]), mouse(11, GameInputMouseLeftButton, 0)], &[], &window(None), 11);
    let frame = reader.read(&[], &[KEYBOARD, MOUSE], &window(None), 20);
    assert_eq!(keys(&frame.events), vec![Event::KeyReleased(Key::A)]);
    assert!(frame.events.contains(&Event::ButtonReleased(MouseButton::Left)));
    // Nothing is held any more, so the next press is a press.
    let frame = reader.read(&[keyboard(30, &[A])], &[], &window(None), 30);
    assert!(frame.events.contains(&Event::KeyPressed { key: Key::A, repeat: false }));
}

// A frame hands out the readings that GameInput's reading callback queued
// and those that the walk of its history found, each once, oldest first. The
// history keeps only about 512 readings, under a tenth of a second of a
// gamepad's, so after a long frame the walk finds only the newest, and the
// queue has the rest. The callback may be later than the history with the
// newest, which the walk finds first, and which the next frame then doesn't
// hand out again.
#[test]
fn a_frame_hands_out_the_queued_and_the_walked_readings_once_each_oldest_first() {
    use user_interface::frame_readings;
    const PAD: usize = 7;
    const OTHER_PAD: usize = 9;
    let reading = |timestamp: u64, device: usize| (timestamp, device, format!("{device}@{timestamp}"));
    let names = |readings: Vec<(u64, usize, String)>| readings.into_iter().map(|(_, _, name)| name).collect::<Vec<_>>();
    // After a long frame: the walk lost 20 and 30, which the queue has.
    let queued = [reading(10, PAD), reading(20, PAD), reading(30, PAD), reading(40, PAD)];
    let walked = [reading(40, PAD), reading(50, PAD)];
    assert_eq!(names(frame_readings(queued, walked, Some((5, PAD)))), ["7@10", "7@20", "7@30", "7@40", "7@50"]);
    // The next frame: the callback catches up with 50, which went already.
    let queued = [reading(50, PAD), reading(60, PAD)];
    assert_eq!(names(frame_readings(queued, [], Some((50, PAD)))), ["7@60"]);
    // Two devices' readings at the same time are both handed out.
    let queued = [reading(70, OTHER_PAD)];
    let walked = [reading(70, PAD), reading(70, OTHER_PAD)];
    assert_eq!(names(frame_readings(queued, walked, Some((60, PAD)))), ["7@70", "9@70"]);
    // The first frame hands out everything.
    assert_eq!(names(frame_readings([reading(1, PAD)], [], None)), ["7@1"]);
}

// GameInput starts with its device and reading callbacks, reads frames, and
// stops, unregistering both. A test doesn't have focus, so with background
// input off it may get no readings.
#[test]
fn the_devices_start_read_frames_and_stop() {
    let mut devices = user_interface::Devices::new().unwrap();
    for _ in 0..3 {
        devices.read();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(devices.frame(), 3);
}

// Only "no reading after this one" means that there's nothing new. Any other
// failure of GetNextReading starts over from the newest reading, so that a
// failure that keeps coming can't stop a kind of input for good.
#[test]
fn only_reading_not_found_means_nothing_newer() {
    use user_interface::{NextReadingFailure, after_next_reading_failed};
    assert_eq!(after_next_reading_failed(GAMEINPUT_E_READING_NOT_FOUND), NextReadingFailure::NothingNewer);
    for code in [GAMEINPUT_E_REFERENCE_READING_TOO_OLD, GAMEINPUT_E_OBJECT_NO_LONGER_EXISTS, GAMEINPUT_E_DEVICE_DISCONNECTED] {
        assert_eq!(after_next_reading_failed(code), NextReadingFailure::FindFromNewest);
    }
}
