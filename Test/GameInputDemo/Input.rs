// What GameInputDemo makes of devices and readings, from a scripted source
// instead of GameInput, so without a gamepad, and without the user's
// keyboard and mouse: which devices it listens to, what it ignores without
// focus, its shortcuts, the rumble test, and the log.

use application::script::compare;
use game_input::*;
use game_input_demo::devices::InputSource;
use game_input_demo::input::{Input, Listen, key_name};
use game_input_demo::scripted_source::{ScriptedSource, key_state};

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
    let problems = compare(&json::parse(expected).unwrap(), &input.state());
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
    step(&mut input, r#"{ "Disconnect": 2 }"#);
    input.update(true);
    expect(&input, r#"{ "Listen": "Pad Two", "Gamepad": { "Device": null } }"#);

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

    // F11, pressed in another window while the demo is minimized, doesn't
    // switch it to fullscreen once it's restored.
    wait(&mut input, 1000);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F11"] } }"#);
    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": [] } }"#);
    wait(&mut input, 1000);
    input.update(true);
    assert!(!input.fullscreen_toggled);
    assert!(!log(&input).iter().any(|line| line.contains("F11")), "{:?}", log(&input));
    // The sticks count as at rest until they move with focus.
    assert!(source(&mut input).rumble(1).is_none());
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

    step(&mut input, r#"{ "Keyboard": { "Id": 3, "Down": ["F11"] } }"#);
    input.update(true);
    assert!(input.fullscreen_toggled);

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

#[test]
fn scripted_readings_are_whole_states() {
    let mut source = ScriptedSource::default();
    let step = |source: &mut ScriptedSource, step: &str| source.apply(&json::parse(step).unwrap()).unwrap();
    step(&mut source, PAD_ONE);
    step(&mut source, RODENT);
    // What a step leaves out is at rest.
    step(&mut source, r#"{ "Gamepad": { "Id": 1, "LeftStick": [0.5, 0], "Buttons": ["A"] } }"#);
    step(&mut source, r#"{ "Gamepad": { "Id": 1, "RightTrigger": 1 } }"#);
    let state = source.gamepad_state(1).unwrap();
    assert_eq!((state.leftThumbstickX, state.buttons, state.rightTrigger), (0.0, 0, 1.0));

    // A mouse's motion adds up, and each reading has the one before it.
    step(&mut source, r#"{ "Mouse": { "Id": 4, "Motion": [10, 0] } }"#);
    step(&mut source, r#"{ "Mouse": { "Id": 4, "Motion": [10, 0] } }"#);
    let readings = source.readings(GameInputKindMouse);
    let totals: Vec<_> = readings
        .iter()
        .map(|(reading, previous)| (reading.mouse.positionX, previous.as_ref().map(|previous| previous.mouse.positionX)))
        .collect();
    assert_eq!(totals, [(10, None), (20, Some(10))]);
    // Later readings have later timestamps, even at the same time.
    assert!(readings[1].0.timestamp > readings[0].0.timestamp);
    assert!(source.readings(GameInputKindMouse).is_empty());

    let events = source.device_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].1.label, "Pad One");
    assert_eq!(events[0].1.rumble_motors, GameInputRumbleLowFrequency | GameInputRumbleHighFrequency);
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
