// The platformer through its frames, as the main loop runs them, with input
// that the tests make up, in the cards' real layout: the law is that a
// frame's input shows its whole effect on that frame's own view.

#include "Platformer/Game.h"
#include "Platformer/Level.h"
#include "Platformer/Physics.h"
#include "Platformer/Program.h"
#include "Testing/Testing.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Testing/Headless.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <filesystem>
#include <string_view>
#include <system_error>

using json::Value;
using platformer::game::Game;
using platformer::level::Zone;
using platformer::physics::Half;
using platformer::program::Platformer;
using user_interface::Headless;
using user_interface::Key;
using user_interface::Rect;
using user_interface::Startup;
using user_interface::Target;

namespace {

std::filesystem::path save_folder(std::string_view test) {
    return std::filesystem::temp_directory_path() / "PlatformerGameTests" / test;
}

// The game after its first frames, on the ground, saving to a fresh folder
// of the test's own.
Headless<Platformer> start(std::string_view test) {
    std::error_code ignored;
    std::filesystem::remove_all(save_folder(test), ignored);
    Headless<Platformer> app = Headless<Platformer>::start(Startup::for_tests("{}", save_folder(test)), [](Startup& startup) { return Platformer::start(startup); });
    app.frames(5);
    CHECK(app.program.game.player.grounded);
    return app;
}

Value state(const Headless<Platformer>& app, std::string_view member) {
    Value all = app.state();
    const Value* found = all.get(member);
    return found != nullptr ? *found : Value();
}

} // namespace

TEST(a_jump_press_rises_on_its_own_frame) {
    Headless<Platformer> app = start("Jump");
    double ground = app.program.game.player.position[1];
    app.hold(Key::Space);
    CHECK(app.program.game.player.position[1] > ground, "higher on the very frame of the press");
    CHECK_EQ(state(app, "Grounded"), Value(Value::Bool{false}));
}

// ESDF: S and F run, E jumps, each on its own frame, and A, D and W do
// nothing.
TEST(esdf_runs_and_jumps) {
    Headless<Platformer> app = start("Esdf");
    for (Key key : {Key::A, Key::D, Key::W}) {
        app.hold(key);
        CHECK_EQ(app.program.game.player.velocity[0], 0.0, "{} doesn't run", testing::describe(key));
        CHECK(app.program.game.player.grounded, "{} doesn't jump", testing::describe(key));
        app.release_key(key);
    }
    app.hold(Key::S);
    CHECK(app.program.game.player.velocity[0] < 0.0, "S runs left on its frame");
    app.release_key(Key::S).frames(60);
    app.hold(Key::F);
    CHECK(app.program.game.player.velocity[0] > 0.0, "F runs right on its frame");
    app.release_key(Key::F).frames(60);
    CHECK(app.program.game.player.grounded);
    app.hold(Key::E);
    CHECK(!app.program.game.player.grounded, "E jumps on its frame");
}

TEST(the_frame_that_collects_a_coin_already_counts_it) {
    Headless<Platformer> app = start("Coin");
    std::array<double, 2> coin = app.program.game.current().coins[0];
    app.program.game.player.position = {coin[0], coin[1] + Half[1]};
    app.program.game.player.velocity = {0.0, 0.0};
    app.frame();
    CHECK_EQ(state(app, "Coins"), Value(Value::Number{1.0}));
    Value events = state(app, "Events");
    CHECK(events.to_json().contains("CoinCollected 0"), "{}", events.to_json());
}

// The banner, the record and the saved file are all there on the frame that
// reaches the goal.
TEST(the_goal_frame_shows_the_banner_and_saves) {
    Headless<Platformer> app = start("Goal");
    Game& game = app.program.game;
    size_t total = game.total_coins();
    for (size_t coin = 0; coin < total; coin++) {
        game.collected[coin] = true;
    }
    game.coins = total;
    game.doors_open[0] = true;
    Zone goal = game.current().goal;
    game.player.position = {goal.x + 0.5, 0.55};
    game.player.velocity = {0.0, 0.0};
    CHECK(!app.has_target("Banner"));
    app.frame();
    CHECK_EQ(state(app, "State"), Value(Value::String{"Finished"}));
    CHECK(app.has_target("Banner"), "the banner is on the goal's own frame");
    CHECK_EQ(state(app, "Status"), Value(Value::String{"Saved Save.json"}));
    CHECK(std::filesystem::is_regular_file(save_folder("Goal") / "Save.json"), "the store was written in the frame");
    CHECK_EQ(state(app, "Unlocked"), Value(Value::Number{2.0}));
}

TEST(f1_hides_the_cards_on_its_frame) {
    Headless<Platformer> app = start("Cards");
    CHECK(app.has_target("Run") && app.has_target("Events"));
    app.key(Key::F1);
    CHECK(!app.has_target("Run") && !app.has_target("Events"));
    app.key(Key::F1);
    CHECK(app.has_target("Run"));
}

TEST(levels_switch_only_once_unlocked) {
    Headless<Platformer> app = start("Levels");
    app.key(Key::PageUp);
    CHECK_EQ(state(app, "Level"), Value(Value::String{"First Steps"}), "locked levels don't open");
    app.program.game.progress.unlocked = 2;
    app.key(Key::PageUp);
    CHECK_EQ(state(app, "Level"), Value(Value::String{"Ridgeline"}));
    CHECK_EQ(state(app, "Coins"), Value(Value::Number{0.0}), "a fresh run there");
    app.key(Key::PageDown);
    CHECK_EQ(state(app, "Level"), Value(Value::String{"First Steps"}));
}

TEST(r_restarts_the_run_on_its_frame) {
    Headless<Platformer> app = start("Restart");
    app.program.game.lives = 1;
    app.program.game.coins = 2;
    app.program.game.collected[0] = true;
    app.program.game.collected[1] = true;
    app.key(Key::R);
    CHECK_EQ(state(app, "Lives"), Value(Value::Number{3.0}));
    CHECK_EQ(state(app, "Coins"), Value(Value::Number{0.0}));
    CHECK_EQ(state(app, "State"), Value(Value::String{"Running"}));
}

TEST(without_focus_nothing_moves) {
    Headless<Platformer> app = start("Focus");
    std::array<double, 2> before = app.program.game.player.position;
    app.focus(false).hold(Key::Right).hold(Key::Space).frames(10);
    CHECK_EQ(app.program.game.player.position[0], before[0], "the keys went nowhere");
    CHECK(app.program.game.player.grounded);
}

TEST(the_run_card_fits_and_the_targets_stay_on_the_page) {
    for (bool fullscreen : {false, true}) {
        Headless<Platformer> app = start("Fits");
        app.set_fullscreen(fullscreen).frame();
        for (const Target& target : app.ui.targets()) {
            if (!target.listed) {
                continue;
            }
            Rect rect = target.rect;
            CHECK(rect.x >= 0.0f && rect.y >= 0.0f && rect.right() <= 2560.0f, "{} at {}", target.name, testing::describe(rect));
        }
    }
}
