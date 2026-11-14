// Typed text, made from keys: the ASCII that a key types on a US keyboard,
// with Shift or without. It needs nothing of Windows, so a key types the same
// on any computer, and a recorded run types the same when it's played again.
// It has no Caps Lock, no accents and no key repeat: a held key types once.
// A key held with Ctrl or Alt types nothing, as those are shortcuts.
//
// The table has the letters, the digits, space, the 11 keys of punctuation,
// each with its form under Shift, and the numpad's digits and operators,
// which Shift doesn't change.

use crate::input::devices::Key;

// The keys of punctuation: each key, what it types, and what it types with
// Shift.
const PUNCTUATION: [(u8, char, char); 11] = [
    (0xBA, ';', ':'),
    (0xBB, '=', '+'),
    (0xBC, ',', '<'),
    (0xBD, '-', '_'),
    (0xBE, '.', '>'),
    (0xBF, '/', '?'),
    (0xC0, '`', '~'),
    (0xDB, '[', '{'),
    (0xDC, '\\', '|'),
    (0xDD, ']', '}'),
    (0xDE, '\'', '"'),
];

// What the digits type with Shift, from 0 to 9.
const SHIFTED_DIGITS: [char; 10] = [')', '!', '@', '#', '$', '%', '^', '&', '*', '('];

// The numpad's operators.
const NUMPAD: [(u8, char); 5] = [(0x6A, '*'), (0x6B, '+'), (0x6D, '-'), (0x6E, '.'), (0x6F, '/')];

// What the key types, with Shift held or not, if anything.
pub fn typed(key: Key, shift: bool) -> Option<char> {
    let code = key.0;
    match code {
        b'A'..=b'Z' => Some(if shift { code as char } else { code.to_ascii_lowercase() as char }),
        b'0'..=b'9' => Some(if shift { SHIFTED_DIGITS[(code - b'0') as usize] } else { code as char }),
        0x20 => Some(' '),
        0x60..=0x69 => Some((b'0' + code - 0x60) as char),
        _ => PUNCTUATION
            .iter()
            .find(|(punctuation, _, _)| *punctuation == code)
            .map(|(_, plain, shifted)| if shift { *shifted } else { *plain })
            .or_else(|| NUMPAD.iter().find(|(numpad, _)| *numpad == code).map(|(_, character)| *character)),
    }
}

// The key that types the character, and whether Shift is held for it, for
// tests and scripts that type text: the main keys rather than the numpad's.
// None for a character that no key in the table types.
pub fn key_for(character: char) -> Option<(Key, bool)> {
    let main = (0x20..=0xDE).map(Key).filter(|key| !(0x60..=0x6F).contains(&key.0));
    for key in main {
        for shift in [false, true] {
            if typed(key, shift) == Some(character) {
                return Some((key, shift));
            }
        }
    }
    None
}
