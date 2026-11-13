// Tests the interface's rules for input with a small program: which widget
// the pointer is on, presses, which act as the button goes down, keyboard
// focus, which only text fields take, popups, menus, dialogs, tooltips,
// notices, editing, and palettes.

use user_interface::json::Value;
use user_interface::controls::{MENU_BAR_HEIGHT, MENU_BAR_ITEM_MARGIN, MENU_BAR_ITEM_PADDING};
use user_interface::tokens::{APP_BAR_HEIGHT, PAGE_PADDING, TOOLTIP_DELAY};
use user_interface::{
    AppBar, Button, Card, CheckBox, Clipboard, Color, ColorField, ComboBox, Context, Cover, Dialog, DialogResult, Element, Event, Form, Group, Headless, Icon,
    Key, ListView, MenuBar, MenuItem, Modifiers, MouseButton, NumberBox, Palette, Point, Program, Rect, Segmented, Sense, Severity, Slider,
    Startup, Text, TextBox, ThemeKind, TooltipLine, Tree, Ui, View, ViewInput,
};

// Where the program's part of the window starts, under the PerfBar.
const PERF_BAR: f32 = user_interface::tokens::PERF_BAR_HEIGHT;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Click,
    Under,
    Maybe,
    Trapped,
    Shortcut,
    Late,
    Text(String),
    Choice(usize),
    Rows(usize),
    Check(bool),
    Level(f64),
    Number(f64),
    Chosen(String),
    Area(ViewInput),
    Ask,
    Answer(DialogResult),
    Pick(usize),
    Shrink(usize),
    Beneath(usize),
    Replace,
}

fn recent() -> Vec<MenuItem<Msg>> {
    vec![MenuItem::new("First").on_choose(Msg::Chosen("First".into())), MenuItem::new("Second").on_choose(Msg::Chosen("Second".into()))]
}

fn menu() -> Vec<MenuItem<Msg>> {
    vec![MenuItem::new("Open").accelerator("Ctrl+O").on_choose(Msg::Chosen("Open".into())), MenuItem::separator(), MenuItem::submenu("Recent", recent())]
}

#[derive(Default)]
struct Test {
    clicks: u32,
    shortcut: u32,
    text: String,
    choice: usize,
    covered: bool,
    disabled: bool,
    rows: Option<usize>,
    checked: bool,
    level: f64,
    number: i32,
    chosen: Vec<String>,
    dialog: bool,
    answer: Option<DialogResult>,
    // Right presses on the area.
    secondary: u32,
    // A combo box's items, which a test can take away while its list is open.
    items: Vec<String>,
    shrink: usize,
    // Replaces the field's text with a two-byte character, once.
    replace: bool,
    // A list that a context menu can open over.
    beneath: Option<usize>,
    // A combo box in the dialog.
    pick: usize,
    // Presses of D, which the dialog gates from the frame it opens.
    late: u32,
}

impl Program for Test {
    type Message = Msg;

    fn title(&self) -> String {
        "Test".to_string()
    }

    fn input(&self, ui: &Ui) -> Vec<Msg> {
        let mut messages = Vec::new();
        if ui.key_pressed(Key::A) {
            messages.push(Msg::Shortcut);
        }
        if ui.key_pressed(Key::D) {
            messages.push(Msg::Late);
        }
        if self.replace {
            messages.push(Msg::Replace);
        }
        messages
    }

    fn update(&mut self, message: Msg, _ui: &mut Context<Msg>) {
        match message {
            Msg::Click => self.clicks += 1,
            Msg::Under => self.covered = true,
            Msg::Maybe => self.clicks += 100,
            Msg::Trapped => self.clicks += 1000,
            Msg::Shortcut => self.shortcut += 1,
            // After the dialog's message in the frame's queue, so the frame
            // that opens the dialog gates it.
            Msg::Late => {
                if !self.dialog {
                    self.late += 1;
                }
            }
            Msg::Text(text) => self.text = text,
            Msg::Choice(choice) => self.choice = choice,
            Msg::Rows(row) => self.rows = Some(row),
            Msg::Check(checked) => self.checked = checked,
            Msg::Level(level) => self.level = level,
            Msg::Number(number) => self.number = number as i32,
            Msg::Chosen(label) => self.chosen.push(label),
            Msg::Area(input) => {
                if input.secondary_pressed {
                    self.secondary += 1;
                }
            }
            Msg::Ask => self.dialog = true,
            Msg::Answer(answer) => {
                self.answer = Some(answer);
                self.dialog = false;
            }
            Msg::Pick(pick) => self.pick = pick,
            Msg::Shrink(shrink) => self.shrink = shrink,
            Msg::Beneath(row) => self.beneath = Some(row),
            Msg::Replace => {
                self.text = "\u{e9}".to_string();
                self.replace = false;
            }
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Msg> {
        let mut tree = Tree::new();
        // Under the PerfBar.
        let top = ui.content_top();
        tree.add("Button", Rect::new(10.0, top + 10.0, 100.0, 32.0), Button::new("Button").on_press(Msg::Click)).set_tooltip("A tip");
        tree.add("Field", Rect::new(10.0, top + 60.0, 200.0, 32.0), TextBox::new(&self.text).on_change(Msg::Text));
        tree.add("Combo", Rect::new(10.0, top + 110.0, 200.0, 32.0), ComboBox::new(self.choice, &["Zero", "One", "Two"]).on_select(Msg::Choice));
        // A surface over a button covers it from the pointer.
        tree.add("Under", Rect::new(300.0, top + 10.0, 100.0, 32.0), Button::new("Under").on_press(Msg::Under));
        tree.draw(Rect::new(290.0, top, 200.0, 60.0), Cover);
        tree.add("Maybe", Rect::new(10.0, top + 160.0, 100.0, 32.0), Button::new("Maybe").on_press(Msg::Maybe)).set_enabled(!self.disabled);
        // A widget can't be enabled inside a disabled group.
        tree.push(Element::unnamed(Rect::new(120.0, top + 160.0, 100.0, 32.0), Group).enabled(false).children({
            let mut group = Tree::new();
            group.add("Trapped", Rect::new(120.0, top + 160.0, 100.0, 32.0), Button::new("Trapped").on_press(Msg::Trapped)).set_enabled(true);
            group
        }));
        let rows = Rect::new(500.0, top + 10.0, 200.0, 100.0);
        let list = ListView::new(50, 20.0).selection(self.rows).on_select(Msg::Rows).labels(|index| format!("Row {index}"));
        let shown = list.rows(ui, "Rows", rows);
        tree.container(Some("Rows"), rows, list, |list| {
            for index in shown.indices() {
                list.draw(shown.content(index), Text::label(format!("Row {index}")));
            }
        });
        tree.add("Check", Rect::new(10.0, top + 210.0, 200.0, 32.0), CheckBox::new(self.checked, "Check").on_change(Msg::Check));
        tree.add("Level", Rect::new(10.0, top + 260.0, 220.0, 32.0), Slider::new(self.level, 0.0, 1.0).step(0.1).allowed(0.2, 0.8).on_change(Msg::Level));
        tree.add("Number", Rect::new(10.0, top + 310.0, 200.0, 32.0), NumberBox::new(self.number as f64).range(0.0, 10.0).on_change(Msg::Number));
        tree.add("MenuButton", Rect::new(10.0, top + 360.0, 100.0, 32.0), Button::new("Menu").menu(menu()));
        tree.add("Area", Rect::new(800.0, top + 400.0, 200.0, 200.0), View::new().on_input(Msg::Area)).set_context_menu(recent());
        tree.sized_card(ui, Some("Sized"), Rect::new(1200.0, top + 10.0, 300.0, 1000.0), Card::new("Sized"), |_, card, area| {
            let mut content = area.content;
            card.draw(content.cut_top(20.0, 8.0), Text::label("One"));
            card.draw(content.cut_top(20.0, 0.0), Text::label("Two"));
            content.y
        });
        tree.add("Ask", Rect::new(10.0, top + 410.0, 100.0, 32.0), Button::new("Ask").on_press(Msg::Ask));
        if self.dialog {
            let dialog = Dialog::new("Sure?", "Really.").primary("Yes").content_height(32.0).on_result(Msg::Answer);
            let area = dialog.area(ui);
            tree.container(Some("Question"), area.rect, dialog, |dialog| {
                dialog.add("Pick", area.content.top_strip(32.0), ComboBox::new(self.pick, &["Yes", "No"]).on_select(Msg::Pick));
            });
        }
        tree.add("Shrink", Rect::new(240.0, top + 110.0, 200.0, 32.0), ComboBox::new(self.shrink, &self.items).on_select(Msg::Shrink));
        let beneath = Rect::new(1000.0, top + 400.0, 300.0, 100.0);
        let list = ListView::new(50, 20.0).selection(self.beneath).on_select(Msg::Beneath);
        let shown = list.rows(ui, "Beneath", beneath);
        tree.container(Some("Beneath"), beneath, list, |list| {
            for index in shown.indices() {
                list.draw(shown.content(index), Text::label(format!("Row {index}")));
            }
        });
        tree
    }
}

fn focused(app: &Headless<Test>) -> Option<Value> {
    app.ui.state().get("Focused").cloned()
}

#[test]
fn a_press_acts_on_the_frame_that_the_button_goes_down() {
    let mut app = Headless::new(Test::default());
    app.press("Button");
    assert_eq!(app.program.clicks, 1, "the press, before its release");
    // Sliding off and letting go elsewhere neither takes it back nor
    // presses again.
    let away = Point::new(5.0, 900.0);
    app.move_point(away).release();
    assert_eq!(app.program.clicks, 1);
    // Nor does letting go on the button.
    app.press("Button").release();
    assert_eq!(app.program.clicks, 2);
    // A press and its release in one frame are one press.
    app.move_to("Button").send(&[Event::ButtonPressed(MouseButton::Left), Event::ButtonReleased(MouseButton::Left)]);
    assert_eq!(app.program.clicks, 3);
}

#[test]
fn a_press_that_opens_a_popup_or_a_dialog_does_nothing_more_as_it_is_let_go_of() {
    let mut app = Headless::new(Test::default());
    // The combo box's list opens over it, with the chosen item under the
    // pointer, which the release doesn't choose.
    app.press("Combo");
    assert_eq!(app.ui.popup_name(), Some("Combo"));
    app.release();
    assert_eq!((app.ui.popup_name(), app.program.choice), (Some("Combo"), 0));
    // A press on an item chooses it, and closes the list, as it goes down.
    app.press("Combo/2");
    assert_eq!((app.ui.popup_name(), app.program.choice), (None, 2));
    app.release();
    // A menu, likewise.
    app.press("MenuButton");
    assert_eq!(app.ui.popup_name(), Some("MenuButton/Menu"));
    app.release();
    assert_eq!(app.ui.popup_name(), Some("MenuButton/Menu"));
    app.press("MenuButton/Menu/Open");
    assert_eq!((app.ui.popup_name(), app.program.chosen.as_slice()), (None, ["Open".to_string()].as_slice()));
    app.release();
    // A dialog that a press opens isn't answered by the release over it.
    app.press("Ask");
    assert_eq!(app.ui.state().get("Dialog"), Some(&Value::Bool(true)));
    app.move_to("Question/Primary").release();
    assert_eq!((app.program.dialog, app.program.answer), (true, None));
}

#[test]
fn a_covered_widget_is_not_under_the_pointer() {
    let mut app = Headless::new(Test::default());
    app.click("Under");
    assert!(!app.program.covered);
}

#[test]
fn a_disabled_widget_takes_no_input() {
    let mut app = Headless::new(Test {
        disabled: true,
        ..Test::default()
    });
    app.click("Maybe");
    assert_eq!(app.program.clicks, 0);
    assert!(!app.is_enabled("Maybe"));
}

#[test]
fn a_widget_cannot_be_enabled_inside_a_disabled_group() {
    let mut app = Headless::new(Test::default());
    app.click("Trapped");
    assert_eq!(app.program.clicks, 0);
    assert!(!app.is_enabled("Trapped"));
}

#[test]
fn a_focused_text_field_takes_the_keys_that_type() {
    let mut app = Headless::new(Test::default());
    app.key(Key::A);
    assert_eq!(app.program.shortcut, 1);
    app.click("Field").type_text("ab cd").key(Key::A);
    assert_eq!(app.program.text, "ab cd");
    assert_eq!(app.program.shortcut, 1);
    // Editing: a word goes with Ctrl+Backspace, everything with Ctrl+A.
    app.key_with(Modifiers::CONTROL, Key::BACKSPACE);
    assert_eq!(app.program.text, "ab ");
    app.key_with(Modifiers::CONTROL, Key::A).key_with(Modifiers::CONTROL, Key::C).key(Key::BACKSPACE);
    assert_eq!(app.program.text, "");
    app.key_with(Modifiers::CONTROL, Key::V);
    assert_eq!(app.program.text, "ab ");
    // A press elsewhere takes focus away, and the keys are the program's
    // again.
    app.click_point(Point::new(5.0, 900.0)).key(Key::A);
    assert_eq!(app.program.shortcut, 2);
    assert_eq!(focused(&app), Some(Value::Null));
}

// A press elsewhere takes focus from a text field only once the field has
// taken what was typed in the press's frame.
#[test]
fn a_field_takes_what_was_typed_in_the_frame_of_a_press_elsewhere() {
    let mut app = Headless::new(Test::default());
    app.click("Field");
    let center = app.target("Button").center();
    app.queue(&[Event::Text('x'), Event::PointerMoved(center), Event::ButtonPressed(MouseButton::Left)]).frame();
    assert_eq!((app.program.text.as_str(), app.program.clicks), ("x", 1));
    assert_eq!(focused(&app), Some(Value::Null));
    app.release();
}

// Cut takes the text away only once it's on the clipboard. Windows'
// clipboard can be unavailable for a moment, while another program has it
// open, and then the selection stays.
#[test]
fn a_cut_keeps_the_text_unless_the_clipboard_takes_it() {
    let mut app = Headless::new(Test::default());
    app.click("Field").type_text("keep");
    app.set_clipboard(Clipboard::Unavailable);
    app.key_with(Modifiers::CONTROL, Key::A).key_with(Modifiers::CONTROL, Key::X);
    assert_eq!(app.program.text, "keep");
    app.set_clipboard(Clipboard::Own(String::new()));
    app.key_with(Modifiers::CONTROL, Key::A).key_with(Modifiers::CONTROL, Key::X);
    assert_eq!(app.program.text, "");
    assert_eq!(app.clipboard_text().as_deref(), Some("keep"));
}

#[test]
fn only_a_text_field_takes_focus_and_a_press_elsewhere_takes_it_away() {
    let mut app = Headless::new(Test::default());
    // Buttons, lists and checkboxes take no focus, and Tab moves none.
    app.click("Button").click("Rows/1").click("Check").key(Key::TAB);
    assert_eq!(focused(&app), Some(Value::Null));
    app.click("Field").type_text("x");
    assert_eq!(focused(&app), Some(Value::from("Field")));
    // A press on the field's own clear button leaves focus in the field.
    app.click("Field/Clear");
    assert_eq!((focused(&app), app.program.text.as_str()), (Some(Value::from("Field")), ""));
    // A press on any other widget takes it away, and so does one that no
    // widget takes, of any button.
    app.click("Button");
    assert_eq!(focused(&app), Some(Value::Null));
    app.click("Field").press_button(Point::new(5.0, 900.0), MouseButton::Right).release_button(MouseButton::Right);
    assert_eq!(focused(&app), Some(Value::Null));
}

#[test]
fn a_popup_takes_the_pointer_from_what_is_under_it() {
    let mut app = Headless::new(Test::default());
    app.click("Combo");
    assert_eq!(app.ui.popup_name(), Some("Combo"));
    // The chosen item is where the combo box's text is.
    let chosen = app.target("Combo/0");
    assert!((chosen.center().y - app.target("Combo").center().y).abs() < 1.0);
    app.click("Combo/2");
    assert_eq!(app.program.choice, 2);
    assert_eq!(app.ui.popup_name(), None);
    assert_eq!(app.label("Combo").as_deref(), Some("Two"));
    // A press outside closes it without choosing.
    app.click("Combo").click_point(Point::new(5.0, 900.0));
    assert_eq!((app.program.choice, app.ui.popup_name()), (2, None));
}

#[test]
fn a_tooltip_waits_for_the_pointer_to_rest_and_sits_above_it() {
    let mut app = Headless::new(Test::default());
    app.move_point(Point::new(60.0, PERF_BAR + 30.0)).wait(TOOLTIP_DELAY * 0.6);
    assert_eq!(app.ui.tooltip_name(), None);
    app.wait(TOOLTIP_DELAY);
    assert_eq!(app.ui.tooltip_name(), Some("Button"));
    // There's no room above a button at the window's top, so it's below.
    let state = app.ui.state();
    let area = state.get("Tooltip").and_then(|tooltip| tooltip.get("Area")).cloned().unwrap();
    let Value::Array(area) = area else { panic!() };
    let Value::Number(y) = area[1] else { panic!() };
    assert!(y >= (PERF_BAR + 30.0 + 20.0) as f64, "{y}");
    app.press_point(Point::new(60.0, PERF_BAR + 30.0));
    assert_eq!(app.ui.tooltip_name(), None);
}

#[test]
fn a_list_shows_only_the_rows_in_view() {
    let mut app = Headless::new(Test::default());
    assert!(app.has_target("Rows/4") && !app.has_target("Rows/6"));
    app.click("Rows/1");
    assert_eq!(app.program.rows, Some(1));
    app.scroll("Rows", -20.0);
    assert!(app.has_target("Rows/49") && !app.has_target("Rows/1"));
    app.scroll("Rows", 2.0);
    assert!(app.has_target("Rows/43"));
}

#[test]
fn toggles_sliders_and_number_boxes_change_their_values() {
    let mut app = Headless::new(Test {
        level: 0.5,
        ..Test::default()
    });
    app.click("Check");
    assert!(app.program.checked);
    // The slider stays inside its allowed values, though its rail is wider.
    let (left, _) = user_interface::slider_rail(app.target("Level"));
    app.press_point(Point::new(left, app.target("Level").center().y)).release();
    assert!((app.program.level - 0.2).abs() < 1e-9, "{}", app.program.level);
    app.scroll("Level", 0.5).scroll("Level", 0.5);
    assert!((app.program.level - 0.3).abs() < 1e-9, "half notches add up: {}", app.program.level);
    assert_eq!(app.label("Level").as_deref(), Some("0.30"));
    // A number box takes what's typed with Enter, inside its range.
    app.click("Number").key_with(Modifiers::CONTROL, Key::A).type_text("42").key(Key::ENTER);
    assert_eq!(app.program.number, 10);
    app.click("Number/Down");
    assert_eq!(app.program.number, 9);
}

#[test]
fn menus_open_choose_and_close() {
    let mut app = Headless::new(Test::default());
    app.click("MenuButton");
    assert_eq!(app.ui.popup_name(), Some("MenuButton/Menu"));
    app.click("MenuButton/Menu/Open");
    assert_eq!(app.program.chosen, ["Open"]);
    assert_eq!(app.ui.popup_name(), None);
    // A submenu opens when the pointer is on its item.
    app.click("MenuButton").move_to("MenuButton/Menu/Recent");
    assert!(app.has_target("MenuButton/Menu/Recent/Menu/Second"));
    app.click("MenuButton/Menu/Recent/Menu/Second");
    assert_eq!(app.program.chosen, ["Open", "Second"]);
    assert!(!app.ui.any_popup());
    // A right press opens a context menu where it is, which stays open as
    // the button comes up.
    let at = Point::new(850.0, 450.0);
    app.press_button(at, MouseButton::Right);
    assert_eq!(app.ui.popup_name(), Some("Area/ContextMenu"));
    app.release_button(MouseButton::Right);
    assert_eq!(app.ui.popup_name(), Some("Area/ContextMenu"));
    let menu = app.target("Area/ContextMenu");
    assert!((menu.x - at.x).abs() < 1.0 && (menu.y - at.y).abs() < 1.0);
    app.click("Area/ContextMenu/First");
    assert_eq!(app.program.chosen, ["Open", "Second", "First"]);
    // A press outside a menu closes it, and does nothing else.
    app.click("MenuButton").click("Button");
    assert_eq!((app.program.clicks, app.ui.any_popup()), (0, false));
}

// A press on what a context menu belongs to, outside the menu, closes it and
// does nothing else, as a press anywhere outside a popup does.
#[test]
fn a_press_on_a_context_menu_s_area_outside_it_closes_it() {
    let mut app = Headless::new(Test::default());
    let at = Point::new(850.0, PERF_BAR + 450.0);
    app.press_button(at, MouseButton::Right).release_button(MouseButton::Right);
    assert_eq!(app.ui.popup_name(), Some("Area/ContextMenu"));
    let secondary = app.program.secondary;
    // In the area, left of the menu.
    let away = Point::new(810.0, PERF_BAR + 590.0);
    app.press_button(away, MouseButton::Right).release_button(MouseButton::Right);
    assert_eq!((app.ui.popup_name(), app.program.secondary), (None, secondary));
    app.press_button(at, MouseButton::Right).release_button(MouseButton::Right);
    app.click_point(away);
    assert_eq!(app.ui.popup_name(), None);
}

#[test]
fn a_dialog_takes_the_input_until_it_is_answered() {
    let mut app = Headless::new(Test::default());
    app.click("Ask");
    assert_eq!(app.ui.state().get("Dialog"), Some(&Value::Bool(true)));
    app.click_point(Point::new(60.0, PERF_BAR + 26.0));
    assert_eq!(app.program.clicks, 0, "the page is under the smoke");
    app.key(Key::A);
    assert_eq!(app.program.shortcut, 0, "the program's keys wait too");
    app.click("Question/Close");
    assert_eq!(app.program.answer, Some(DialogResult::Close));
    app.click("Ask").click("Question/Primary");
    assert_eq!(app.program.answer, Some(DialogResult::Primary));
    app.click("Button");
    assert_eq!(app.program.clicks, 1);
}

#[test]
fn a_dialog_gates_the_keys_the_frame_it_opens() {
    let mut app = Headless::new(Test::default());
    app.key(Key::D);
    assert_eq!(app.program.late, 1, "without a dialog the key counts");
    // The press that opens the dialog and a press of D land in one frame.
    // The widgets' messages come before the program's own, so the dialog
    // is open when the key's message is applied, that very frame.
    app.move_to("Ask").frame();
    app.send(&[
        Event::ButtonPressed(MouseButton::Left),
        Event::KeyPressed { key: Key::D, repeat: false },
        Event::KeyReleased(Key::D),
    ]);
    assert_eq!(app.ui.state().get("Dialog"), Some(&Value::Bool(true)));
    assert_eq!(app.program.late, 1);
}

#[test]
fn a_card_sizes_itself_to_its_content() {
    let app = Headless::new(Test::default());
    // Padding, the header and its gap, two rows, and padding.
    assert_eq!(app.target("Sized").height, 16.0 + 32.0 + 8.0 + 20.0 + 8.0 + 20.0 + 16.0);
    let state = app.ui.state();
    let kind = state.get("Targets").and_then(|targets| targets.get("Sized")).and_then(|card| card.get("Kind")).cloned();
    assert_eq!(kind, Some(Value::from("Card")));
}

const COLORS: [Color; 5] = [Color::hex(0xD13438), Color::hex(0xCA5010), Color::hex(0xC19C00), Color::hex(0x107C10), Color::hex(0x0078D4)];

#[derive(Default)]
struct Colors {
    picked: Option<usize>,
}

impl Program for Colors {
    type Message = usize;

    fn title(&self) -> String {
        "Colors".to_string()
    }

    fn update(&mut self, picked: usize, _ui: &mut Context<usize>) {
        self.picked = Some(picked);
    }

    fn view(&self, ui: &mut Ui) -> Tree<usize> {
        let mut tree = Tree::new();
        let top = ui.content_top();
        tree.add("Colors", Rect::new(10.0, top + 10.0, 100.0, 200.0), Palette::new(self.picked, &COLORS).on_pick(|index| index));
        tree
    }
}

#[test]
fn a_palette_lays_its_swatches_out_in_rows_and_picks_the_one_clicked() {
    let mut app = Headless::new(Colors::default());
    app.frame();
    // Swatches a control tall and 8 apart: two fit across 100 pixels, so
    // five make three rows.
    let (first, second, third) = (app.target("Colors/0"), app.target("Colors/1"), app.target("Colors/2"));
    assert_eq!((first.width, first.height), (32.0, 32.0));
    assert_eq!((second.x - first.x, second.y), (40.0, first.y));
    assert_eq!((third.x, third.y - first.y), (first.x, 40.0));
    assert_eq!(Palette::<usize>::new(None, &COLORS).height(100.0), 3.0 * 32.0 + 2.0 * 8.0);
    assert_eq!(app.label("Colors/3").as_deref(), Some("#107C10"));
    assert_eq!(app.program.picked, None);
    app.click("Colors/3");
    assert_eq!(app.program.picked, Some(3));
}

struct Notices;

impl Program for Notices {
    type Message = ();

    fn title(&self) -> String {
        "Notices".to_string()
    }

    fn update(&mut self, _message: (), _ui: &mut Context<()>) {}

    fn view(&self, ui: &mut Ui) -> Tree<()> {
        let mut tree = Tree::new();
        tree.shell(ui, AppBar::new(Icon::Info, "Notices"), false, None);
        tree
    }
}

// A menu bar in the menu row under the app bar.
#[derive(Default)]
struct MenuRow {
    chosen: Option<String>,
}

impl Program for MenuRow {
    type Message = String;

    fn title(&self) -> String {
        "MenuRow".to_string()
    }

    fn update(&mut self, label: String, _ui: &mut Context<String>) {
        self.chosen = Some(label);
    }

    fn view(&self, ui: &mut Ui) -> Tree<String> {
        let mut tree = Tree::new();
        let shell = tree.shell(ui, AppBar::new(Icon::Info, "MenuRow"), true, None);
        let row = shell.menu_row.unwrap();
        let bar = MenuBar::new(vec![("File", vec![MenuItem::new("Open").on_choose("Open".to_string()), MenuItem::separator()])]);
        let width = bar.width(ui);
        tree.add("Menus", row.left_strip(width), bar);
        // The page, as a card that a test can find.
        tree.add("Page", shell.page, Card::new(""));
        tree
    }
}

#[test]
fn the_menu_row_puts_a_menu_bar_in_line_with_the_page_under_the_app_bar() {
    let mut app = Headless::new(MenuRow::default());
    app.frame();
    let row_top = PERF_BAR + APP_BAR_HEIGHT;
    let file = app.target("Menus/File");
    // The first menu's name starts on the page's left edge, and its
    // backplate out in the padding.
    assert_eq!((file.x + MENU_BAR_ITEM_PADDING.left, file.y), (PAGE_PADDING, row_top + MENU_BAR_ITEM_MARGIN.top));
    assert_eq!(app.target("Page").y, row_top + MENU_BAR_HEIGHT + PAGE_PADDING);
    app.click("Menus/File").click("Menus/File/Menu/Open");
    assert_eq!(app.program.chosen.as_deref(), Some("Open"));
}

#[test]
fn notices_from_the_start_show_and_close() {
    let folder = std::env::temp_dir();
    let mut startup = Startup::for_tests(r#"{ "Volume": "loud" }"#, &folder);
    startup.settings.number("Volume", 1.0, |_| Ok(()));
    startup.notify(Severity::Error, "Paint.json", "It's broken.");
    let mut app = Headless::start(startup, |_| Ok(Notices));
    assert!(app.has_target("Notice/0") && app.has_target("Notice/1"));
    assert!(app.label("Notice/0").unwrap().contains("Volume must be a number"));
    app.click("Notice/0/Close");
    assert_eq!(app.ui.notices().len(), 1);
    assert_eq!(app.ui.notices()[0].title, "Paint.json");
}

// A program starts in the theme that its settings say, from its first frame,
// and the settings every program has are the program's to read too.
#[test]
fn a_program_starts_in_the_theme_that_its_settings_say() {
    let folder = std::env::temp_dir();
    let dark = Headless::start(Startup::for_tests(r#"{ "Theme": "Dark" }"#, &folder), |startup| {
        assert_eq!(startup.common.theme, ThemeKind::Dark);
        Ok(Notices)
    });
    assert_eq!(dark.ui.theme().kind, ThemeKind::Dark);
    assert_eq!(dark.ui.state().get("Theme"), Some(&Value::from("Dark")));
    assert!(dark.ui.notices().is_empty());
    let mut light = Headless::start(Startup::for_tests(r#"{ "Theme": "Light" }"#, &folder), |_| Ok(Notices));
    assert_eq!(light.ui.theme().kind, ThemeKind::Light);
    // F9 switches whichever it started in.
    light.key(Key::F9);
    assert_eq!(light.ui.theme().kind, ThemeKind::Dark);
    // A wrong one is light, and a notice says why.
    let wrong = Headless::start(Startup::for_tests(r#"{ "Theme": "Dim" }"#, &folder), |_| Ok(Notices));
    assert_eq!(wrong.ui.theme().kind, ThemeKind::Light);
    assert!(wrong.label("Notice/0").unwrap().contains("Theme must be \"Light\" or \"Dark\""), "{:?}", wrong.label("Notice/0"));
}

// What porting the programs found: segments, which take no focus, a color
// field left while it's typed in, a slider's steps of a thousandth, and a
// wide tooltip.
#[derive(Clone, Debug, PartialEq)]
enum PortedMsg {
    Segment(usize),
    Color(Color),
    Other(String),
    Fine(f64),
}

#[derive(Default)]
struct Ported {
    segment: usize,
    color: Color,
    other: String,
    fine: f64,
}

impl Program for Ported {
    type Message = PortedMsg;

    fn title(&self) -> String {
        "Ported".to_string()
    }

    fn update(&mut self, message: PortedMsg, _ui: &mut Context<PortedMsg>) {
        match message {
            PortedMsg::Segment(segment) => self.segment = segment,
            PortedMsg::Color(color) => self.color = color,
            PortedMsg::Other(other) => self.other = other,
            PortedMsg::Fine(fine) => self.fine = fine,
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<PortedMsg> {
        let mut tree = Tree::new();
        // Under the PerfBar.
        let top = ui.content_top();
        tree.add("Segments", Rect::new(10.0, top + 10.0, 300.0, 32.0), Segmented::new(self.segment, &["A", "B", "C"]).on_select(PortedMsg::Segment));
        tree.add("Hex", Rect::new(10.0, top + 60.0, 200.0, 32.0), ColorField::new(self.color).on_change(PortedMsg::Color));
        tree.add("Other", Rect::new(10.0, top + 110.0, 200.0, 32.0), TextBox::new(&self.other).on_change(PortedMsg::Other));
        tree.add("Fine", Rect::new(10.0, top + 160.0, 300.0, Slider::<PortedMsg>::height(false)), Slider::new(self.fine, 0.0, 1.0).step(0.001).on_change(PortedMsg::Fine));
        let long = "A device's path, like HID#VID_045E&PID_0B13&IG_00#8&2f6c1e3b&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}, and more that goes on after it.";
        tree.add("Wide", Rect::new(10.0, top + 600.0, 100.0, 32.0), Button::new("Wide")).set_tooltip_wide(vec![TooltipLine::text(long)], 640.0);
        tree
    }
}

#[test]
fn segments_take_no_focus() {
    let mut app = Headless::new(Ported::default());
    app.frame();
    app.click("Segments/B");
    assert_eq!(app.program.segment, 1);
    assert_eq!(app.ui.focused_name(), None);
}

#[test]
fn a_color_field_takes_what_was_typed_when_it_loses_focus() {
    let mut app = Headless::new(Ported::default());
    app.frame();
    app.click("Hex");
    app.key_with(Modifiers::CONTROL, Key::A);
    app.type_text("#FF8000");
    assert_eq!(app.program.color, Color::default());
    app.click("Other");
    assert_eq!(app.program.color.to_hex(), "#FF8000");
}

// However focus leaves a number box - a press on a button, or on a view -
// the typed draft commits on that very frame, and no stale draft revives on
// the next edit.
#[test]
fn a_number_box_commits_its_draft_however_focus_leaves() {
    let mut app = Headless::new(Test::default());
    app.frame();
    app.click("Number").key_with(Modifiers::CONTROL, Key::A);
    app.type_text("7");
    assert_eq!(app.program.number, 0, "not before it commits");
    app.press("Button");
    assert_eq!(app.program.number, 7, "the button's press committed the draft on its frame");
    app.release();
    // A fresh edit starts from the value, not a revived draft.
    app.click("Number").key_with(Modifiers::CONTROL, Key::A);
    app.type_text("3");
    app.press("Area");
    assert_eq!(app.program.number, 3, "the view's press committed the draft on its frame");
    app.release();
}

// F10's very frame already says the new setting, PerfBar and state alike.
#[test]
fn f10_shows_the_new_vertical_sync_on_its_own_frame() {
    let mut app = Headless::new(Test::default());
    app.frame();
    app.queue(&[Event::KeyPressed { key: Key::F10, repeat: false }]).frame();
    assert!(!app.vertical_sync());
    assert_eq!(app.ui.state().get("VerticalSync"), Some(&Value::Bool(false)));
}

#[test]
fn f10_switches_vertical_sync_even_while_a_text_field_has_focus() {
    let mut app = Headless::new(Test::default());
    // It starts on, as Headless::new starts a program.
    assert!(app.vertical_sync());
    app.key(Key::F10);
    assert!(!app.vertical_sync());
    assert_eq!(app.ui.state().get("VerticalSync"), Some(&Value::Bool(false)));
    // A text field takes what's typed, but not the function keys.
    app.click("Field").key(Key::F10);
    assert!(app.vertical_sync());
    assert_eq!(app.ui.state().get("VerticalSync"), Some(&Value::Bool(true)));
    assert!(app.program.text.is_empty());
}

#[test]
fn f9_switches_the_theme_even_while_a_text_field_has_focus() {
    let mut app = Headless::new(Test::default());
    // It starts light, as Headless::new starts a program.
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
    app.key(Key::F9);
    assert_eq!(app.ui.theme().kind, ThemeKind::Dark);
    assert_eq!(app.ui.state().get("Theme"), Some(&Value::from("Dark")));
    // A text field takes what's typed, but not the function keys.
    app.click("Field").key(Key::F9);
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
    assert_eq!(app.ui.state().get("Theme"), Some(&Value::from("Light")));
    assert!(app.program.text.is_empty());
}

// Notes the theme that each frame's view is in, and switches it with
// toggle_theme when asked, as a menu item does.
#[derive(Default)]
struct Themes {
    frames: std::cell::RefCell<Vec<ThemeKind>>,
}

impl Program for Themes {
    type Message = ();

    fn title(&self) -> String {
        "Themes".to_string()
    }

    fn update(&mut self, _toggle: (), ui: &mut Context<()>) {
        ui.toggle_theme();
    }

    fn view(&self, ui: &mut Ui) -> Tree<()> {
        self.frames.borrow_mut().push(ui.theme().kind);
        Tree::new()
    }
}

#[test]
fn a_frame_is_drawn_whole_in_the_theme_its_messages_leave() {
    let mut app = Headless::new(Themes::default());
    // F9 switches the theme from the frame that it's pressed in.
    app.send(&[Event::KeyPressed { key: Key::F9, repeat: false }]);
    let frames = app.program.frames.borrow().clone();
    assert_eq!(frames[frames.len() - 2..], [ThemeKind::Light, ThemeKind::Dark]);
    app.send(&[Event::KeyReleased(Key::F9)]);
    // A message's toggle_theme switches it for the frame that applies the
    // message, which is drawn in the new theme throughout.
    app.send_message(());
    assert_eq!(app.program.frames.borrow().last(), Some(&ThemeKind::Light));
    // Two toggles in one frame leave the theme as it was.
    app.queue_message(()).queue_message(()).frame();
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
}

#[test]
fn a_slider_s_steps_land_on_their_decimals() {
    let mut app = Headless::new(Ported::default());
    app.frame();
    for _ in 0..107 {
        app.scroll("Fine", 1.0);
    }
    assert_eq!(app.program.fine, 0.107);
}

#[test]
fn headed_columns_sit_side_by_side_under_their_headers() {
    let mut form = Form::new(Rect::new(400.0, PERF_BAR + 10.0, 400.0, 200.0));
    let [left, right] = form.headed_columns(&mut Tree::<()>::new(), ["Left", "Right"], 32.0);
    let header = user_interface::tokens::BODY.line_height + user_interface::controls::TEXT_BOX_HEADER_GAP;
    assert_eq!(left.y, PERF_BAR + 10.0 + header);
    assert_eq!((left.height, right.y), (32.0, left.y));
    assert_eq!(right.x - left.right(), user_interface::tokens::CARD_PADDING);
    assert_eq!(right.right(), 800.0);
}

#[test]
fn a_gamepad_s_combination_has_a_keycap_for_each_part() {
    let mut app = Headless::new(Ported::default());
    let fonts = app.ui.fonts();
    let keys = user_interface::Keycaps::width(fonts, "RB+Left stick");
    let parts = user_interface::Keycaps::key_width(fonts, "RB") + user_interface::Keycaps::key_width(fonts, "Left stick");
    assert!(keys > parts && keys < parts + 16.0, "{keys} {parts}");
}

#[test]
fn a_wide_tooltip_is_wider_than_winui_s() {
    let mut app = Headless::new(Ported::default());
    app.frame();
    app.move_to("Wide").wait(TOOLTIP_DELAY * 2.0);
    let state = app.ui.state();
    let Some(Value::Array(area)) = state.get("Tooltip").and_then(|tooltip| tooltip.get("Area")).cloned() else { panic!("no tooltip") };
    let Value::Number(width) = area[2] else { panic!() };
    assert!(width > 320.0 && width <= 640.0, "{width}");
}

#[test]
fn a_right_press_that_closes_a_menu_goes_nowhere_else() {
    let mut app = Headless::new(Test::default());
    app.click("MenuButton");
    assert!(app.ui.any_popup());
    app.right_click("Area");
    assert!(!app.ui.any_popup());
    assert_eq!(app.program.secondary, 0, "the press that closed the menu");
    app.right_click("Area");
    assert_eq!(app.program.secondary, 1);
}

#[test]
fn a_list_whose_items_go_while_it_is_open_closes() {
    let mut app = Headless::new(Test {
        items: vec!["Only".to_string()],
        ..Test::default()
    });
    app.click("Shrink");
    assert!(app.ui.any_popup());
    app.program.items.clear();
    // The frame that shows no items, and the one whose input finds the
    // list empty and closes it.
    app.frame().frame();
    assert!(!app.ui.any_popup());
    assert_eq!(app.label("Shrink").as_deref(), Some(""));
}

#[test]
fn a_caret_in_text_the_program_replaced_lands_on_a_character() {
    let mut app = Headless::new(Test::default());
    app.click("Field").type_text("abc").key(Key::LEFT).key(Key::LEFT);
    // The caret was inside the two-byte character's bytes.
    app.program.replace = true;
    app.frame().type_text("x");
    assert_eq!(app.program.text, "x\u{e9}");
}

#[test]
fn the_wheel_over_a_menu_leaves_the_list_under_it_alone() {
    let mut app = Headless::new(Test::default());
    let at = Point::new(999.0, PERF_BAR + 450.0);
    app.press_button(at, MouseButton::Right).release_button(MouseButton::Right);
    assert_eq!(app.ui.popup_name(), Some("Area/ContextMenu"));
    let over = app.target("Area/ContextMenu/First").center();
    assert!(app.target("Beneath").contains(over), "the menu opens over the list");
    app.scroll_point(over, -2.0);
    assert!(app.has_target("Beneath/0"), "the list didn't scroll under the menu");
    app.click_point(Point::new(5.0, 900.0));
    app.scroll_point(over, -2.0);
    assert!(!app.has_target("Beneath/0"), "the list scrolls once the menu is gone");
}

#[test]
fn a_number_box_takes_the_wheel_only_with_focus_and_the_pointer_on_it() {
    let mut app = Headless::new(Test::default());
    app.scroll("Number", 1.0);
    assert_eq!(app.program.number, 0, "without focus");
    app.click("Number").scroll("Check", 1.0);
    assert_eq!(app.program.number, 0, "with the pointer elsewhere");
    app.scroll("Number", 1.0);
    assert_eq!(app.program.number, 1);
}

#[test]
fn a_message_s_effects_all_show_on_its_frame() {
    // Every widget shows what the input did, whatever order the tree lists
    // them in: the field's text, which the view builds from the model, is
    // the typed text on the very frame that types it.
    let mut app = Headless::new(Test::default());
    app.click("Field").type_text("abc");
    assert_eq!(app.label("Field").as_deref(), Some("abc"));
    assert_eq!(app.messages_applied(), 1);
}

#[test]
fn a_view_that_sends_nothing_applies_no_messages() {
    let mut app = Headless::new(Test::default());
    app.frame();
    assert_eq!(app.messages_applied(), 0);
    let _ = Sense::NONE;
}

// A list of the names that a program has, which may get fewer while one of
// the later ones is still what the program has selected.
struct Names {
    names: Vec<String>,
    selected: Option<usize>,
    activated: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
enum NamesMsg {
    Select(usize),
    Activate(usize),
}

impl Program for Names {
    type Message = NamesMsg;

    fn title(&self) -> String {
        "Names".to_string()
    }

    fn update(&mut self, message: NamesMsg, _ui: &mut Context<NamesMsg>) {
        match message {
            NamesMsg::Select(row) => self.selected = Some(row),
            NamesMsg::Activate(row) => self.activated.push(row),
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<NamesMsg> {
        let mut tree = Tree::new();
        let rect = Rect::new(10.0, ui.content_top() + 10.0, 200.0, 100.0);
        let names = self.names.clone();
        // Indexing, as a program's labels do: a row past the names stops
        // the program.
        let list = ListView::new(self.names.len(), 20.0)
            .selection(self.selected)
            .on_select(NamesMsg::Select)
            .on_activate(NamesMsg::Activate)
            .labels(move |index| names[index].clone());
        let shown = list.rows(ui, "Names", rect);
        tree.container(Some("Names"), rect, list, |list| {
            for index in shown.indices() {
                list.draw(shown.content(index), Text::label(&self.names[index]));
            }
        });
        tree
    }
}

// A program's selection may be past its list's rows, when the list got
// shorter and the selection wasn't moved. The list takes it as no selection,
// and asks for no label of it.
#[test]
fn a_list_takes_a_selection_past_its_rows_as_none() {
    let names = |count: usize| (0..count).map(|index| format!("Name {index}")).collect::<Vec<_>>();
    let mut app = Headless::new(Names { names: names(3), selected: None, activated: Vec::new() });
    app.click("Names/2");
    assert_eq!(app.program.selected, Some(2));
    app.program.names = names(1);
    // The frame asks the list for its label, which is its selected row's.
    app.frame();
    assert_eq!(app.program.selected, Some(2), "the program's selection is the program's to move");
    assert_eq!(app.label("Names"), None);
    // A double-click activates the row it's on.
    app.double_click("Names/0");
    assert_eq!(app.program.activated, [0]);
}