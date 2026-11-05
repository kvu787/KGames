// Tests UiShowcase through its cards' real layout, without a window: presses
// on each kind of widget, drags, the wheel, the keyboard, the combo boxes'
// lists, typing, the list, tooltips, focus, and the shortcuts.

use ui_showcase::showcase::Showcase;
use user_interface::json::Value;
use user_interface::tokens::{FULLSCREEN_HEIGHT, MAXIMIZED_HEIGHT};
use user_interface::{Gamepad, GamepadButtons, Headless, Key, Modifiers, Point, ThemeKind, slider_rail};

fn showcase() -> Headless<Showcase> {
    Headless::new(Showcase::default())
}

#[test]
fn every_widget_fits_in_the_window_at_both_heights_and_in_both_themes() {
    for theme in [ThemeKind::Light, ThemeKind::Dark] {
        for fullscreen in [false, true] {
            let mut app = Headless::with_theme(Showcase::default(), theme);
            app.set_fullscreen(fullscreen);
            let height = if fullscreen { FULLSCREEN_HEIGHT } else { MAXIMIZED_HEIGHT };
            for target in app.ui.targets().iter().filter(|target| target.listed) {
                let rect = target.rect;
                assert!(
                    rect.x >= 0.0 && rect.y >= 0.0 && rect.right() <= 2560.0 && rect.bottom() <= height,
                    "{} is at {rect:?}, outside the window",
                    target.name
                );
            }
        }
    }
}

#[test]
fn buttons_count_their_presses() {
    let mut app = showcase();
    app.click("Standard").click("Accent").click("Accent").click("Save");
    assert_eq!(app.program.presses, [1, 2, 0, 1]);
    app.click("Add").click("Add").click("Remove");
    assert_eq!(app.program.count, 1);
    app.click("Pin");
    assert!(app.program.pinned);
    // A disabled button takes no presses.
    app.click("Disabled");
    assert!(!app.is_enabled("Disabled"));
}

#[test]
fn a_press_that_slides_off_a_button_does_not_press_it() {
    let mut app = showcase();
    app.press("Standard");
    let away = app.target("Accent").center();
    app.move_point(away).release();
    assert_eq!(app.program.presses, [0, 0, 0, 0]);
}

#[test]
fn the_first_checkbox_disables_the_options() {
    let mut app = showcase();
    assert!(app.is_enabled("Option/Show rulers"));
    app.click("OptionsEnabled");
    assert!(!app.program.options_enabled);
    assert!(!app.is_enabled("Option/Show rulers"));
    app.click("Option/Show rulers");
    assert!(!app.program.options[1]);
    app.click("OptionsEnabled").click("Option/Show rulers");
    assert!(app.program.options[1]);
}

#[test]
fn switches_turn_tooltips_and_the_dark_theme_on_and_off() {
    let mut app = showcase();
    app.click("Dark");
    assert!(app.program.dark);
    assert_eq!(app.ui.theme().kind, ThemeKind::Dark);
    app.key(Key::F2);
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
    app.click("Tooltips");
    assert!(!app.ui.tooltips_enabled());
    app.key(Key::F1);
    assert!(app.ui.tooltips_enabled());
}

#[test]
fn radio_buttons_and_segments_pick_one() {
    let mut app = showcase();
    app.click("Size/Large");
    assert_eq!(app.program.size, 2);
    app.click("Alignment/Right");
    assert_eq!(app.program.alignment, 2);
    app.click("Tool/Erase");
    assert_eq!(app.program.tool, 2);
    // The arrows move through a segmented control's segments.
    app.key(Key::LEFT);
    assert_eq!(app.program.tool, 1);
    assert_eq!(app.ui.state().get("Focused"), Some(&Value::from("Tool/Fill")));
}

#[test]
fn a_press_on_a_slider_sets_it_and_a_drag_moves_it_past_its_ends() {
    let mut app = showcase();
    let rect = app.target("Volume");
    let (left, right) = slider_rail(rect);
    app.press_point(Point::new((left + right) / 2.0, rect.center().y));
    assert_eq!(app.program.volume, 50.0);
    app.move_point(Point::new(right + 400.0, rect.center().y));
    assert_eq!(app.program.volume, 100.0);
    app.move_point(Point::new(left - 400.0, rect.center().y)).release();
    assert_eq!(app.program.volume, 0.0);
    app.press_point(Point::new(left + (right - left) * 0.25, rect.center().y)).release();
    assert_eq!(app.program.volume, 25.0);
}

#[test]
fn the_wheel_and_the_keys_move_a_slider_by_its_steps() {
    let mut app = showcase();
    app.scroll("Opacity", -3.0);
    assert!((app.program.opacity - 0.747).abs() < 1e-5);
    app.click("Angle");
    let angle = app.program.angle;
    app.key(Key::RIGHT);
    assert!((app.program.angle - angle - 0.1).abs() < 1e-3);
    app.key_with(Modifiers::SHIFT, Key::RIGHT);
    assert!((app.program.angle - angle - 10.1).abs() < 1e-3);
    // The range stops at its last step.
    app.key(Key::END);
    assert!((app.program.angle - 359.9).abs() < 1e-3);
    app.key(Key::HOME);
    assert_eq!(app.program.angle, 0.0);
}

#[test]
fn a_combo_box_opens_its_list_over_itself_and_a_press_picks_from_it() {
    let mut app = showcase();
    app.click("Density");
    assert_eq!(app.ui.state().get("Popup"), Some(&Value::from("Density")));
    // The chosen item is where the box is.
    let box_center = app.target("Density").center().y;
    let chosen = app.target("Density/1");
    assert!((chosen.center().y - box_center).abs() < 4.0, "{chosen:?} isn't over the box");
    app.click("Density/2");
    assert_eq!(app.program.density, 2);
    assert_eq!(app.ui.state().get("Popup"), Some(&Value::Null));
    assert_eq!(app.label("Density").as_deref(), Some("Comfortable"));
}

#[test]
fn a_press_outside_a_combo_box_list_closes_it_and_does_nothing_else() {
    let mut app = showcase();
    app.click("Role");
    assert!(app.has_target("Role/3"));
    let far = app.target("Standard").center();
    app.click_point(far);
    assert_eq!(app.ui.state().get("Popup"), Some(&Value::Null));
    assert_eq!(app.program.presses, [0, 0, 0, 0]);
    // Esc closes it too, and the keys pick from it.
    app.click("Role").key(Key::DOWN).key(Key::DOWN).key(Key::ENTER);
    assert_eq!(app.program.role, 2);
    app.click("Role").key(Key::ESCAPE);
    assert_eq!(app.ui.state().get("Popup"), Some(&Value::Null));
}

#[test]
fn a_text_field_takes_typing_and_enter_greets() {
    let mut app = showcase();
    app.click("Name").type_text("Ada").key(Key::ENTER);
    assert_eq!(app.program.name, "Ada");
    assert_eq!(app.program.greeting.as_deref(), Some("Hello, Ada!"));
    // Keys that the field takes don't reach the program's shortcuts.
    app.type_text(" B").key(Key::BACKSPACE).key(Key::BACKSPACE);
    assert_eq!(app.program.name, "Ada");
    app.key_with(Modifiers::CONTROL, Key::A).key_with(Modifiers::CONTROL, Key::C).key(Key::END).key_with(Modifiers::CONTROL, Key::V);
    assert_eq!(app.program.name, "AdaAda");
    app.key_with(Modifiers::CONTROL, Key::BACKSPACE);
    assert_eq!(app.program.name, "");
}

#[test]
fn a_double_click_selects_a_word() {
    let mut app = showcase();
    app.click("Name").type_text("Grace Hopper");
    let field = app.target("Name");
    app.double_click("Name");
    // The pointer is in the middle of the field, past the text, so the
    // word before it is selected.
    app.type_text("Lovelace");
    assert_eq!(app.program.name, "Grace Lovelace");
    let _ = field;
}

#[test]
fn the_search_shows_the_items_it_matches_and_its_button_clears_it() {
    let mut app = showcase();
    app.click("Search").type_text("cobalt");
    assert_eq!(app.program.matches().len(), 10);
    assert_eq!(app.label("Items/0").as_deref(), Some("Cobalt 1"));
    app.click("Search/Clear");
    assert_eq!(app.program.search, "");
    assert_eq!(app.program.matches().len(), 120);
}

#[test]
fn the_list_selects_scrolls_and_opens() {
    let mut app = showcase();
    app.click("Items/3");
    assert_eq!(app.program.selected, Some(3));
    app.key(Key::DOWN).key(Key::DOWN);
    assert_eq!(app.program.selected, Some(5));
    app.double_click("Items/4");
    assert_eq!(app.program.opened, Some(4));
    assert!(!app.has_target("Items/60"));
    app.scroll("Items", -20.0);
    assert!(app.has_target("Items/60"));
    assert!(!app.has_target("Items/0"));
}

#[test]
fn the_caution_closes_and_comes_back() {
    let mut app = showcase();
    app.click("Caution/Close");
    assert!(!app.program.caution_shown);
    app.click("ShowCaution");
    assert!(app.program.caution_shown);
}

#[test]
fn a_tooltip_shows_once_the_pointer_rests() {
    let mut app = showcase();
    app.move_to("TipSentence");
    assert_eq!(app.ui.state().get("Tooltip"), Some(&Value::Null));
    app.wait(0.6);
    let tooltip = app.ui.state().get("Tooltip").cloned().unwrap();
    assert_eq!(tooltip.get("Name"), Some(&Value::from("TipSentence")));
    // A press puts it away.
    app.press("TipSentence").release();
    assert_eq!(app.ui.state().get("Tooltip"), Some(&Value::Null));
}

#[test]
fn a_tooltip_without_room_under_the_pointer_goes_above_it() {
    let mut app = showcase();
    app.move_to("TipAbove").wait(0.6);
    let state = app.ui.state();
    let area = state.get("Tooltip").and_then(|tooltip| tooltip.get("Area")).cloned().unwrap();
    let Value::Array(area) = area else { panic!() };
    let (Value::Number(y), Value::Number(height)) = (&area[1], &area[3]) else { panic!() };
    assert!(y + height <= app.pointer().y as f64, "the tooltip isn't above the pointer");
}

#[test]
fn tab_moves_focus_through_the_widgets_and_esc_takes_it_away() {
    let mut app = showcase();
    app.key(Key::TAB);
    assert_eq!(app.ui.state().get("Focused"), Some(&Value::from("Theme")));
    app.key(Key::TAB);
    assert_eq!(app.ui.state().get("Focused"), Some(&Value::from("Standard")));
    app.key(Key::SPACE);
    assert_eq!(app.program.presses[0], 1);
    app.key_with(Modifiers::SHIFT, Key::TAB);
    assert_eq!(app.ui.state().get("Focused"), Some(&Value::from("Theme")));
    app.key(Key::ESCAPE);
    assert_eq!(app.ui.state().get("Focused"), Some(&Value::Null));
}

#[test]
fn ctrl_r_resets_every_widget() {
    let mut app = showcase();
    app.click("Standard").click("Size/Small").click("Dark");
    app.key_with(Modifiers::CONTROL, Key::R);
    assert_eq!(app.program.presses, [0, 0, 0, 0]);
    assert_eq!(app.program.size, 1);
    assert!(!app.program.dark);
}

#[test]
fn f11_switches_to_fullscreen_and_back() {
    let mut app = showcase();
    app.key(Key::F11);
    assert!(app.fullscreen());
    app.key(Key::F11);
    assert!(!app.fullscreen());
}

#[test]
fn without_focus_nothing_responds() {
    let mut app = showcase();
    app.focus(false);
    app.click("Standard").key(Key::F2);
    assert_eq!(app.program.presses, [0, 0, 0, 0]);
    assert!(!app.program.dark);
    // Keys held when focus comes count once they're let go of.
    app.focus(true).click("Standard");
    assert_eq!(app.program.presses[0], 1);
}

#[test]
fn the_interface_card_shows_held_keys_and_the_gamepad() {
    let mut app = showcase();
    app.hold(Key::W);
    assert!(app.ui.input().key_down(Key::W));
    app.release_key(Key::W);
    let gamepad = Gamepad {
        connected: true,
        left_stick: Point::new(0.5, -0.5),
        buttons: GamepadButtons(GamepadButtons::A),
        ..Gamepad::default()
    };
    app.set_gamepad(gamepad);
    assert_eq!(app.ui.gamepad().buttons.names(), vec!["A"]);
}

#[test]
#[ignore]
fn benchmark_frame_time() {
    let mut app = Headless::new(Showcase::default());
    app.frames(120);
    let start = std::time::Instant::now();
    app.frames(1000);
    println!("FRAME {:.1} us", start.elapsed().as_secs_f64() / 1000.0 * 1e6);
}
