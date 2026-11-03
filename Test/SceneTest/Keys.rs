// The keys that every scene test has, which Library/SceneTest handles.

use application::Event;
use scene_test::{Action, action};

fn key(key: u32, control: bool) -> Event {
    Event::Key {
        key,
        control,
        shift: false,
        repeat: false,
    }
}

#[test]
fn f1_toggles_the_text_and_f11_fullscreen() {
    assert_eq!(action(&key(0x70, false)), Some(Action::ToggleText));
    assert_eq!(action(&key(0x70, true)), Some(Action::ToggleText));
    assert_eq!(action(&key(0x7A, false)), Some(Action::ToggleFullscreen));
    // The tests' own keys go to their scenes.
    assert_eq!(action(&key('S' as u32, false)), None);
    assert_eq!(action(&key('V' as u32, false)), None, "VSync isn't a setting here");
    assert_eq!(action(&key(0x1B, false)), None, "Esc doesn't quit");
    assert_eq!(action(&Event::Press([10.0, 10.0])), None);
    // Holding a key down doesn't flip it back and forth.
    for code in [0x70, 0x7A] {
        let held = Event::Key {
            key: code,
            control: false,
            shift: false,
            repeat: true,
        };
        assert_eq!(action(&held), None);
    }
}
