// Tests scripts: what they parse to, the input their steps make, and how
// expectations match the state.

use user_interface::json;
use user_interface::testing::script::compare;
use user_interface::input::events::moved_pointer;
use user_interface::{Button, Context, Event, Headless, Key, MouseButton, Point, Program, Rect, Script, ScriptRunner, Tree, Ui};

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

#[test]
fn a_point_can_be_a_place_in_a_target() {
    let app = Headless::new(Page);
    let script = Script::parse(r#"{ "Steps": [ { "Frame": 1, "Move": { "Target": "Save", "At": [0.25, 0.5] } } ] }"#).unwrap();
    let events = ScriptRunner::new(script).frame(1, &app.ui).unwrap().events;
    let save = app.target("Save");
    assert_eq!(lands(&app, &events[0]), Point::new(save.x + save.width / 4.0, save.center().y));
}
