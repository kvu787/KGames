// MinimalSceneTest's keys, through its interface without a window: F2 shows
// and hides the card, and holding it down doesn't do it again.

use minimal_scene_test::moving_sphere::MovingSphere;
use minimal_scene_test::settings::SceneSettings;
use user_interface::{Event, Headless, InfoCard, Input, Key, Modifiers};

fn sphere() -> Headless<MovingSphere> {
    Headless::new(MovingSphere::new(SceneSettings::default()))
}

#[test]
fn f2_hides_and_shows_the_card() {
    let mut app = sphere();
    assert!(app.program.text_shown);
    assert_eq!(app.label(InfoCard::NAME).as_deref(), Some("Minimal Scene Test"));
    app.key(Key::F2);
    assert!(!app.program.text_shown);
    assert!(!app.has_target(InfoCard::NAME));
    // With Ctrl too.
    app.key_with(Modifiers::CONTROL, Key::F2);
    assert!(app.program.text_shown);
    assert!(app.has_target(InfoCard::NAME));
}

#[test]
fn holding_f2_down_does_not_show_and_hide_the_card_again() {
    let mut input = Input::default();
    input.begin(0.0, &[Event::FocusGained(Vec::new()), Event::KeyPressed(Key::F2)]);
    assert!(InfoCard::toggled(&input));

    let mut app = sphere();
    app.hold(Key::F2).wait(1.0).release_key(Key::F2);
    assert!(!app.program.text_shown);
}

#[test]
fn other_keys_do_nothing() {
    let mut app = sphere();
    // V doesn't turn on vertical sync, and Esc doesn't quit.
    for key in [Key::V, Key::S, Key::ESCAPE, Key::SPACE, Key::ENTER, Key::F3] {
        app.key(key);
        assert!(app.program.text_shown, "{}", key.name());
        assert!(app.vertical_sync(), "{}", key.name());
    }
}

#[test]
fn without_focus_the_keys_do_nothing() {
    let mut app = sphere();
    app.focus(false);
    app.key(Key::F2).key(Key::F10);
    assert!(app.program.text_shown);
    assert!(app.vertical_sync());
    // The sphere keeps moving.
    let position = app.program.motion.position;
    app.wait(0.25);
    assert!(app.program.motion.position > position);
    app.focus(true);
    app.key(Key::F2);
    assert!(!app.program.text_shown);
}
