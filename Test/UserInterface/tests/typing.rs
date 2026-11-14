// Tests typed text, which the interface makes from keys with a US keyboard's
// table (input/typing.rs): what each key types, with Shift and without, that
// Ctrl and Alt type nothing, and that a key's press and its text are one
// step.

use user_interface::input::typing::{key_for, typed};
use user_interface::testing::headless::Headless;
use user_interface::{Context, Event, Input, Key, Program, Tree, Ui};

// Every printable ASCII character has a key that types it, with Shift or
// without, and that key types it.
#[test]
fn every_printable_ascii_character_has_a_key_that_types_it() {
    for character in ' '..='~' {
        let (key, shift) = key_for(character).unwrap_or_else(|| panic!("no key types {character:?}"));
        assert_eq!(typed(key, shift), Some(character), "{key:?} with Shift {shift}");
    }
}

// Nothing else is typed: no accents, no control characters, nothing but
// ASCII.
#[test]
fn only_printable_ascii_is_typed() {
    for character in ['é', '\t', '\n', '\u{7F}', '€', '中'] {
        assert_eq!(key_for(character), None, "{character:?}");
    }
    for code in 0..=255u8 {
        for shift in [false, true] {
            if let Some(character) = typed(Key(code), shift) {
                assert!((' '..='~').contains(&character), "{code:#04X} types {character:?}");
            }
        }
    }
}

#[test]
fn shift_types_a_key_s_other_character() {
    assert_eq!((typed(Key::A, false), typed(Key::A, true)), (Some('a'), Some('A')));
    assert_eq!((typed(Key::DIGIT_1, false), typed(Key::DIGIT_1, true)), (Some('1'), Some('!')));
    assert_eq!((typed(Key(0xBD), false), typed(Key(0xBD), true)), (Some('-'), Some('_')));
    assert_eq!((typed(Key(0xDE), false), typed(Key(0xDE), true)), (Some('\''), Some('"')));
    assert_eq!((typed(Key::SPACE, false), typed(Key::SPACE, true)), (Some(' '), Some(' ')));
}

// The numpad's digits and operators type the same with Shift or without.
#[test]
fn the_numpad_types_its_digits_and_operators() {
    for (code, character) in [(0x60, '0'), (0x69, '9'), (0x6A, '*'), (0x6B, '+'), (0x6D, '-'), (0x6E, '.'), (0x6F, '/')] {
        assert_eq!(typed(Key(code), false), Some(character));
        assert_eq!(typed(Key(code), true), Some(character));
    }
    // Keys with no character type nothing.
    for key in [Key::ENTER, Key::BACKSPACE, Key::TAB, Key::ESCAPE, Key::F2, Key::LEFT, Key::SHIFT, Key::CAPS_LOCK] {
        assert_eq!(typed(key, false), None, "{key:?}");
    }
}

// A frame's text, as the interface's steps type it from the keys.
fn text_of(events: &[Event]) -> String {
    let mut input = Input::default();
    let mut text = String::new();
    for step in input.steps(events) {
        input.begin(0.0, step);
        text.push_str(&input.text);
    }
    text
}

fn tap(key: Key) -> [Event; 2] {
    [Event::KeyPressed(key), Event::KeyReleased(key)]
}

// Shift types the other character while it's held, Ctrl and Alt type
// nothing, as they make shortcuts, and a key's release types nothing.
#[test]
fn the_interface_types_with_shift_and_never_with_ctrl_or_alt() {
    let mut events = tap(Key::A).to_vec();
    events.push(Event::KeyPressed(Key::SHIFT));
    events.extend(tap(Key::A));
    events.extend(tap(Key::DIGIT_2));
    events.push(Event::KeyReleased(Key::SHIFT));
    for modifier in [Key::CONTROL, Key::ALT] {
        events.push(Event::KeyPressed(modifier));
        events.extend(tap(Key::B));
        events.push(Event::KeyReleased(modifier));
    }
    events.extend(tap(Key::C));
    assert_eq!(text_of(&events), "aA@c");
}

// A key's press and the text that it types are one step, so a widget that
// hears the press hears its text with it.
#[test]
fn a_key_s_press_and_its_text_are_one_step() {
    let mut input = Input::default();
    let events = [Event::KeyPressed(Key::X), Event::KeyReleased(Key::X)];
    let steps = input.steps(&events);
    assert_eq!(steps, [&events[0..1], &events[1..2]]);
    input.begin(0.0, steps[0]);
    assert!(input.key_pressed(Key::X));
    assert_eq!(input.text, "x");
    input.begin(0.0, steps[1]);
    assert_eq!(input.text, "");
}

// A test types text with the keys that type it, and fails on a character
// that no key types.
#[derive(Default)]
struct Typing {
    typed: String,
}

impl Program for Typing {
    type Message = String;

    fn title(&self) -> String {
        "Typing".to_string()
    }

    fn input(&self, ui: &Ui) -> Vec<String> {
        vec![ui.input().text.clone()]
    }

    fn update(&mut self, message: String, _ui: &mut Context<String>) {
        self.typed.push_str(&message);
    }

    fn view(&self, _ui: &mut Ui) -> Tree<String> {
        Tree::new()
    }
}

#[test]
fn a_test_types_text_with_its_keys() {
    let mut app = Headless::new(Typing::default());
    app.type_text("Hi, 42!");
    assert_eq!(app.program.typed, "Hi, 42!");
}

// A Shift that the test holds stays held after the text is typed, and the
// characters that need no Shift are typed without it.
#[test]
fn typing_keeps_a_held_shift() {
    let mut app = Headless::new(Typing::default());
    app.hold(Key::SHIFT).type_text("aB!c");
    assert_eq!(app.program.typed, "aB!c");
    assert!(app.ui.input().key_down(Key::SHIFT));
    app.release_key(Key::SHIFT).type_text("AB");
    assert_eq!(app.program.typed, "aB!cAB");
    assert!(!app.ui.input().key_down(Key::SHIFT));
}

// A script's Type step after a Hold of Shift keeps it held too.
#[test]
fn a_script_types_with_a_held_shift() {
    let app = Headless::new(Typing::default());
    let script = user_interface::Script::parse(r#"{ "Steps": [ { "Frame": 1, "Hold": ["Shift"] }, { "Frame": 2, "Type": "aB" } ] }"#).unwrap();
    let mut runner = user_interface::ScriptRunner::new(script);
    runner.frame(1, &app.ui).unwrap();
    let typed: Vec<Event> = runner.frame(2, &app.ui).unwrap().events.into_iter().filter(|event| !matches!(event, Event::Gamepad(_))).collect();
    assert_eq!(
        typed,
        [
            Event::KeyReleased(Key::SHIFT),
            Event::KeyPressed(Key::A),
            Event::KeyReleased(Key::A),
            Event::KeyPressed(Key::SHIFT),
            Event::KeyPressed(Key::B),
            Event::KeyReleased(Key::B),
        ]
    );
}

#[test]
#[should_panic(expected = "can't type 'é'")]
fn a_test_can_t_type_what_no_key_types() {
    Headless::new(Typing::default()).type_text("café");
}
