// Tests the interface's rules for input with a small program: which widget
// the pointer is on, presses and clicks, keyboard focus and the keys that
// widgets take, popups, menus, dialogs, tooltips, notices, editing, and
// motion.

use user_interface::json::Value;
use user_interface::controls::{MENU_BAR_HEIGHT, MENU_BAR_ITEM_MARGIN, MENU_BAR_ITEM_PADDING};
use user_interface::tokens::{APP_BAR_HEIGHT, CURVE_LINEAR, DURATION_NORMAL, PAGE_PADDING, TOOLTIP_DELAY};
use user_interface::{
    Button, Card, CheckBox, ComboBox, Dialog, DialogResult, Event, Headless, Key, ListView, MenuBar, MenuItem, Modifiers, MouseButton,
    NumberBox, Point, Program, Rect, Sense, Severity, Slider, Startup, TextBox, ThemeKind, Ui, id,
};

// Where the program's part of the window starts, under the PerfBar.
const PERF_BAR: f32 = user_interface::tokens::PERF_BAR_HEIGHT;

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
    // Right presses on the button.
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
    // Presses of D, checked after the dialog is shown.
    late: u32,
}

impl Program for Test {
    fn title(&self) -> String {
        "Test".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        // Under the PerfBar.
        let top = ui.content_top();
        if ui.key_pressed(Key::A) {
            self.shortcut += 1;
        }
        let response = ui.add("Button", Rect::new(10.0, top + 10.0, 100.0, 32.0), Button::new("Button"));
        if response.clicked {
            self.clicks += 1;
        }
        if response.secondary_pressed {
            self.secondary += 1;
        }
        ui.tooltip(&response, "A tip");
        if self.replace {
            self.text = "\u{e9}".to_string();
            self.replace = false;
        }
        ui.add("Field", Rect::new(10.0, top + 60.0, 200.0, 32.0), TextBox::new(&mut self.text));
        ui.add("Combo", Rect::new(10.0, top + 110.0, 200.0, 32.0), ComboBox::new(&mut self.choice, &["Zero", "One", "Two"]));
        // A surface over a button covers it from the pointer.
        if ui.add("Under", Rect::new(300.0, top + 10.0, 100.0, 32.0), Button::new("Under")).clicked {
            self.covered = true;
        }
        ui.cover(Rect::new(290.0, top, 200.0, 60.0));
        if ui.add_enabled(!self.disabled, "Maybe", Rect::new(10.0, top + 160.0, 100.0, 32.0), Button::new("Maybe")).clicked {
            self.clicks += 100;
        }
        // A widget can't be enabled inside a disabled scope.
        ui.with_enabled(false, |ui| {
            if ui.add_enabled(true, "Trapped", Rect::new(120.0, top + 160.0, 100.0, 32.0), Button::new("Trapped")).clicked {
                self.clicks += 1000;
            }
        });
        ui.add("Rows", Rect::new(500.0, top + 10.0, 200.0, 100.0), ListView::new(50, 20.0, |ui, index, row, _| {
            ui.label(row, &format!("Row {index}"));
        }).selection(&mut self.rows));
        ui.add("Check", Rect::new(10.0, top + 210.0, 200.0, 32.0), CheckBox::new(&mut self.checked, "Check"));
        ui.add("Level", Rect::new(10.0, top + 260.0, 220.0, 32.0), Slider::new(&mut self.level, 0.0, 1.0).step(0.1).allowed(0.2, 0.8));
        ui.add("Number", Rect::new(10.0, top + 310.0, 200.0, 32.0), NumberBox::new(&mut self.number).range(0.0, 10.0));
        let menu_button = ui.add("MenuButton", Rect::new(10.0, top + 360.0, 100.0, 32.0), Button::new("Menu").chevron());
        if let Some(label) = ui.flyout_menu("Menu", &menu_button, MENU) {
            self.chosen.push(label.to_string());
        }
        let area = ui.interact("Area", Rect::new(800.0, top + 400.0, 200.0, 200.0), Sense::VIEW, "Area");
        if let Some(label) = ui.context_menu("Context", &area, RECENT) {
            self.chosen.push(label.to_string());
        }
        let open = ui.begin_card(Rect::new(1200.0, top + 10.0, 300.0, 1000.0), Card::new("Sized").named("Sized"));
        let mut content = open.area.content;
        ui.label(content.cut_top(20.0, 8.0), "One");
        ui.label(content.cut_top(20.0, 0.0), "Two");
        self.card = ui.end_card(open, content.y);
        if ui.add("Ask", Rect::new(10.0, top + 410.0, 100.0, 32.0), Button::new("Ask")).clicked {
            self.dialog = true;
        }
        if self.dialog
            && let Some(answer) = ui.dialog_with("Question", Dialog::new("Sure?", "Really.").primary("Yes"), 32.0, |ui, area| {
                ui.add("Pick", area.top_strip(32.0), ComboBox::new(&mut self.pick, &["Yes", "No"]));
            })
        {
            self.answer = Some(answer);
            self.dialog = false;
        }
        // After the dialog, which gates it from the frame it opens.
        if ui.key_pressed(Key::D) {
            self.late += 1;
        }
        self.eased = ui.animate(id("Eased"), 0, self.target, DURATION_NORMAL, CURVE_LINEAR);
        // Last, so that the tab order of the widgets above stays.
        let items: Vec<&str> = self.items.iter().map(String::as_str).collect();
        ui.add("Shrink", Rect::new(240.0, top + 110.0, 200.0, 32.0), ComboBox::new(&mut self.shrink, &items));
        ui.add("Beneath", Rect::new(1000.0, top + 400.0, 300.0, 100.0), ListView::new(50, 20.0, |ui, index, row, _| {
            ui.label(row, &format!("Row {index}"));
        }).selection(&mut self.beneath));
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
fn a_widget_cannot_be_enabled_inside_a_disabled_scope() {
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
    app.click_point(Point::new(60.0, PERF_BAR + 26.0));
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
fn a_dialog_gates_the_keys_the_frame_it_opens() {
    let mut app = Headless::new(Test::default());
    app.key(Key::D);
    assert_eq!(app.program.late, 1, "without a dialog the check after it sees the key");
    // The click that opens the dialog and a press of D land in one frame.
    // The check runs after the dialog is up, so the key is gated that very
    // frame, not just from the next one on.
    app.move_to("Ask").frame();
    app.send(&[Event::ButtonPressed(MouseButton::Left)]);
    app.send(&[
        Event::ButtonReleased(MouseButton::Left),
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

// A menu bar in the menu row under the app bar.
#[derive(Default)]
struct MenuRow {
    page: Rect,
    chosen: Option<String>,
}

const MENUS: &[(&str, &[MenuItem])] = &[("File", MENU)];

impl Program for MenuRow {
    fn title(&self) -> String {
        "MenuRow".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        ui.app_bar(user_interface::AppBar::new(user_interface::Icon::Info, "MenuRow"));
        let row = ui.menu_row();
        let bar = MenuBar::new(MENUS);
        let width = bar.width(ui);
        if let Some(label) = ui.add("Menus", row.left_strip(width), bar) {
            self.chosen = Some(label.to_string());
        }
        self.page = ui.page();
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
    assert_eq!(app.program.page.y, row_top + MENU_BAR_HEIGHT + PAGE_PADDING);
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


// What porting the programs found: segments where widgets can't take focus,
// a color field left while it's typed in, a slider's steps of a thousandth,
// controls side by side under their headers, and a gamepad's combination's
// keycaps.
#[derive(Default)]
struct Ported {
    segment: usize,
    color: user_interface::Color,
    other: String,
    fine: f64,
    columns: Vec<Rect>,
    keys: f32,
    parts: f32,
}

impl Program for Ported {
    fn title(&self) -> String {
        "Ported".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        // Under the PerfBar.
        let top = ui.content_top();
        ui.with_focusable(false, |ui| {
            ui.add("Segments", Rect::new(10.0, top + 10.0, 300.0, 32.0), user_interface::Segmented::new(&mut self.segment, &["A", "B", "C"]));
        });
        ui.add("Hex", Rect::new(10.0, top + 60.0, 200.0, 32.0), user_interface::ColorField::new(&mut self.color));
        ui.add("Other", Rect::new(10.0, top + 110.0, 200.0, 32.0), TextBox::new(&mut self.other));
        ui.add("Fine", Rect::new(10.0, top + 160.0, 300.0, Slider::<f64>::height(false)), Slider::new(&mut self.fine, 0.0, 1.0).step(0.001));
        let mut form = user_interface::Form::new(Rect::new(400.0, top + 10.0, 400.0, 200.0));
        self.columns = form.headed_columns(ui, ["Left", "Right"], 32.0).to_vec();
        self.keys = ui.keys_width("RB+Left stick");
        self.parts = ui.keycap_width("RB") + ui.keycap_width("Left stick");
        let wide = ui.add("Wide", Rect::new(10.0, top + 600.0, 100.0, 32.0), Button::new("Wide"));
        let long = "A device's path, like HID#VID_045E&PID_0B13&IG_00#8&2f6c1e3b&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}, and more that goes on after it.";
        ui.tooltip_lines_wide(&wide, &[user_interface::TooltipLine::Text(long)], 640.0);
    }
}

#[test]
fn segments_take_no_focus_where_widgets_can_t() {
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
    assert_eq!(app.program.color, user_interface::Color::default());
    app.click("Other");
    assert_eq!(app.program.color.to_hex(), "#FF8000");
}

#[test]
fn f10_switches_vertical_sync_even_while_a_text_field_has_focus() {
    let mut app = Headless::new(Test::default());
    // It starts on, as every program does.
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
    // It starts light, as every program does.
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

// Notes the theme of each frame, at its start and at its end, and switches
// it with toggle_theme when asked, mid-frame, like a menu item does.
#[derive(Default)]
struct Themes {
    frames: Vec<(ThemeKind, ThemeKind)>,
    toggle: bool,
}

impl Program for Themes {
    fn title(&self) -> String {
        "Themes".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        let start = ui.theme().kind;
        if std::mem::take(&mut self.toggle) {
            ui.toggle_theme();
        }
        self.frames.push((start, ui.theme().kind));
    }
}

#[test]
fn no_frame_mixes_the_two_themes() {
    let mut app = Headless::new(Themes::default());
    // F9 switches the theme from the frame that it's pressed in, before
    // the program draws anything.
    app.send(&[Event::KeyPressed { key: Key::F9, repeat: false }]);
    let pressed = app.program.frames.len() - 2;
    assert_eq!(app.program.frames[pressed - 1], (ThemeKind::Light, ThemeKind::Light));
    assert_eq!(app.program.frames[pressed], (ThemeKind::Dark, ThemeKind::Dark));
    app.send(&[Event::KeyReleased(Key::F9)]);
    // A program's toggle_theme switches it from the next frame on, so the
    // frame that asked stays in one theme.
    app.program.toggle = true;
    let asked = app.program.frames.len();
    app.frame().frame();
    assert_eq!(app.program.frames[asked], (ThemeKind::Dark, ThemeKind::Dark));
    assert_eq!(app.program.frames[asked + 1], (ThemeKind::Light, ThemeKind::Light));
    // Two toggles before the next frame leave the theme as it was.
    app.program.toggle = true;
    app.frame();
    app.ui.toggle_theme();
    app.frame();
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
    let mut app = Headless::new(Ported::default());
    app.frame();
    let [left, right] = [app.program.columns[0], app.program.columns[1]];
    let header = user_interface::tokens::BODY.line_height + user_interface::controls::TEXT_BOX_HEADER_GAP;
    assert_eq!(left.y, PERF_BAR + 10.0 + header);
    assert_eq!((left.height, right.y), (32.0, left.y));
    assert_eq!(right.x - left.right(), user_interface::tokens::CARD_PADDING);
    assert_eq!(right.right(), 800.0);
}

#[test]
fn a_gamepad_s_combination_has_a_keycap_for_each_part() {
    let mut app = Headless::new(Ported::default());
    app.frame();
    assert!(app.program.keys > app.program.parts && app.program.keys < app.program.parts + 16.0, "{} {}", app.program.keys, app.program.parts);
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
    app.right_click("Button");
    assert!(!app.ui.any_popup());
    assert_eq!(app.program.secondary, 0, "the press that closed the menu");
    app.right_click("Button");
    assert_eq!(app.program.secondary, 1);
}

#[test]
fn enter_goes_to_an_open_list_in_a_dialog_not_to_its_default_button() {
    let mut app = Headless::new(Test::default());
    app.click("Ask").click("Pick");
    assert!(app.ui.any_popup());
    app.key(Key::DOWN).key(Key::ENTER);
    assert_eq!((app.program.pick, app.program.answer), (1, None));
    assert!(!app.ui.any_popup());
    app.click("Question/Primary");
    assert_eq!(app.program.answer, Some(DialogResult::Primary));
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
    app.frame();
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
    assert_eq!(app.ui.popup_name(), Some("Context"));
    let over = app.target("Context/First").center();
    assert!(app.target("Beneath").contains(over), "the menu opens over the list");
    app.scroll_point(over, -2.0);
    assert!(app.has_target("Beneath/0"), "the list didn't scroll under the menu");
    app.key(Key::ESCAPE);
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

