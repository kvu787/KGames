// Tests the interface's rules for input with a small program: which widget
// the pointer is on, presses and clicks, keyboard focus and the keys that
// widgets take, popups, tooltips, notices, and motion.

use user_interface::json::Value;
use user_interface::tokens::{CURVE_LINEAR, DURATION_NORMAL};
use user_interface::{Button, Headless, Icon, Key, Modifiers, Point, Program, Rect, Severity, Startup, TextBox, Ui, id};

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
}

impl Program for Test {
    fn title(&self) -> String {
        "Test".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        if ui.key_pressed(Key::A) {
            self.shortcut += 1;
        }
        if ui.button("Button", Rect::new(10.0, 10.0, 100.0, 32.0), Button::new("Button")).clicked {
            self.clicks += 1;
        }
        ui.tooltip("Button", "A tip");
        ui.text_box("Field", Rect::new(10.0, 60.0, 200.0, 32.0), &mut self.text, TextBox::new());
        ui.combo_box("Combo", Rect::new(10.0, 110.0, 200.0, 32.0), &mut self.choice, &["Zero", "One", "Two"]);
        // A surface over a button covers it from the pointer.
        if ui.button("Under", Rect::new(300.0, 10.0, 100.0, 32.0), Button::new("Under")).clicked {
            self.covered = true;
        }
        ui.cover(Rect::new(290.0, 0.0, 200.0, 60.0));
        ui.set_enabled(!self.disabled);
        if ui.button("Maybe", Rect::new(10.0, 160.0, 100.0, 32.0), Button::new("Maybe")).clicked {
            self.clicks += 100;
        }
        ui.set_enabled(true);
        let mut selected = self.rows;
        ui.list_view("Rows", Rect::new(500.0, 10.0, 200.0, 100.0), 50, 20.0, &mut selected, |ui, index, row, _| {
            ui.label(row, &format!("Row {index}"));
        });
        self.rows = selected;
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
    // A press that ends off the widget doesn't click it.
    app.press("Button").move_point(Point::new(400.0, 400.0)).release();
    assert_eq!(app.program.clicks, 1);
}

#[test]
fn a_covered_widget_is_not_under_the_pointer() {
    let mut app = Headless::new(Test::default());
    app.click_point(Point::new(350.0, 26.0));
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
    app.click("Field").key(Key::A).type_text("a");
    assert_eq!(app.program.shortcut, 1);
    assert_eq!(app.program.text, "a");
    // Esc takes focus away, and the shortcut works again.
    app.key(Key::ESCAPE).key(Key::A);
    assert_eq!(app.program.shortcut, 2);
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
    app.key(Key::TAB).key(Key::TAB).key(Key::TAB);
    assert_eq!(focused(&app), Some(Value::from("Rows")));
    app.key_with(Modifiers::SHIFT, Key::TAB);
    assert_eq!(focused(&app), Some(Value::from("Under")));
    // Space presses a focused button.
    app.key(Key::SPACE);
    assert!(app.program.covered);
}

#[test]
fn a_popup_takes_the_pointer_from_what_is_under_it() {
    let mut app = Headless::new(Test::default());
    app.click("Combo");
    assert!(app.has_target("Combo/2"));
    app.click("Combo/2");
    assert_eq!(app.program.choice, 2);
    assert!(!app.has_target("Combo/2"));
}

#[test]
fn a_tooltip_waits_for_the_pointer_to_rest() {
    let mut app = Headless::new(Test::default());
    app.move_to("Button").wait(0.3);
    assert_eq!(app.ui.state().get("Tooltip"), Some(&Value::Null));
    app.wait(0.3);
    assert_ne!(app.ui.state().get("Tooltip"), Some(&Value::Null));
}

#[test]
fn a_list_shows_only_the_rows_in_view_and_the_keys_scroll_to_the_selection() {
    let mut app = Headless::new(Test::default());
    assert!(app.has_target("Rows/4") && !app.has_target("Rows/6"));
    app.click("Rows/2");
    assert_eq!(app.program.rows, Some(2));
    app.key(Key::END);
    assert_eq!(app.program.rows, Some(49));
    assert!(app.has_target("Rows/49") && !app.has_target("Rows/2"));
}

#[test]
fn animations_move_to_their_target_over_their_duration() {
    let mut app = Headless::new(Test::default());
    assert_eq!(app.program.eased, 0.0);
    app.program.target = 1.0;
    app.frame();
    assert!(app.program.eased < 0.1);
    app.wait(DURATION_NORMAL as f64 / 2.0);
    assert!(app.program.eased > 0.3 && app.program.eased < 0.8, "{}", app.program.eased);
    app.wait(DURATION_NORMAL as f64);
    assert_eq!(app.program.eased, 1.0);
}

struct Shell;

impl Program for Shell {
    fn title(&self) -> String {
        "Shell".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        ui.app_bar(Icon::Home, "Shell");
        ui.page();
    }
}

#[test]
fn notices_from_the_start_show_and_close() {
    let folder = std::env::temp_dir();
    let startup = Startup::for_tests(r#"{ "Volume": "loud" }"#, &folder);
    let mut app = Headless::start(startup, |startup| {
        startup.settings.number("Volume", 1.0, |_| Ok(()));
        startup.notify(Severity::Informational, "Hello", "A notice");
        Ok(Shell)
    });
    assert_eq!(app.ui.notices().len(), 2);
    assert!(app.has_target("Notice/0") && app.has_target("Notice/1"));
    app.click("Notice/0/Close");
    assert_eq!(app.ui.notices().len(), 1);
    assert_eq!(app.ui.notices()[0].title, "Hello");
}
