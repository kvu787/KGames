// What typing and the keys do to the tester, as the window's keyboard
// messages give them to it: everything an input causes is in the tester's
// state as soon as it's handled, so the frame after it shows all of it.

mod common;

use common::real_font;
use pixel_font_tester::font::Font;
use pixel_font_tester::layout::{self, Style};
use pixel_font_tester::tester::{Input, Key, MAXIMUM_SCALE, Modifiers, Shortcut, Tester, Theme, shortcut};
use pixel_font_tester::text::{Position, Text};

fn tester(text: &str, font: &Font) -> Tester {
    Tester::new(Text::typed(text), 1, Theme::Light, true, font)
}

fn type_text(tester: &mut Tester, font: &Font, text: &str) {
    for unit in text.encode_utf16() {
        tester.handle(Input::Character(unit), font);
    }
}

fn press(tester: &mut Tester, font: &Font, key: Key) {
    tester.handle(Input::Key(key, Modifiers::NONE), font);
}

fn position(line: usize, column: usize) -> Position {
    Position { line, column }
}

#[test]
fn ascii_types_and_everything_past_it_is_ignored() {
    let font = real_font();
    let mut tester = tester("", &font);
    type_text(&mut tester, &font, "Hi! é😀 ~{}\u{7F}\u{1B}");
    assert_eq!(tester.text.to_text(), "Hi!  ~{}");
    // Ctrl and a letter type control characters, which do nothing.
    tester.handle(Input::Character(0x01), &font);
    tester.handle(Input::Character(0x1A), &font);
    assert_eq!(tester.text.to_text(), "Hi!  ~{}");
}

#[test]
fn number_keys_type_digits() {
    let font = real_font();
    let mut tester = tester("", &font);
    // The key goes down, then TranslateMessage's WM_CHAR types the digit.
    press(&mut tester, &font, Key::Digit(4));
    tester.handle(Input::Character(u16::from(b'4')), &font);
    assert_eq!(tester.text.to_text(), "4");
    assert_eq!(tester.scale, 1);
}

#[test]
fn enter_breaks_the_line_backspace_joins_it_and_tab_types_the_box() {
    let font = real_font();
    let mut tester = tester("", &font);
    type_text(&mut tester, &font, "ab\rcd\ne\tf");
    assert_eq!(tester.text.to_text(), "ab\ncd\ne\tf");
    assert_eq!(tester.text.caret(), position(2, 3));
    press(&mut tester, &font, Key::Home);
    type_text(&mut tester, &font, "\u{8}");
    assert_eq!(tester.text.to_text(), "ab\ncde\tf");
    assert_eq!(tester.text.caret(), position(1, 2));
    press(&mut tester, &font, Key::Delete);
    assert_eq!(tester.text.to_text(), "ab\ncd\tf");
    // Delete at a line's end joins the next one, and Backspace at the
    // text's start does nothing.
    tester.handle(Input::Key(Key::Home, Modifiers::CONTROL), &font);
    press(&mut tester, &font, Key::End);
    press(&mut tester, &font, Key::Delete);
    assert_eq!(tester.text.to_text(), "abcd\tf");
    tester.handle(Input::Key(Key::Home, Modifiers::CONTROL), &font);
    type_text(&mut tester, &font, "\u{8}");
    assert_eq!(tester.text.to_text(), "abcd\tf");
    assert_eq!(tester.text.caret(), position(0, 0));
}

#[test]
fn the_arrows_move_the_caret_through_lines_and_up_and_down_keep_their_column() {
    let font = real_font();
    let mut tester = tester("long line\nab\nanother line", &font);
    assert_eq!(tester.text.caret(), position(2, 12));
    press(&mut tester, &font, Key::Left);
    assert_eq!(tester.text.caret(), position(2, 11));
    press(&mut tester, &font, Key::Up);
    assert_eq!(tester.text.caret(), position(1, 2));
    press(&mut tester, &font, Key::Up);
    assert_eq!(tester.text.caret(), position(0, 9));
    press(&mut tester, &font, Key::Down);
    press(&mut tester, &font, Key::Down);
    assert_eq!(tester.text.caret(), position(2, 11));
    // Up on the first line goes to its start, and Down on the last to its end.
    press(&mut tester, &font, Key::Down);
    assert_eq!(tester.text.caret(), position(2, 12));
    tester.handle(Input::Key(Key::Home, Modifiers::CONTROL), &font);
    press(&mut tester, &font, Key::Right);
    press(&mut tester, &font, Key::Up);
    assert_eq!(tester.text.caret(), position(0, 0));
    // Left from a line's start goes to the end of the line before, and Right
    // from its end to the start of the next.
    press(&mut tester, &font, Key::Down);
    press(&mut tester, &font, Key::Left);
    assert_eq!(tester.text.caret(), position(0, 9));
    press(&mut tester, &font, Key::Right);
    assert_eq!(tester.text.caret(), position(1, 0));
    tester.handle(Input::Key(Key::End, Modifiers::CONTROL), &font);
    assert_eq!(tester.text.caret(), position(2, 12));
}

#[test]
fn ctrl_and_a_number_sets_the_size_and_zero_is_ten() {
    let font = real_font();
    let mut tester = tester("text", &font);
    for digit in 1..=9 {
        tester.handle(Input::Key(Key::Digit(digit), Modifiers::CONTROL), &font);
        assert_eq!(tester.scale, u32::from(digit));
    }
    tester.handle(Input::Key(Key::Digit(0), Modifiers::CONTROL), &font);
    assert_eq!(tester.scale, MAXIMUM_SCALE);
    // Ctrl's control characters that follow type nothing.
    tester.handle(Input::Character(0x00), &font);
    tester.handle(Input::Character(0x1E), &font);
    assert_eq!(tester.text.to_text(), "text");
    assert_eq!(tester.spacing, font.spacing());
}

const CONTROL_SHIFT: Modifiers = Modifiers {
    control: true,
    shift: true,
    alt: false,
};

#[test]
fn ctrl_shift_and_a_number_sets_the_spacing_from_none_to_nine() {
    let font = real_font();
    let mut tester = tester("text", &font);
    // From the sheet's.
    assert_eq!(tester.spacing, font.spacing());
    for digit in [0, 1, 5, 9, 3] {
        tester.handle(Input::Key(Key::Digit(digit), CONTROL_SHIFT), &font);
        assert_eq!(tester.spacing, u32::from(digit));
        assert_eq!(tester.scale, 1, "Ctrl+Shift+{digit} set the size");
    }
    // Ctrl+Shift+2's NUL, and Ctrl+Shift+6's record separator, type nothing.
    tester.handle(Input::Character(0x00), &font);
    tester.handle(Input::Character(0x1E), &font);
    assert_eq!(tester.text.to_text(), "text");
    assert_eq!(shortcut(Key::Digit(4), CONTROL_SHIFT), Some(Shortcut::Spacing(4)));
    assert_eq!(shortcut(Key::Digit(4), Modifiers::CONTROL), Some(Shortcut::Scale(4)));
    assert_eq!(shortcut(Key::Left, CONTROL_SHIFT), None);
}

#[test]
fn shift_and_a_number_types_what_the_layout_says() {
    let font = real_font();
    let mut tester = tester("", &font);
    let shift = Modifiers { shift: true, ..Modifiers::NONE };
    tester.handle(Input::Key(Key::Digit(1), shift), &font);
    tester.handle(Input::Character(u16::from(b'!')), &font);
    assert_eq!(tester.text.to_text(), "!");
    assert_eq!((tester.scale, tester.spacing), (1, font.spacing()));
    assert_eq!(shortcut(Key::Digit(1), shift), None);
    // Shift+AltGr, which is Ctrl+Shift+Alt to Windows, types too.
    let shift_altgr = Modifiers { alt: true, ..CONTROL_SHIFT };
    tester.handle(Input::Key(Key::Digit(1), shift_altgr), &font);
    assert_eq!(tester.spacing, font.spacing());
    assert_eq!(shortcut(Key::Digit(1), shift_altgr), None);
}

#[test]
fn a_new_spacing_scrolls_to_show_the_caret_at_once() {
    let font = real_font();
    let mut tester = tester(&"x".repeat(200), &font);
    for digit in [9, 0, 4] {
        tester.handle(Input::Key(Key::Digit(digit), CONTROL_SHIFT), &font);
        assert!(caret_shows(&tester, &font), "Ctrl+Shift+{digit}: {:?}", tester.scroll);
        let columns = layout::visible_columns(&font, tester.style());
        assert_eq!(tester.scroll.column, 201usize.saturating_sub(columns), "Ctrl+Shift+{digit}");
    }
    // Spacing 9 shows fewer columns than none.
    let columns = |spacing| layout::visible_columns(&font, Style { scale: 1, spacing });
    assert!(columns(9) < columns(0));
}

#[test]
fn altgr_and_a_number_types_instead_of_setting_the_size() {
    let font = real_font();
    let mut tester = tester("", &font);
    // AltGr is Ctrl+Alt to Windows: German's AltGr+7 types {.
    let altgr = Modifiers { alt: true, ..Modifiers::CONTROL };
    tester.handle(Input::Key(Key::Digit(7), altgr), &font);
    tester.handle(Input::Character(u16::from(b'{')), &font);
    assert_eq!(tester.scale, 1);
    assert_eq!(tester.text.to_text(), "{");
}

#[test]
fn f9_switches_the_theme_f10_vertical_sync_and_ctrl_shift_alt_f12_quits() {
    let font = real_font();
    let mut tester = tester("", &font);
    press(&mut tester, &font, Key::F9);
    assert_eq!(tester.theme, Theme::Dark);
    press(&mut tester, &font, Key::F9);
    assert_eq!(tester.theme, Theme::Light);
    press(&mut tester, &font, Key::F10);
    assert!(!tester.vsync);
    press(&mut tester, &font, Key::F10);
    assert!(tester.vsync);
    for modifiers in [Modifiers::NONE, Modifiers::CONTROL, Modifiers { shift: true, ..Modifiers::CONTROL }] {
        tester.handle(Input::Key(Key::F12, modifiers), &font);
        assert!(!tester.quit, "{modifiers:?}");
    }
    tester.handle(Input::Key(Key::F12, Modifiers { control: true, shift: true, alt: true }), &font);
    assert!(tester.quit);
}

// The caret, and the character after it, are always whole in the box, from
// the input that moves them.
fn caret_shows(tester: &Tester, font: &Font) -> bool {
    let caret = tester.text.caret();
    let scroll = tester.scroll;
    caret.line >= scroll.line
        && caret.column >= scroll.column
        && caret.line < scroll.line + layout::visible_lines(font, tester.style())
        && caret.column < scroll.column + layout::visible_columns(font, tester.style())
}

#[test]
fn the_box_scrolls_with_the_caret_as_it_moves() {
    let font = real_font();
    let mut tester = tester("", &font);
    let columns = layout::visible_columns(&font, tester.style());
    // Typing past the right edge scrolls right, with that character.
    type_text(&mut tester, &font, &"x".repeat(columns - 1));
    assert_eq!(tester.scroll, position(0, 0));
    type_text(&mut tester, &font, "x");
    assert_eq!(tester.scroll, position(0, 1));
    assert!(caret_shows(&tester, &font));
    // Home scrolls back at once.
    press(&mut tester, &font, Key::Home);
    assert_eq!(tester.scroll, position(0, 0));
    // So do lines, down and up.
    let lines = layout::visible_lines(&font, tester.style());
    press(&mut tester, &font, Key::End);
    type_text(&mut tester, &font, &"\r".repeat(lines));
    assert_eq!(tester.scroll, position(1, 0));
    assert!(caret_shows(&tester, &font));
    tester.handle(Input::Key(Key::Home, Modifiers::CONTROL), &font);
    assert_eq!(tester.scroll, position(0, 0));
}

#[test]
fn a_new_size_scrolls_to_show_the_caret_at_once() {
    let font = real_font();
    let mut tester = tester(&format!("{}\n{}", "x".repeat(200), "y".repeat(100)), &font);
    assert!(caret_shows(&tester, &font));
    for digit in [9, 3, 0, 1, 5] {
        tester.handle(Input::Key(Key::Digit(digit), Modifiers::CONTROL), &font);
        assert!(caret_shows(&tester, &font), "Ctrl+{digit}: {:?}", tester.scroll);
        // As far left and up as shows it.
        let columns = layout::visible_columns(&font, tester.style());
        assert_eq!(tester.scroll.column, 101usize.saturating_sub(columns), "Ctrl+{digit}");
    }
    // Small enough, it all shows from the top left.
    tester.handle(Input::Key(Key::Digit(1), Modifiers::CONTROL), &font);
    assert_eq!(tester.scroll, position(0, 0));
}

#[test]
fn text_from_the_command_line_types_like_the_keyboard() {
    let text = Text::typed("a\tb\r\nc\u{e9}d");
    assert_eq!(text.lines(), [b"a\tb".to_vec(), b"cd".to_vec()]);
    assert_eq!(text.caret(), position(1, 2));
}
