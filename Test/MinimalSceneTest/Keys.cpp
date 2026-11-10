// MinimalSceneTest's keys, through its interface without a window: F1 shows
// and hides the card, F11 switches between maximized and borderless
// fullscreen, and holding a key down doesn't do it again.

#include "MinimalSceneTest/MinimalSceneTest.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

using minimal_scene_test::moving_sphere::MovingSphere;
using minimal_scene_test::settings::SceneSettings;
using user_interface::Event;
using user_interface::Headless;
using user_interface::InfoCard;
using user_interface::Input;
using user_interface::Key;
using user_interface::Modifiers;

namespace {

Headless<MovingSphere> sphere() {
    return Headless<MovingSphere>(MovingSphere(SceneSettings{}));
}

} // namespace

TEST(f1_hides_and_shows_the_card) {
    Headless<MovingSphere> app = sphere();
    CHECK(app.program.text_shown);
    CHECK_EQ(app.label(InfoCard::Name), "Minimal Scene Test");
    app.key(Key::F1);
    CHECK(!app.program.text_shown);
    CHECK(!app.has_target(InfoCard::Name));
    // With Ctrl too.
    app.key_with(Modifiers::Control, Key::F1);
    CHECK(app.program.text_shown);
    CHECK(app.has_target(InfoCard::Name));
}

TEST(f11_switches_between_maximized_and_borderless_fullscreen) {
    Headless<MovingSphere> app = sphere();
    CHECK(!app.fullscreen());
    app.key(Key::F11);
    CHECK(app.fullscreen());
    CHECK_EQ(app.ui.size().height, 1440.0f);
    // The card stays at the bottom left.
    CHECK_EQ(app.target(InfoCard::Name).bottom(), 1440.0f - 16.0f);
    app.key(Key::F11);
    CHECK(!app.fullscreen());
    CHECK(app.program.text_shown);
}

TEST(holding_f1_down_does_not_show_and_hide_the_card_again) {
    Input input;
    input.begin(0.0, {{Event::FocusGained{}, Event::KeyPressed{.key = Key::F1, .repeat = false}}});
    CHECK(InfoCard::toggled(input));
    // Windows repeats a held key.
    input.begin(0.5, {{Event::KeyPressed{.key = Key::F1, .repeat = true}}});
    CHECK(!InfoCard::toggled(input));

    Headless<MovingSphere> app = sphere();
    app.hold(Key::F1).wait(1.0).release_key(Key::F1);
    CHECK(!app.program.text_shown);
}

TEST(other_keys_do_nothing) {
    Headless<MovingSphere> app = sphere();
    // V doesn't turn on vertical sync, and Esc doesn't quit.
    for (Key key : {Key::V, Key::S, Key::Escape, Key::Space, Key::Enter, Key::F2}) {
        app.key(key);
        CHECK(app.program.text_shown, "{}", key.name());
        CHECK(!app.fullscreen(), "{}", key.name());
    }
}

TEST(without_focus_the_keys_do_nothing) {
    Headless<MovingSphere> app = sphere();
    app.focus(false);
    app.key(Key::F1).key(Key::F11);
    CHECK(app.program.text_shown);
    CHECK(!app.fullscreen());
    // The sphere keeps moving.
    float position = app.program.motion.position;
    app.wait(0.25);
    CHECK(app.program.motion.position > position);
    app.focus(true);
    app.key(Key::F1);
    CHECK(!app.program.text_shown);
}
