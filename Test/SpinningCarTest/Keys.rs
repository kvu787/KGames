// SpinningCarTest's keys, through its interface without a window: S keeps the
// car in the middle or lets it move, F1 shows and hides the card, F11
// switches between maximized and borderless fullscreen, and holding a key
// down doesn't do it again.

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

#[test]
fn f11_switches_between_maximized_and_borderless_fullscreen() {
    let mut app = car();
    assert!(!app.fullscreen());
    app.key(Key::F11);
    assert!(app.fullscreen());
    assert_eq!(app.ui.size().height, 1440.0);
    // The card stays at the bottom left.
    assert_eq!(app.target(InfoCard::NAME).bottom(), 1440.0 - 16.0);
    app.key(Key::F11);
    assert!(!app.fullscreen());
    assert!(app.program.text_shown);
    assert!(!app.program.stationary);
}

#[test]
fn holding_a_key_down_does_not_do_it_again() {
    let mut input = Input::default();
    input.begin(
        0.0,
        &[
            Event::FocusGained(Vec::new()),
            Event::KeyPressed { key: Key::S, repeat: false },
            Event::KeyPressed { key: Key::F1, repeat: false },
        ],
    );
    assert!(toggles_stationary(&input));
    assert!(InfoCard::toggled(&input));
    // Windows repeats a held key.
    input.begin(
        0.5,
        &[Event::KeyPressed { key: Key::S, repeat: true }, Event::KeyPressed { key: Key::F1, repeat: true }],
    );
    assert!(!toggles_stationary(&input));
    assert!(!InfoCard::toggled(&input));

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
        assert!(!app.fullscreen(), "{}", key.name());
    }
}

#[test]
fn without_focus_the_keys_do_nothing() {
    let mut app = car();
    app.focus(false);
    app.key(Key::S).key(Key::F1).key(Key::F11);
    assert!(!app.program.stationary);
    assert!(app.program.text_shown);
    assert!(!app.fullscreen());
    // The car keeps moving.
    assert_eq!(app.program.placement, car_placement(app.time(), false));
    app.focus(true);
    app.key(Key::S);
    assert!(app.program.stationary);
}
