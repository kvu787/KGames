// Library/Application's gamepad: its buttons' names and bits, what went down
// since the frame before, and what frames see of it with and without focus.

use application::{FocusFilter, Gamepad, GamepadButton};

fn held(buttons: &[GamepadButton]) -> Gamepad {
    Gamepad {
        connected: true,
        right_stick: [0.25, -0.5],
        left_trigger: 0.75,
        buttons: buttons.iter().fold(0, |bits, button| bits | button.bit()),
        ..Gamepad::default()
    }
}

#[test]
fn buttons_have_names_and_bits_of_their_own() {
    for button in GamepadButton::ALL {
        assert_eq!(GamepadButton::from_name(button.name()), Some(button));
        let others = GamepadButton::ALL.iter().filter(|other| other.bit() & button.bit() != 0);
        assert_eq!(others.count(), 1, "{button:?} shares its bit");
    }
    assert_eq!(GamepadButton::from_name("LB"), Some(GamepadButton::LeftShoulder));
    assert_eq!(GamepadButton::from_name("Start"), None);
    assert_eq!(held(&[GamepadButton::Up, GamepadButton::A]).held_names(), ["A", "Up"]);
}

// A button is pressed in the frame that it goes down, and held while it's
// down.
#[test]
fn a_button_is_pressed_once_and_held_until_it_is_let_go_of() {
    let gamepad = held(&[GamepadButton::A, GamepadButton::X]).after(GamepadButton::X.bit());
    assert!(gamepad.held(GamepadButton::A) && gamepad.pressed(GamepadButton::A));
    assert!(gamepad.held(GamepadButton::X) && !gamepad.pressed(GamepadButton::X));
    assert!(!gamepad.held(GamepadButton::B) && !gamepad.pressed(GamepadButton::B));
}

// Without focus, frames see the gamepad at rest, though connected. A button
// that's still down when the window gets focus counts once it's pressed
// again, but sticks and triggers count right away. Buttons held when the
// program starts are ignored the same way.
#[test]
fn buttons_down_without_focus_count_once_pressed_again() {
    let mut filter = FocusFilter::default();
    let start = filter.frame(held(&[GamepadButton::A]), true);
    assert!(!start.held(GamepadButton::A), "a button held at the start doesn't count");
    assert_eq!((start.right_stick, start.left_trigger), ([0.25, -0.5], 0.75));
    let pressed = filter.frame(held(&[GamepadButton::B]), true);
    assert!(pressed.pressed(GamepadButton::B) && pressed.held(GamepadButton::B));
    let again = filter.frame(held(&[GamepadButton::A, GamepadButton::B]), true);
    assert!(again.pressed(GamepadButton::A) && !again.pressed(GamepadButton::B));

    let unfocused = filter.frame(held(&[GamepadButton::A, GamepadButton::B]), false);
    assert_eq!(
        unfocused,
        Gamepad {
            connected: true,
            ..Gamepad::default()
        }
    );
    let back = filter.frame(held(&[GamepadButton::B]), true);
    assert_eq!(back.buttons, 0, "B was down without focus");
    assert_eq!(back.right_stick, [0.25, -0.5]);
    filter.frame(held(&[]), true);
    let pressed = filter.frame(held(&[GamepadButton::B]), true);
    assert!(pressed.pressed(GamepadButton::B));

    // Like when the window was minimized, with no frame to see it.
    filter.lose_focus();
    assert_eq!(filter.frame(held(&[GamepadButton::B]), true).buttons, 0);
}
