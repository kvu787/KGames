// The event graph, against the two laws: a frame's input produces the
// correct state by the end of that frame, and every event's whole graph of
// consequences - however deep - resolves inside the frame that published it.
// Each test runs ONE Game::update and asserts everything it should have
// caused, the file on disk included.

#include "Base/Files.h"
#include "Base/Sum.h"
#include "Platformer/Controls.h"
#include "Platformer/Game.h"
#include "Platformer/Level.h"
#include "Platformer/Physics.h"
#include "Platformer/Progress.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <format>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <system_error>
#include <tuple>
#include <utility>
#include <vector>

using platformer::controls::Controls;
using platformer::game::EffectKind;
using platformer::game::Game;
using platformer::game::Lives;
using platformer::game::RunState;
using platformer::level::Level;
using platformer::level::Zone;
using platformer::physics::Half;
using platformer::progress::Progress;

namespace {

// A frame at 60 Hz.
constexpr double Frame = 1.0 / 60.0;

std::filesystem::path save_folder(std::string_view test) {
    return std::filesystem::temp_directory_path() / "PlatformerEventTests" / test;
}

void remove_dir_all(const std::filesystem::path& folder) {
    std::error_code ignored;
    std::filesystem::remove_all(folder, ignored);
}

// A game on level 1, saving to a fresh folder of the test's own, settled on
// the ground after a few frames.
Game new_game(std::string_view test) {
    remove_dir_all(save_folder(test));
    std::vector<Level> levels = platformer::level::levels();
    Progress progress(levels.size());
    Game game(std::move(levels), std::move(progress), save_folder(test) / "Save.json");
    for (int number = 1; number <= 5; number++) {
        game.update(Controls{}, static_cast<double>(number) * Frame);
    }
    CHECK(game.player.grounded);
    return game;
}

// One more frame, with these controls.
void frame(Game& game, Controls controls) {
    double next = game.simulated + Frame;
    game.update(controls, next);
}

// Whether the log, newest first, holds lines starting with the labels, in
// order, oldest first.
bool logged(const Game& game, std::span<const std::string_view> labels) {
    std::vector<std::string_view> log;
    for (size_t index = game.log.size(); index > 0; index--) {
        log.push_back(game.log[index - 1].second);
    }
    std::span<const std::string_view> rest = log;
    for (std::string_view label : labels) {
        auto found = std::ranges::find_if(rest, [&](std::string_view line) { return line.starts_with(label); });
        if (found == rest.end()) {
            return false;
        }
        rest = rest.subspan(static_cast<size_t>(found - rest.begin()) + 1);
    }
    return true;
}

size_t count(const Game& game, std::string_view label) {
    size_t lines = 0;
    for (const std::pair<double, std::string>& entry : game.log) {
        if (entry.second == label) {
            lines += 1;
        }
    }
    return lines;
}

// Puts the player's feet at a point, standing still.
void place(Game& game, std::array<double, 2> feet) {
    game.player.position = {feet[0], feet[1] + Half[1]};
    game.player.velocity = {0.0, 0.0};
}

// Collects every coin but the last by hand, as if the run had.
void collect_all_but_last(Game& game) {
    size_t total = game.total_coins();
    for (size_t coin = 0; coin < total - 1; coin++) {
        game.collected[coin] = true;
    }
    game.coins = total - 1;
}

} // namespace

TEST(collecting_the_last_coin_opens_the_door_in_the_same_frame) {
    Game game = new_game("LastCoin");
    collect_all_but_last(game);
    CHECK(!game.doors_open[0]);
    std::array<double, 2> coin = game.current().coins.back();
    place(game, {coin[0] - 0.2, 0.0});
    frame(game, Controls{});
    CHECK_EQ(game.coins, game.total_coins());
    CHECK(game.doors_open[0], "the chain coin -> all coins -> door open ran inside the frame");
    CHECK(logged(game, {{"CoinCollected 4", "AllCoinsCollected", "DoorOpened 0"}}), "{}", testing::describe(game.log));
}

// The deepest chain: the frame's input collects the last coin, which opens
// the door, which lets the same frame's movement reach the goal, which ends
// the run, sets the record, unlocks the next level, saves the file, and
// flashes the save light - all before this frame's view.
TEST(the_last_coin_and_the_goal_resolve_in_one_frame) {
    Game game = new_game("OneFrame");
    collect_all_but_last(game);
    place(game, {57.6, 0.0});
    Controls run{
        .push = 1.0,
    };
    frame(game, run);

    // The whole graph, in the one frame.
    CHECK_EQ(game.coins, game.total_coins());
    CHECK(game.doors_open[0]);
    CHECK(base::is<RunState::Finished>(game.state_));
    CHECK(game.progress.best[0].has_value(), "the record was set");
    CHECK_EQ(game.progress.unlocked, 2, "the next level was unlocked");
    CHECK(logged(game, {{"CoinCollected 4", "AllCoinsCollected", "DoorOpened 0", "GoalReached", "RecordSet", "ProgressSaved Save.json"}}), "{}", testing::describe(game.log));
    // The persistent store was really written, inside the frame.
    std::expected<std::string, std::string> saved = base::read_to_string(save_folder("OneFrame") / "Save.json");
    remove_dir_all(save_folder("OneFrame"));
    CHECK(saved.has_value(), "Save.json was written in the goal's frame");
    const std::string& text = *saved;
    CHECK(text.contains("\"Unlocked\": 2"), "{}", text);
    // And the frame's effects are already showing.
    std::vector<EffectKind> kinds;
    for (const platformer::game::Effect& effect : game.effects) {
        kinds.push_back(effect.kind);
    }
    CHECK(std::ranges::contains(kinds, EffectKind::Sparkle), "{}", testing::describe(kinds));
    CHECK(std::ranges::contains(kinds, EffectKind::Burst), "{}", testing::describe(kinds));
    CHECK(std::ranges::contains(kinds, EffectKind::Confetti), "{}", testing::describe(kinds));
    CHECK(std::ranges::contains(kinds, EffectKind::SaveFlash), "{}", testing::describe(kinds));
    // RecordSet's label carries the seconds, so match it loosely.
    CHECK(std::ranges::any_of(game.log, [](const std::pair<double, std::string>& entry) { return entry.second.starts_with("RecordSet "); }), "{}", testing::describe(game.log));
}

// Two consumers mark the progress dirty in one cascade - the record and the
// unlock - but the cascade saves once: the second SaveProgress finds the
// store clean.
TEST(one_cascade_saves_once) {
    Game game = new_game("SaveOnce");
    collect_all_but_last(game);
    place(game, {57.6, 0.0});
    frame(game, Controls{.push = 1.0});
    CHECK(base::is<RunState::Finished>(game.state_));
    CHECK_EQ(count(game, "SaveProgress"), 2, "the record and the unlock each asked, {}", testing::describe(game.log));
    CHECK_EQ(count(game, "ProgressSaved Save.json"), 1, "but one write served both, {}", testing::describe(game.log));
    remove_dir_all(save_folder("SaveOnce"));
}

// A run that doesn't beat the best sets no record and writes nothing.
TEST(a_slower_run_sets_no_record_and_saves_nothing) {
    Game game = new_game("Slower");
    game.progress.best[0] = 0.001;
    game.progress.unlocked = 2;
    collect_all_but_last(game);
    place(game, {57.6, 0.0});
    frame(game, Controls{.push = 1.0});
    CHECK(base::is<RunState::Finished>(game.state_));
    CHECK_EQ(game.progress.best[0], std::optional<double>(0.001));
    CHECK_EQ(count(game, "ProgressSaved Save.json"), 0, "{}", testing::describe(game.log));
    CHECK(!std::filesystem::exists(save_folder("Slower") / "Save.json"));
}

TEST(death_and_respawn_resolve_in_the_frame) {
    Game game = new_game("Death");
    // The checkpoint is reached, so the respawn goes there.
    game.reached[0] = true;
    game.checkpoint = size_t{0};
    Zone spikes = game.current().spikes[0];
    place(game, {spikes.center()[0], spikes.y + 0.1});
    frame(game, Controls{});
    CHECK_EQ(game.lives, Lives - 1);
    Zone checkpoint = game.current().checkpoints[0];
    CHECK(std::abs(game.player.position[0] - checkpoint.center()[0]) < 1e-9, "respawned at the checkpoint, in the frame");
    CHECK(logged(game, {{"Died Spikes", "Respawned"}}), "{}", testing::describe(game.log));
    CHECK_EQ(game.state_, RunState(RunState::Running{}));
}

TEST(the_last_death_fails_the_run_in_the_frame) {
    Game game = new_game("OutOfLives");
    game.lives = 1;
    Zone spikes = game.current().spikes[0];
    place(game, {spikes.center()[0], spikes.y + 0.1});
    frame(game, Controls{});
    CHECK_EQ(game.lives, 0);
    CHECK_EQ(game.state_, RunState(RunState::Failed{}));
    CHECK(logged(game, {{"Died Spikes", "OutOfLives"}}), "{}", testing::describe(game.log));
    // Failed means frozen: more frames change nothing but the clock.
    std::array<double, 2> position = game.player.position;
    frame(game, Controls{.push = 1.0});
    CHECK_EQ(game.player.position, position);
}

TEST(falling_out_of_the_level_is_a_death) {
    Game game = new_game("Fell");
    place(game, {16.5, -8.0});
    frame(game, Controls{});
    CHECK_EQ(game.lives, Lives - 1);
    CHECK(logged(game, {{"Died Fell", "Respawned"}}), "{}", testing::describe(game.log));
    // No checkpoint yet, so the respawn is the spawn.
    CHECK(std::abs(game.player.position[0] - game.current().spawn[0]) < 1e-9);
}

TEST(a_checkpoint_counts_once) {
    Game game = new_game("Checkpoint");
    Zone checkpoint = game.current().checkpoints[0];
    place(game, {checkpoint.center()[0], checkpoint.y});
    frame(game, Controls{});
    frame(game, Controls{});
    CHECK_EQ(count(game, "CheckpointReached 0"), 1, "{}", testing::describe(game.log));
    CHECK_EQ(game.checkpoint, std::optional<size_t>(0));
}

TEST(a_spring_bounces_in_its_frame) {
    Game game = new_game("Spring");
    Zone spring = game.current().springs[0];
    // Falling onto the spring, a step above it.
    game.player.position = {spring.center()[0], spring.top() + 0.57};
    game.player.velocity = {0.0, -6.0};
    frame(game, Controls{});
    CHECK(game.player.velocity[1] > 0.0, "bounced up in the frame, {}", testing::describe(game.player));
    CHECK_EQ(count(game, "SpringBounced 0"), 1, "{}", testing::describe(game.log));
}

TEST(restarting_resets_the_run_and_keeps_the_progress) {
    Game game = new_game("Restart");
    game.progress.best[0] = 9.0;
    game.collected[0] = true;
    game.coins = 1;
    game.lives = 1;
    frame(game, Controls{.restart = true});
    CHECK_EQ(game.coins, 0);
    CHECK_EQ(game.lives, Lives);
    CHECK_EQ(game.state_, RunState(RunState::Running{}));
    CHECK_EQ(game.progress.best[0], std::optional<double>(9.0));
    CHECK(logged(game, {{"RunStarted"}}), "{}", testing::describe(game.log));
}

// The whole simulation, events included, is the same at 60 and 240 frames a
// second: same positions, same log, same times on the log's clock.
TEST(the_game_runs_the_same_at_any_frame_rate) {
    auto run = [](std::uint32_t rate) {
        std::filesystem::path folder = save_folder(std::format("Rate{}", rate));
        remove_dir_all(folder);
        std::vector<Level> levels = platformer::level::levels();
        Game game(std::move(levels), Progress(2), folder / "Save.json");
        Controls controls{
            .push = 1.0,
            .jump_held = true,
        };
        std::uint32_t frames = 5 * rate;
        for (std::uint32_t number = 1; number <= frames; number++) {
            // A jump pressed at the same wall times, a push all along: runs
            // off the first ground, jumps, collects the first coin.
            bool pressed = number == rate / 2 || number == 2 * rate;
            Controls with_press = controls;
            with_press.jump_pressed = pressed;
            game.update(with_press, static_cast<double>(number) / static_cast<double>(rate));
        }
        remove_dir_all(folder);
        // The log as a vector, which compares.
        std::vector<std::pair<double, std::string>> log(game.log.begin(), game.log.end());
        return std::tuple(game.player.position, game.player.velocity, game.coins, game.lives, std::move(log));
    };
    auto slow = run(60);
    auto fast = run(240);
    CHECK_EQ(std::get<0>(slow), std::get<0>(fast), "the same place");
    CHECK_EQ(std::get<1>(slow), std::get<1>(fast), "the same velocity");
    CHECK_EQ(std::tuple(std::get<2>(slow), std::get<3>(slow)), std::tuple(std::get<2>(fast), std::get<3>(fast)));
    CHECK_EQ(std::get<4>(slow), std::get<4>(fast), "the same events at the same times");
}
