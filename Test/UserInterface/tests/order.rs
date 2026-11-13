// Tests that a frame's input applies in the order it happened, each event on
// the state that the events before it left, as a Win32 window's messages do:
// typing and Backspace, clicks, a click on a menu or a dialog that a click
// before it opened, shortcuts and the modifiers held as their key went down,
// a gamepad button tapped within a frame, and the frame's time, which passes
// once, after its events. Each test puts all of its input in one frame, and
// checks everything it caused on that frame.

use user_interface::{
    Button, Context, Dialog, DialogResult, Event, Gamepad, GamepadButtons, Headless, Key, MenuItem, Modifiers, MouseButton, Point, Program, Rect,
    TextBox, Tree, Ui,
};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Add,
    Text(String),
    Chosen(String),
    Undo,
    Jump,
    Tick(f32, f64),
    Ask,
    Answer(DialogResult),
}

#[derive(Default)]
struct Order {
    clicks: u32,
    text: String,
    chosen: Vec<String>,
    undos: u32,
    jumps: u32,
    // Each step's time since the step before, and time, as `input` saw them.
    ticks: Vec<(f32, f64)>,
    dialog: bool,
}

impl Program for Order {
    type Message = Msg;

    fn title(&self) -> String {
        "Order".to_string()
    }

    fn input(&self, ui: &Ui) -> Vec<Msg> {
        let mut messages = Vec::new();
        if ui.shortcut(Modifiers::CONTROL, Key::Z) {
            messages.push(Msg::Undo);
        }
        if ui.input().gamepad_pressed(GamepadButtons::A) {
            messages.push(Msg::Jump);
        }
        messages.push(Msg::Tick(ui.delta(), ui.time()));
        messages
    }

    fn update(&mut self, message: Msg, _ui: &mut Context<Msg>) {
        match message {
            Msg::Add => self.clicks += 1,
            Msg::Text(text) => self.text = text,
            Msg::Chosen(label) => self.chosen.push(label),
            Msg::Undo => self.undos += 1,
            Msg::Jump => self.jumps += 1,
            Msg::Tick(delta, time) => self.ticks.push((delta, time)),
            Msg::Ask => self.dialog = true,
            Msg::Answer(_) => self.dialog = false,
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Msg> {
        let mut tree = Tree::new();
        let top = ui.content_top();
        tree.add("Add", Rect::new(10.0, top + 10.0, 100.0, 32.0), Button::new("Add").on_press(Msg::Add));
        tree.add("Field", Rect::new(10.0, top + 60.0, 200.0, 32.0), TextBox::new(&self.text).on_change(Msg::Text));
        let menu = vec![MenuItem::new("Open").on_choose(Msg::Chosen("Open".into()))];
        tree.add("Menu", Rect::new(10.0, top + 110.0, 100.0, 32.0), Button::new("Menu").menu(menu));
        tree.add("Ask", Rect::new(10.0, top + 160.0, 100.0, 32.0), Button::new("Ask").on_press(Msg::Ask));
        if self.dialog {
            let dialog = Dialog::new("Sure?", "Really.").primary("Yes").on_result(Msg::Answer);
            let area = dialog.area(ui);
            tree.container(Some("Question"), area.rect, dialog, |_| {});
        }
        tree
    }
}

// Where nothing is.
const AWAY: Point = Point::new(5.0, 900.0);

fn press(key: Key) -> Event {
    Event::KeyPressed { key, repeat: false }
}

fn click_at(point: Point) -> [Event; 3] {
    [Event::PointerMoved(point), Event::ButtonPressed(MouseButton::Left), Event::ButtonReleased(MouseButton::Left)]
}

// Typing "a", Backspace and "b" in one frame deletes the "a": Backspace
// deletes what's before the caret as the keys before it left it.
#[test]
fn keys_and_their_text_apply_in_the_order_they_were_typed() {
    let mut app = Headless::new(Order::default());
    app.click("Field").type_text("xy");
    app.send(&[
        press(Key::A),
        Event::Text('a'),
        Event::KeyReleased(Key::A),
        press(Key::BACKSPACE),
        Event::KeyReleased(Key::BACKSPACE),
        press(Key::B),
        Event::Text('b'),
        Event::KeyReleased(Key::B),
    ]);
    assert_eq!(app.program.text, "xyb");
    assert_eq!(app.label("Field").as_deref(), Some("xyb"));
}

// Two clicks on a button in one frame press it twice: an intent that comes
// twice applies twice, and never collapses into one.
#[test]
fn two_clicks_in_one_frame_are_two_presses() {
    let mut app = Headless::new(Order::default());
    let add = app.target("Add").center();
    let mut events = click_at(add).to_vec();
    events.extend([Event::ButtonPressed(MouseButton::Left), Event::ButtonReleased(MouseButton::Left)]);
    app.send(&events);
    assert_eq!(app.program.clicks, 2);
}

// A press lands where the pointer was as the button went down, not where it
// went after, in the same frame.
#[test]
fn a_press_lands_where_the_pointer_was_as_it_went_down() {
    let mut app = Headless::new(Order::default());
    let add = app.target("Add").center();
    app.send(&[Event::PointerMoved(add), Event::ButtonPressed(MouseButton::Left), Event::PointerMoved(AWAY)]);
    assert_eq!(app.program.clicks, 1);
    assert_eq!(app.pointer(), AWAY);
    app.release();
    assert_eq!(app.program.clicks, 1);
}

// A click that opens a menu, and a click on its item, in one frame, choose the
// item: the second click lands on the menu that the first one opened.
#[test]
fn a_click_lands_on_the_menu_that_a_click_before_it_in_the_frame_opened() {
    let mut app = Headless::new(Order::default());
    app.click("Menu");
    let open = app.target("Menu/Menu/Open").center();
    app.click_point(AWAY);
    assert_eq!(app.ui.popup_name(), None);
    let menu = app.target("Menu").center();
    let mut events = click_at(menu).to_vec();
    events.extend(click_at(open));
    app.send(&events);
    assert_eq!(app.program.chosen, ["Open"]);
    assert_eq!(app.ui.popup_name(), None);
}

// A click that opens a dialog, and a click on the page under it, in one frame:
// the dialog takes the second click from the page.
#[test]
fn a_dialog_that_a_click_opened_takes_the_next_click_in_the_frame() {
    let mut app = Headless::new(Order::default());
    let ask = app.target("Ask").center();
    let add = app.target("Add").center();
    let mut events = click_at(ask).to_vec();
    events.extend(click_at(add));
    app.send(&events);
    assert!(app.program.dialog);
    assert_eq!(app.program.clicks, 0);
    assert_eq!(app.ui.state().get("Dialog"), Some(&user_interface::json::Value::Bool(true)));
}

// A shortcut is its key's press with the modifiers held as the key went down,
// whatever's let go of later in the frame, and two of its presses in a frame
// are two.
#[test]
fn a_shortcut_sees_the_modifiers_held_as_its_key_went_down() {
    let mut app = Headless::new(Order::default());
    app.send(&[press(Key::CONTROL), press(Key::Z), Event::KeyReleased(Key::CONTROL), Event::KeyReleased(Key::Z)]);
    assert_eq!(app.program.undos, 1, "Ctrl was held as Z went down");
    app.send(&[press(Key::Z), press(Key::CONTROL), Event::KeyReleased(Key::Z), Event::KeyReleased(Key::CONTROL)]);
    assert_eq!(app.program.undos, 1, "Ctrl went down after Z");
    app.send(&[press(Key::CONTROL), press(Key::Z), Event::KeyReleased(Key::Z), press(Key::Z), Event::KeyReleased(Key::Z), Event::KeyReleased(Key::CONTROL)]);
    assert_eq!(app.program.undos, 3, "two presses of Ctrl+Z");
}

// A gamepad button pressed and let go of within one frame is a press.
#[test]
fn a_gamepad_button_tapped_within_a_frame_is_pressed() {
    let mut app = Headless::new(Order::default());
    let holding_a = Gamepad {
        connected: true,
        buttons: GamepadButtons(GamepadButtons::A),
        ..Gamepad::default()
    };
    app.send(&[Event::Gamepad(holding_a), Event::Gamepad(Gamepad::default())]);
    assert_eq!(app.program.jumps, 1);
    assert!(!app.ui.gamepad().buttons.contains(GamepadButtons::A));
}

// The frame's time passes once, in its last step, after its events: the
// steps before it see no time pass, and the last frame's time, so that a
// simulation that `input` ticks moves once a frame, by the frame's time.
#[test]
fn a_frame_s_time_passes_once_after_its_events() {
    let mut app = Headless::new(Order::default());
    let before = app.time();
    app.program.ticks.clear();
    app.send(&[press(Key::Q), Event::KeyReleased(Key::Q), press(Key::W)]);
    let now = app.time();
    assert_eq!(app.program.ticks, [(0.0, before), (0.0, before), ((now - before) as f32, now)]);
    // A frame of one step: its time passes in it.
    app.program.ticks.clear();
    app.frame();
    assert_eq!(app.program.ticks, [((app.time() - now) as f32, app.time())]);
}
