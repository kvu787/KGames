// Tests the showcase without a window: what each widget does with input, in
// the cards' real layout, and in both themes.

use ui_showcase::showcase::Showcase;
use user_interface::json::Value;
use user_interface::controls::{INFO_BAR_MIN_HEIGHT, MENU_BAR_HEIGHT, MENU_BAR_ITEM_MARGIN, MENU_BAR_ITEM_PADDING};
use user_interface::graphics::{AdapterChoice, Gpu};
use user_interface::tokens::{APP_BAR_HEIGHT, PAGE_PADDING, PERF_BAR_HEIGHT, TOOLTIP_DELAY, WINDOW_HEIGHT};
use user_interface::{Cursor, Gamepad, GamepadButtons, Headless, Key, Modifiers, MouseButton, Point, Rect, Startup, Theme, ThemeKind, slider_rail};

fn showcase() -> Headless<Showcase> {
    Headless::new(Showcase::default())
}

fn interface(app: &Headless<Showcase>, member: &str) -> Option<Value> {
    app.ui.state().get(member).cloned()
}

#[test]
fn every_widget_fits_in_the_window_in_both_themes() {
    for theme in [ThemeKind::Light, ThemeKind::Dark] {
        let mut app = Headless::with_theme(Showcase::default(), theme);
        app.frame();
        for target in app.ui.targets().iter().filter(|target| target.listed) {
            let rect = target.rect;
            assert!(rect.x >= 0.0 && rect.y >= 0.0 && rect.right() <= 2560.0 && rect.bottom() <= WINDOW_HEIGHT, "{} is at {rect:?}", target.name);
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
fn a_press_presses_a_button_as_it_goes_down_even_if_it_slides_off() {
    let mut app = showcase();
    app.press("Standard");
    assert_eq!(app.program.presses[0], 1);
    app.move_point(Point::new(1200.0, 900.0)).release();
    assert_eq!(app.program.presses[0], 1);
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
    // The press's own frame switches the theme, and logs it with the press,
    // newest first.
    app.press("DarkTheme");
    assert_eq!(app.ui.theme().kind, ThemeKind::Dark);
    let newest: Vec<&str> = app.program.log.iter().take(2).map(String::as_str).collect();
    assert_eq!(newest, ["Press on DarkTheme", "Dark theme"]);
    app.release();
    app.click("Tooltips");
    assert!(!app.ui.tooltips_enabled());
    // F9 is the library's, which the theme's switch follows.
    app.key(Key::F9).key(Key::F2);
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
    assert!(app.ui.tooltips_enabled());
    assert!(app.program.log.iter().any(|line| line == "Light theme"));
    app.click("Theme");
    assert_eq!(app.ui.theme().kind, ThemeKind::Dark);
}

// Starting in the dark theme, as "Theme": "Dark" in Settings.json makes it,
// is no change of theme, which the log would tell of, and the switch and F9
// still go from there.
#[test]
fn starting_in_the_dark_theme_logs_no_change_of_theme() {
    let folder = std::env::temp_dir();
    let mut app = Headless::start(Startup::for_tests(r#"{ "Theme": "Dark" }"#, &folder), |startup| Ok(Showcase::new(startup)));
    app.frames(3);
    assert_eq!(app.ui.theme().kind, ThemeKind::Dark);
    let themed = |app: &Headless<Showcase>| app.program.log.iter().filter(|line| line.ends_with("theme")).cloned().collect::<Vec<_>>();
    assert_eq!(themed(&app), Vec::<String>::new());
    assert_eq!(app.label("DarkTheme").as_deref(), Some("On"));
    app.key(Key::F9);
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
    assert_eq!(themed(&app), ["Light theme"]);
    // And starting light is no change either.
    let mut app = Headless::start(Startup::for_tests(r#"{ "Theme": "Light" }"#, &folder), |startup| Ok(Showcase::new(startup)));
    app.frames(3);
    assert_eq!(themed(&app), Vec::<String>::new());
}

// F9 and the program's own theme command in one frame are two toggles, which
// apply in order and cancel: F9 switches to dark, which the program sees, and
// the button's press after it switches back, so the frame ends, and is
// drawn, in the theme it started in.
#[test]
fn two_theme_toggles_in_one_frame_cancel() {
    use user_interface::Event;
    let mut app = showcase();
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
    // The button fires on its press, so its press frame also presses F9.
    let themed = app.program.log.iter().filter(|line| line.ends_with("theme")).count();
    app.queue(&[Event::KeyPressed(Key::F9)]);
    app.press("Theme");
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
    // The watcher logged the dark theme that F9 left, and the button's toggle
    // the light one that it left, newest first; the watcher re-reads the
    // theme when it applies, so it adds nothing more, this frame or the next.
    app.release();
    let after: Vec<&String> = app.program.log.iter().filter(|line| line.ends_with("theme")).collect();
    assert_eq!(after.len(), themed + 2, "{after:?}");
    assert_eq!(after[..2], ["Light theme", "Dark theme"]);
    // Either alone still switches.
    app.queue(&[Event::KeyReleased(Key::F9)]).frame();
    app.key(Key::F9);
    assert_eq!(app.ui.theme().kind, ThemeKind::Dark);
    app.click("Theme");
    assert_eq!(app.ui.theme().kind, ThemeKind::Light);
}

// A frame can hold both a typed letter, which re-filters the list, and a
// press on a row after it. The letter applies first, as it was typed first,
// so the press lands on the row as the filtered list shows it: the first
// row shows Basalt, the first item with a "b", where it showed Aurora.
#[test]
fn a_press_after_a_letter_in_one_frame_lands_on_the_list_the_letter_filtered() {
    use user_interface::{Event, Key};
    let mut app = showcase();
    app.click("Search");
    let center = app.target("Items/0").center();
    app.queue(&[Event::KeyPressed(Key::B), Event::KeyReleased(Key::B)]).queue_move(center).queue(&[Event::ButtonPressed(MouseButton::Left)]).frame();
    assert_eq!(app.program.search, "b", "the letter reached the search box");
    assert_eq!(app.program.matches()[0], 1);
    assert_eq!(app.program.selected, Some(1), "the press selected what the row showed once the letter filtered the list");
    app.release();
}

#[test]
fn radio_buttons_and_segments_pick_one() {
    let mut app = showcase();
    app.click("Size/Large").click("Alignment/Right").click("Tool/Pick");
    assert_eq!((app.program.size, app.program.alignment, app.program.tool), (2, 2, 3));
    // They take no keyboard focus.
    app.click("Tool/Erase");
    assert_eq!((app.program.tool, interface(&app, "Focused")), (2, Some(Value::Null)));
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
fn the_wheel_moves_a_slider_by_its_steps() {
    let mut app = showcase();
    app.scroll("Opacity", -3.0);
    assert!((app.program.opacity - 0.747).abs() < 1e-9, "{}", app.program.opacity);
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
fn a_text_field_takes_typing_and_backspace() {
    let mut app = showcase();
    app.click("Name").type_text("Ada Lovelace");
    for _ in 0.."Lovelace".len() {
        app.key(Key::BACKSPACE);
    }
    app.type_text("Byron").click("Greet");
    assert_eq!(app.program.name, "Ada Byron");
    assert_eq!(app.program.greeting.as_deref(), Some("Hello, Ada Byron!"));
    app.click("Greeting/Close");
    assert_eq!(app.program.greeting, None);
}

// A greeting of a long name wraps, and its info bar is as tall as it wraps
// to, on the frame of the press on Greet.
#[test]
fn a_greeting_of_a_long_name_is_as_tall_as_it_wraps_to() {
    let mut app = showcase();
    app.click("Name").type_text(&"W".repeat(40)).press("Greet");
    let greeting = app.target("Greeting");
    assert!(greeting.height > INFO_BAR_MIN_HEIGHT, "{greeting:?}");
    app.release();
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
    app.click("Scale").click("Scale/Clear").type_text("9").key(Key::ENTER);
    assert_eq!(app.program.scale, 4.0);
    app.click("Scale/Down");
    assert_eq!(app.program.scale, 3.75);
}

#[test]
fn the_list_selects_scrolls_and_opens() {
    let mut app = showcase();
    app.click("Items/2");
    assert_eq!(app.program.selected, Some(2));
    app.scroll("Items", -10.0);
    assert!(!app.has_target("Items/0"));
    let row = app.ui.targets().iter().filter(|target| target.name.starts_with("Items/") && target.listed).nth(2).unwrap().name.clone();
    app.double_click(&row);
    let index: usize = row.trim_start_matches("Items/").parse().unwrap();
    assert_eq!(app.program.opened, Some(index));
}

// A search that leaves the list too few rows to scroll scrolls it to its
// top, though the pointer was never on the list, and the list stays there as
// the search is cleared.
#[test]
fn a_list_that_a_search_shortens_stays_at_its_top_as_the_search_is_cleared() {
    let mut app = showcase();
    app.scroll("Items", -10.0);
    assert!(!app.has_target("Items/0"));
    app.click("Search").type_text("ember 1");
    assert_eq!(app.program.matches().len(), 2);
    app.click("Search/Clear");
    assert_eq!(app.program.matches().len(), 120);
    assert!(app.has_target("Items/0"));
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
    app.click("Menu/Help").click("Menu/Help/Menu/Ask a question").click("Question/Close");
    assert_eq!(app.program.dialog_result.as_deref(), Some("Cancel"));
}

// How many pixels of the Interface card's rows, from the menu row down to
// its Details button, differ between the frame that just ran and the next,
// which has no input: none, if the rows show the frame that they're drawn
// in.
fn interface_rows_changed_a_frame_later(app: &mut Headless<Showcase>, gpu: &mut Gpu) -> usize {
    let details = app.target("Details");
    let top = PERF_BAR_HEIGHT + APP_BAR_HEIGHT + MENU_BAR_HEIGHT;
    let rows = Rect::new(details.x, top, 2560.0 - PAGE_PADDING - details.x, details.y - top);
    let drawn = app.render(gpu).unwrap();
    app.frame();
    let after = app.render(gpu).unwrap();
    let pixels = (rows.y as u32..rows.bottom() as u32).flat_map(|y| (rows.x as u32..rows.right() as u32).map(move |x| (x, y)));
    pixels.filter(|&(x, y)| drawn.pixel(x, y) != after.pixel(x, y)).count()
}

// The Interface card says what the frame that it's drawn in has: the frame
// whose press opens the dialog draws it as the frame after it does, with the
// dialog open, and nothing under the pointer, which the dialog's smoke
// covers.
#[test]
fn the_interface_card_shows_the_frame_that_it_is_drawn_in() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = showcase();
    app.press("Ask");
    assert!(app.ui.modal());
    assert_eq!(interface_rows_changed_a_frame_later(&mut app, &mut gpu), 0, "the card's rows were a frame late");
    app.release();
}

// The Interface card's Tooltip row says the tooltip of the frame that it's
// drawn in, as the frame after it does: the frame that shows it, and the one
// whose press puts it away.
#[test]
fn the_interface_card_shows_the_tooltip_of_the_frame_that_it_is_drawn_in() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = showcase();
    app.move_to("Save");
    let mut frames = 0;
    while app.ui.tooltip_name().is_none() {
        assert!(frames < 120, "no tooltip showed");
        app.frame();
        frames += 1;
    }
    assert_eq!(app.ui.tooltip_name(), Some("Save"));
    assert_eq!(interface_rows_changed_a_frame_later(&mut app, &mut gpu), 0, "the Tooltip row was a frame late as the tooltip showed");
    app.press("Save");
    assert_eq!(app.ui.tooltip_name(), None);
    assert_eq!(interface_rows_changed_a_frame_later(&mut app, &mut gpu), 0, "the Tooltip row was a frame late as a press put the tooltip away");
    app.release();
}

// The Cursors card is over the tabs, in their column, with every cursor, and
// the pointer takes the look of the one that it's on, in the frame that
// moves it there, and is the arrow again in the frame that moves it off.
#[test]
fn the_cursors_card_has_every_cursor_over_the_tabs_and_the_pointer_takes_the_look_of_the_one_it_is_on() {
    let mut app = showcase();
    app.frame();
    let tabs = app.target("Tabs");
    for cursor in Cursor::ALL {
        let name = format!("Cursor/{}", cursor.name());
        let rect = app.target(&name);
        assert!(rect.bottom() <= tabs.y && rect.x >= tabs.x && rect.right() <= tabs.right(), "{name} is at {rect:?}, and the tabs at {tabs:?}");
        app.move_to(&name);
        assert_eq!(app.ui.cursor(), cursor);
        assert_eq!(app.ui.hovered_name(), Some(name.as_str()));
        app.move_point(Point::new(rect.right() + 4.0, rect.center().y));
        assert_eq!(app.ui.cursor(), Cursor::Arrow, "off {name}");
    }
}

// Each cursor of the Cursors card is drawn in the accent, inside its own
// area, in both themes, without the pointer.
#[test]
fn the_cursors_card_draws_every_cursor_in_its_area() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    for kind in [ThemeKind::Light, ThemeKind::Dark] {
        let mut app = Headless::with_theme(Showcase::default(), kind);
        app.frame();
        let image = app.render(&mut gpu).unwrap();
        let accent = Theme::new(kind).accent.to_bytes();
        let is_accent = |x: u32, y: u32| image.pixel(x, y).iter().zip(accent).take(3).all(|(a, b)| (*a as i32 - b as i32).abs() <= 2);
        for cursor in Cursor::ALL {
            let rect = app.target(&format!("Cursor/{}", cursor.name()));
            let (left, top, right, bottom) = (rect.x as u32, rect.y as u32, rect.right() as u32, rect.bottom() as u32);
            let inside = (top..bottom).flat_map(|y| (left..right).map(move |x| (x, y))).filter(|&(x, y)| is_accent(x, y)).count();
            assert!(inside > 0, "{} isn't drawn, in the {} theme", cursor.name(), kind.name());
            // All of it is inside its area: none of it is on the area's
            // border.
            let border = (left..right).flat_map(|x| [(x, top), (x, bottom - 1)]).chain((top..bottom).flat_map(|y| [(left, y), (right - 1, y)]));
            let card = image.pixel(left, top);
            assert!(border.clone().all(|(x, y)| image.pixel(x, y) == card), "{} reaches its area's border, in the {} theme", cursor.name(), kind.name());
        }
    }
}

#[test]
fn the_color_picker_and_field_and_palette_agree() {
    let mut app = showcase();
    app.click("Palette/3");
    assert_eq!(app.program.color.to_hex(), "#107C10");
    assert_eq!(app.label("Hex").as_deref(), Some("#107C10"));
    app.click("Hex").click("Hex/Clear").type_text("#FF0000").key(Key::ENTER);
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

// GameInput doesn't repeat a held key, and neither does KGames: holding
// Backspace deletes one character.
#[test]
fn a_held_key_does_not_repeat() {
    let mut app = showcase();
    app.click("Name").type_text("abc");
    app.hold(Key::BACKSPACE).wait(1.0).release_key(Key::BACKSPACE);
    assert_eq!(app.program.name, "ab");
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
