// Tests the interface's rules for input with a small program: which widget
// the pointer is on, presses and clicks, keyboard focus and the keys that
// widgets take, popups, menus, dialogs, tooltips, notices, editing, and
// motion.

use user_interface::json::Value;
use user_interface::tokens::{CURVE_LINEAR, DURATION_NORMAL, TOOLTIP_DELAY};
use user_interface::{
    Button, Card, CheckBox, ComboBox, Dialog, DialogResult, Headless, Key, ListView, MenuItem, Modifiers, MouseButton, NumberBox, Point,
    Program, Rect, Sense, Severity, Slider, Startup, TextBox, Ui, id,
};

const MENU: &[MenuItem] = &[MenuItem::new("Open").accelerator("Ctrl+O"), MenuItem::separator(), MenuItem::submenu("Recent", RECENT)];
const RECENT: &[MenuItem] = &[MenuItem::new("First"), MenuItem::new("Second")];

#[derive(Default)]
struct Test {
    clicks: u32,
    shortcut: u32,
    text: String,
    choice: usize,
    covered: bool,
    disabled: bool,
    rows: Option<usize>,
    eased: f32,
    target: f32,
    checked: bool,
    level: f64,
    number: i32,
    chosen: Vec<String>,
    dialog: bool,
    answer: Option<DialogResult>,
    card: Rect,
}

impl Program for Test {
    fn title(&self) -> String {
        "Test".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        if ui.key_pressed(Key::A) {
            self.shortcut += 1;
        }
        let response = ui.add("Button", Rect::new(10.0, 10.0, 100.0, 32.0), Button::new("Button"));
        if response.clicked {
            self.clicks += 1;
        }
        ui.tooltip(&response, "A tip");
        ui.add("Field", Rect::new(10.0, 60.0, 200.0, 32.0), TextBox::new(&mut self.text));
        ui.add("Combo", Rect::new(10.0, 110.0, 200.0, 32.0), ComboBox::new(&mut self.choice, &["Zero", "One", "Two"]));
        // A surface over a button covers it from the pointer.
        if ui.add("Under", Rect::new(300.0, 10.0, 100.0, 32.0), Button::new("Under")).clicked {
            self.covered = true;
        }
        ui.cover(Rect::new(290.0, 0.0, 200.0, 60.0));
        ui.set_enabled(!self.disabled);
        if ui.add("Maybe", Rect::new(10.0, 160.0, 100.0, 32.0), Button::new("Maybe")).clicked {
            self.clicks += 100;
        }
        ui.set_enabled(true);
        ui.add("Rows", Rect::new(500.0, 10.0, 200.0, 100.0), ListView::new(50, 20.0, |ui, index, row, _| {
            ui.label(row, &format!("Row {index}"));
        }).selection(&mut self.rows));
        ui.add("Check", Rect::new(10.0, 210.0, 200.0, 32.0), CheckBox::new(&mut self.checked, "Check"));
        ui.add("Level", Rect::new(10.0, 260.0, 220.0, 32.0), Slider::new(&mut self.level, 0.0, 1.0).step(0.1).allowed(0.2, 0.8));
        ui.add("Number", Rect::new(10.0, 310.0, 200.0, 32.0), NumberBox::new(&mut self.number).range(0.0, 10.0));
        let menu_button = ui.add("MenuButton", Rect::new(10.0, 360.0, 100.0, 32.0), Button::new("Menu").chevron());
        if let Some(label) = ui.flyout_menu("Menu", &menu_button, MENU) {
            self.chosen.push(label.to_string());
        }
        let area = ui.interact("Area", Rect::new(800.0, 400.0, 200.0, 200.0), Sense::VIEW, "Area");
        if let Some(label) = ui.context_menu("Context", &area, RECENT) {
            self.chosen.push(label.to_string());
        }
        let open = ui.begin_card(Rect::new(1200.0, 10.0, 300.0, 1000.0), Card::new("Sized").named("Sized"));
        let mut content = open.area.content;
        ui.label(content.cut_top(20.0, 8.0), "One");
        ui.label(content.cut_top(20.0, 0.0), "Two");
        self.card = ui.end_card(open, content.y);
        if ui.add("Ask", Rect::new(10.0, 410.0, 100.0, 32.0), Button::new("Ask")).clicked {
            self.dialog = true;
        }
        if self.dialog && let Some(answer) = ui.dialog("Question", Dialog::new("Sure?", "Really.").primary("Yes")) {
            self.answer = Some(answer);
            self.dialog = false;
        }
        self.eased = ui.animate(id("Eased"), 0, self.target, DURATION_NORMAL, CURVE_LINEAR);
    }
}

fn focused(app: &Headless<Test>) -> Option<Value> {
    app.ui.state().get("Focused").cloned()
}

#[test]
fn a_click_is_a_press_and_a_release_on_the_widget() {
    let mut app = Headless::new(Test::default());
    app.click("Button");
    assert_eq!(app.program.clicks, 1);
    // A press that's let go of somewhere else isn't a click.
    let away = Point::new(5.0, 900.0);
    app.press("Button").move_point(away).release();
    assert_eq!(app.program.clicks, 1);
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
    // Esc takes focus away, and the keys are the program's again.
    app.key(Key::ESCAPE).key(Key::A);
    assert_eq!(app.program.shortcut, 2);
    assert_eq!(focused(&app), Some(Value::Null));
}

#[test]
fn tab_goes_through_the_widgets_in_order_and_skips_disabled_ones() {
    let mut app = Headless::new(Test {
        disabled: true,
        ..Test::default()
    });
    app.key(Key::TAB);
    assert_eq!(focused(&app), Some(Value::from("Button")));
    app.key(Key::TAB);
    assert_eq!(focused(&app), Some(Value::from("Field")));
    // Under is covered from the pointer, but not from the keyboard, and
    // Maybe is disabled.
    app.key(Key::TAB).key(Key::TAB).key(Key::TAB);
    assert_eq!(focused(&app), Some(Value::from("Rows")));
    app.key_with(Modifiers::SHIFT, Key::TAB);
    assert_eq!(focused(&app), Some(Value::from("Under")));
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
    // Esc closes it without choosing.
    app.click("Combo").key(Key::UP).key(Key::ESCAPE);
    assert_eq!((app.program.choice, app.ui.popup_name()), (2, None));
}

#[test]
fn a_tooltip_waits_for_the_pointer_to_rest_and_sits_above_it() {
    let mut app = Headless::new(Test::default());
    app.move_point(Point::new(60.0, 30.0)).wait(TOOLTIP_DELAY * 0.6);
    assert_eq!(app.ui.tooltip_name(), None);
    app.wait(TOOLTIP_DELAY);
    assert_eq!(app.ui.tooltip_name(), Some("Button"));
    // There's no room above a button at the window's top, so it's below.
    let state = app.ui.state();
    let area = state.get("Tooltip").and_then(|tooltip| tooltip.get("Area")).cloned().unwrap();
    let Value::Array(area) = area else { panic!() };
    let Value::Number(y) = area[1] else { panic!() };
    assert!(y >= 30.0 + 20.0, "{y}");
    app.press_point(Point::new(60.0, 30.0));
    assert_eq!(app.ui.tooltip_name(), None);
}

#[test]
fn a_list_shows_only_the_rows_in_view_and_the_keys_scroll_to_the_selection() {
    let mut app = Headless::new(Test::default());
    assert!(app.has_target("Rows/4") && !app.has_target("Rows/6"));
    app.click("Rows/1");
    assert_eq!(app.program.rows, Some(1));
    app.key(Key::END);
    assert_eq!(app.program.rows, Some(49));
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
    app.key(Key::RIGHT);
    assert!((app.program.level - 0.3).abs() < 1e-9);
    app.scroll("Level", 0.5).scroll("Level", 0.5);
    assert!((app.program.level - 0.4).abs() < 1e-9, "half notches add up: {}", app.program.level);
    assert_eq!(app.label("Level").as_deref(), Some("0"));
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
    assert_eq!(app.ui.popup_name(), Some("Menu"));
    app.click("Menu/Open");
    assert_eq!(app.program.chosen, ["Open"]);
    assert_eq!(app.ui.popup_name(), None);
    // A submenu opens when the pointer is on its item.
    app.click("MenuButton").move_to("Menu/Recent");
    assert!(app.has_target("Menu/Recent/Menu/Second"));
    app.click("Menu/Recent/Menu/Second");
    assert_eq!(app.program.chosen, ["Open", "Second"]);
    assert!(!app.ui.any_popup());
    // A right-click opens a context menu where it is.
    let at = Point::new(850.0, 450.0);
    app.press_button(at, MouseButton::Right).release_button(MouseButton::Right);
    assert_eq!(app.ui.popup_name(), Some("Context"));
    let menu = app.target("Context");
    assert!((menu.x - at.x).abs() < 1.0 && (menu.y - at.y).abs() < 1.0);
    app.key(Key::DOWN).key(Key::ENTER);
    assert_eq!(app.program.chosen, ["Open", "Second", "First"]);
    // A press outside a menu closes it, and does nothing else.
    app.click("MenuButton").click("Button");
    assert_eq!((app.program.clicks, app.ui.any_popup()), (0, false));
}

#[test]
fn a_dialog_takes_the_input_until_it_is_answered() {
    let mut app = Headless::new(Test::default());
    app.click("Ask");
    assert_eq!(app.ui.state().get("Dialog"), Some(&Value::Bool(true)));
    app.click_point(Point::new(60.0, 26.0));
    assert_eq!(app.program.clicks, 0, "the page is under the smoke");
    app.key(Key::A);
    assert_eq!(app.program.shortcut, 0, "the program's keys wait too");
    app.key(Key::ESCAPE);
    assert_eq!(app.program.answer, Some(DialogResult::Close));
    app.click("Ask").click("Question/Primary");
    assert_eq!(app.program.answer, Some(DialogResult::Primary));
    app.click("Button");
    assert_eq!(app.program.clicks, 1);
}

#[test]
fn a_card_sizes_itself_to_its_content() {
    let app = Headless::new(Test::default());
    // Padding, the header and its gap, two rows, and padding.
    assert_eq!(app.program.card.height, 16.0 + 32.0 + 8.0 + 20.0 + 8.0 + 20.0 + 16.0);
    let state = app.ui.state();
    let kind = state.get("Targets").and_then(|targets| targets.get("Sized")).and_then(|card| card.get("Kind")).cloned();
    assert_eq!(kind, Some(Value::from("Card")));
}

#[test]
fn animations_move_to_their_target_over_their_duration() {
    let mut app = Headless::new(Test::default());
    app.program.target = 1.0;
    app.frame();
    assert!(app.program.eased < 0.1);
    app.wait(DURATION_NORMAL as f64 / 2.0);
    assert!(app.program.eased > 0.3 && app.program.eased < 0.8, "{}", app.program.eased);
    app.wait(DURATION_NORMAL as f64);
    assert_eq!(app.program.eased, 1.0);
}

struct Notices;

impl Program for Notices {
    fn title(&self) -> String {
        "Notices".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        ui.app_bar(user_interface::AppBar::new(user_interface::Icon::Info, "Notices"));
        ui.page();
    }
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

