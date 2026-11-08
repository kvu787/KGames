// Tests the showcase without a window: what each widget does with input, in
// the cards' real layout, at both window heights and in both themes.

use ui_showcase::showcase::Showcase;
use user_interface::json::Value;
use user_interface::controls::{MENU_BAR_ITEM_MARGIN, MENU_BAR_ITEM_PADDING};
use user_interface::tokens::{APP_BAR_HEIGHT, FULLSCREEN_HEIGHT, MAXIMIZED_HEIGHT, PAGE_PADDING, PERF_BAR_HEIGHT, TOOLTIP_DELAY};
use user_interface::{Gamepad, GamepadButtons, Headless, Key, Modifiers, MouseButton, Point, ThemeKind, slider_rail};

fn showcase() -> Headless<Showcase> {
    Headless::new(Showcase::default())
}

fn interface(app: &Headless<Showcase>, member: &str) -> Option<Value> {
    app.ui.state().get(member).cloned()
}

#[test]
fn every_widget_fits_in_the_window_at_both_heights_and_in_both_themes() {
    for theme in [ThemeKind::Light, ThemeKind::Dark] {
        for fullscreen in [false, true] {
            let mut app = Headless::with_theme(Showcase::default(), theme);
            app.set_fullscreen(fullscreen).frame();
            let height = if fullscreen { FULLSCREEN_HEIGHT } else { MAXIMIZED_HEIGHT };
            for target in app.ui.targets().iter().filter(|target| target.listed) {
                let rect = target.rect;
                assert!(rect.x >= 0.0 && rect.y >= 0.0 && rect.right() <= 2560.0 && rect.bottom() <= height, "{} is at {rect:?}", target.name);
            }
        }
    }
}

#[test]
fn buttons_count_their_presses() {
    let mut app = showcase();
    app.click("Standard").click("Accent").click("Subtle").click("Save").click("Add").click("Add").click("Remove");
    assert_eq!(app.program.presses, [1, 1, 1, 1]);
    assert_eq!(app.program.count, 1);
    app.click("Pin");
    assert!(app.program.pinned);
    app.click("Link");
    assert_eq!(app.program.link_clicks, 1);
    app.click("Disabled");
    assert_eq!(app.program.presses, [1, 1, 1, 1]);
}

#[test]
fn a_press_that_slides_off_a_button_does_not_press_it() {
    let mut app = showcase();
    app.press("Standard").move_point(Point::new(1200.0, 900.0)).release();
    assert_eq!(app.program.presses[0], 0);
}

#[test]
fn the_first_checkbox_disables_the_options() {
    let mut app = showcase();
    app.click("Option/Show rulers");
    assert!(app.program.options[1]);
    app.click("OptionsEnabled").click("Option/Spell check");
    assert!(!app.program.options_enabled && !app.program.options[3]);
    assert!(!app.is_enabled("Option/Spell check"));
}

#[test]
fn switches_turn_tooltips_and_the_dark_theme_on_and_off() {
    let mut app = showcase();
    app.click("DarkTheme");
    assert_eq!(app.ui.theme().kind, ThemeKind::Dark);
    assert_eq!(app.program.log.front().map(String::as_str), Some("Dark theme"), "{:?}", app.program.log);
    app.click("Tooltips");
    assert!(!app.ui.tooltips_enabled());
    // F9 is the library's, which the theme's switch follows.
    app.key(Key::F9).key(Key::F1);
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
    assert!(app.ui.tooltips_enabled());
    assert!(app.program.log.iter().any(|line| line == "Light theme"));
    app.click("Theme");
    assert_eq!(app.ui.theme().kind, ThemeKind::Dark);
}

// F9 and the program's own theme command in one frame are two toggles, which
// cancel: the theme is the same at the frame's end, so it doesn't change.
#[test]
fn two_theme_toggles_in_one_frame_cancel() {
    use user_interface::Event;
    let mut app = showcase();
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
    // The button fires on its click, so its release frame also presses F9.
    app.press("Theme");
    let themed = app.program.log.iter().filter(|line| line.ends_with("theme")).count();
    app.queue(&[Event::KeyPressed { key: Key::F9, repeat: false }]);
    app.release();
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
    // The button's toggle logged the theme the frame draws in, once; the
    // watcher re-reads the theme when it applies, so it adds nothing, this
    // frame or the next.
    app.frame();
    let after: Vec<&String> = app.program.log.iter().filter(|line| line.ends_with("theme")).collect();
    assert_eq!(after.len(), themed + 1, "{after:?}");
    assert_eq!(after.first().map(|line| line.as_str()), Some("Light theme"));
    // Either alone still switches.
    app.queue(&[Event::KeyReleased(Key::F9)]).frame();
    app.key(Key::F9);
    assert_eq!(app.ui.theme().kind, ThemeKind::Dark);
    app.click("Theme");
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
}

// A frame can hold both a typed letter, which re-filters the list, and a
// press on a row. The press means the item the row showed, so it still
// lands there.
#[test]
fn a_press_lands_on_the_item_its_row_showed_even_as_the_search_changes() {
    use user_interface::Event;
    let mut app = showcase();
    app.click("Search");
    let center = app.target("Items/0").center();
    app.queue(&[Event::Text('b'), Event::PointerMoved(center), Event::ButtonPressed(MouseButton::Left)]).frame();
    assert_eq!(app.program.search, "b", "the letter reached the search box");
    assert_eq!(app.program.selected, Some(0), "the press selected what the row was showing");
    app.release();
}

#[test]
fn radio_buttons_and_segments_pick_one() {
    let mut app = showcase();
    app.click("Size/Large").click("Alignment/Right").click("Tool/Pick");
    assert_eq!((app.program.size, app.program.alignment, app.program.tool), (2, 2, 3));
    // A segmented control is one focus stop, whose arrows move it.
    app.click("Tool/Erase");
    assert_eq!(interface(&app, "Focused"), Some(Value::from("Tool")));
    app.key(Key::LEFT);
    assert_eq!(app.program.tool, 1);
}

#[test]
fn a_press_on_a_slider_sets_it_and_a_drag_moves_it_past_its_ends() {
    let mut app = showcase();
    let rect = app.target("Volume");
    let (left, right) = slider_rail(rect);
    app.press_point(Point::new(left + (right - left) * 0.25, rect.center().y));
    assert_eq!(app.program.volume, 25.0);
    app.move_point(Point::new(2400.0, rect.center().y)).release();
    assert_eq!(app.program.volume, 100.0);
    assert_eq!(app.label("Volume").as_deref(), Some("100"));
}

#[test]
fn the_wheel_and_the_keys_move_a_slider_by_its_steps() {
    let mut app = showcase();
    app.scroll("Opacity", -3.0);
    assert!((app.program.opacity - 0.747).abs() < 1e-9, "{}", app.program.opacity);
    app.click("Angle").key(Key::HOME).key(Key::RIGHT).key(Key::PAGE_UP);
    assert!((app.program.angle - 15.1).abs() < 1e-3, "{}", app.program.angle);
    app.key(Key::END);
    assert_eq!(app.program.angle, 360.0);
}

#[test]
fn parts_of_notches_add_up_on_a_slider() {
    let mut app = showcase();
    for _ in 0..4 {
        app.scroll("Volume", 0.25);
    }
    assert_eq!(app.program.volume, 51.0);
}

#[test]
fn a_combo_box_opens_its_list_over_itself_and_a_press_picks_from_it() {
    let mut app = showcase();
    app.click("Role");
    assert_eq!(app.ui.popup_name(), Some("Role"));
    assert!((app.target("Role/0").center().y - app.target("Role").center().y).abs() < 1.0);
    app.click("Role/3");
    assert_eq!(app.program.role, 3);
    assert_eq!(app.ui.popup_name(), None);
}

#[test]
fn a_press_outside_a_combo_box_list_closes_it_and_does_nothing_else() {
    let mut app = showcase();
    app.click("Role").click("Standard");
    assert_eq!(app.ui.popup_name(), None);
    assert_eq!(app.program.presses[0], 0);
}

#[test]
fn a_text_field_takes_typing_and_enter_greets() {
    let mut app = showcase();
    app.click("Name").type_text("Ada Lovelace").key_with(Modifiers::CONTROL, Key::LEFT).key_with(Modifiers::SHIFT, Key::END).type_text("Byron");
    app.key(Key::ENTER);
    assert_eq!(app.program.name, "Ada Byron");
    assert_eq!(app.program.greeting.as_deref(), Some("Hello, Ada Byron!"));
    app.click("Greeting/Close");
    assert_eq!(app.program.greeting, None);
}

#[test]
fn a_double_click_selects_a_word() {
    let mut app = showcase();
    app.click("Name").type_text("one two");
    app.double_click("Name").type_text("three");
    assert!(app.program.name == "one three" || app.program.name == "three two", "{}", app.program.name);
}

#[test]
fn the_search_shows_the_items_it_matches_and_its_button_clears_it() {
    let mut app = showcase();
    app.click("Search").type_text("ember");
    assert_eq!(app.program.matches().len(), 10);
    assert_eq!(app.label("Items/0").as_deref(), Some("Ember 1"));
    app.click("Search/Clear");
    assert_eq!(app.program.matches().len(), 120);
}

#[test]
fn number_boxes_take_typing_and_their_spin_buttons() {
    let mut app = showcase();
    app.click("Quantity/Up").click("Quantity/Up");
    assert_eq!(app.program.quantity, 5);
    app.click("Scale").key_with(Modifiers::CONTROL, Key::A).type_text("9").key(Key::ENTER);
    assert_eq!(app.program.scale, 4.0);
    app.key(Key::DOWN);
    assert_eq!(app.program.scale, 3.75);
}

#[test]
fn the_list_selects_scrolls_and_opens() {
    let mut app = showcase();
    app.click("Items/2").key(Key::DOWN);
    assert_eq!(app.program.selected, Some(3));
    app.scroll("Items", -10.0);
    assert!(!app.has_target("Items/0"));
    let row = app.ui.targets().iter().filter(|target| target.name.starts_with("Items/") && target.listed).nth(2).unwrap().name.clone();
    app.double_click(&row);
    let index: usize = row.trim_start_matches("Items/").parse().unwrap();
    assert_eq!(app.program.opened, Some(index));
}

#[test]
fn the_caution_closes_and_comes_back() {
    let mut app = showcase();
    app.click("Caution/Close");
    assert!(!app.program.caution_shown);
    app.click("BringBack");
    assert!(app.program.caution_shown);
}

#[test]
fn a_tooltip_shows_once_the_pointer_rests_and_sits_above_it() {
    let mut app = showcase();
    app.move_to("Save").wait(TOOLTIP_DELAY / 2.0);
    assert_eq!(app.ui.tooltip_name(), None);
    app.wait(TOOLTIP_DELAY);
    assert_eq!(app.ui.tooltip_name(), Some("Save"));
    let Some(Value::Object(tooltip)) = interface(&app, "Tooltip") else { panic!() };
    let Value::Array(area) = &tooltip.iter().find(|(name, _)| name == "Area").unwrap().1 else { panic!() };
    let (Value::Number(y), Value::Number(height)) = (&area[1], &area[3]) else { panic!() };
    assert_eq!((*y + *height) as f32, app.pointer().y - 20.0);
}

#[test]
fn the_menu_bar_lines_up_with_the_cards_under_the_title() {
    let app = showcase();
    // The File menu's name starts on the cards' left edge.
    let file = app.target("Menu/File");
    assert_eq!(file.x + MENU_BAR_ITEM_PADDING.left, PAGE_PADDING);
    assert_eq!(file.y, PERF_BAR_HEIGHT + APP_BAR_HEIGHT + MENU_BAR_ITEM_MARGIN.top);
    // The theme's button is at the other end of the menu bar's row, on the
    // cards' right edge.
    let theme = app.target("Theme");
    assert_eq!(theme.center().y, file.center().y);
    assert_eq!(theme.right(), 2560.0 - PAGE_PADDING);
}

#[test]
fn menus_choose_commands() {
    let mut app = showcase();
    app.click("Menu/View").move_to("Menu/View/Menu/Theme");
    app.click("Menu/View/Menu/Theme/Menu/Dark theme");
    assert_eq!(app.ui.theme().kind, ThemeKind::Dark);
    assert_eq!(app.program.menu_choice.as_deref(), Some("Dark theme"));
    app.click("More").click("More/Menu/Pinned");
    assert!(app.program.pinned);
    // The context menu opens at a right-click.
    let area = app.target("ContextArea").center();
    app.press_button(area, MouseButton::Right).release_button(MouseButton::Right);
    assert_eq!(app.ui.popup_name(), Some("ContextArea/ContextMenu"));
    app.click("ContextArea/ContextMenu/Copy");
    assert_eq!(app.program.menu_choice.as_deref(), Some("Copy"));
}

#[test]
fn the_dialog_asks_and_takes_the_page_until_answered() {
    let mut app = showcase();
    app.click("Ask");
    assert!(app.program.dialog_open && app.ui.modal());
    app.click("Standard");
    assert_eq!(app.program.presses[0], 0);
    app.click("Question/Secondary");
    assert_eq!(app.program.dialog_result.as_deref(), Some("Don't save"));
    assert!(!app.program.dialog_open);
    app.click("Menu/Help").click("Menu/Help/Menu/Ask a question").key(Key::ESCAPE);
    assert_eq!(app.program.dialog_result.as_deref(), Some("Cancel"));
}

#[test]
fn the_color_picker_and_field_and_palette_agree() {
    let mut app = showcase();
    app.click("Palette/3");
    assert_eq!(app.program.color.to_hex(), "#107C10");
    assert_eq!(app.label("Hex").as_deref(), Some("#107C10"));
    app.click("Hex").key_with(Modifiers::CONTROL, Key::A).type_text("#FF0000").key(Key::ENTER);
    assert_eq!(app.program.color.to_hex(), "#FF0000");
    // The spectrum's top left is red, at full saturation.
    let spectrum = app.target("Picker/Spectrum");
    app.click_point(Point::new(spectrum.x + spectrum.width / 3.0, spectrum.y + 1.0));
    let green = app.program.color;
    assert!(green.g > 0.99 && green.r < 0.02 && green.b < 0.02, "{}", green.to_hex());
}

#[test]
fn the_expander_shows_and_hides() {
    let mut app = showcase();
    assert!(app.program.expanded);
    app.click("Expander");
    assert!(!app.program.expanded);
}

#[test]
fn tab_moves_focus_through_the_widgets_and_esc_takes_it_away() {
    let mut app = showcase();
    for _ in 0..5 {
        app.key(Key::TAB);
    }
    assert_eq!(interface(&app, "Focused"), Some(Value::from("Standard")));
    app.key(Key::SPACE);
    assert_eq!(app.program.presses[0], 1);
    app.key_with(Modifiers::SHIFT, Key::TAB);
    assert_eq!(interface(&app, "Focused"), Some(Value::from("Theme")));
    app.key(Key::ESCAPE);
    assert_eq!(interface(&app, "Focused"), Some(Value::Null));
}

#[test]
fn ctrl_r_resets_every_widget() {
    let mut app = showcase();
    app.click("Standard").click("Size/Large");
    app.key_with(Modifiers::CONTROL, Key::R);
    assert_eq!(app.program.presses[0], 0);
    assert_eq!(app.program.size, 1);
    assert_eq!(app.program.log.front().map(String::as_str), Some("Reset every widget"));
}

#[test]
fn f10_and_the_view_menu_switch_vertical_sync() {
    let mut app = showcase();
    assert!(app.vertical_sync());
    app.key(Key::F10);
    assert!(!app.vertical_sync());
    assert_eq!(interface(&app, "VerticalSync"), Some(Value::Bool(false)));
    app.click("Menu/View").click("Menu/View/Menu/VSync");
    assert!(app.vertical_sync());
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
    app.focus(false).click("Standard").key(Key::F9);
    assert_eq!(app.program.presses[0], 0);
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
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
        left_stick: Point::new(0.5, 0.5),
        buttons: GamepadButtons(GamepadButtons::A),
        ..Gamepad::default()
    };
    app.set_gamepad(gamepad);
    assert_eq!(app.ui.gamepad().buttons.names(), ["A"]);
}

#[test]
fn a_held_key_repeats() {
    let mut app = showcase();
    app.click("Name").type_text("abc");
    app.hold(Key::BACKSPACE).repeat(Key::BACKSPACE).release_key(Key::BACKSPACE);
    assert_eq!(app.program.name, "a");
}

#[test]
fn benchmark_frame_time() {
    let mut app = showcase();
    app.frames(100);
    let start = std::time::Instant::now();
    let frames = 1000;
    let mut phases = user_interface::FramePhases::default();
    for _ in 0..frames {
        app.frame();
        let frame = app.ui.phases();
        phases.input += frame.input;
        phases.update += frame.update;
        phases.view += frame.view;
        phases.layout += frame.layout;
        phases.draw += frame.draw;
    }
    let per_frame = start.elapsed().as_secs_f64() * 1000.0 / frames as f64;
    let each = frames as f64;
    eprintln!("A frame of the showcase's interface takes {per_frame:.3} ms to build");
    eprintln!(
        "  input {:.3}, update {:.3}, view {:.3}, layout {:.3}, draw {:.3}",
        phases.input / each,
        phases.update / each,
        phases.view / each,
        phases.layout / each,
        phases.draw / each
    );
}
