// SpinningCarTest's keys, through its interface without a window: S keeps the
// car in the middle or lets it move, F1 shows and hides the card, and holding
// a key down doesn't do it again.

use std::f32::consts::FRAC_PI_2;

use spinning_car_test::controls::toggles_stationary;
use spinning_car_test::scene::car_placement;
use spinning_car_test::settings::SceneSettings;
use spinning_car_test::spinning_car::SpinningCar;
use user_interface::{Event, Headless, InfoCard, Input, Key, Modifiers};

fn car() -> Headless<SpinningCar> {
    Headless::new(SpinningCar::new(SceneSettings::default()))
}

#[test]
fn s_keeps_the_car_in_the_middle_and_lets_it_move() {
    let mut app = car();
    app.key(Key::S);
    assert!(app.program.stationary);
    assert_eq!(app.program.placement, (0.0, FRAC_PI_2));
    app.wait(1.0);
    assert_eq!(app.program.placement, (0.0, FRAC_PI_2));
    // It moves on from where the time puts it.
    app.key(Key::S);
    assert!(!app.program.stationary);
    assert_eq!(app.program.placement, car_placement(app.time(), false));
}

#[test]
fn f1_hides_and_shows_the_card() {
    let mut app = car();
    assert!(app.program.text_shown);
    assert_eq!(app.label(InfoCard::NAME).as_deref(), Some("Spinning Car Test"));
    app.key(Key::F1);
    assert!(!app.program.text_shown);
    assert!(!app.has_target(InfoCard::NAME));
    // With Ctrl too.
    app.key_with(Modifiers::CONTROL, Key::F1);
    assert!(app.program.text_shown);
    assert!(app.has_target(InfoCard::NAME));
}

// A held key doesn't repeat, so holding S down toggles once.
#[test]
fn holding_a_key_down_does_not_do_it_again() {
    let mut input = Input::default();
    input.begin(0.0, &[Event::FocusGained(Vec::new()), Event::KeyPressed(Key::S), Event::KeyPressed(Key::F1)]);
    assert!(toggles_stationary(&input));
    assert!(InfoCard::toggled(&input));

    let mut app = car();
    app.hold(Key::S).wait(1.0).release_key(Key::S);
    assert!(app.program.stationary);
}

#[test]
fn other_keys_do_nothing() {
    let mut app = car();
    // V doesn't turn on vertical sync, and Esc doesn't quit.
    for key in [Key::V, Key::ESCAPE, Key::SPACE, Key::ENTER, Key::F2, Key::D] {
        app.key(key);
        assert!(app.program.text_shown, "{}", key.name());
        assert!(!app.program.stationary, "{}", key.name());
        assert!(app.vertical_sync(), "{}", key.name());
    }
}

#[test]
fn without_focus_the_keys_do_nothing() {
    let mut app = car();
    app.focus(false);
    app.key(Key::S).key(Key::F1).key(Key::F10);
    assert!(!app.program.stationary);
    assert!(app.program.text_shown);
    assert!(app.vertical_sync());
    // The car keeps moving.
    assert_eq!(app.program.placement, car_placement(app.time(), false));
    app.focus(true);
    app.key(Key::S);
    assert!(app.program.stationary);
}
