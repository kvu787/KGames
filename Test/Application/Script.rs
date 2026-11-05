// Library/Application's scripts: reading them, what each frame gets, and
// how a program's state is checked against what a script expects.

use std::path::Path;

use application::script::{Checkpoint, Script, compare};
use application::{Button, Event, Gamepad, key_name, virtual_key};
use json::{Value, object, parse};

fn script(text: &str) -> Script {
    Script::parse(text, Path::new(r"C:\Output")).unwrap_or_else(|problem| panic!("{problem}"))
}

fn problem(text: &str) -> String {
    match Script::parse(text, Path::new(r"C:\Output")) {
        Ok(_) => panic!("{text} parsed"),
        Err(problem) => problem,
    }
}

#[test]
fn gives_each_frame_its_steps_and_time() {
    let mut script = script(
        r#"{
            "FrameSeconds": 0.25,
            "Steps": [
                { "Frame": 1, "Move": [10, 20] },
                { "Frame": 2, "Press": [30, 40] },
                { "Frame": 2, "Key": "F11", "Shift": true },
                { "Frame": 3, "Key": "Left", "Repeat": true },
                { "Frame": 4, "Wheel": [50, 60], "Notches": -2 },
                { "Frame": 4, "Capture": "End" },
                { "Frame": 4, "Expect": { "A": 1 } },
                { "Frame": 4, "Program": { "B": 2 } }
            ]
        }"#,
    );
    let frames: Vec<_> = (0..5).map(|_| script.next_frame()).collect();
    assert!(script.finished());
    assert_eq!(frames.iter().map(|frame| frame.seconds).collect::<Vec<_>>(), [0.0, 0.25, 0.5, 0.75, 1.0]);
    assert!(frames[0].events.is_empty() && frames[0].cursor.is_none());
    // A move only moves the cursor, which stays until something moves it.
    assert!(frames[1].events.is_empty());
    assert_eq!(frames[1].cursor, Some([10.0, 20.0]));
    assert_eq!(
        frames[2].events,
        [
            Event::Press([30.0, 40.0]),
            Event::Key {
                key: 0x7A,
                control: false,
                shift: true,
                repeat: false
            }
        ]
    );
    assert_eq!(
        frames[3].events,
        [Event::Key {
            key: 0x25,
            control: false,
            shift: false,
            repeat: true
        }]
    );
    // A click moves the cursor to it, like the window's messages do.
    assert_eq!(frames[2].cursor, Some([30.0, 40.0]));
    assert_eq!(frames[3].cursor, Some([30.0, 40.0]));
    assert_eq!(frames[4].events, [Event::Wheel([50.0, 60.0], -2.0)]);
    assert_eq!(frames[4].program_steps, [object([("B", 2.0.into())])]);
    match frames[4].checkpoints.as_slice() {
        [Checkpoint::Capture { image, state }, Checkpoint::Expect(expected)] => {
            assert_eq!(image, Path::new(r"C:\Output\End.png"));
            assert_eq!(state, Path::new(r"C:\Output\End.json"));
            assert_eq!(expected, &object([("A", 1.0.into())]));
        }
        _ => panic!("frame 4 should capture, then expect"),
    }
}

// Window::finish fails a run whose program exited with a step still to do.
#[test]
fn knows_which_step_is_next() {
    let mut script = script(r#"{ "Steps": [{ "Frame": 1, "Key": "S" }, { "Frame": 3, "Key": "S" }] }"#);
    assert_eq!(script.pending_frame(), Some(1));
    script.next_frame();
    script.next_frame();
    assert_eq!(script.pending_frame(), Some(3));
    script.next_frame();
    script.next_frame();
    assert_eq!(script.pending_frame(), None);
    assert!(script.finished());
}

// A Text step types its characters, without the keys that type them, and
// leaves the cursor where it was.
#[test]
fn types_text_as_characters() {
    let mut script = script(
        r#"{ "Steps": [{ "Frame": 0, "Move": [5, 6] }, { "Frame": 0, "Text": "Hi é" }, { "Frame": 0, "Key": "Home" }] }"#,
    );
    let frame = script.next_frame();
    assert_eq!(
        frame.events,
        [
            Event::Character('H'),
            Event::Character('i'),
            Event::Character(' '),
            Event::Character('\u{E9}'),
            Event::Key {
                key: 0x24,
                control: false,
                shift: false,
                repeat: false
            }
        ]
    );
    assert_eq!(frame.cursor, Some([5.0, 6.0]));
    for (text, expected) in [
        (r#"{ "Steps": [{ "Frame": 1, "Text": "" }] }"#, "step 1: Text must be characters to type, without control characters, not \"\""),
        (r#"{ "Steps": [{ "Frame": 1, "Text": "a\nb" }] }"#, "step 1: Text must be characters to type, without control characters, not \"a\nb\""),
        (r#"{ "Steps": [{ "Frame": 1, "Text": 7 }] }"#, "step 1: Text must be characters to type, without control characters, not 7"),
    ] {
        assert_eq!(problem(text), expected, "for {text}");
    }
}

// A Hold step says which keys are held from its frame on: a key that goes
// down is pressed, with Control and Shift if they're held with it, and a key
// that it leaves out lets go.
#[test]
fn holds_keys_until_the_next_hold() {
    let mut script = script(
        r#"{
            "Steps": [
                { "Frame": 1, "Hold": ["Up"] },
                { "Frame": 3, "Hold": ["Up", "Shift", "Space"] },
                { "Frame": 4, "Hold": ["Space"] },
                { "Frame": 5, "Hold": [] }
            ]
        }"#,
    );
    let key = |key, shift| Event::Key {
        key,
        control: false,
        shift,
        repeat: false,
    };
    let frames: Vec<_> = (0..6).map(|_| script.next_frame()).collect();
    let held: Vec<&[u32]> = frames.iter().map(|frame| frame.held_keys.as_slice()).collect();
    assert_eq!(held, [&[][..], &[0x26], &[0x26], &[0x26, 0x10, 0x20], &[0x20], &[]]);
    assert_eq!(frames[1].events, [key(0x26, false)]);
    assert!(frames[2].events.is_empty(), "a held key isn't pressed again");
    assert_eq!(frames[3].events, [key(0x10, true), key(0x20, true)]);
    assert!(frames[4].events.is_empty() && frames[5].events.is_empty());
}

// A Gamepad step says where the gamepad is from its frame on: what it leaves
// out is at rest, a button is pressed in the frame it goes down, and null
// disconnects the gamepad.
#[test]
fn gives_the_gamepad_as_its_steps_say() {
    let mut script = script(
        r#"{
            "Steps": [
                { "Frame": 1, "Gamepad": { "RightStick": [0.5, -1], "LeftTrigger": 0.25, "Buttons": ["A", "LB"] } },
                { "Frame": 3, "Gamepad": { "LeftStick": [-1, 0], "Buttons": ["A", "Up"] } },
                { "Frame": 4, "Gamepad": null }
            ]
        }"#,
    );
    let frames: Vec<_> = (0..5).map(|_| script.next_frame().gamepad).collect();
    assert_eq!(frames[0], Gamepad::default());
    let bits = |buttons: &[Button]| buttons.iter().fold(0, |bits, button| bits | button.bit());
    let first = Gamepad {
        connected: true,
        right_stick: [0.5, -1.0],
        left_trigger: 0.25,
        buttons: bits(&[Button::A, Button::LeftShoulder]),
        went_down: bits(&[Button::A, Button::LeftShoulder]),
        ..Gamepad::default()
    };
    assert_eq!(frames[1], first);
    assert_eq!(frames[2], Gamepad { went_down: 0, ..first });
    assert_eq!(
        frames[3],
        Gamepad {
            connected: true,
            left_stick: [-1.0, 0.0],
            buttons: bits(&[Button::A, Button::Up]),
            went_down: Button::Up.bit(),
            ..Gamepad::default()
        }
    );
    assert_eq!(frames[4], Gamepad::default());
    for (text, expected) in [
        (r#"{ "Steps": [{ "Frame": 1, "Gamepad": 1 }] }"#, "step 1: Gamepad's state must be an object, or null, not 1"),
        (r#"{ "Steps": [{ "Frame": 1, "Gamepad": { "LeftStick": [2, 0] } }] }"#, "step 1: Gamepad's LeftStick must be [x, y], each from -1 to 1, not a list of 2"),
        (r#"{ "Steps": [{ "Frame": 1, "Gamepad": { "RightTrigger": -0.5 } }] }"#, "step 1: Gamepad's RightTrigger must be a number from 0 to 1, not -0.5"),
        (r#"{ "Steps": [{ "Frame": 1, "Gamepad": { "Buttons": ["Start"] } }] }"#, "step 1: Gamepad's \"Start\" isn't a button. They're A, B, X, Y, View, Menu, LB, RB, LS, RS, Up, Down, Left, Right"),
        (r#"{ "Steps": [{ "Frame": 1, "Gamepad": { "Paddle": 1 } }] }"#, "step 1: Gamepad's Paddle isn't part of a gamepad. They're LeftStick, RightStick, LeftTrigger, RightTrigger and Buttons"),
    ] {
        assert_eq!(problem(text), expected, "for {text}");
    }
}

#[test]
fn runs_at_60_frames_a_second_by_default() {
    let mut script = script(r#"{ "Steps": [{ "Frame": 1, "Key": "S" }] }"#);
    script.next_frame();
    assert!((script.next_frame().seconds - 1.0 / 60.0).abs() < 1e-12);
}

#[test]
fn says_what_is_wrong_with_a_script() {
    for (text, expected) in [
        (r#"{}"#, "a script needs \"Steps\", a list of steps"),
        (r#"{ "Steps": [], "FrameSeconds": 0 }"#, "FrameSeconds must be a number of seconds above 0, not 0"),
        (r#"{ "Steps": [{ "Key": "S" }] }"#, "step 1: needs a Frame"),
        (r#"{ "Steps": [{ "Frame": 1.5, "Key": "S" }] }"#, "step 1: Frame must be a whole number, not 1.5"),
        (r#"{ "Steps": [{ "Frame": 1 }] }"#, "step 1: needs one of Move, Press, Release, Wheel, Key, Hold, Gamepad, Text, Capture, Expect and Program"),
        (r#"{ "Steps": [{ "Frame": 1, "Key": "S", "Move": [1, 2] }] }"#, "step 1: needs one of Move, Press, Release, Wheel, Key, Hold, Gamepad, Text, Capture, Expect and Program"),
        (r#"{ "Steps": [{ "Frame": 1, "Hold": "Up" }] }"#, "step 1: Hold must be a list of the keys held down, like [\"Up\", \"Space\"], not \"Up\""),
        (r#"{ "Steps": [{ "Frame": 1, "Hold": ["Up", "Hyper"] }] }"#, "step 1: Hyper isn't a key"),
        (r#"{ "Steps": [{ "Frame": 1, "Hold": [true] }] }"#, "step 1: Each of Hold's keys must be a key's name, like \"F11\", not true"),
        (r#"{ "Steps": [{ "Frame": 1, "Hold": ["Up", "Up"] }] }"#, "step 1: Hold has \"Up\" twice"),
        (r#"{ "Steps": [{ "Frame": 1, "Jump": true }] }"#, "step 1: Jump isn't a step"),
        (r#"{ "Steps": [{ "Frame": 1, "Key": "Hyper" }] }"#, "step 1: Hyper isn't a key"),
        (r#"{ "Steps": [{ "Frame": 1, "Press": [1] }] }"#, "step 1: a point must be [x, y], not a list of 1"),
        (r#"{ "Steps": [{ "Frame": 1, "Capture": "a/b" }] }"#, "step 1: Capture must be a name of letters and digits, for its files, not \"a/b\""),
        (r#"{ "Steps": [{ "Frame": 1, "Key": "S", "Control": 1 }] }"#, "step 1: Control must be true or false, not 1"),
        (r#"{ "Steps": [{ "Frame": 1, "Key": 65.7 }] }"#, "step 1: Key must be a key's name, like \"F11\", not 65.7"),
        (r#"{ "Steps": [{ "Frame": 2, "Key": "S" }, { "Frame": 1, "Key": "S" }] }"#, "step 2 comes before the step above it"),
        (r#"{ "Steps": [{ "Frame": 1, "Capture": "A" }, { "Frame": 1, "Capture": "B" }] }"#, "step 2: frame 1 has a Capture already"),
        (r#"{ "Steps": [{ "Frame": 1, "Capture": "A" }, { "Frame": 2, "Capture": "A" }] }"#, "step 2: another Capture is called A"),
        ("{ \"Steps\": [", "line 1, column 13: expected a value, but the file ended"),
    ] {
        assert_eq!(problem(text), expected, "for {text}");
    }
}

#[test]
fn names_keys_like_scripts_do() {
    for (name, code) in [
        ("F1", Some(0x70)),
        ("F11", Some(0x7A)),
        ("F24", Some(0x87)),
        ("F25", None),
        ("F+1", None),
        ("F", Some(0x46)),
        ("S", Some(0x53)),
        ("7", Some(0x37)),
        ("s", None),
        ("Left", Some(0x25)),
        ("Escape", Some(0x1B)),
        ("Space", Some(0x20)),
        ("Home", Some(0x24)),
        ("End", Some(0x23)),
        ("PageUp", Some(0x21)),
        ("PageDown", Some(0x22)),
        ("", None),
    ] {
        assert_eq!(virtual_key(name), code, "for {name:?}");
        // A key's name is the one that scripts read.
        if let Some(code) = code {
            assert_eq!(key_name(code), name);
        }
    }
    assert_eq!(key_name(0xBA), "0xBA", "a key without a name of its own");
}

#[test]
fn compares_only_what_is_expected() {
    let actual = parse(
        r#"{ "Editor": { "Shift": 0.5, "Name": "Body", "Color": [0.1, 0.2] }, "Flag": true, "Extra": null }"#,
    )
    .unwrap();
    let matches = |expected: &str| compare(&parse(expected).unwrap(), &actual);
    assert!(matches(r#"{ "Flag": true }"#).is_empty());
    assert!(matches(r#"{ "Editor": { "Color": [0.1, 0.2] } }"#).is_empty());
    // Within a billionth, for rounding.
    assert!(matches(r#"{ "Editor": { "Shift": 0.5000000001 } }"#).is_empty());
    assert_eq!(matches(r#"{ "Editor": { "Shift": 0.6 } }"#), ["Editor.Shift is 0.5, not 0.6"]);
    assert_eq!(matches(r#"{ "Editor": { "Name": "Cabin" } }"#), ["Editor.Name is \"Body\", not \"Cabin\""]);
    assert_eq!(matches(r#"{ "Editor": { "Color": [0.1] } }"#), ["Editor.Color has 2 items, not 1: [0.1, 0.2]"]);
    assert_eq!(matches(r#"{ "Editor": { "Color": [0.1, 0.3] } }"#), ["Editor.Color[1] is 0.2, not 0.3"]);
    assert_eq!(matches(r#"{ "Missing": 1 }"#), ["Missing is missing"]);
    assert_eq!(matches(r#"{ "Flag": "true" }"#), ["Flag is true, not \"true\""]);
    assert_eq!(matches(r#"{ "Extra": 0 }"#), ["Extra is null, not 0.0"]);
    // A state that went wrong enough to be infinite or NaN matches nothing.
    for wrong in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        let differences = compare(&object([("X", 4.0.into())]), &object([("X", wrong.into())]));
        assert_eq!(differences, [format!("X is {wrong}, not 4")]);
    }
    assert_eq!(
        compare(&Value::from(1.0), &Value::from("one")),
        ["the state is \"one\", not 1.0"]
    );
}
