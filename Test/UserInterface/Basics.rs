// Tests the library's parts that need no window: geometry, colors, easing,
// keys, how events fold into a frame's input, settings, and text layout.

use user_interface::input::parse_shortcut;
use user_interface::text::TextStyle;
use user_interface::tokens::{BODY, CURVE_DECELERATE, CURVE_EASY_EASE, CURVE_LINEAR};
use user_interface::{Color, Event, Fonts, Input, Key, Modifiers, MouseButton, Point, Rect, Settings};

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
    assert!(Rect::new(0.0, 0.0, 10.0, 10.0).contains(Point::new(0.0, 9.9)));
    assert!(!Rect::new(0.0, 0.0, 10.0, 10.0).contains(Point::new(10.0, 5.0)));
    assert!(Rect::new(0.0, 0.0, 10.0, 10.0).intersect(Rect::new(20.0, 0.0, 5.0, 5.0)).is_empty());
}

#[test]
fn colors_parse_and_blend_like_fluent_writes_them() {
    assert_eq!(Color::parse("#1CDA61"), Some(Color::hex(0x1CDA61)));
    assert_eq!(Color::parse("#E4000000"), Some(Color::argb(0xE400_0000)));
    assert_eq!(Color::parse("1CDA61"), None);
    assert_eq!(Color::parse("#12345"), None);
    assert_eq!(Color::hex(0x1CDA61).to_hex(), "#1CDA61");
    let half_black = Color::argb(0x8000_0000).over(Color::WHITE);
    assert!((half_black.r - 0.498).abs() < 0.01 && half_black.a == 1.0);
    // Premultiplied for the GPU: a half-transparent white is half bright.
    assert_eq!(Color::rgba(1.0, 1.0, 1.0, 0.5).pack_premultiplied(), 0x8080_8080);
    let [r, g, b] = Color::from_linear(0.5, 0.0, 1.0).to_linear();
    assert!((r - 0.5).abs() < 0.01 && g == 0.0 && (b - 1.0).abs() < 1e-6);
}

#[test]
fn curves_go_from_0_to_1() {
    for curve in [CURVE_LINEAR, CURVE_DECELERATE, CURVE_EASY_EASE] {
        assert!(curve.ease(0.0).abs() < 1e-4);
        assert!((curve.ease(1.0) - 1.0).abs() < 1e-4);
    }
    // Decelerating covers most of the way early.
    assert!(CURVE_DECELERATE.ease(0.25) > 0.6);
    assert!((CURVE_EASY_EASE.ease(0.5) - 0.5).abs() < 0.01);
}

#[test]
fn keys_have_names_that_parse_back() {
    for key in [Key::A, Key::F11, Key::LEFT, Key::SPACE, Key::ENTER, Key::ESCAPE, Key(b'7'), Key::PAGE_DOWN] {
        assert_eq!(Key::parse(&key.name()), Some(key), "{}", key.name());
    }
    assert_eq!(Key::parse("f11"), Some(Key::F11));
    assert_eq!(Key::parse("Nope"), None);
    assert_eq!(parse_shortcut("Ctrl+S"), Some((Modifiers::CONTROL, Key::S)));
    assert_eq!(parse_shortcut("Ctrl+Shift+Z"), Some((Modifiers::CONTROL_SHIFT, Key::Z)));
    assert_eq!(parse_shortcut("Hyper+S"), None);
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
    // Without focus, there's no input, and losing it lets go of everything.
    input.begin(0.3, &[Event::KeyPressed { key: Key::W, repeat: false }, Event::FocusLost, Event::KeyPressed { key: Key::A, repeat: false }]);
    assert!(input.focus_lost && !input.key_down(Key::W) && !input.key_down(Key::A) && !input.button_down(MouseButton::Left));
    // Keys held when focus comes count once they're let go of.
    input.begin(0.4, &[Event::FocusGained(vec![Key::ALT]), Event::KeyPressed { key: Key::ALT, repeat: true }]);
    assert!(!input.key_down(Key::ALT));
    input.begin(0.5, &[Event::KeyReleased(Key::ALT), Event::KeyPressed { key: Key::ALT, repeat: false }]);
    assert!(input.key_down(Key::ALT));
}

#[test]
fn settings_keep_their_defaults_when_missing_or_wrong() {
    let mut settings = Settings::from_text(r#"{ "Volume": 3, "Mode": "fast", "Name": 5 }"#);
    assert_eq!(settings.number("Volume", 1.0, |value| if value <= 2.0 { Ok(()) } else { Err("must be at most 2".to_string()) }), 1.0);
    assert_eq!(settings.choice("Mode", "Slow", &["Slow", "Fast"]), "Fast");
    assert_eq!(settings.string("Name", "Ada"), "Ada");
    assert!(settings.boolean("Missing", true));
    assert_eq!(settings.problems().len(), 2, "{:?}", settings.problems());
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
    settings.boolean("AllowTearing", true);
    settings.number("DeadZone", 0.1, |_| Ok(()));
    settings.write_if_missing().unwrap();
    let written = std::fs::read_to_string(&path).unwrap();
    assert!(written.contains("\"AllowTearing\": true") && written.contains("\"DeadZone\": 0.1"), "{written}");
    std::fs::remove_dir_all(&folder).unwrap();
}

#[test]
fn text_lays_out_with_directwrite() {
    let mut fonts = Fonts::new().unwrap();
    let one = fonts.measure("Hello", BODY);
    let two = fonts.measure("Hello Hello", BODY);
    assert!(one.width > 20.0 && two.width > one.width * 1.8);
    assert_eq!(one.height, BODY.line_height);
    let wrapped = fonts.layout("one two three four five six seven", BODY, Some(60.0)).size();
    assert!(wrapped.height >= BODY.line_height * 3.0);
    // A caret goes between characters, and a hit finds the nearest one.
    let layout = fonts.layout("Hello", BODY, None);
    assert_eq!(layout.caret_x(0), 0.0);
    let end = layout.caret_x(5);
    assert!((end - one.width).abs() < 0.5);
    assert_eq!(layout.hit(Point::new(end + 20.0, 5.0)), 5);
    assert_eq!(layout.hit(Point::new(-5.0, 5.0)), 0);
    let _: TextStyle = BODY;
}
