// The tester's command line, which RunInBackground's runs give the options
// that every program takes, and the keys that the window reads.

use pixel_font_tester::arguments::Arguments;
use pixel_font_tester::tester::{Key, Theme};
use pixel_font_tester::window::key;
use windows_api_raw::*;

fn parse(arguments: &[&str]) -> Result<Arguments, String> {
    Arguments::parse(arguments.iter().map(|argument| argument.to_string()))
}

#[test]
fn without_options_it_opens_on_the_primary_display_light_and_vertically_synced() {
    let arguments = parse(&[]).unwrap();
    assert_eq!(arguments, Arguments::default());
    assert_eq!(arguments.display, "Primary");
    assert!(arguments.vsync && !arguments.no_activate && !arguments.no_input);
    assert_eq!((arguments.theme, arguments.scale, arguments.text.as_str()), (Theme::Light, 1, ""));
}

#[test]
fn it_takes_what_run_in_background_gives_every_program() {
    let arguments = parse(&["--display", "CVVNY2", "--no-activate", "--no-input", "--vsync", "--theme", "Light"]).unwrap();
    assert_eq!(arguments.display, "CVVNY2");
    assert!(arguments.no_activate && arguments.no_input && arguments.vsync);
    assert!(!parse(&["--no-vsync"]).unwrap().vsync);
    assert_eq!(parse(&["--theme", "dark"]).unwrap().theme, Theme::Dark);
}

#[test]
fn the_size_the_spacing_and_the_text_to_start_with_can_be_given() {
    assert_eq!(parse(&["--size", "10"]).unwrap().scale, 10);
    assert_eq!(parse(&["--text", r"one\ntwo \\n"]).unwrap().text, "one\ntwo \\n");
    for size in ["0", "11", "two"] {
        let error = parse(&["--size", size]).unwrap_err();
        assert!(error.contains("--size"), "{error}");
    }
    assert_eq!(parse(&[]).unwrap().spacing, None);
    assert_eq!(parse(&["--spacing", "0"]).unwrap().spacing, Some(0));
    assert_eq!(parse(&["--spacing", "9"]).unwrap().spacing, Some(9));
    for spacing in ["10", "-1", "two"] {
        let error = parse(&["--spacing", spacing]).unwrap_err();
        assert!(error.contains("--spacing"), "{error}");
    }
}

#[test]
fn what_it_doesn_t_take_fails_and_says_why() {
    assert!(parse(&["--display"]).unwrap_err().contains("needs a value"));
    assert!(parse(&["--theme", "Blue"]).unwrap_err().contains("Light or Dark"));
    assert!(parse(&["--frobnicate"]).unwrap_err().contains("--frobnicate"));
    assert!(parse(&["--script", "Some.json"]).unwrap_err().contains("doesn't run scripts"));
}

#[test]
fn the_window_reads_the_number_keys_of_the_top_row_and_the_numpad() {
    for digit in 0..10u8 {
        assert_eq!(key(0x30 + i32::from(digit)), Some(Key::Digit(digit)));
        assert_eq!(key(VK_NUMPAD0 + i32::from(digit)), Some(Key::Digit(digit)));
    }
    let keys = [
        (VK_LEFT, Key::Left),
        (VK_RIGHT, Key::Right),
        (VK_UP, Key::Up),
        (VK_DOWN, Key::Down),
        (VK_HOME, Key::Home),
        (VK_END, Key::End),
        (VK_DELETE, Key::Delete),
        (VK_F9, Key::F9),
        (VK_F10, Key::F10),
        (VK_F12, Key::F12),
    ];
    for (virtual_key, expected) in keys {
        assert_eq!(key(virtual_key), Some(expected));
    }
    // Letters, and keys that type, like Enter, come as WM_CHAR's characters.
    for virtual_key in [0x41, 0x0D, 0x08, 0x09, VK_CONTROL] {
        assert_eq!(key(virtual_key), None, "{virtual_key}");
    }
}
