// SpinningCarTest's keys, through its interface without a window: S keeps the
// car in the middle or lets it move, F1 shows and hides the card, F11
// switches between maximized and borderless fullscreen, and holding a key
// down doesn't do it again.

#include "SpinningCarTest/SpinningCarTest.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

#include <numbers>
#include <utility>

using spinning_car_test::controls::toggles_stationary;
using spinning_car_test::scene::car_placement;
using spinning_car_test::settings::SceneSettings;
using spinning_car_test::spinning_car::SpinningCar;
using user_interface::Event;
using user_interface::Headless;
using user_interface::InfoCard;
using user_interface::Input;
using user_interface::Key;
using user_interface::Modifiers;

namespace {

constexpr float FracPi2 = std::numbers::pi_v<float> / 2.0f;

Headless<SpinningCar> car() {
    return Headless<SpinningCar>(SpinningCar(SceneSettings{}));
}

} // namespace

TEST(s_keeps_the_car_in_the_middle_and_lets_it_move) {
    Headless<SpinningCar> app = car();
    app.key(Key::S);
    CHECK(app.program.stationary);
    CHECK_EQ(app.program.placement, std::pair(0.0f, FracPi2));
    app.wait(1.0);
    CHECK_EQ(app.program.placement, std::pair(0.0f, FracPi2));
    // It moves on from where the time puts it.
    app.key(Key::S);
    CHECK(!app.program.stationary);
    CHECK_EQ(app.program.placement, car_placement(app.time(), false));
}

TEST(f1_hides_and_shows_the_card) {
    Headless<SpinningCar> app = car();
    CHECK(app.program.text_shown);
    CHECK_EQ(app.label(InfoCard::Name), "Spinning Car Test");
    app.key(Key::F1);
    CHECK(!app.program.text_shown);
    CHECK(!app.has_target(InfoCard::Name));
    // With Ctrl too.
    app.key_with(Modifiers::Control, Key::F1);
    CHECK(app.program.text_shown);
    CHECK(app.has_target(InfoCard::Name));
}

TEST(f11_switches_between_maximized_and_borderless_fullscreen) {
    Headless<SpinningCar> app = car();
    CHECK(!app.fullscreen());
    app.key(Key::F11);
    CHECK(app.fullscreen());
    CHECK_EQ(app.ui.size().height, 1440.0f);
    // The card stays at the bottom left.
    CHECK_EQ(app.target(InfoCard::Name).bottom(), 1440.0f - 16.0f);
    app.key(Key::F11);
    CHECK(!app.fullscreen());
    CHECK(app.program.text_shown);
    CHECK(!app.program.stationary);
}

TEST(holding_a_key_down_does_not_do_it_again) {
    Input input;
    input.begin(0.0, {{
                         Event::FocusGained{},
                         Event::KeyPressed{.key = Key::S, .repeat = false},
                         Event::KeyPressed{.key = Key::F1, .repeat = false},
                     }});
    CHECK(toggles_stationary(input));
    CHECK(InfoCard::toggled(input));
    // Windows repeats a held key.
    input.begin(0.5, {{Event::KeyPressed{.key = Key::S, .repeat = true}, Event::KeyPressed{.key = Key::F1, .repeat = true}}});
    CHECK(!toggles_stationary(input));
    CHECK(!InfoCard::toggled(input));

    Headless<SpinningCar> app = car();
    app.hold(Key::S).wait(1.0).release_key(Key::S);
    CHECK(app.program.stationary);
}

TEST(other_keys_do_nothing) {
    Headless<SpinningCar> app = car();
    // V doesn't turn on vertical sync, and Esc doesn't quit.
    for (Key key : {Key::V, Key::Escape, Key::Space, Key::Enter, Key::F2, Key::D}) {
        app.key(key);
        CHECK(app.program.text_shown, "{}", key.name());
        CHECK(!app.program.stationary, "{}", key.name());
        CHECK(!app.fullscreen(), "{}", key.name());
    }
}

TEST(without_focus_the_keys_do_nothing) {
    Headless<SpinningCar> app = car();
    app.focus(false);
    app.key(Key::S).key(Key::F1).key(Key::F11);
    CHECK(!app.program.stationary);
    CHECK(app.program.text_shown);
    CHECK(!app.fullscreen());
    // The car keeps moving.
    CHECK_EQ(app.program.placement, car_placement(app.time(), false));
    app.focus(true);
    app.key(Key::S);
    CHECK(app.program.stationary);
}
