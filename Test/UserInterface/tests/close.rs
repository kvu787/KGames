// Tests closing. F12 and the taskbar's Close quit at once, under the
// interface, without asking the program (see application/run.rs): F12 never
// reaches the interface. Alt+F4 is only two keys, and F1 one. A program can
// close itself with Context::close, and the steps of its frame after the one
// that closes don't happen.

use user_interface::{Button, Context, Event, Headless, Key, Program, Rect, Tree, Ui};

// A program that closes itself on Q, and counts the presses of A that no
// widget takes, and the keys it was given.
#[derive(Default)]
struct Closing {
    typed: u32,
    keys: Vec<Key>,
}

#[derive(Clone)]
enum Message {
    Key(Key),
    Close,
}

impl Program for Closing {
    type Message = Message;

    fn title(&self) -> String {
        "Closing".to_string()
    }

    fn input(&self, ui: &Ui) -> Vec<Message> {
        let mut messages: Vec<Message> = ui.input().keys_pressed().iter().map(|key| Message::Key(*key)).collect();
        if ui.key_pressed(Key::Q) {
            messages.push(Message::Close);
        }
        messages
    }

    fn update(&mut self, message: Message, ui: &mut Context<Message>) {
        match message {
            Message::Key(key) => {
                self.keys.push(key);
                if key == Key::A {
                    self.typed += 1;
                }
            }
            Message::Close => ui.close(),
        }
    }

    fn view(&self, _ui: &mut Ui) -> Tree<Message> {
        let mut tree = Tree::new();
        tree.add("Button", Rect::new(100.0, 100.0, 200.0, 32.0), Button::new("Button"));
        tree
    }
}

// Alt+F4 is two keys, which the program gets like any others, and which
// close nothing.
#[test]
fn alt_f4_is_only_two_keys() {
    let mut app = Headless::new(Closing::default());
    app.send(&[Event::KeyPressed(Key::ALT), Event::KeyPressed(Key::F4), Event::KeyReleased(Key::F4), Event::KeyReleased(Key::ALT)]);
    assert!(!app.closed());
    assert_eq!(app.program.keys, [Key::ALT, Key::F4]);
}

// The steps after one that closes the program don't happen: a key pressed
// after it, in the same frame, came after the program closed, and mustn't
// change what it closes with, like making its work unsaved.
#[test]
fn nothing_happens_after_a_step_that_closes() {
    let mut app = Headless::new(Closing::default());
    app.send(&[Event::KeyPressed(Key::Q), Event::KeyPressed(Key::A)]);
    assert!(app.closed());
    assert_eq!(app.program.typed, 0);
}

// A key before the step that closes, in the same frame, happens, as it came
// first.
#[test]
fn what_comes_before_a_step_that_closes_happens() {
    let mut app = Headless::new(Closing::default());
    app.send(&[Event::KeyPressed(Key::A), Event::KeyReleased(Key::A), Event::KeyPressed(Key::Q)]);
    assert!(app.closed());
    assert_eq!(app.program.typed, 1);
}

// F1 is a key, which the program gets like any other, and which closes
// nothing.
#[test]
fn f1_is_only_a_key() {
    let mut app = Headless::new(Closing::default());
    app.send(&[Event::KeyPressed(Key::F1), Event::KeyReleased(Key::F1)]);
    assert!(!app.closed());
    assert_eq!(app.program.keys, [Key::F1]);
}

// F12 is the platform's: whoever made a frame's events, it never reaches the
// interface, as a press or a release.
#[test]
#[should_panic(expected = "F12 is the platform's")]
fn f12_never_reaches_the_interface_as_a_press() {
    Headless::new(Closing::default()).send(&[Event::KeyPressed(Key::F12)]);
}

#[test]
#[should_panic(expected = "F12 is the platform's")]
fn f12_never_reaches_the_interface_as_a_release() {
    Headless::new(Closing::default()).send(&[Event::KeyReleased(Key::F12)]);
}

// A script can't press it either: one that does says so before it runs. It
// can press F1.
#[test]
fn a_script_can_t_press_f12() {
    for step in [r#"{ "Frame": 1, "Key": "F12" }"#, r#"{ "Frame": 1, "Key": "Ctrl+F12" }"#, r#"{ "Frame": 1, "Hold": ["W", "F12"] }"#] {
        let script = format!(r#"{{ "Steps": [ {step} ] }}"#);
        let error = user_interface::Script::parse(&script).unwrap_err();
        assert!(error.contains("F12 is the platform's"), "{error}");
    }
    for step in [r#"{ "Frame": 1, "Key": "F1" }"#, r#"{ "Frame": 1, "Hold": ["W", "F1"] }"#] {
        let script = format!(r#"{{ "Steps": [ {step} ] }}"#);
        assert!(user_interface::Script::parse(&script).is_ok(), "{step}");
    }
}
