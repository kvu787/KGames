// What GameInputDemo makes of devices and readings, from a scripted source
// instead of GameInput, so without a gamepad, and without the user's
// keyboard and mouse: which devices it listens to, what it ignores without
// focus, its shortcuts, the rumble test, and the log.

use game_input::*;
use game_input_demo::devices::InputSource;
use game_input_demo::input::{Input, Listen, key_name};
use game_input_demo::scripted_source::{ScriptedSource, key_state};
use user_interface::testing::script::compare;

const PAD_ONE: &str = r#"{ "Connect": { "Id": 1, "Name": "Pad One", "Input": ["Gamepad"],
                                        "Motors": ["LowFrequency", "HighFrequency"] } }"#;
const PAD_TWO: &str = r#"{ "Connect": { "Id": 2, "Name": "Pad Two", "Input": ["Gamepad"],
                                        "Motors": ["LowFrequency", "HighFrequency"] } }"#;
const KEYS: &str = r#"{ "Connect": { "Id": 3, "Name": "Keys", "Input": ["Keyboard"] } }"#;
const RODENT: &str = r#"{ "Connect": { "Id": 4, "Name": "Rodent", "Input": ["Mouse"] } }"#;

// The demo's input, from a test's devices and readings, with the rumble
// test's dead zone at a tenth of the sticks' deflection.
fn input() -> Input {
    Input::new(Box::new(ScriptedSource::default()), 0.1)
}

fn source(input: &mut Input) -> &mut ScriptedSource {
    input.scripted().unwrap()
}

// Does what a script's "Program" step says.
fn step(input: &mut Input, step: &str) {
    source(input).apply(&json::parse(step).unwrap()).unwrap();
}

// Moves the source's clock on.
fn wait(input: &mut Input, microseconds: u64) {
    source(input).now += microseconds;
}

// Checks the demo's state, as a script's "Expect" step would.
fn expect(input: &Input, expected: &str) {
    let mut problems = Vec::new();
    compare(&json::parse(expected).unwrap(), &input.state(), &mut String::new(), &mut problems);
    assert!(problems.is_empty(), "{problems:#?}");
}

fn log(input: &Input) -> Vec<&str> {
    input.log.iter().map(|entry| entry.text.as_str()).collect()
}

fn gamepad(input: &Input, label: &str) -> Listen {
    let device = input.gamepads().find(|device| device.label == label).unwrap();
    Listen::Gamepad(device.clone())
}

#[test]
fn listens_to_the_first_gamepad_until_one_is_picked() {
    let mut input = input();
    for device in [KEYS, PAD_ONE, PAD_TWO] {
        step(&mut input, device);
    }
    input.update(true);
    // Gamepads come first.
    expect(&input, r#"{ "Devices": ["Pad One", "Pad Two", "Keys"], "Listen": "Pad One" }"#);
    assert!(!input.listen_picked);
    assert_eq!(log(&input)[0], "Listening to Pad One only");

    step(&mut input, r#"{ "Disconnect": 1 }"#);
    input.update(true);
    expect(&input, r#"{ "Listen": "Pad Two" }"#);
}

#[test]
fn a_picked_gamepad_is_kept_while_it_is_disconnected() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    step(&mut input, PAD_TWO);
    input.update(true);
    input.pick_listen(gamepad(&input, "Pad Two"));
    step(&mut input, r#"{ "Gamepad": { "Id": 2, "Buttons": ["A"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Gamepad": { "Device": "Pad Two", "Buttons": ["A"] } }"#);
    step(&mut input, r#"{ "Disconnect": 2 }"#);
    input.update(true);
    // Its view starts over, without what it held.
    expect(&input, r#"{ "Listen": "Pad Two", "Gamepad": { "Device": null, "Buttons": [] } }"#);

    step(&mut input, PAD_TWO);
    step(&mut input, r#"{ "Gamepad": { "Id": 2, "Buttons": ["B"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Listen": "Pad Two", "Gamepad": { "Device": "Pad Two", "Buttons": ["B"] } }"#);
}

#[test]
fn ignores_gamepads_that_it_does_not_listen_to() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    step(&mut input, PAD_TWO);
    // A gamepad that's also a keyboard, like a GameSir in Xbox 360 mode.
    step(&mut input, r#"{ "Connect": { "Id": 5, "Name": "Combo", "Input": ["Gamepad", "Keyboard"] } }"#);
    step(&mut input, KEYS);
    input.update(true);
    input.pick_listen(gamepad(&input, "Pad One"));
    step(&mut input, r#"{ "Gamepad": { "Id": 2, "Buttons": ["A"] } }"#);
    step(&mut input, r#"{ "Keyboard": { "Id": 5, "Down": ["A"] } }"#);
    input.update(true);
    expect(&input, r#"{ "ReadingCount": 0, "Latest": null, "Gamepad": { "Device": "Pad One", "Buttons": [] } }"#);

    // Keyboards that aren't gamepads are always listened to.
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["A"] } }"#);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "Buttons": ["X"] } }"#);
    input.update(true);
    expect(
        &input,
        r#"{ "ReadingCount": 2, "Keyboard": { "Device": "Keys", "Keys": ["A"] },
             "Gamepad": { "Buttons": ["X"] } }"#,
    );
}

#[test]
fn a_device_that_disconnects_lets_go_of_its_keys_and_buttons() {
    let mut input = input();
    step(&mut input, KEYS);
    step(&mut input, RODENT);
    input.update(true);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["A"] } }"#);
    step(&mut input, r#"{ "Mouse": { "Id": 4, "Buttons": ["Left"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Keyboard": { "Keys": ["A"] }, "Mouse": { "Buttons": ["Left"] } }"#);
    step(&mut input, r#"{ "Disconnect": 3 }"#);
    step(&mut input, r#"{ "Disconnect": 4 }"#);
    input.update(true);
    expect(&input, r#"{ "Keyboard": { "Device": null, "Keys": [] }, "Mouse": { "Device": null, "Buttons": [] } }"#);
}

#[test]
fn a_gamepad_no_longer_listened_to_lets_go_of_its_keys() {
    let mut input = input();
    step(&mut input, r#"{ "Connect": { "Id": 5, "Name": "Combo", "Input": ["Gamepad", "Keyboard"] } }"#);
    step(&mut input, PAD_ONE);
    input.update(true);
    input.pick_listen(gamepad(&input, "Combo"));
    step(&mut input, r#"{ "Keyboard": { "Id": 5, "Down": ["A"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Keyboard": { "Device": "Combo", "Keys": ["A"] } }"#);
    // Its release isn't handled from now on.
    input.pick_listen(gamepad(&input, "Pad One"));
    expect(&input, r#"{ "Keyboard": { "Device": null, "Keys": [] } }"#);
}

#[test]
fn a_device_that_reconnects_starts_idle_even_after_a_reading_with_its_disconnect() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    input.update(true);
    // The reading and the disconnect come in one update, which handles the
    // disconnect first.
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "Buttons": ["A"] } }"#);
    step(&mut input, r#"{ "Disconnect": 1 }"#);
    input.update(true);
    step(&mut input, PAD_ONE);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "Buttons": ["A"] } }"#);
    input.update(true);
    assert_eq!(log(&input)[0], "Pad One: A pressed");
    expect(&input, r#"{ "Gamepad": { "Device": "Pad One", "Buttons": ["A"] } }"#);
}

#[test]
fn no_gamepad_ignores_every_gamepad() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    step(&mut input, KEYS);
    input.update(true);
    input.pick_listen(Listen::NoGamepad);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "Buttons": ["A"] } }"#);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["Space"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Listen": "None", "ReadingCount": 1, "Gamepad": { "Device": null } }"#);
    assert_eq!(log(&input)[..2], ["Keys: Space down", "Ignoring all gamepads"]);
    // Picking the same thing again changes nothing.
    input.pick_listen(Listen::NoGamepad);
    assert_eq!(log(&input)[0], "Keys: Space down");
}

#[test]
fn input_without_focus_is_dropped() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    input.update(false);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "Buttons": ["A"] } }"#);
    input.update(false);
    expect(&input, r#"{ "ReadingCount": 0, "Gamepad": { "Buttons": [] } }"#);

    // Input from before the window got focus, like the click that gave it,
    // is dropped too.
    wait(&mut input, 1000);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "Buttons": ["B"] } }"#);
    wait(&mut input, 1000);
    input.update(true);
    expect(&input, r#"{ "ReadingCount": 0, "Gamepad": { "Buttons": [] } }"#);

    wait(&mut input, 1000);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "Buttons": ["X"] } }"#);
    input.update(true);
    expect(&input, r#"{ "ReadingCount": 1, "Gamepad": { "Buttons": ["X"] } }"#);
}

#[test]
fn losing_focus_lets_go_of_held_input() {
    let mut input = input();
    for device in [PAD_ONE, KEYS, RODENT] {
        step(&mut input, device);
    }
    input.update(true);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [1, 0], "Buttons": ["A"] } }"#);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["A"] } }"#);
    step(&mut input, r#"{ "Mouse": { "Id": 4, "Buttons": ["Left"] } }"#);
    input.update(true);
    expect(
        &input,
        r#"{ "Gamepad": { "LeftStick": [1.0, 0.0], "Buttons": ["A"] }, "Keyboard": { "Keys": ["A"] },
             "Mouse": { "Buttons": ["Left"] } }"#,
    );

    // What's let go of without focus is never seen, so nothing is shown
    // held, even once the window has focus again.
    let released = r#"{ "Gamepad": { "Device": "Pad One", "LeftStick": [0.0, 0.0], "Buttons": [] },
                        "Keyboard": { "Keys": [] }, "Mouse": { "Buttons": [] } }"#;
    input.update(false);
    expect(&input, released);
    input.update(true);
    expect(&input, released);
}

#[test]
fn input_while_minimized_is_dropped_and_restoring_counts_as_getting_focus() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    step(&mut input, KEYS);
    input.update(true);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F1"] } }"#);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": [] } }"#);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [1, 0] } }"#);
    input.update(true);
    assert_eq!(source(&mut input).rumble(1).unwrap().lowFrequency, 1.0);

    // Minimized, the window doesn't have focus, and the main loop says so.
    // The motors stop.
    wait(&mut input, 1000);
    input.update(false);
    assert!(source(&mut input).rumble(1).is_none());

    // F1, pressed in another window while the demo is minimized, doesn't
    // turn the rumble test off once it's restored.
    wait(&mut input, 1000);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F1"] } }"#);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": [] } }"#);
    wait(&mut input, 1000);
    input.update(true);
    assert!(input.rumble_enabled);
    let presses = log(&input).iter().filter(|line| line.contains("F1")).count();
    assert_eq!(presses, 2, "{:?}", log(&input));
    // The sticks count as at rest until they move with focus.
    assert!(source(&mut input).rumble(1).is_none());
}

#[test]
fn the_click_that_gives_focus_does_only_that() {
    let mut input = input();
    step(&mut input, RODENT);
    input.update(false);
    // The press that gives the window focus comes just before it has it.
    wait(&mut input, 1000);
    step(&mut input, r#"{ "Mouse": { "Id": 4, "Buttons": ["Left"] } }"#);
    wait(&mut input, 1000);
    input.update(true);

    // While it's still held, the button doesn't show as held, but motion does.
    wait(&mut input, 1000);
    step(&mut input, r#"{ "Mouse": { "Id": 4, "Motion": [5, 0], "Buttons": ["Left"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Mouse": { "Buttons": [], "Delta": [5, 0] }, "Latest": ["moved (+5, +0)"] }"#);

    // Letting go of it isn't reported either.
    step(&mut input, r#"{ "Mouse": { "Id": 4 } }"#);
    input.update(true);
    expect(&input, r#"{ "Mouse": { "Buttons": [] }, "Latest": ["moved (+5, +0)"] }"#);
    assert!(!log(&input).iter().any(|line| line.contains("Left")), "{:?}", log(&input));

    // Pressed again, it's a click like any other.
    step(&mut input, r#"{ "Mouse": { "Id": 4, "Buttons": ["Left"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Mouse": { "Buttons": ["Left"] }, "Latest": ["Left pressed"] }"#);
}

#[test]
fn keys_held_when_the_window_gets_focus_are_ignored_until_let_go_of() {
    let mut input = input();
    step(&mut input, KEYS);
    input.update(true);
    // Held from before the window loses focus, through it getting it back.
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["Shift"] } }"#);
    input.update(true);
    input.update(false);
    // F1 held down in another window, and the Alt of the Alt+Tab that
    // switches back to the demo.
    wait(&mut input, 1000);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["Shift", "F1", "Alt"] } }"#);
    wait(&mut input, 1000);
    input.update(true);
    expect(&input, r#"{ "Keyboard": { "Keys": [] } }"#);

    wait(&mut input, 1000);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["Shift", "F1", "Alt", "A"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Keyboard": { "Keys": ["A"] }, "Latest": ["A down"], "RumbleTest": false }"#);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F1"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Keyboard": { "Keys": [] }, "Latest": ["A up"] }"#);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": [] } }"#);
    input.update(true);
    expect(&input, r#"{ "Latest": ["A up"], "RumbleTest": false }"#);

    // Once let go of, F1 works again.
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F1"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Keyboard": { "Keys": ["F1"] }, "Latest": ["F1 down"], "RumbleTest": true }"#);
}

#[test]
fn gamepad_buttons_held_when_the_window_gets_focus_are_ignored_until_let_go_of() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    input.update(false);
    wait(&mut input, 1000);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [1, 0], "Buttons": ["A"] } }"#);
    wait(&mut input, 1000);
    input.update(true);

    // The stick shows as usual, without A.
    wait(&mut input, 1000);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0.5, 0], "Buttons": ["A", "B"] } }"#);
    input.update(true);
    expect(
        &input,
        r#"{ "Gamepad": { "LeftStick": [0.5, 0.0], "Buttons": ["B"] },
             "Latest": ["B pressed", "left stick (+0.500, +0.000)"] }"#,
    );
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0.5, 0] } }"#);
    input.update(true);
    expect(&input, r#"{ "Gamepad": { "Buttons": [] }, "Latest": ["B released"] }"#);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0.5, 0], "Buttons": ["A"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Gamepad": { "Buttons": ["A"] }, "Latest": ["A pressed"] }"#);
}

#[test]
fn a_gamepad_listened_to_shows_none_of_its_input_from_without_focus() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    step(&mut input, PAD_TWO);
    input.update(false);
    wait(&mut input, 1000);
    step(&mut input, r#"{ "Gamepad": { "Id": 2, "LeftStick": [1, 0], "Buttons": ["A"] } }"#);
    input.update(false);
    // The demo switches to Pad Two while it doesn't have focus, and shows
    // it at rest.
    step(&mut input, r#"{ "Disconnect": 1 }"#);
    input.update(false);
    let idle = r#"{ "Listen": "Pad Two", "Gamepad": { "Device": "Pad Two", "LeftStick": [0.0, 0.0], "Buttons": [] } }"#;
    expect(&input, idle);
    wait(&mut input, 1000);
    input.update(true);
    expect(&input, idle);

    // With focus, a gamepad picked shows its last reading, if the window had
    // focus for it.
    step(&mut input, PAD_ONE);
    input.update(true);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0, 1], "Buttons": ["B"] } }"#);
    input.update(true);
    input.pick_listen(gamepad(&input, "Pad One"));
    expect(&input, r#"{ "Gamepad": { "Device": "Pad One", "LeftStick": [0.0, 1.0], "Buttons": ["B"] } }"#);
    input.pick_listen(gamepad(&input, "Pad Two"));
    expect(&input, idle);
}

#[test]
fn readings_of_a_disconnected_gamepad_are_ignored() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    input.update(true);
    // Its last reading comes in the same frame as its disconnect.
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [1, 0], "Buttons": ["A"] } }"#);
    step(&mut input, r#"{ "Disconnect": 1 }"#);
    input.update(true);
    expect(
        &input,
        r#"{ "ReadingCount": 0, "Listen": "Pad One", "Gamepad": { "Device": null, "Buttons": [] }, "Rumble": [] }"#,
    );
    assert_eq!(log(&input)[0], "Disconnected: Pad One");
}

#[test]
fn readings_are_compared_with_the_devices_own_last_one() {
    let mut input = input();
    step(&mut input, KEYS);
    step(&mut input, RODENT);
    input.update(true);
    // GameInput forgets readings after a short while, so after a pause, it
    // no longer has the one before. The demo keeps it, so a key held through
    // the pause isn't pressed again, and the first motion after it moves.
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["Shift"] } }"#);
    step(&mut input, r#"{ "Mouse": { "Id": 4 } }"#);
    input.update(true);
    wait(&mut input, 10_000_000);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["Shift", "A"] } }"#);
    step(&mut input, r#"{ "Mouse": { "Id": 4, "Motion": [3, 4] } }"#);
    input.update(true);
    assert_eq!(log(&input)[..2], ["Rodent: moved (+3, +4)", "Keys: A down"]);

    // Readings without focus count too, so a key let go of meanwhile isn't
    // let go of again.
    input.update(false);
    wait(&mut input, 1000);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": [] } }"#);
    wait(&mut input, 1000);
    input.update(true);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["B"] } }"#);
    input.update(true);
    expect(&input, r#"{ "Latest": ["B down"] }"#);

    // A device that reconnects starts at rest.
    step(&mut input, r#"{ "Disconnect": 3 }"#);
    step(&mut input, KEYS);
    input.update(true);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["B"] } }"#);
    input.update(true);
    assert_eq!(log(&input)[..2], ["Keys: B down", "Connected: Keys [keyboard]"]);
}

#[test]
fn the_rumble_test_drives_the_motors_past_the_dead_zone() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    step(&mut input, KEYS);
    input.update(true);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F1"] } }"#);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": [] } }"#);
    input.update(true);
    assert!(input.rumble_enabled);

    // Halfway from the dead zone's edge to the stick's, and inside it.
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0.55, 0], "RightStick": [0, 0.05] } }"#);
    input.update(true);
    let rumble = source(&mut input).rumble(1).unwrap();
    assert!((rumble.lowFrequency - 0.5).abs() < 1e-6, "{}", rumble.lowFrequency);
    assert_eq!(rumble.highFrequency, 0.0);

    // Diagonal deflection counts, up to full.
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0, 0], "RightStick": [-0.8, 0.8] } }"#);
    input.update(true);
    let rumble = source(&mut input).rumble(1).unwrap();
    assert_eq!((rumble.lowFrequency, rumble.highFrequency), (0.0, 1.0));
    assert_eq!((rumble.leftTrigger, rumble.rightTrigger), (0.0, 0.0));

    // Without focus the motors stop, and they stay stopped with focus
    // until the sticks move again.
    input.update(false);
    assert!(source(&mut input).rumble(1).is_none());
    input.update(true);
    assert!(source(&mut input).rumble(1).is_none());
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "RightStick": [1, 0] } }"#);
    input.update(true);
    assert_eq!(source(&mut input).rumble(1).unwrap().highFrequency, 1.0);

    // Off, the test stops the motors, and the sticks no longer drive them.
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F1"] } }"#);
    input.update(true);
    assert!(!input.rumble_enabled);
    assert_eq!(source(&mut input).rumble(1).unwrap().highFrequency, 0.0);
}

#[test]
fn gamepads_no_longer_listened_to_stop_rumbling() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    step(&mut input, PAD_TWO);
    step(&mut input, KEYS);
    input.update(true);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F1"] } }"#);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [1, 0] } }"#);
    input.update(true);
    assert_eq!(source(&mut input).rumble(1).unwrap().lowFrequency, 1.0);

    input.pick_listen(gamepad(&input, "Pad Two"));
    assert!(source(&mut input).rumble(1).is_none());
    expect(&input, r#"{ "Rumble": [] }"#);
}

#[test]
fn shortcuts_run_once_for_each_press() {
    let mut input = input();
    step(&mut input, KEYS);
    input.update(true);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F1"] } }"#);
    input.update(true);
    assert!(input.rumble_enabled);
    // Held while another key goes down.
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F1", "A"] } }"#);
    input.update(true);
    assert!(input.rumble_enabled);
    // Let go of and pressed again, between two frames.
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": [] } }"#);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F1"] } }"#);
    input.update(true);
    assert!(!input.rumble_enabled);

    // F11 is the library's, which reads the same readings, so to the demo
    // it's only a key, which is logged.
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F11"] } }"#);
    input.update(true);
    assert!(log(&input)[0].starts_with("Keys: F11 down"), "{:?}", log(&input));

    // The log is cleared after the key press is logged.
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F2"] } }"#);
    input.update(true);
    assert!(input.log.is_empty());
}

#[test]
fn keys_are_named_like_the_keyboard_layout_names_them() {
    for name in ["A", "F1", "Left", "Delete"] {
        assert_eq!(key_name(&key_state(name).unwrap()), name);
    }
    // The arrow keys have the prefix that sets them apart from the keypad's.
    assert_eq!(key_state("Left").unwrap().scanCode, 0xE04B);
    assert_eq!(key_state("A").unwrap().codePoint, 'a' as u32);
    assert!(key_state("Nope").is_none());
}

#[test]
fn runs_of_motion_share_a_log_line() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    step(&mut input, RODENT);
    input.update(true);
    // The first reading is the mouse's starting point.
    step(&mut input, r#"{ "Mouse": { "Id": 4 } }"#);
    step(&mut input, r#"{ "Mouse": { "Id": 4, "Motion": [10, -5] } }"#);
    step(&mut input, r#"{ "Mouse": { "Id": 4, "Motion": [3, 4] } }"#);
    input.update(true);
    expect(&input, r#"{ "ReadingCount": 3, "Mouse": { "Device": "Rodent", "Delta": [3, 4] } }"#);
    assert_eq!(input.log[0].text, "Rodent: moved (+3, +4)");
    assert_eq!(input.log[0].count, 2);

    // A press starts a line of its own, and so does motion after it.
    step(&mut input, r#"{ "Mouse": { "Id": 4, "Buttons": ["Left"] } }"#);
    step(&mut input, r#"{ "Mouse": { "Id": 4, "Wheel": [0, -120] } }"#);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0.25, 0] } }"#);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0.5, 0] } }"#);
    input.update(true);
    assert_eq!(
        log(&input)[..3],
        [
            "Pad One: left stick (+0.500, +0.000)",
            "Rodent: Left released, wheel -120",
            "Rodent: Left pressed",
        ]
    );
    assert_eq!(input.log[0].count, 2);
    expect(&input, r#"{ "Mouse": { "Delta": [0, 0], "Wheel": [0, -120] } }"#);
}

#[test]
fn readings_are_handled_in_the_order_they_came() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    step(&mut input, KEYS);
    input.update(true);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["A"] } }"#);
    step(&mut input, r#"{ "Gamepad": { "Id": 1, "Buttons": ["A"] } }"#);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": [] } }"#);
    input.update(true);
    assert_eq!(log(&input)[..3], ["Keys: A up", "Pad One: A pressed", "Keys: A down"]);
    expect(&input, r#"{ "ReadingCount": 3, "Latest": ["A up"] }"#);
}

// The Gamepad card's readings a second, from the demo's state.
fn readings_per_second(input: &Input) -> f64 {
    match input.state().get("Gamepad").and_then(|gamepad| gamepad.get("ReadingsPerSecond")) {
        Some(json::Value::Number(rate)) => *rate,
        other => panic!("no ReadingsPerSecond: {other:?}"),
    }
}

// Moves the clock on by `length` microseconds, with a reading of the gamepad
// every `spacing` microseconds, each at its own time, and adds their times
// to `times`.
fn read_for(input: &mut Input, id: usize, length: u64, spacing: u64, times: &mut Vec<u64>) {
    let start = source(input).now;
    let end = start + length;
    let mut next = start + spacing;
    while next <= end {
        source(input).now = next;
        // The stick moves, as a real one's noise does.
        let x = (next % 1000) as f64 / 1000.0;
        step(input, &format!(r#"{{ "Gamepad": {{ "Id": {id}, "LeftStick": [{x}, 0] }} }}"#));
        times.push(next);
        next += spacing;
    }
    source(input).now = end;
}

// A frame with focus, `length` microseconds long, with the gamepad's
// readings every `spacing` microseconds, if any.
fn frame_of_readings(input: &mut Input, id: usize, length: u64, spacing: Option<u64>, times: &mut Vec<u64>) {
    match spacing {
        Some(spacing) => read_for(input, id, length, spacing, times),
        None => source(input).now += length,
    }
    input.update(true);
}

// The time of the demo's last update, in the source's microseconds, which
// start at 0.
fn now(input: &Input) -> u64 {
    (input.now * 1_000_000.0).round() as u64
}

#[test]
fn the_gamepad_s_readings_a_second_are_averaged_over_the_last_two_seconds() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    input.update(true);
    assert_eq!(readings_per_second(&input), 0.0);

    // 100 frames a second, of uneven lengths, and a reading every
    // millisecond, like a gamepad that GameInput polls every millisecond.
    // On every frame, the average is the readings of the last two seconds,
    // that frame's too, divided by two.
    let lengths = [7_000, 13_000, 10_000, 4_000, 16_000];
    let mut times = Vec::new();
    let check = |input: &Input, times: &[u64]| {
        let now = now(input);
        let recent = times.iter().filter(|&&time| now - time < 2_000_000).count();
        assert_eq!(readings_per_second(input), recent as f64 / 2.0, "at {now} microseconds");
    };
    for frame in 0..300 {
        frame_of_readings(&mut input, 1, lengths[frame % lengths.len()], Some(1_000), &mut times);
        check(&input, &times);
        if now(&input) == 1_000_000 {
            // A second in, a second's readings over two seconds.
            assert_eq!(readings_per_second(&input), 500.0);
        }
    }
    // Three seconds in: a reading every millisecond is 1,000 a second.
    assert_eq!(now(&input), 3_000_000);
    assert_eq!(readings_per_second(&input), 1_000.0);

    // Once the readings stop, they leave the average as they turn two
    // seconds old: half of them a second later, all of them two.
    for frame in 0..200 {
        frame_of_readings(&mut input, 1, lengths[frame % lengths.len()], None, &mut times);
        check(&input, &times);
        if now(&input) == 4_000_000 {
            assert_eq!(readings_per_second(&input), 500.0);
        }
    }
    assert_eq!(now(&input), 5_000_000);
    assert_eq!(readings_per_second(&input), 0.0);
}

#[test]
fn the_gamepad_s_average_counts_only_the_gamepad_the_card_shows() {
    let mut input = input();
    step(&mut input, PAD_ONE);
    step(&mut input, PAD_TWO);
    input.update(true);
    let mut one = Vec::new();
    // The first reading counts, on its own frame.
    frame_of_readings(&mut input, 1, 1_000, Some(1_000), &mut one);
    assert_eq!(readings_per_second(&input), 0.5);
    for _ in 0..99 {
        // Pad Two reads too, but the card shows Pad One.
        step(&mut input, r#"{ "Gamepad": { "Id": 2, "LeftStick": [0.5, 0] } }"#);
        frame_of_readings(&mut input, 1, 10_000, Some(1_000), &mut one);
    }
    // A second of Pad One's readings, and none of Pad Two's.
    assert_eq!(readings_per_second(&input), 495.5);

    // Picking another gamepad starts over, with that one's readings.
    input.pick_listen(gamepad(&input, "Pad Two"));
    frame_of_readings(&mut input, 1, 10_000, Some(1_000), &mut one);
    assert_eq!(readings_per_second(&input), 0.0);
    let mut two = Vec::new();
    frame_of_readings(&mut input, 2, 10_000, Some(500), &mut two);
    assert_eq!(readings_per_second(&input), 10.0);

    // A gamepad that disconnects and reconnects starts over too, and the
    // reading that shows it again counts.
    step(&mut input, r#"{ "Disconnect": 2 }"#);
    input.update(true);
    assert_eq!(readings_per_second(&input), 0.0);
    step(&mut input, PAD_TWO);
    frame_of_readings(&mut input, 2, 500, Some(500), &mut two);
    expect(&input, r#"{ "Gamepad": { "Device": "Pad Two", "ReadingsPerSecond": 0.5 } }"#);

    // Losing focus starts over, and readings without focus aren't handled.
    frame_of_readings(&mut input, 2, 10_000, Some(500), &mut two);
    assert_eq!(readings_per_second(&input), 10.5);
    input.update(false);
    assert_eq!(readings_per_second(&input), 0.0);
    read_for(&mut input, 2, 10_000, 500, &mut two);
    input.update(false);
    assert_eq!(readings_per_second(&input), 0.0);
    // Nor are the ones from before the frame that gets focus back.
    read_for(&mut input, 2, 10_000, 500, &mut two);
    source(&mut input).now += 1;
    input.update(true);
    assert_eq!(readings_per_second(&input), 0.0);
    frame_of_readings(&mut input, 2, 10_000, Some(500), &mut two);
    assert_eq!(readings_per_second(&input), 10.0);
}

#[test]
fn scripted_readings_are_whole_states() {
    let mut source = ScriptedSource::default();
    let step = |source: &mut ScriptedSource, step: &str| source.apply(&json::parse(step).unwrap()).unwrap();
    step(&mut source, PAD_ONE);
    step(&mut source, RODENT);
    // What a step leaves out is at rest.
    step(&mut source, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0.5, 0], "Buttons": ["A"] } }"#);
    step(&mut source, r#"{ "Gamepad": { "Id": 1, "RightTrigger": 1 } }"#);
    let state = source.readings(GameInputKindGamepad)[1].gamepad;
    assert_eq!((state.leftThumbstickX, state.buttons, state.rightTrigger), (0.0, 0, 1.0));

    // A mouse's motion adds up.
    step(&mut source, r#"{ "Mouse": { "Id": 4, "Motion": [10, 0] } }"#);
    step(&mut source, r#"{ "Mouse": { "Id": 4, "Motion": [10, 0] } }"#);
    let readings = source.readings(GameInputKindMouse);
    let totals: Vec<_> = readings.iter().map(|reading| (reading.device, reading.mouse.positionX)).collect();
    assert_eq!(totals, [(Some(4), 10), (Some(4), 20)]);
    // Later readings have later timestamps, even at the same time.
    assert!(readings[1].timestamp > readings[0].timestamp);
    assert!(source.readings(GameInputKindMouse).is_empty());

    let events = source.device_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].1.label, "Pad One");
    assert_eq!(events[0].1.rumble_motors, GameInputRumbleLowFrequency | GameInputRumbleHighFrequency);
}

// A mouse's position and wheel are running totals, which go around an i64's
// ends when a script moves it that far, and the motion that the demo shows,
// the difference of two totals, goes around with them. Adding and
// subtracting them as they are stops a debug build there.
#[test]
fn a_scripted_mouse_s_totals_go_around_an_i64_s_ends() {
    const FAR: &str = r#"{ "Mouse": { "Id": 4, "Motion": [1e300, -1e300], "Wheel": [-1e300, 1e300] } }"#;
    let mut source = ScriptedSource::default();
    for step in [RODENT, FAR, FAR] {
        source.apply(&json::parse(step).unwrap()).unwrap();
    }
    let totals: Vec<_> = source
        .readings(GameInputKindMouse)
        .iter()
        .map(|reading| (reading.mouse.positionX, reading.mouse.positionY, reading.mouse.wheelX, reading.mouse.wheelY))
        .collect();
    // A step's motion is the most that an i64 holds, and two of them go
    // around.
    assert_eq!(totals, [(i64::MAX, i64::MIN, i64::MIN, i64::MAX), (-2, 0, 0, -2)]);

    // The demo's motion is from the first reading, its starting point, to
    // the second.
    let mut input = input();
    step(&mut input, RODENT);
    input.update(true);
    step(&mut input, FAR);
    step(&mut input, FAR);
    input.update(true);
    assert_eq!(
        input.log[0].text,
        "Rodent: moved (+9223372036854775807, -9223372036854775808), wheel +9223372036854775807, horizontal wheel -9223372036854775808"
    );
}

#[test]
fn scripted_steps_say_what_is_wrong_with_them() {
    let mut source = ScriptedSource::default();
    let mut problem = |step: &str| source.apply(&json::parse(step).unwrap()).unwrap_err();
    assert_eq!(problem(r#"{ "Gamepad": { "Id": 9 } }"#), "device 9 isn't connected");
    assert_eq!(problem(r#"{ "Connect": { "Id": 1 } }"#), "Connect needs a Name");
    assert_eq!(
        problem(r#"{ "Connect": { "Id": 0, "Name": "Pad" } }"#),
        "a device's Id must be a whole number from 1"
    );
    assert_eq!(problem(r#"{ "Connect": { "Id": 1, "Name": "Pad", "Input": ["Wheel"] } }"#), "Input can't have Wheel");
    assert_eq!(problem(r#"{ "Keyboard": { "Id": 1, "Down": ["Nope"] } }"#), "Nope isn't a key");
    assert_eq!(
        problem(r#"{ "Gamepad": { "Id": 1, "LeftStick": [1] } }"#),
        "LeftStick must be [x, y], not a list of 1"
    );
    assert!(problem(r#"{ "Mouse": { "Id": 1, "Buttons": "Left" } }"#).starts_with("Buttons must be a list of names, not "));
    assert!(problem(r#"{ "Disconnect": "Pad" }"#).starts_with("Disconnect needs a device's Id, not "));
    assert_eq!(problem(r#"{ "Wave": 1 }"#), "Wave isn't a step for the demo");
    assert!(problem(r#"{ "Disconnect": 1, "Wave": 1 }"#).starts_with("a step for the demo has one of "));
    assert!(problem("[]").starts_with("a step for the demo must be an object, not "));
}
