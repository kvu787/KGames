// Tests Alt+F4, which comes from GameInput's keys, as Windows' own handling of
// it is swallowed: it asks the program whether it may close, in the step with
// the key, once the step's messages are applied, so that what the program
// shows if it may not, like a prompt about unsaved work, is in that step's
// view, on the frame of the key.

use user_interface::{Button, Context, Event, Headless, Key, Program, Rect, Tree, Ui};

// A program that may close, or not, and shows a prompt when it says no, and
// counts the presses of A that no widget takes.
#[derive(Default)]
struct Closing {
    allowed: bool,
    asked: u32,
    prompt: bool,
    typed: u32,
}

impl Program for Closing {
    type Message = ();

    fn title(&self) -> String {
        "Closing".to_string()
    }

    fn input(&self, ui: &Ui) -> Vec<()> {
        if ui.key_pressed(Key::A) { vec![()] } else { Vec::new() }
    }

    fn update(&mut self, _message: (), _ui: &mut Context<()>) {
        self.typed += 1;
    }

    fn view(&self, _ui: &mut Ui) -> Tree<()> {
        let mut tree = Tree::new();
        if self.prompt {
            tree.add("Prompt", Rect::new(100.0, 100.0, 200.0, 32.0), Button::new("Save first?"));
        }
        tree
    }

    fn may_close(&mut self) -> bool {
        self.asked += 1;
        self.prompt |= !self.allowed;
        self.allowed
    }
}

fn alt_f4() -> Vec<Event> {
    vec![Event::KeyPressed(Key::ALT), Event::KeyPressed(Key::F4)]
}

#[test]
fn alt_f4_closes_a_program_that_may_close() {
    let mut app = Headless::new(Closing {
        allowed: true,
        ..Closing::default()
    });
    app.send(&alt_f4());
    assert_eq!(app.program.asked, 1);
    assert!(app.closed());
}

// The steps after an Alt+F4 that closes don't happen: a key pressed after
// it, in the same frame, came after the program said it may close, and
// mustn't change what it closes with, like making its work unsaved.
#[test]
fn nothing_happens_after_an_alt_f4_that_closes() {
    let mut app = Headless::new(Closing {
        allowed: true,
        ..Closing::default()
    });
    let mut events = alt_f4();
    events.push(Event::KeyPressed(Key::A));
    app.send(&events);
    assert!(app.closed());
    assert_eq!(app.program.typed, 0);
}

// A key before an Alt+F4 in the same frame happens, as it came first.
#[test]
fn what_comes_before_an_alt_f4_that_closes_happens() {
    let mut app = Headless::new(Closing {
        allowed: true,
        ..Closing::default()
    });
    let mut events = vec![Event::KeyPressed(Key::A), Event::KeyReleased(Key::A)];
    events.extend(alt_f4());
    app.send(&events);
    assert!(app.closed());
    assert_eq!(app.program.typed, 1);
}

#[test]
fn alt_f4_that_the_program_refuses_shows_its_prompt_on_the_same_frame() {
    let mut app = Headless::new(Closing::default());
    app.send(&alt_f4());
    assert_eq!(app.program.asked, 1);
    assert!(!app.closed());
    assert!(app.has_target("Prompt"));
}

// Two Alt+F4s in a frame ask twice, as two clicks on a window's close button
// would, and never collapse into one.
#[test]
fn two_alt_f4s_in_a_frame_ask_twice() {
    let mut app = Headless::new(Closing::default());
    let mut events = alt_f4();
    events.extend([Event::KeyReleased(Key::F4), Event::KeyPressed(Key::F4)]);
    app.send(&events);
    assert_eq!(app.program.asked, 2);
    assert!(!app.closed());
}

// F4 alone, or Alt alone, is a key like any other.
#[test]
fn f4_without_alt_asks_nothing() {
    let mut app = Headless::new(Closing::default());
    app.send(&[Event::KeyPressed(Key::F4), Event::KeyReleased(Key::F4), Event::KeyPressed(Key::ALT)]);
    assert_eq!(app.program.asked, 0);
    assert!(!app.has_target("Prompt"));
}
