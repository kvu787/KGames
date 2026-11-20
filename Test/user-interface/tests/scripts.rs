// Tests scripts: what they parse to, the input their steps make, and how
// expectations match the state.

use user_interface::json;
use user_interface::testing::script::compare;
use user_interface::input::events::moved_pointer;
use user_interface::{Button, Context, Event, Headless, Key, Modifiers, MouseButton, Point, Program, Rect, Script, ScriptRunner, Shortcut, Tree, Ui};

struct Page;

impl Program for Page {
    type Message = ();

    fn title(&self) -> String {
        "Page".to_string()
    }

    fn update(&mut self, _message: (), _ui: &mut Context<()>) {}

    fn view(&self, _ui: &mut Ui) -> Tree<()> {
        let mut tree = Tree::new();
        tree.add("Save", Rect::new(100.0, 100.0, 80.0, 32.0), Button::new("Save"));
        tree
    }
}

#[test]
fn scripts_say_what_is_wrong_with_them() {
    assert!(Script::parse("{").is_err());
    assert!(Script::parse(r#"{ "Steps": [ { "Move": [1, 2] } ] }"#).unwrap_err().contains("Frame"));
    assert!(Script::parse(r#"{ "Steps": [ { "Frame": 1, "Fly": true } ] }"#).unwrap_err().contains("Fly"));
    assert!(Script::parse(r#"{ "Steps": [ { "Frame": 1, "Move": [1, 2], "Click": [1, 2] } ] }"#).is_err());
    // A step that's wrong fails as the script is read, before the run.
    let error = Script::parse(r#"{ "Steps": [ { "Frame": 1, "Key": "F3" }, { "Frame": 9, "Drag": { "From": [0, 0] } } ] }"#).unwrap_err();
    assert!(error.contains("frame 9") && error.contains("Drag") && error.contains("To"), "{error}");
    let script = Script::parse(r#"{ "FrameTime": 0.01, "Steps": [ { "Frame": 7, "Key": "F3" }, { "Frame": 3, "Key": "F2" } ] }"#).unwrap();
    assert_eq!(script.frame_time, 0.01);
    assert_eq!(script.last_frame(), 7);
}

#[test]
fn steps_make_input_on_their_frames() {
    let app = Headless::new(Page);
    let script = Script::parse(
        r#"{ "Steps": [
            { "Frame": 1, "Click": "Save" },
            { "Frame": 3, "Key": "Ctrl+S" },
            { "Frame": 5, "Hold": ["W"] },
            { "Frame": 6, "Hold": [] },
            { "Frame": 7, "Drag": { "From": [0, 0], "To": [10, 0], "Frames": 2, "Button": "Middle" } },
            { "Frame": 12, "Press": { "At": [3, 4], "Button": "Right" } }
        ] }"#,
    )
    .unwrap();
    let mut runner = ScriptRunner::new(script);
    let mut frame = |number| runner.frame(number, &app.ui).unwrap().events;
    // Frame 0 has nothing but the gamepad, which reads every frame.
    assert!(matches!(frame(0)[..], [Event::Gamepad(_)]));
    let first = frame(1);
    assert_eq!(lands(&app, &first[0]), app.target("Save").center());
    assert_eq!(first[1], Event::ButtonPressed(MouseButton::Left));
    assert_eq!(frame(2)[0], Event::ButtonReleased(MouseButton::Left));
    let third = frame(3);
    assert_eq!(&third[..2], &[Event::KeyPressed(Key::CONTROL), Event::KeyPressed(Key::S)]);
    assert_eq!(&frame(4)[..2], &[Event::KeyReleased(Key::S), Event::KeyReleased(Key::CONTROL)]);
    assert_eq!(frame(5)[0], Event::KeyPressed(Key::W));
    assert_eq!(frame(6)[0], Event::KeyReleased(Key::W));
    assert_eq!(frame(7)[1], Event::ButtonPressed(MouseButton::Middle));
    frame(8);
    assert_eq!(lands(&app, &frame(9)[0]), Point::new(10.0, 0.0));
    assert_eq!(frame(10)[0], Event::ButtonReleased(MouseButton::Middle));
    frame(11);
    let twelfth = frame(12);
    assert_eq!((lands(&app, &twelfth[0]), &twelfth[1]), (Point::new(3.0, 4.0), &Event::ButtonPressed(MouseButton::Right)));
}

// Where the mice's motion moves the pointer from where the app has it, as a
// step that points at a place makes the motion that takes it there.
fn lands(app: &Headless<Page>, event: &Event) -> Point {
    let Event::MouseMoved { x, y } = event else {
        panic!("{event:?} isn't the mice's motion");
    };
    moved_pointer(app.ui.pointer(), *x, *y, app.size())
}

#[test]
fn a_step_that_points_at_a_widget_the_frame_did_not_show_fails() {
    let app = Headless::new(Page);
    let missing = Script::parse(r#"{ "Steps": [ { "Frame": 1, "Click": "Open" } ] }"#).unwrap();
    let error = ScriptRunner::new(missing).frame(1, &app.ui).err().unwrap();
    assert!(error.contains("Open") && error.contains("Save"), "{error}");
}

#[test]
fn expectations_match_what_they_name() {
    let state = json::parse(r#"{ "Volume": 50.0000001, "Name": "Ada", "List": [1, 2], "Interface": { "Popup": null, "Focused": "Name" } }"#).unwrap();
    let check = |expected: &str| {
        let mut mismatches = Vec::new();
        compare(&json::parse(expected).unwrap(), &state, &mut String::new(), &mut mismatches);
        mismatches
    };
    assert!(check(r#"{ "Volume": 50, "Interface": { "Popup": null } }"#).is_empty());
    assert_eq!(check(r#"{ "Name": "Grace" }"#), vec![r#"Name is "Ada", not "Grace""#.to_string()]);
    assert_eq!(check(r#"{ "Interface": { "Focused": null } }"#).len(), 1);
    assert_eq!(check(r#"{ "Missing": 1 }"#), vec!["Missing is missing".to_string()]);
    assert_eq!(check(r#"{ "List": [1] }"#).len(), 1);
}

// A script's drag, run frame by frame, moves the pointer to each of its
// points from where the frame before left it, as the motion that takes it
// there.
#[test]
fn a_drag_lands_on_each_of_its_points_frame_by_frame() {
    let mut app = Headless::new(Page);
    let script = Script::parse(r#"{ "Steps": [ { "Frame": 1, "Drag": { "From": [100, 300], "To": [400, 600], "Frames": 3 } } ] }"#).unwrap();
    let mut runner = ScriptRunner::new(script);
    let mut pointers = Vec::new();
    for number in 0..=4 {
        let events = runner.frame(number, &app.ui).unwrap().events;
        app.send(&events);
        pointers.push(app.pointer());
    }
    assert_eq!(pointers[1..], [Point::new(100.0, 300.0), Point::new(200.0, 400.0), Point::new(300.0, 500.0), Point::new(400.0, 600.0)]);
}

// A run goes on until the last frame that its steps' input reaches, as the
// main loop ends it, so a drag, a click or a key that's the last step lets
// go, and nothing stays held.
#[test]
fn a_run_ends_once_its_last_gesture_lets_go() {
    let app = Headless::new(Page);
    let run = |steps: &str| {
        let mut runner = ScriptRunner::new(Script::parse(&format!(r#"{{ "Steps": [ {steps} ] }}"#)).unwrap());
        let mut releases = Vec::new();
        for number in 0.. {
            let events = runner.frame(number, &app.ui).unwrap().events;
            if events.iter().any(|event| matches!(event, Event::ButtonReleased(_) | Event::KeyReleased(_))) {
                releases.push(number);
            }
            if number >= runner.last_frame() {
                return (number, releases);
            }
        }
        unreachable!()
    };
    assert_eq!(run(r#"{ "Frame": 1, "Key": "F3" }, { "Frame": 5, "Drag": { "From": [0, 0], "To": [10, 0], "Frames": 10 } }"#), (16, vec![2, 16]));
    assert_eq!(run(r#"{ "Frame": 3, "Click": "Save" }"#), (4, vec![4]));
    assert_eq!(run(r#"{ "Frame": 3, "Key": "Ctrl+S" }"#), (4, vec![4]));
    assert_eq!(run(r#"{ "Frame": 3, "Press": "Save" }"#), (3, vec![]));
}

// A script's keys are what a keyboard's readings give the interface, which
// doesn't tell the left Shift, Ctrl and Alt from the right ones: a held
// LeftCtrl makes S Ctrl+S, on the frame of the S, as a real one does.
#[test]
fn a_script_s_left_and_right_modifiers_are_the_modifiers() {
    let mut app = Headless::new(Page);
    let script = Script::parse(
        r#"{ "Steps": [
            { "Frame": 1, "Hold": ["LeftCtrl", "RightCtrl", "LeftShift"] },
            { "Frame": 2, "Key": "S" },
            { "Frame": 4, "Hold": [] },
            { "Frame": 5, "Key": "Shift+RightAlt" }
        ] }"#,
    )
    .unwrap();
    let mut runner = ScriptRunner::new(script);
    let mut frame = |app: &mut Headless<Page>, number| {
        let events = runner.frame(number, &app.ui).unwrap().events;
        app.send(&events);
        events
    };
    frame(&mut app, 0);
    assert_eq!(frame(&mut app, 1)[..2], [Event::KeyPressed(Key::CONTROL), Event::KeyPressed(Key::SHIFT)]);
    frame(&mut app, 2);
    assert!(app.ui.input().shortcut_pressed(Shortcut::new(Modifiers::CONTROL_SHIFT, Key::S)));
    frame(&mut app, 3);
    frame(&mut app, 4);
    assert_eq!(frame(&mut app, 5)[..2], [Event::KeyPressed(Key::SHIFT), Event::KeyPressed(Key::ALT)]);
    assert_eq!(app.ui.modifiers(), Modifiers::new(false, true, true));
}

// A program that saves with Save, and closes itself with Quit.
#[derive(Default)]
struct Quitting {
    saved: u32,
}

#[derive(Clone)]
enum Quit {
    Save,
    Quit,
}

impl Program for Quitting {
    type Message = Quit;

    fn title(&self) -> String {
        "Quitting".to_string()
    }

    fn update(&mut self, message: Quit, ui: &mut Context<Quit>) {
        match message {
            Quit::Save => self.saved += 1,
            Quit::Quit => ui.close(),
        }
    }

    fn view(&self, _ui: &mut Ui) -> Tree<Quit> {
        let mut tree = Tree::new();
        tree.add("Save", Rect::new(100.0, 100.0, 80.0, 32.0), Button::new("Save").on_press(Quit::Save));
        tree.add("Quit", Rect::new(200.0, 100.0, 80.0, 32.0), Button::new("Quit").on_press(Quit::Quit));
        tree
    }

    fn state(&self) -> json::Value {
        json::Value::Object(vec![("Saved".to_string(), json::Value::Number(self.saved as f64))])
    }
}

// Runs the script's frames as the main loop does, until the run is done:
// the frame number it ended at, and what failed.
fn run_until_done(app: &mut Headless<Quitting>, script: &str) -> (u64, Vec<String>) {
    let mut runner = ScriptRunner::new(Script::parse(script).unwrap());
    for number in 0..100 {
        let frame = runner.frame(number, &app.ui).unwrap();
        app.send(&frame.events);
        if runner.end_frame(number, &frame, &app.state(), app.closed()) {
            return (number, runner.failures);
        }
    }
    panic!("the run never ended");
}

// A program that closes itself in a scripted run ends it after the frame
// that closes it, whose expectations are still checked, and the steps after
// it never run, which fails the run.
#[test]
fn a_program_that_closes_itself_ends_the_run_once_its_frame_is_checked() {
    let mut app = Headless::new(Quitting::default());
    let (ended, failures) = run_until_done(
        &mut app,
        r#"{ "Steps": [
            { "Frame": 1, "Click": "Quit" },
            { "Frame": 1, "Expect": { "Saved": 1 } },
            { "Frame": 30, "Click": "Save" },
            { "Frame": 31, "Expect": { "Saved": 1 } }
        ] }"#,
    );
    assert_eq!(ended, 1);
    assert_eq!(failures, [
        "Frame 1: Saved is 0, not 1",
        "Frame 1: the program closed before 2 of the script's steps ran, from the Click step at frame 30",
    ]);

    // A script that ends with the frame that closes passes, without the
    // release of its click, which comes after the program closed.
    let mut app = Headless::new(Quitting::default());
    let (ended, failures) = run_until_done(&mut app, r#"{ "Steps": [ { "Frame": 1, "Click": "Quit" }, { "Frame": 1, "Expect": { "Saved": 0 } } ] }"#);
    assert_eq!((ended, failures), (1, Vec::<String>::new()));
}

#[test]
fn a_point_can_be_a_place_in_a_target() {
    let app = Headless::new(Page);
    let script = Script::parse(r#"{ "Steps": [ { "Frame": 1, "Move": { "Target": "Save", "At": [0.25, 0.5] } } ] }"#).unwrap();
    let events = ScriptRunner::new(script).frame(1, &app.ui).unwrap().events;
    let save = app.target("Save");
    assert_eq!(lands(&app, &events[0]), Point::new(save.x + save.width / 4.0, save.center().y));
}
