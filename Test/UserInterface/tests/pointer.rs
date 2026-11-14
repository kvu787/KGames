// Tests the pointer, which is the interface's (input/events.rs): the mice's
// motion moves it, in GameInput's counts times MOUSE_SENSITIVITY, inside the
// frame, from the frame's middle, where it starts with the mice's first
// event. And the motion that tests and scripts make to point at a place,
// which lands on it exactly.

use user_interface::input::events::{MOUSE_SENSITIVITY, motion_to, moved_pointer};
use user_interface::{Context, Event, Headless, Input, Key, MouseButton, Point, Program, Size, Tree, Ui};

const SIZE: Size = Size::new(2560.0, 1440.0);
const MIDDLE: Point = Point::new(1280.0, 720.0);

fn new_input() -> Input {
    let mut input = Input::default();
    input.resize(SIZE);
    input
}

fn moved(x: f64, y: f64) -> Event {
    Event::MouseMoved { x, y }
}

// There's no pointer until the mice's first event, which starts it in the
// frame's middle, and it carries on from where it is after that.
#[test]
fn the_pointer_starts_in_the_middle_with_the_mice_s_first_event() {
    let mut input = new_input();
    input.begin(0.0, &[Event::KeyPressed(Key::A)]);
    assert_eq!(input.pointer, None);
    input.begin(0.0, &[moved(2.0, -1.0)]);
    assert_eq!(input.pointer, Some(Point::new(1280.0 + 1.6, 720.0 - 0.8)));
    input.begin(0.0, &[moved(40.0, 0.0)]);
    assert_eq!(input.pointer, Some(Point::new(1280.0 + 33.6, 720.0 - 0.8)));
    // A press is the mice's first event too, in the middle.
    for first in [Event::ButtonPressed(MouseButton::Left), Event::Wheel(Point::new(0.0, 1.0))] {
        let mut input = new_input();
        input.begin(0.0, &[first]);
        assert_eq!(input.pointer, Some(MIDDLE));
    }
}

// The pointer moves by the counts times MOUSE_SENSITIVITY, in parts of a
// pixel, worked out in f64.
#[test]
fn the_pointer_moves_by_the_counts_times_the_mouse_sensitivity() {
    assert_eq!(MOUSE_SENSITIVITY, 0.8);
    let mut input = new_input();
    input.begin(0.0, &[moved(0.0, 0.0), moved(1.0, 1.0)]);
    assert_eq!(input.pointer, Some(Point::new(1280.8, 720.8)));
    // The pointer is kept as f32s, so moves add up to within a rounding of
    // each.
    input.begin(0.0, &[moved(1.0, 1.0), moved(1.0, 1.0)]);
    let pointer = input.pointer.unwrap();
    assert!((pointer.x - 1282.4).abs() < 1e-3 && (pointer.y - 722.4).abs() < 1e-3, "{pointer:?}");
    // A step's motion says how far it went.
    input.begin(0.0, &[moved(-10.0, 5.0)]);
    assert_eq!(input.pointer_delta, Point::new(-8.0, 4.0));
}

// The pointer stays in the frame, as Windows' cursor, which the window keeps
// inside it, does: motion past an edge is lost, and motion back moves it off
// the edge at once. A smaller frame takes it in.
#[test]
fn the_pointer_stays_inside_the_frame() {
    let mut input = new_input();
    input.begin(0.0, &[moved(-5000.0, 5000.0)]);
    assert_eq!(input.pointer, Some(Point::new(0.0, 1439.0)));
    input.begin(0.0, &[moved(1.0, -1.0)]);
    assert_eq!(input.pointer, Some(Point::new(0.8, 1438.2)));
    input.begin(0.0, &[moved(5000.0, -5000.0)]);
    assert_eq!(input.pointer, Some(Point::new(2559.0, 0.0)));
    input.resize(Size::new(1000.0, 500.0));
    assert_eq!(input.pointer, Some(Point::new(999.0, 0.0)));
}

// The motion that a test or a script makes to point at a place lands on it
// exactly, wherever the pointer is, in pixels or parts of them, and a place
// outside the frame is as far as the pointer goes toward it.
#[test]
fn motion_to_a_place_lands_on_it_exactly() {
    let places = [Point::new(0.0, 0.0), Point::new(2559.0, 1439.0), Point::new(123.25, 456.75), Point::new(1279.5, 719.5), Point::new(0.1, 1438.9), Point::new(2000.3, 3.7)];
    let froms = [None, Some(MIDDLE), Some(Point::new(0.3, 0.7)), Some(Point::new(2558.9, 1.1))];
    for from in froms {
        for to in places {
            let Event::MouseMoved { x, y } = motion_to(from, to, SIZE) else {
                unreachable!()
            };
            assert_eq!(moved_pointer(from, x, y, SIZE), to, "from {from:?}");
        }
        let Event::MouseMoved { x, y } = motion_to(from, Point::new(-50.0, 2000.0), SIZE) else {
            unreachable!()
        };
        assert_eq!(moved_pointer(from, x, y, SIZE), Point::new(0.0, 1439.0));
    }
}

// A program that counts the presses on its button.
#[derive(Default)]
struct Pointing {
    clicks: u32,
}

impl Program for Pointing {
    type Message = ();

    fn title(&self) -> String {
        "Pointing".to_string()
    }

    fn update(&mut self, _message: (), _ui: &mut Context<()>) {
        self.clicks += 1;
    }

    fn view(&self, _ui: &mut Ui) -> Tree<()> {
        let mut tree = Tree::new();
        tree.add("Button", user_interface::Rect::new(100.0, 100.0, 200.0, 32.0), user_interface::Button::new("Button").on_press(()));
        tree
    }
}

// Headless points at places as a script does: its moves land exactly, one
// after another in a frame too, and a press goes where the pointer was as
// it went down.
#[test]
fn headless_moves_land_exactly() {
    let mut app = Headless::new(Pointing::default());
    assert_eq!(app.ui.pointer(), None);
    app.move_point(Point::new(123.25, 456.75));
    assert_eq!(app.pointer(), Point::new(123.25, 456.75));
    let button = app.target("Button").center();
    app.queue_move(button).queue(&[Event::ButtonPressed(MouseButton::Left)]).queue_move(Point::new(5.0, 900.0)).frame();
    assert_eq!(app.pointer(), Point::new(5.0, 900.0));
    assert_eq!(app.program.clicks, 1);
    app.release();
    app.click("Button");
    assert_eq!(app.program.clicks, 2);
}
