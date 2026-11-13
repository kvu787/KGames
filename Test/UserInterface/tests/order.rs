// Tests that a frame's input applies in the order it happened, each event on
// the state that the events before it left, as a Win32 window's messages do:
// typing and Backspace, clicks, a click on a menu or a dialog that a click
// before it opened, shortcuts and the modifiers held as their key went down,
// a gamepad button tapped within a frame, text typed after a key, and the
// frame's time, which passes once, after its events, in its last step. Each
// test puts all of its input in one frame, and checks everything it caused
// on that frame.

use user_interface::{
    Button, ComboBox, Context, Cursor, Dialog, DialogResult, Event, Gamepad, GamepadButtons, Headless, Key, MenuItem, Modifiers, MouseButton, NumberBox, Point,
    Program, Rect, TextBox, Tree, Ui,
};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Add,
    Text(String),
    Chosen(String),
    Undo,
    Jump,
    Tick(f32, f64, bool),
    Number(f64),
    Pick(usize),
    // Sends itself, without end.
    Echo,
    Swap,
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
    // Each step's time since the step before, its time, and whether it was
    // its frame's last, as `input` saw them.
    ticks: Vec<(f32, f64, bool)>,
    number: f64,
    pick: usize,
    swapped: bool,
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
        messages.push(Msg::Tick(ui.delta(), ui.time(), ui.last_step()));
        messages
    }

    fn update(&mut self, message: Msg, ui: &mut Context<Msg>) {
        match message {
            Msg::Add => self.clicks += 1,
            Msg::Text(text) => self.text = text,
            Msg::Chosen(label) => self.chosen.push(label),
            Msg::Undo => self.undos += 1,
            Msg::Jump => self.jumps += 1,
            Msg::Tick(delta, time, last) => self.ticks.push((delta, time, last)),
            Msg::Number(number) => self.number = number,
            Msg::Pick(pick) => self.pick = pick,
            Msg::Echo => ui.send(Msg::Echo),
            Msg::Swap => self.swapped = true,
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
        tree.add("Number", Rect::new(10.0, top + 210.0, 200.0, 32.0), NumberBox::new(self.number).range(0.0, 100.0).on_change(Msg::Number));
        let items: Vec<String> = (0..40).map(|index| format!("Item {index}")).collect();
        tree.add("Long", Rect::new(300.0, top + 10.0, 200.0, 32.0), ComboBox::new(self.pick, &items).on_select(Msg::Pick));
        let swap = Rect::new(520.0, top + 10.0, 120.0, 32.0);
        if self.swapped {
            tree.add("Swapped", swap, TextBox::new(&self.text).on_change(Msg::Text));
        } else {
            tree.add("Swap", swap, Button::new("Swap").on_press(Msg::Swap));
        }
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
    Event::KeyPressed(key)
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

// Text typed after a key in one frame, like a script's Enter and then "2",
// is a step after the key's: Enter takes what was typed before it, and the 2
// waits for the next Enter.
#[test]
fn text_typed_after_a_key_in_one_frame_comes_after_it() {
    let mut app = Headless::new(Order::default());
    app.click("Number").key(Key::BACKSPACE);
    app.send(&[Event::Text('4'), press(Key::ENTER), Event::Text('2'), Event::KeyReleased(Key::ENTER)]);
    assert_eq!(app.program.number, 4.0);
    app.key(Key::ENTER);
    assert_eq!(app.program.number, 42.0);
}

// The frame's time passes once, in its last step, after its events: the
// steps before it see no time pass, and the last frame's time, so that a
// simulation that `input` ticks moves once a frame, by the frame's time.
// `last_step` says which step is the last, for what a program does once a
// frame.
#[test]
fn a_frame_s_time_passes_once_after_its_events() {
    let mut app = Headless::new(Order::default());
    let before = app.time();
    app.program.ticks.clear();
    app.send(&[press(Key::Q), Event::KeyReleased(Key::Q), press(Key::W)]);
    let now = app.time();
    assert_eq!(app.program.ticks, [(0.0, before, false), (0.0, before, false), ((now - before) as f32, now, true)]);
    // A frame of one step: its time passes in it.
    app.program.ticks.clear();
    app.frame();
    assert_eq!(app.program.ticks, [((app.time() - now) as f32, app.time(), true)]);
}

// A frame can have any number of steps, like a burst of wheel readings after
// a long stall, each with its own messages, which all settle: the frame
// applies them all. Only a chain of messages that never ends, in one step,
// stops the program.
#[test]
fn a_frame_of_many_steps_applies_all_their_messages() {
    let mut app = Headless::new(Order::default());
    app.program.ticks.clear();
    app.send(&vec![Event::KeyReleased(Key::Q); 10_001]);
    assert_eq!(app.program.ticks.len(), 10_001);
}

#[test]
#[should_panic(expected = "messages without end")]
fn a_chain_of_messages_that_never_ends_stops_the_program() {
    let mut app = Headless::new(Order::default());
    app.send_message(Msg::Echo);
}

// A wheel that turns by less than a notch at a time gives each turn in a step
// of its own. A number box steps, and a combo box's list scrolls, once its
// turns add up to a notch.
#[test]
fn a_wheel_s_turns_add_up_to_notches_across_steps() {
    let mut app = Headless::new(Order::default());
    app.click("Number").move_to("Number");
    app.send(&vec![Event::Wheel(Point::new(0.0, 0.25)); 4]);
    assert_eq!(app.program.number, 1.0);
    app.send(&vec![Event::Wheel(Point::new(0.0, 0.25)); 3]);
    assert_eq!(app.program.number, 1.0, "three quarters of a notch");
    app.send(&[Event::Wheel(Point::new(0.0, 0.25))]);
    assert_eq!(app.program.number, 2.0);
    // The list shows its first items, and scrolls down by one.
    app.click("Long");
    assert!(app.has_target("Long/0"));
    let list = app.target("Long/1").center();
    app.move_point(list);
    app.send(&vec![Event::Wheel(Point::new(0.0, -0.25)); 4]);
    assert!(!app.has_target("Long/0") && app.has_target("Long/1"));
}

// A press in a text field puts the caret where the text is under it as the
// steps before it left the text, scroll and all: typing that scrolls the
// field, and a press in it, in one frame, put the caret where the same
// press in the next frame does.
#[test]
fn a_press_in_a_field_lands_on_the_text_as_the_typing_before_it_scrolled_it() {
    let typed = |count: usize| vec![Event::Text('w'); count];
    let mut apart = Headless::new(Order::default());
    apart.click("Field").send(&typed(60));
    let field = apart.target("Field");
    let near_left = Point::new(field.x + user_interface::controls::TEXT_BOX_PADDING.left + 3.0, field.center().y);
    apart.send(&typed(10)).click_point(near_left).type_text("Z");
    let mut together = Headless::new(Order::default());
    together.click("Field").send(&typed(60));
    let mut events = typed(10);
    events.extend([Event::PointerMoved(near_left), Event::ButtonPressed(MouseButton::Left), Event::ButtonReleased(MouseButton::Left)]);
    together.send(&events).type_text("Z");
    assert_eq!(together.program.text, apart.program.text);
    assert_ne!(together.program.text.find('Z'), Some(0), "the field had scrolled");
}

// The pointer's look is the drawn tree's: a press that puts a text field
// under the pointer shows the text cursor on its own frame, not the next.
#[test]
fn the_cursor_is_the_drawn_tree_s() {
    let mut app = Headless::new(Order::default());
    app.move_to("Swap");
    assert_eq!(app.ui.cursor(), Cursor::Arrow);
    app.press("Swap");
    assert!(app.has_target("Swapped"));
    assert_eq!(app.ui.cursor(), Cursor::Text);
}
