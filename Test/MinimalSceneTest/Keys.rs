// MinimalSceneTest's keys, through its interface without a window: F1 shows
// and hides the card, F11 switches between maximized and borderless
// fullscreen, and holding a key down doesn't do it again.

use minimal_scene_test::info_card;
use minimal_scene_test::moving_sphere::MovingSphere;
use minimal_scene_test::settings::SceneSettings;
use user_interface::{Event, Headless, Input, Key, Modifiers};

fn sphere() -> Headless<MovingSphere> {
    Headless::new(MovingSphere::new(SceneSettings::default()))
}

#[test]
fn f1_hides_and_shows_the_card() {
    let mut app = sphere();
    assert!(app.program.text_shown);
    assert_eq!(app.label(info_card::NAME).as_deref(), Some("Minimal Scene Test"));
    app.key(Key::F1);
    assert!(!app.program.text_shown);
    assert!(!app.has_target(info_card::NAME));
    // With Ctrl too.
    app.key_with(Modifiers::CONTROL, Key::F1);
    assert!(app.program.text_shown);
    assert!(app.has_target(info_card::NAME));
}

#[test]
fn f11_switches_between_maximized_and_borderless_fullscreen() {
    let mut app = sphere();
    assert!(!app.fullscreen());
    app.key(Key::F11);
    assert!(app.fullscreen());
    assert_eq!(app.ui.size().height, 1440.0);
    // The card stays at the bottom left.
    assert_eq!(app.target(info_card::NAME).bottom(), 1440.0 - 16.0);
    app.key(Key::F11);
    assert!(!app.fullscreen());
    assert!(app.program.text_shown);
}

#[test]
fn holding_f1_down_does_not_show_and_hide_the_card_again() {
    let mut input = Input::default();
    input.begin(0.0, &[Event::FocusGained(Vec::new()), Event::KeyPressed { key: Key::F1, repeat: false }]);
    assert!(info_card::toggled(&input));
    // Windows repeats a held key.
    input.begin(0.5, &[Event::KeyPressed { key: Key::F1, repeat: true }]);
    assert!(!info_card::toggled(&input));

    let mut app = sphere();
    app.hold(Key::F1).wait(1.0).release_key(Key::F1);
    assert!(!app.program.text_shown);
}

#[test]
fn other_keys_do_nothing() {
    let mut app = sphere();
    // V doesn't turn on vertical sync, and Esc doesn't quit.
    for key in [Key::V, Key::S, Key::ESCAPE, Key::SPACE, Key::ENTER, Key::F2] {
        app.key(key);
        assert!(app.program.text_shown, "{}", key.name());
        assert!(!app.fullscreen(), "{}", key.name());
    }
}

#[test]
fn without_focus_the_keys_do_nothing() {
    let mut app = sphere();
    app.focus(false);
    app.key(Key::F1).key(Key::F11);
    assert!(app.program.text_shown);
    assert!(!app.fullscreen());
    // The sphere keeps moving.
    let position = app.program.motion.position;
    app.wait(0.25);
    assert!(app.program.motion.position > position);
    app.focus(true);
    app.key(Key::F1);
    assert!(!app.program.text_shown);
}
