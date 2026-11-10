// The movement, in fixed steps: jumping only from the ground, the buffered
// jump firing on the very step of the landing, coyote time, the jump cut,
// the spring, the moving platform that carries the player, and every top in
// every level holding a player who runs along it.

#include "Platformer/Physics.h"
#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Platformer/Controls.h"
#include "Platformer/Game.h"
#include "Platformer/Level.h"
#include "Platformer/Progress.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <filesystem>
#include <format>
#include <limits>
#include <string>
#include <string_view>
#include <system_error>
#include <utility>
#include <vector>

using platformer::controls::Controls;
using platformer::game::Game;
using platformer::level::Level;
using platformer::level::Patrol;
using platformer::level::Zone;
using platformer::physics::Buffer;
using platformer::physics::Coyote;
using platformer::physics::Half;
using platformer::physics::JumpSpeed;
using platformer::physics::Player;
using platformer::physics::RunSpeed;
using platformer::physics::Solid;
using platformer::physics::Standing;
using platformer::physics::Step;
using platformer::progress::Progress;

namespace {

constexpr double Frame = 1.0 / 60.0;

Game new_game(std::string_view test) {
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "PlatformerPhysicsTests" / test;
    std::error_code ignored;
    std::filesystem::remove_all(folder, ignored);
    Game game(platformer::level::levels(), Progress(2), folder / "Save.json");
    for (int number = 1; number <= 5; number++) {
        game.update(Controls{}, static_cast<double>(number) * Frame);
    }
    CHECK(game.player.grounded);
    return game;
}

void frame(Game& game, Controls controls) {
    double next = game.simulated + Frame;
    game.update(controls, next);
}

// Running left, toward the first ground's open left edge.
constexpr Controls RunLeft{
    .push = -1.0,
    .jump_pressed = false,
    .jump_held = false,
    .restart = false,
    .level = 0,
    .toggle_cards = false,
};

} // namespace

TEST(a_jump_rises_on_the_frame_of_its_press) {
    Game game = new_game("Jump");
    double ground = game.player.position[1];
    frame(game, Controls{.jump_pressed = true, .jump_held = true});
    CHECK(game.player.velocity[1] > 0.0);
    CHECK(game.player.position[1] > ground, "already higher on the press's frame");
    CHECK(!game.player.grounded);
}

TEST(a_jump_needs_the_ground_or_a_fresh_ledge) {
    Game game = new_game("Air");
    // Jump once, then press again mid-air: nothing.
    frame(game, Controls{.jump_pressed = true, .jump_held = true});
    double rising = game.player.velocity[1];
    CHECK(rising > 0.0);
    frame(game, Controls{.jump_pressed = true, .jump_held = true});
    CHECK(game.player.velocity[1] < rising, "the second press did nothing but fall along");
}

// A press just before the landing jumps on the very step of the landing.
TEST(a_buffered_jump_fires_on_the_landing_step) {
    Game game = new_game("Buffer");
    // A short hop without holding, so the fall starts soon.
    frame(game, Controls{.jump_pressed = true});
    // Fall until just above the ground, then press while still in the air.
    while (game.player.velocity[1] > -3.0) {
        frame(game, Controls{});
    }
    CHECK(!game.player.grounded);
    game.player.jump_until = game.simulated + Buffer;
    // Within the buffer the landing itself relaunches: watch for a frame
    // where the player is airborne RISING without a press this frame.
    bool relaunched = false;
    for (int count = 0; count < 30; count++) {
        frame(game, Controls{});
        if (game.player.velocity[1] > JumpSpeed - 2.0) {
            relaunched = true;
            break;
        }
    }
    CHECK(relaunched, "the buffered press jumped on the landing, {}", testing::describe(game.player));
}

TEST(coyote_time_jumps_after_the_ledge_but_not_much_after) {
    {
        Game game = new_game("Coyote");
        // Walk off the ground's open left edge.
        while (game.player.grounded) {
            frame(game, RunLeft);
        }
        // Just off the ledge: a press inside the coyote window still jumps.
        CHECK(game.simulated <= game.player.coyote_until);
        frame(game, Controls{.push = -1.0, .jump_pressed = true, .jump_held = true});
        CHECK(game.player.velocity[1] > 0.0, "the coyote jump fired, {}", testing::describe(game.player));
    }

    // Do it again, but wait the window out: the press does nothing.
    Game game = new_game("CoyoteLate");
    while (game.player.grounded) {
        frame(game, RunLeft);
    }
    size_t frames_past = base::saturating_cast<size_t>(std::ceil(Coyote / Frame)) + 1;
    for (size_t count = 0; count < frames_past; count++) {
        frame(game, Controls{});
    }
    frame(game, Controls{.jump_pressed = true, .jump_held = true});
    CHECK(game.player.velocity[1] < 0.0, "too late: still falling, {}", testing::describe(game.player));
}

TEST(letting_go_of_jump_cuts_the_rise) {
    double full = [] {
        Game game = new_game("FullJump");
        double top = game.player.position[1];
        frame(game, Controls{.jump_pressed = true, .jump_held = true});
        for (int count = 0; count < 60; count++) {
            frame(game, Controls{.jump_held = true});
            top = std::max(top, game.player.position[1]);
        }
        return top;
    }();
    double cut = [] {
        Game game = new_game("CutJump");
        double top = game.player.position[1];
        frame(game, Controls{.jump_pressed = true, .jump_held = true});
        // Let go right away.
        for (int count = 0; count < 60; count++) {
            frame(game, Controls{});
            top = std::max(top, game.player.position[1]);
        }
        return top;
    }();
    CHECK(cut < full - 0.5, "a tap hops lower: {} against {}", base::display(cut), base::display(full));
}

TEST(a_moving_platform_carries_the_player) {
    Game game = new_game("Carry");
    // Stand on level 1's patrol, which moves along X.
    Patrol patrol = game.current().patrols[0];
    Zone zone = patrol.at(game.simulated);
    game.player.position = {zone.center()[0], zone.top() + 0.56};
    game.player.velocity = {0.0, 0.0};
    // Land on it.
    for (int count = 0; count < 10; count++) {
        frame(game, Controls{});
        if (game.player.grounded) {
            break;
        }
    }
    CHECK(game.player.grounded, "{}", testing::describe(game.player));
    double before = game.player.position[0];
    double platform_before = patrol.at(game.simulated).x;
    for (int count = 0; count < 30; count++) {
        frame(game, Controls{});
    }
    double platform_moved = patrol.at(game.simulated).x - platform_before;
    double player_moved = game.player.position[0] - before;
    CHECK(std::abs(platform_moved) > 0.3, "the platform moved, {}", base::display(platform_moved));
    CHECK(std::abs(player_moved - platform_moved) < 0.05, "the player rode it: {} against {}", base::display(player_moved), base::display(platform_moved));
}

TEST(walls_stop_the_run_where_they_stand) {
    Game game = new_game("Wall");
    // Run right into the closed door.
    Zone door = game.current().doors[0];
    game.player.position = {door.x - 1.0, 0.55};
    game.player.velocity = {0.0, 0.0};
    Controls run{.push = 1.0};
    for (int count = 0; count < 30; count++) {
        frame(game, run);
    }
    CHECK(std::abs(game.player.position[0] - (door.x - 0.35)) < 1e-9, "stopped at the door's face, {}", testing::describe(game.player.position));
}

// Every top in every level, the doors' and the patrols' at each step of
// their way included, holds a player who stands on it and runs along it,
// both ways. The player's box is stored as its center, and a center resting
// on a top doesn't always round back to it: 2.0 + 0.55 - 0.55 is
// 1.9999999999999998. That hair of overlap once made the run's X pass take
// the platform under the player for a wall, and push them back to its edge
// and off it: an invisible wall on First Steps' platform over the first pit
// and on Ridgeline's checkpoint block, the two tops at 2.0.
TEST(every_top_holds_a_runner_both_ways) {
    std::vector<std::pair<std::string, Zone>> tops;
    for (const Level& level : platformer::level::levels()) {
        for (size_t index = 0; index < level.solids.size(); index++) {
            tops.push_back({std::format("{} solid {}", level.name, index), level.solids[index]});
        }
        for (size_t index = 0; index < level.doors.size(); index++) {
            tops.push_back({std::format("{} door {}", level.name, index), level.doors[index]});
        }
        for (size_t index = 0; index < level.patrols.size(); index++) {
            const Patrol& patrol = level.patrols[index];
            size_t steps = base::saturating_cast<size_t>(std::round(patrol.period / Step));
            for (size_t step = 0; step < steps; step++) {
                double seconds = static_cast<double>(step) * Step;
                tops.push_back({std::format("{} patrol {} at {} s", level.name, index, base::display(seconds)), patrol.at(seconds)});
            }
        }
    }
    // One 60 Hz frame's worth of steps.
    constexpr size_t Steps = 4;
    for (const auto& [name, zone] : tops) {
        std::array<Solid, 1> solids = {Solid(zone, Standing::Still{})};
        for (double push : {1.0, -1.0}) {
            // Where a landing puts the player: the top plus the half height.
            Player player = Player::at({zone.center()[0], zone.top()});
            std::array<double, 2> start = player.position;
            for (size_t step = 1; step <= Steps; step++) {
                platformer::physics::step(player, static_cast<double>(step) * Step, push, false, {0.0, 0.0}, solids);
            }
            double ran = push * RunSpeed * Step * static_cast<double>(Steps);
            CHECK(player.grounded, "{}, running {}: still on it, {}", name, base::display(push), testing::describe(player));
            CHECK(std::abs(player.position[0] - (start[0] + ran)) < 1e-9 && player.position[1] == start[1], "{}, running {}: ran {} along the top from {}, {}", name,
                  base::display(push), base::display(ran), testing::describe(start), testing::describe(player));
        }
    }
}

// The way the bug was found: jump from the first ground onto First Steps'
// platform over the pit, holding right, and keep running along it.
TEST(a_runner_lands_on_the_platform_over_the_pit_and_runs_along_it) {
    Game game = new_game("PitPlatform");
    Zone platform = game.current().solids[4];
    CHECK_EQ(platform, Zone(15.5, 1.5, 3.0, 0.5), "the platform over the first pit");
    game.player.position = {12.0, Half[1]};
    game.player.velocity = {0.0, 0.0};
    frame(game, Controls{.push = 1.0, .jump_pressed = true, .jump_held = true});
    bool landed = false;
    double farthest = -std::numeric_limits<double>::infinity();
    for (int count = 0; count < 60; count++) {
        frame(game, Controls{.push = 1.0, .jump_held = true});
        bool on_top = game.player.grounded && game.player.position[1] == platform.top() + Half[1];
        if (on_top) {
            landed = true;
            farthest = std::max(farthest, game.player.position[0]);
        } else if (landed) {
            break;
        }
    }
    CHECK(landed, "landed on the platform, {}", testing::describe(game.player));
    CHECK(farthest > platform.right(), "ran to its far end, not stopped on it: {}", base::display(farthest));
}

// A patrol carries the player by how far it moved, which is rounded too.
// Ride Ridgeline's lift, which rises and sinks through heights that round
// every which way, for a whole period, running a frame each way in turn:
// the player stays on it, never pushed off it as if it were a wall.
TEST(a_runner_rides_the_lift_through_every_height) {
    Game game = new_game("Lift");
    game.level = 1;
    game.restart();
    Patrol patrol = game.current().patrols[0];
    CHECK(patrol.to[1] != patrol.zone.y && patrol.to[0] == patrol.zone.x, "Ridgeline's first patrol goes up and down");
    Zone start = patrol.at(game.simulated);
    game.player.position = {start.center()[0], start.top() + Half[1]};
    game.player.velocity = {0.0, 0.0};
    frame(game, Controls{});
    CHECK(game.player.grounded, "{}", testing::describe(game.player));
    size_t frames = base::saturating_cast<size_t>(std::round(patrol.period / Frame));
    for (size_t index = 0; index < frames; index++) {
        double push = index % 2 == 0 ? 1.0 : -1.0;
        frame(game, Controls{.push = push});
        Zone zone = patrol.at(game.simulated);
        CHECK(game.player.standing == Standing(Standing::Patrol{0}) && std::abs(game.player.position[0] - zone.center()[0]) < 0.2, "frame {}: still riding the lift at {}, {}",
              index, testing::describe(zone), testing::describe(game.player));
    }
}

TEST(the_patrol_is_a_pure_function_of_time) {
    Patrol patrol = platformer::level::levels()[0].patrols[0];
    Zone there = patrol.at(1.0);
    Zone again = patrol.at(1.0 + patrol.period * 7.0);
    CHECK(std::abs(there.x - again.x) < 1e-9 && std::abs(there.y - again.y) < 1e-9);
    // The wave reaches both ends.
    Zone start = patrol.at(0.0);
    Zone far_ = patrol.at(patrol.period / 2.0);
    CHECK(std::abs(start.x - patrol.zone.x) < 1e-9);
    CHECK(std::abs(far_.x - patrol.to[0]) < 1e-9);
}

// One fixed step at a time: Step never varies, so a frame's work is a whole
// number of steps and the clock never drifts from the step count.
TEST(the_clock_moves_in_whole_steps) {
    Game game = new_game("Clock");
    double before = game.simulated;
    frame(game, Controls{});
    double steps = (game.simulated - before) / Step;
    CHECK(std::abs(steps - std::round(steps)) < 1e-9, "{}", base::display(steps));
}
