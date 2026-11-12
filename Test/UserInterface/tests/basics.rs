// Tests the library's parts that need no window: geometry, colors, easing,
// the design system's values, keys, how events fold into a frame's input,
// settings, the command line, and text layout.

use user_interface::controls::{PROGRESS_BAR_CURVE, PROGRESS_RING_CURVE};
use user_interface::tokens::{BODY, Curve, ELEVATION_DIALOG, ELEVATION_FLYOUT, ELEVATION_TOOLTIP, elevation};
use user_interface::json::{self, Value};
use user_interface::{Arguments, Color, CommonSettings, Event, Fonts, Input, Insets, Key, Modifiers, MouseButton, Point, Rect, Settings, Shortcut, Startup, Theme, ThemeKind, WindowMode};

#[test]
fn rectangles_cut_into_strips_and_columns() {
    let mut area = Rect::new(10.0, 20.0, 300.0, 200.0);
    let top = area.cut_top(32.0, 8.0);
    assert_eq!(top, Rect::new(10.0, 20.0, 300.0, 32.0));
    assert_eq!(area, Rect::new(10.0, 60.0, 300.0, 160.0));
    let right = area.cut_right(100.0, 10.0);
    assert_eq!(right, Rect::new(210.0, 60.0, 100.0, 160.0));
    assert_eq!(area.width, 190.0);
    let [a, b, c] = Rect::new(0.0, 0.0, 320.0, 10.0).columns::<3>(10.0);
    assert_eq!((a.x, a.width), (0.0, 100.0));
    assert_eq!((b.x, c.x), (110.0, 220.0));
    let [first, second] = Rect::new(0.0, 0.0, 10.0, 100.0).rows::<2>(10.0);
    assert_eq!((first.height, second.y), (45.0, 55.0));
    let weighted = Rect::new(0.0, 0.0, 310.0, 10.0).columns_weighted(&[1.0, 2.0], 10.0);
    assert_eq!((weighted[0].width, weighted[1].right()), (100.0, 310.0));
    assert!(Rect::new(0.0, 0.0, 10.0, 10.0).contains(Point::new(0.0, 9.9)));
    assert!(!Rect::new(0.0, 0.0, 10.0, 10.0).contains(Point::new(10.0, 5.0)));
    assert!(Rect::new(0.0, 0.0, 10.0, 10.0).intersect(Rect::new(20.0, 0.0, 5.0, 5.0)).is_empty());
    assert_eq!(Rect::new(0.0, 0.0, 100.0, 40.0).pad(Insets::new(11.0, 5.0, 11.0, 6.0)), Rect::new(11.0, 5.0, 78.0, 29.0));
    assert_eq!(Rect::new(10.0, 10.0, 100.0, 20.0).at_fraction(Point::new(0.25, 0.5)), Point::new(35.0, 20.0));
}

#[test]
fn colors_parse_and_blend_like_xaml_writes_them() {
    assert_eq!(Color::parse("#1CDA61"), Some(Color::hex(0x1CDA61)));
    assert_eq!(Color::parse("#E4000000"), Some(Color::argb(0xE400_0000)));
    assert_eq!(Color::parse("1CDA61"), Some(Color::hex(0x1CDA61)));
    assert_eq!(Color::parse("#12345"), None);
    assert_eq!(Color::hex(0x1CDA61).to_hex(), "#1CDA61");
    assert_eq!(Color::argb(0x8000_0000).to_hex(), "#80000000");
    let half_black = Color::argb(0x8000_0000).over(Color::WHITE);
    assert!((half_black.r - 0.498).abs() < 0.01 && half_black.a == 1.0);
    // Premultiplied for the GPU: a half-transparent white is half bright.
    assert_eq!(Color::rgba(1.0, 1.0, 1.0, 0.5).pack_premultiplied(), 0x8080_8080);
    let [r, g, b] = Color::from_linear(0.5, 0.0, 1.0).to_linear();
    assert!((r - 0.5).abs() < 0.01 && g == 0.0 && (b - 1.0).abs() < 1e-6);
    // Hue, saturation and value, and back.
    let color = Color::hex(0x1E90FF);
    let [hue, saturation, value] = color.to_hsv();
    assert!((hue - 209.6).abs() < 0.1 && (saturation - 0.882).abs() < 0.01 && value == 1.0);
    assert_eq!(Color::from_hsv(hue, saturation, value).to_hex(), "#1E90FF");
    assert_eq!(Color::from_hsv(120.0, 1.0, 1.0).to_hex(), "#00FF00");
}

#[test]
fn curves_go_from_0_to_1() {
    let linear = Curve(0.0, 0.0, 1.0, 1.0);
    for curve in [linear, PROGRESS_BAR_CURVE, PROGRESS_RING_CURVE] {
        assert!(curve.ease(0.0).abs() < 1e-4);
        assert!((curve.ease(1.0) - 1.0).abs() < 1e-4);
        // Each is symmetric, so it's half way at half the time.
        assert!((curve.ease(0.5) - 0.5).abs() < 0.01);
    }
    assert!((linear.ease(0.25) - 0.25).abs() < 0.01);
    // The indeterminate bar's indicators start slowly.
    assert!(PROGRESS_BAR_CURVE.ease(0.1) < 0.05);
}

#[test]
fn the_themes_have_winui_3s_values() {
    let light = Theme::light();
    let dark = Theme::dark();
    // AccentFillColorDefault is Windows' default blue's Dark1 in the light
    // theme and Light2 in the dark one.
    assert_eq!(light.accent.to_hex(), "#0067C0");
    assert_eq!(dark.accent.to_hex(), "#4CC2FF");
    assert_eq!(light.accent_secondary.to_hex(), "#E60067C0");
    assert_eq!(light.text_primary.to_hex(), "#E4000000");
    assert_eq!(dark.text_secondary.to_hex(), "#C5FFFFFF");
    assert_eq!(light.control_fill.to_hex(), "#B3FFFFFF");
    assert_eq!(light.control_stroke_secondary.to_hex(), "#29000000");
    assert_eq!(light.background.to_hex(), "#F3F3F3");
    assert_eq!(dark.background.to_hex(), "#202020");
    assert_eq!(light.acrylic.to_hex(), "#F9F9F9");
    assert_eq!(light.accent_selected_text_background.to_hex(), "#0078D4");
}

#[test]
fn elevations_follow_themeshadows_recipe() {
    // Up to 16, only the directional shadow: half the Z blurs, down a
    // quarter of the Z.
    assert_eq!(ELEVATION_TOOLTIP.shadows[0].opacity_light, 0.0);
    assert_eq!((ELEVATION_TOOLTIP.shadows[1].blur, ELEVATION_TOOLTIP.shadows[1].offset_y), (8.0, 4.0));
    assert_eq!(ELEVATION_TOOLTIP.shadows[1].opacity_light, 0.14);
    assert_eq!((ELEVATION_FLYOUT.shadows[1].blur, ELEVATION_FLYOUT.shadows[1].offset_y), (16.0, 8.0));
    assert_eq!(ELEVATION_FLYOUT.shadows[1].opacity_dark, 0.26);
    // Above, an ambient one too, and both stronger.
    assert_eq!(ELEVATION_DIALOG.shadows[0].opacity_light, 0.15);
    assert_eq!((ELEVATION_DIALOG.shadows[1].blur, ELEVATION_DIALOG.shadows[1].offset_y), (64.0, 32.0));
    assert_eq!(ELEVATION_DIALOG.shadows[1].opacity_dark, 0.37);
    assert!(elevation(2.0).shadows.iter().all(|shadow| shadow.opacity_light == 0.0));
}

#[test]
fn keys_have_names_that_parse_back() {
    for key in [Key::A, Key::F11, Key::LEFT, Key::SPACE, Key::ENTER, Key::ESCAPE, Key(b'7'), Key::PAGE_DOWN, Key::DELETE] {
        assert_eq!(Key::parse(&key.name()), Some(key), "{}", key.name());
    }
    assert_eq!(Key::parse("f11"), Some(Key::F11));
    assert_eq!(Key::parse("Nope"), None);
    assert_eq!(Shortcut::parse("Ctrl+S"), Some(Shortcut::new(Modifiers::CONTROL, Key::S)));
    assert_eq!(Shortcut::parse("Ctrl+Shift+Z"), Some(Shortcut::new(Modifiers::CONTROL_SHIFT, Key::Z)));
    assert_eq!(Shortcut::parse("Hyper+S"), None);
    assert_eq!(Shortcut::new(Modifiers::CONTROL_SHIFT, Key::S).name(), "Ctrl+Shift+S");
}

#[test]
fn events_fold_into_a_frame_of_input() {
    let mut input = Input::default();
    input.begin(0.0, &[Event::FocusGained(Vec::new()), Event::PointerMoved(Point::new(5.0, 5.0)), Event::ButtonPressed(MouseButton::Left)]);
    assert!(input.button_pressed(MouseButton::Left) && input.button_down(MouseButton::Left));
    assert_eq!(input.clicks(MouseButton::Left), 1);
    input.begin(0.1, &[Event::ButtonReleased(MouseButton::Left)]);
    assert!(input.button_released(MouseButton::Left) && !input.button_down(MouseButton::Left));
    // A second press, soon and close by, is a double-click.
    input.begin(0.2, &[Event::ButtonPressed(MouseButton::Left)]);
    assert_eq!(input.clicks(MouseButton::Left), 2);
    // Without focus there's no input, and losing it lets go of everything,
    // with a release.
    input.begin(0.3, &[Event::KeyPressed { key: Key::W, repeat: false }, Event::FocusLost, Event::KeyPressed { key: Key::A, repeat: false }]);
    assert!(input.focus_lost && !input.key_down(Key::W) && !input.key_down(Key::A) && !input.button_down(MouseButton::Left));
    assert!(input.key_released(Key::W) && input.button_released(MouseButton::Left));
    // Keys held when focus comes count once they're let go of.
    input.begin(0.4, &[Event::FocusGained(vec![Key::ALT]), Event::KeyPressed { key: Key::ALT, repeat: true }]);
    assert!(!input.key_down(Key::ALT));
    input.begin(0.5, &[Event::KeyReleased(Key::ALT), Event::KeyPressed { key: Key::ALT, repeat: false }]);
    assert!(input.key_down(Key::ALT));
    input.begin(0.6, &[Event::KeyPressed { key: Key::CONTROL, repeat: false }, Event::KeyPressed { key: Key::S, repeat: false }]);
    assert!(input.shortcut_pressed(Shortcut::new(Modifiers::new(true, false, true), Key::S)));
}

#[test]
fn settings_keep_their_defaults_when_missing_or_wrong() {
    let mut settings = Settings::from_text(r#"{ "Volume": 3, "Mode": "fast", "Name": 5 }"#);
    assert_eq!(settings.number("Volume", 1.0, |value| if value <= 2.0 { Ok(()) } else { Err("must be at most 2".to_string()) }), 1.0);
    assert_eq!(settings.choice("Mode", "Slow", &["Slow", "Fast"]), "Fast");
    assert_eq!(settings.string("Name", "Ada"), "Ada");
    assert!(settings.boolean("Missing", true));
    assert_eq!(settings.problems(), ["Volume must be at most 2, so it's the default.", "Name must be text, not 5, so it's the default."]);
    let broken = Settings::from_text("{ \"A\": ");
    assert_eq!(broken.problems().len(), 1);
}

#[test]
fn a_missing_settings_file_is_written_with_the_defaults() {
    let folder = std::env::temp_dir().join(format!("UserInterfaceTests{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("Settings.json");
    let _ = std::fs::remove_file(&path);
    let mut settings = Settings::load(&path);
    settings.boolean("VerticalSync", true);
    settings.number("DeadZone", 0.1, |_| Ok(()));
    settings.write_if_missing().unwrap();
    let written = std::fs::read_to_string(&path).unwrap();
    assert!(written.contains("\"VerticalSync\": true") && written.contains("\"DeadZone\": 0.1"), "{written}");
    std::fs::remove_dir_all(&folder).unwrap();
}

#[test]
fn the_settings_every_program_has_start_its_theme_window_and_vertical_sync() {
    let read = |text: &str| {
        let mut settings = Settings::from_text(text);
        (CommonSettings::read(&mut settings), settings.problems().to_vec())
    };
    let (defaults, problems) = read("{}");
    let light = CommonSettings {
        display: "Primary".to_string(),
        window_mode: WindowMode::WindowedMaximized,
        vertical_sync: true,
        theme: ThemeKind::Light,
    };
    assert_eq!((defaults, problems.len()), (light.clone(), 0));
    // Words ignore case.
    let (dark, problems) = read(r#"{ "Display": "CVVNY2", "WindowMode": "borderlessfullscreen", "VerticalSync": false, "Theme": "dark" }"#);
    assert_eq!(
        dark,
        CommonSettings {
            display: "CVVNY2".to_string(),
            window_mode: WindowMode::BorderlessFullscreen,
            vertical_sync: false,
            theme: ThemeKind::Dark,
        }
    );
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(read(r#"{ "Theme": "Light" }"#).0.theme, ThemeKind::Light);
    // A wrong theme is the default, and a notice says why.
    let (wrong, problems) = read(r#"{ "Theme": "Blue" }"#);
    assert_eq!(wrong.theme, ThemeKind::Light);
    assert_eq!(problems, ["Theme must be \"Light\" or \"Dark\", not \"Blue\", so it's the default."]);
    let (wrong, problems) = read(r#"{ "Theme": true }"#);
    assert_eq!(wrong.theme, ThemeKind::Light);
    assert_eq!(problems, ["Theme must be \"Light\" or \"Dark\", not true, so it's the default."]);
    // The window mode that was called Maximized is WindowedMaximized.
    let (old, problems) = read(r#"{ "WindowMode": "Maximized" }"#);
    assert_eq!(old.window_mode, WindowMode::WindowedMaximized);
    assert_eq!(problems, ["WindowMode must be \"WindowedMaximized\" or \"BorderlessFullscreen\", not \"Maximized\", so it's the default."]);
}

#[test]
fn a_missing_settings_file_is_written_with_the_settings_every_program_has() {
    let folder = std::env::temp_dir().join(format!("UserInterfaceCommonSettings{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("Settings.json");
    let _ = std::fs::remove_file(&path);
    // A program's start reads them, as `run` does, before the program's own.
    let mut startup = Startup::new(Settings::load(&path), &folder, false);
    startup.settings.boolean("Own", false);
    startup.settings.write_if_missing().unwrap();
    let written = std::fs::read_to_string(&path).unwrap();
    let expected = "{\n  \"Display\": \"Primary\",\n  \"WindowMode\": \"WindowedMaximized\",\n  \"VerticalSync\": true,\n  \"Theme\": \"Light\",\n  \"Own\": false\n}\n";
    assert_eq!(written, expected);
    std::fs::remove_dir_all(&folder).unwrap();
}

// Every program's Settings.json lists the settings that every program has,
// each valid, so that each can be seen and changed there.
#[test]
fn every_programs_settings_file_has_the_settings_every_program_has() {
    let programs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Program");
    let mut files = 0;
    for entry in std::fs::read_dir(&programs).unwrap() {
        let path = entry.unwrap().path().join("Settings.json");
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        files += 1;
        let Ok(Value::Object(members)) = json::parse(&text) else {
            panic!("{} isn't a JSON object", path.display());
        };
        let mut settings = Settings::from_text(&text);
        CommonSettings::read(&mut settings);
        assert!(settings.problems().is_empty(), "{}: {:?}", path.display(), settings.problems());
        for name in ["Display", "WindowMode", "VerticalSync", "Theme"] {
            assert!(members.iter().any(|(member, _)| member == name), "{} doesn't have {name}", path.display());
        }
    }
    assert!(files >= 8, "found only {files} programs' Settings.json in {}", programs.display());
}

#[test]
fn vsync_arguments_override_the_setting_only_when_given() {
    let parse = |arguments: &[&str]| Arguments::parse(arguments.iter().map(|argument| argument.to_string()), &[]).unwrap().vsync;
    assert_eq!(parse(&[]), None);
    assert_eq!(parse(&["--vsync"]), Some(true));
    assert_eq!(parse(&["--no-vsync"]), Some(false));
    assert_eq!(parse(&["--no-vsync", "--vsync"]), Some(true));
}

// RunInBackground starts every program with --vsync, --theme Light and
// --window-mode WindowedMaximized, so that its scripts and screenshots start
// the same way whatever the program's Settings.json says.
#[test]
fn the_command_line_overrides_the_theme_window_mode_and_vertical_sync_settings() {
    let parse = |arguments: &[&str]| Arguments::parse(arguments.iter().map(|argument| argument.to_string()), &[]);
    let started = |arguments: &[&str]| {
        let settings = r#"{ "Theme": "Dark", "WindowMode": "BorderlessFullscreen", "VerticalSync": false }"#;
        let mut startup = Startup::for_tests(settings, &std::env::temp_dir());
        parse(arguments).unwrap().override_settings(&mut startup.common);
        let common = startup.common;
        (common.theme, common.window_mode, common.vertical_sync)
    };
    assert_eq!(started(&[]), (ThemeKind::Dark, WindowMode::BorderlessFullscreen, false));
    let run_in_background = ["--vsync", "--theme", "Light", "--window-mode", "WindowedMaximized"];
    assert_eq!(started(&run_in_background), (ThemeKind::Light, WindowMode::WindowedMaximized, true));
    // Words ignore case, and the last one says, so RunInBackground's
    // --arguments, after its own, can say otherwise.
    assert_eq!(started(&["--theme", "light", "--theme", "DARK", "--window-mode", "windowedmaximized"]), (ThemeKind::Dark, WindowMode::WindowedMaximized, false));
    assert_eq!(parse(&["--theme", "Blue"]).unwrap_err(), "--theme takes Light or Dark, not Blue.");
    assert_eq!(parse(&["--window-mode", "Maximized"]).unwrap_err(), "--window-mode takes WindowedMaximized or BorderlessFullscreen, not Maximized.");
    assert_eq!(parse(&["--theme"]).unwrap_err(), "--theme needs a value after it.");
    assert!(parse(&["--light"]).unwrap_err().contains("--theme <Light or Dark>, --window-mode <WindowedMaximized or BorderlessFullscreen>"));
}

#[test]
fn text_lays_out_with_directwrite() {
    let mut fonts = Fonts::new().unwrap();
    let one = fonts.measure("Hello", BODY);
    let two = fonts.measure("Hello Hello", BODY);
    assert!(one.width > 20.0 && two.width > one.width * 1.8);
    assert_eq!(one.height, BODY.line_height);
    let wrapped = fonts.measure_wrapped("one two three four five six seven", BODY, 60.0);
    assert!(wrapped.height >= BODY.line_height * 3.0);
    // A caret goes between characters, and a hit finds the nearest one.
    let layout = fonts.layout("Hello", BODY, None);
    assert_eq!(layout.caret_x(0), 0.0);
    let end = layout.caret_x(5);
    assert!((end - one.width).abs() < 0.5);
    assert_eq!(layout.hit(Point::new(end + 20.0, 5.0)), 5);
    assert_eq!(layout.hit(Point::new(-5.0, 5.0)), 0);
    assert_eq!(fonts.family(user_interface::Font::Text), "Segoe UI Variable Text");
}

// The README's example program, which must keep working as it says.
mod readme {
    use user_interface::{AppBar, Button, Card, Context, Headless, Icon, Key, Program, Slider, Status, Tree, Ui};

    #[derive(Clone)]
    enum Message {
        Level(f64),
        Mute,
    }

    struct Volume {
        level: f64,
    }

    impl Program for Volume {
        type Message = Message;

        fn title(&self) -> String {
            "Volume".to_string()
        }

        fn update(&mut self, message: Message, _ui: &mut Context<Message>) {
            match message {
                Message::Level(level) => self.level = level,
                Message::Mute => self.level = 0.0,
            }
        }

        fn view(&self, ui: &mut Ui) -> Tree<Message> {
            let mut tree = Tree::new();
            let shell = tree.shell(ui, AppBar::new(Icon::Volume, "Volume"), false, Some(vec![Status::shortcut("F11", "Fullscreen")]));
            tree.card(ui, None, shell.page.left_strip(400.0).top_strip(200.0), Card::new("Output"), |_, card, area| {
                let mut area = area.content;
                let slider = Slider::new(self.level, 0.0, 100.0).step(1.0).header("Level").show_value(0).on_change(Message::Level);
                card.add("Level", area.cut_top(Slider::<Message>::height(true), 8.0), slider);
                card.add("Mute", area.cut_top(32.0, 0.0).left_strip(120.0), Button::new("Mute").on_press(Message::Mute));
            });
            tree
        }
    }

    #[test]
    fn the_readme_s_example_works_as_it_says() {
        let mut app = Headless::new(Volume { level: 50.0 });
        app.frame();
        app.click("Mute");
        assert_eq!(app.program.level, 0.0);
        app.key(Key::F11);
        assert!(app.fullscreen());
    }
}

// QuickHasher is for the library's own small keys, and a HashMap picks a
// bucket by a hash's low bits and tells the keys in it apart by its top 7.
// Keys that differ only in a low bit, or only in a high one, as glyph numbers
// and sizes do, must spread over both: a hasher that left them together
// would be right, and slow.
#[test]
fn the_quick_hasher_spreads_keys_that_are_close() {
    use std::hash::{BuildHasher, BuildHasherDefault};
    use user_interface::drawing::hashing::QuickHasher;

    let build = BuildHasherDefault::<QuickHasher>::default();
    let buckets = 1024;
    let crowded = |hashes: &[u64]| {
        let mut low = vec![0u32; buckets];
        let mut high = vec![0u32; 128];
        for hash in hashes {
            low[(hash % buckets as u64) as usize] += 1;
            high[(hash >> 57) as usize] += 1;
        }
        (low.into_iter().max().unwrap(), high.into_iter().max().unwrap())
    };
    // 4,096 keys: 4 a bucket and 32 a tag if they spread evenly.
    let cases: [(&str, Vec<u64>); 4] = [
        ("numbers in a row", (0..4096u64).map(|number| build.hash_one(number)).collect()),
        ("numbers that differ in their high bits", (0..4096u64).map(|number| build.hash_one(number << 52)).collect()),
        ("glyphs of one face at one size", (0..4096u16).map(|glyph| build.hash_one((3u16, glyph, 14.0f32.to_bits(), 1u8))).collect()),
        ("one glyph at many sizes", (0..4096u32).map(|size| build.hash_one((3u16, 40u16, (size as f32 * 0.25).to_bits(), 0u8))).collect()),
    ];
    for (name, hashes) in cases {
        let (low, high) = crowded(&hashes);
        assert!(low <= 16, "{name}: {low} keys in one bucket");
        assert!(high <= 96, "{name}: {high} keys with one tag");
        let mut distinct = hashes.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), hashes.len(), "{name}: two keys with one hash");
    }
}