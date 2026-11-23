// The tester's state, and what input does to it. Input comes from the
// window's keyboard messages, as the window procedure turns them into
// `Input`, which tests make up instead. Everything an input causes, like the
// box scrolling to keep the caret in it, happens as it's handled, so the
// frame that follows shows all of it.
//
// Ctrl and a number key sets the text's size, and Ctrl+Shift and a number
// key the spacing between its characters, in the font's pixels.

use crate::font::Font;
use crate::layout::{self, Style};
use crate::text::{Position, Text};

// The biggest size: Ctrl+0's, after Ctrl+1 to Ctrl+9's 1 to 9 times the
// font's own.
pub const MAXIMUM_SCALE: u32 = 10;
// The most spacing between characters: Ctrl+Shift+9's. Ctrl+Shift+0's is
// none.
pub const MAXIMUM_SPACING: u32 = 9;

// What a number key with Ctrl sets: the size, or with Ctrl+Shift, the spacing
// between characters. Without Ctrl, it types, like Shift+1's !. AltGr, which
// some layouts type characters with, is Ctrl+Alt to Windows, so nothing with
// Alt is a shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shortcut {
    Scale(u32),
    Spacing(u32),
}

pub fn shortcut(key: Key, modifiers: Modifiers) -> Option<Shortcut> {
    let Key::Digit(digit) = key else {
        return None;
    };
    let digit = u32::from(digit);
    match modifiers {
        Modifiers { control: false, .. } | Modifiers { alt: true, .. } => None,
        Modifiers { shift: true, .. } => Some(Shortcut::Spacing(digit)),
        Modifiers { shift: false, .. } => Some(Shortcut::Scale(if digit == 0 { MAXIMUM_SCALE } else { digit })),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn toggled(self) -> Theme {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }

    pub fn colors(self) -> Colors {
        match self {
            Theme::Light => Colors {
                background: [243, 243, 243],
                field: [255, 255, 255],
                border: [204, 204, 204],
                text: [0, 0, 0],
            },
            Theme::Dark => Colors {
                background: [32, 32, 32],
                field: [44, 44, 44],
                border: [96, 96, 96],
                text: [255, 255, 255],
            },
        }
    }
}

// A theme's colors, as sRGB bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Colors {
    // The window's, around the box.
    pub background: [u8; 3],
    // The box's, inside its border.
    pub field: [u8; 3],
    pub border: [u8; 3],
    // The text's and the caret's.
    pub text: [u8; 3],
}

// The modifiers held as a key went down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub control: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Modifiers {
    pub const NONE: Modifiers = Modifiers {
        control: false,
        shift: false,
        alt: false,
    };
    pub const CONTROL: Modifiers = Modifiers {
        control: true,
        shift: false,
        alt: false,
    };
}

// The keys that do something other than type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Delete,
    // A number key, on the keyboard's top row or its numpad.
    Digit(u8),
    F9,
    F10,
    F12,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    // What the keyboard typed, as WM_CHAR has it: a UTF-16 code unit, which
    // the keyboard's layout made from the keys, like Shift+1's !. Enter,
    // Backspace and Tab type control characters, which edit the text.
    Character(u16),
    // A key that went down, or repeated as it's held.
    Key(Key, Modifiers),
}

pub struct Tester {
    pub text: Text,
    // How many of the screen's pixels each of the font's pixels is, across
    // and down.
    pub scale: u32,
    // How many of the font's pixels are between characters.
    pub spacing: u32,
    pub theme: Theme,
    pub vsync: bool,
    // Whether Ctrl+Shift+Alt+F12 asked to quit.
    pub quit: bool,
    // The first line and column that the box shows.
    pub scroll: Position,
}

impl Tester {
    // With the spacing between characters that the font's sheet has.
    pub fn new(text: Text, scale: u32, theme: Theme, vsync: bool, font: &Font) -> Tester {
        let mut tester = Tester {
            text,
            scale: scale.clamp(1, MAXIMUM_SCALE),
            spacing: font.spacing(),
            theme,
            vsync,
            quit: false,
            scroll: Position::default(),
        };
        tester.show_caret(font);
        tester
    }

    pub fn style(&self) -> Style {
        Style {
            scale: self.scale,
            spacing: self.spacing,
        }
    }

    // Sets how many of the font's pixels are between characters, and scrolls
    // from the top left again, as far as it takes to show the caret.
    pub fn set_spacing(&mut self, spacing: u32, font: &Font) {
        self.spacing = spacing.min(MAXIMUM_SPACING);
        self.scroll = Position::default();
        self.show_caret(font);
    }

    pub fn handle(&mut self, input: Input, font: &Font) {
        match input {
            Input::Character(character) => self.typed(character),
            Input::Key(key, modifiers) => self.pressed(key, modifiers),
        }
        self.show_caret(font);
    }

    // Only ASCII types: anything past it, like é, or half of an emoji's
    // surrogate pair, is ignored, and so are the control characters that
    // Ctrl and a letter type, and Escape's.
    fn typed(&mut self, character: u16) {
        match character {
            0x08 => self.text.backspace(),
            // Enter, and Ctrl+Enter's line feed.
            0x0D | 0x0A => self.text.new_line(),
            0x09 | 0x20..=0x7E => self.text.insert(character as u8),
            _ => {}
        }
    }

    fn pressed(&mut self, key: Key, modifiers: Modifiers) {
        match key {
            Key::Digit(_) => {
                match shortcut(key, modifiers) {
                    Some(Shortcut::Scale(scale)) => self.scale = scale,
                    Some(Shortcut::Spacing(spacing)) => self.spacing = spacing,
                    // It types, with the WM_CHAR that comes after it.
                    None => return,
                }
                // From the top left again, as far as the caret lets it.
                self.scroll = Position::default();
            }
            Key::F12 if modifiers.control && modifiers.shift && modifiers.alt => self.quit = true,
            Key::F12 => {}
            Key::F9 => self.theme = self.theme.toggled(),
            Key::F10 => self.vsync = !self.vsync,
            Key::Left => self.text.left(),
            Key::Right => self.text.right(),
            Key::Up => self.text.up(),
            Key::Down => self.text.down(),
            Key::Home if modifiers.control => self.text.start(),
            Key::Home => self.text.home(),
            Key::End if modifiers.control => self.text.finish(),
            Key::End => self.text.end(),
            Key::Delete => self.text.delete(),
        }
    }

    // Scrolls the box as little as it takes to show the caret, and the
    // character after it, whole.
    fn show_caret(&mut self, font: &Font) {
        let caret = self.text.caret();
        let columns = layout::visible_columns(font, self.style());
        let lines = layout::visible_lines(font, self.style());
        self.scroll.column = self.scroll.column.clamp((caret.column + 1).saturating_sub(columns), caret.column);
        self.scroll.line = self.scroll.line.clamp((caret.line + 1).saturating_sub(lines), caret.line);
    }
}
