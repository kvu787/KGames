#include "Platformer/Game.h"

#include "Base/Panic.h"
#include "Base/Text.h"

#include <algorithm>
#include <expected>
#include <format>
#include <string_view>

namespace platformer::game {

using controls::Controls;
using events::Cause;
using events::Event;
using events::MostEvents;
using level::Level;
using level::Zone;
using physics::Buffer;
using physics::Half;
using physics::Player;
using physics::Solid;
using physics::SpringSpeed;
using physics::Standing;
using physics::Step;
using progress::Progress;

namespace {

// How many recent events the log keeps for the Events card.
constexpr size_t LogLength = 40;

} // namespace

std::string describe_value(const RunState& state) {
    return base::match(
        state, [](const RunState::Running&) { return std::string("Running"); },
        [](const RunState::Finished& finished) { return std::format("Finished({})", base::debug(finished.seconds)); },
        [](const RunState::Failed&) { return std::string("Failed"); });
}

double life(EffectKind kind) {
    switch (kind) {
    case EffectKind::Confetti:
        return 2.0;
    case EffectKind::SaveFlash:
        return 1.2;
    default:
        return 0.6;
    }
}

std::string describe_value(EffectKind kind) {
    switch (kind) {
    case EffectKind::Sparkle:
        return "Sparkle";
    case EffectKind::Burst:
        return "Burst";
    case EffectKind::Bounce:
        return "Bounce";
    case EffectKind::Flag:
        return "Flag";
    case EffectKind::Poof:
        return "Poof";
    case EffectKind::Shimmer:
        return "Shimmer";
    case EffectKind::Confetti:
        return "Confetti";
    case EffectKind::SaveFlash:
        return "SaveFlash";
    }
    base::unreachable();
}

Game::Game(std::vector<Level> levels, Progress progress, std::filesystem::path save_path) :
    levels(std::move(levels)),
    level(0),
    progress(std::move(progress)),
    player(Player::at(this->levels[0].spawn)),
    collected(this->levels[0].coins.size(), false),
    coins(0),
    lives(Lives),
    doors_open(this->levels[0].doors.size(), false),
    reached(this->levels[0].checkpoints.size(), false),
    checkpoint(std::nullopt),
    state_(RunState::Running{}),
    started(0.0),
    simulated(0.0),
    show_cards(true),
    save_path(std::move(save_path)),
    drained_this_frame(0) {
    publish(Event::RunStarted{});
    drain();
}

const Level& Game::current() const {
    return levels[level];
}

size_t Game::total_coins() const {
    return current().coins.size();
}

double Game::run_seconds() const {
    if (const RunState::Finished* finished = base::as<RunState::Finished>(state_)) {
        return finished->seconds;
    }
    return simulated - started;
}

void Game::restart() {
    const Level& current_level = levels[level];
    player = Player::at(current_level.spawn);
    collected = std::vector<bool>(current_level.coins.size(), false);
    coins = 0;
    lives = Lives;
    doors_open = std::vector<bool>(current_level.doors.size(), false);
    reached = std::vector<bool>(current_level.checkpoints.size(), false);
    checkpoint = std::nullopt;
    state_ = RunState::Running{};
    started = simulated;
    effects.clear();
    publish(Event::RunStarted{});
    drain();
}

void Game::go_to_level(size_t target) {
    if (target < std::min(progress.unlocked, levels.size()) && target != level) {
        level = target;
        restart();
    }
}

void Game::update(const Controls& controls, double seconds) {
    drained_this_frame = 0;
    if (controls.toggle_cards) {
        show_cards = !show_cards;
    }
    if (controls.restart) {
        restart();
    }
    if (controls.level != 0) {
        std::int64_t target = static_cast<std::int64_t>(level) + std::int64_t{controls.level};
        if (target >= 0 && static_cast<size_t>(target) < std::min(progress.unlocked, levels.size())) {
            level = static_cast<size_t>(target);
            restart();
        }
    }
    if (controls.jump_pressed) {
        // The press counts for a moment, so one just before a landing
        // jumps on the very step of the landing.
        player.jump_until = simulated + Buffer;
    }
    while (simulated + Step <= seconds) {
        simulated += Step;
        step(controls);
        drain();
    }
    std::erase_if(effects, [&](const Effect& spawned) { return !(spawned.started + life(spawned.kind) > simulated); });
}

void Game::step(const Controls& controls) {
    if (!base::is<RunState::Running>(state_)) {
        return;
    }
    double now = simulated;
    const Level& current_level = levels[level];

    // The platform the player stood on carries them by how far it moved.
    std::array<double, 2> carry = base::match(
        player.standing,
        [&](const Standing::Patrol& patrol) {
            Zone before = current_level.patrols[patrol.index].at(now - Step);
            Zone after = current_level.patrols[patrol.index].at(now);
            return std::array{after.x - before.x, after.y - before.y};
        },
        [](const Standing::Still&) { return std::array{0.0, 0.0}; });

    // What's solid this step: the ground, the doors still closed, and
    // the platforms where they are now.
    std::vector<Solid> solids;
    solids.reserve(current_level.solids.size());
    for (const Zone& zone : current_level.solids) {
        solids.push_back(Solid(zone, Standing::Still{}));
    }
    for (size_t door = 0; door < current_level.doors.size(); door++) {
        if (!doors_open[door]) {
            solids.push_back(Solid(current_level.doors[door], Standing::Still{}));
        }
    }
    for (size_t patrol = 0; patrol < current_level.patrols.size(); patrol++) {
        solids.push_back(Solid(current_level.patrols[patrol].at(now), Standing::Patrol{patrol}));
    }

    physics::step(player, now, controls.push, controls.jump_held, carry, solids);

    // What the movement did, as events. Each check reads the state as
    // the last drain left it, so nothing is published twice.
    std::array<double, 2> position = player.position;
    bool falling = player.velocity[1] < -1.0;
    std::vector<Event> found;
    for (size_t coin = 0; coin < current_level.coins.size(); coin++) {
        std::array<double, 2> at = current_level.coins[coin];
        if (!collected[coin] && Zone(at[0] - 0.3, at[1] - 0.3, 0.6, 0.6).overlaps(position, Half)) {
            found.push_back(Event::CoinCollected{coin});
        }
    }
    for (size_t spring = 0; spring < current_level.springs.size(); spring++) {
        if (falling && current_level.springs[spring].overlaps(position, Half)) {
            found.push_back(Event::SpringBounced{spring});
        }
    }
    for (size_t index = 0; index < current_level.checkpoints.size(); index++) {
        if (!reached[index] && current_level.checkpoints[index].overlaps(position, Half)) {
            found.push_back(Event::CheckpointReached{index});
        }
    }
    if (current_level.goal.overlaps(position, Half)) {
        found.push_back(Event::GoalReached{});
    }
    bool on_spikes = std::ranges::any_of(current_level.spikes, [&](const Zone& zone) {
        // A touch of the points themselves, not a brush past the base.
        Zone shrunk(zone.x + 0.1, zone.y, zone.width - 0.2, zone.height - 0.1);
        return shrunk.overlaps(position, Half);
    });
    if (on_spikes) {
        found.push_back(Event::Died{Cause::Spikes});
    } else if (position[1] < current_level.bounds.y) {
        found.push_back(Event::Died{Cause::Fell});
    }
    for (Event& event : found) {
        publish(std::move(event));
    }
}

void Game::publish(Event event) {
    queue.push_back(std::move(event));
}

void Game::drain() {
    while (std::optional<Event> event = queue.pop_front()) {
        drained_this_frame += 1;
        if (drained_this_frame > MostEvents) {
            base::panic(std::format("the event cascade didn't settle: {} events in one frame, last {}", drained_this_frame, event->label()));
        }
        log.push_front({simulated, event->label()});
        log.truncate(LogLength);
        apply(std::move(*event));
    }
}

void Game::apply(Event event) {
    base::match(
        std::move(event), [](const Event::RunStarted&) {},
        [&](const Event::CoinCollected& coin_collected) {
            size_t coin = coin_collected.coin;
            if (!collected[coin]) {
                collected[coin] = true;
                coins += 1;
                std::array<double, 2> at = levels[level].coins[coin];
                effect(EffectKind::Sparkle, at);
                if (coins == total_coins()) {
                    publish(Event::AllCoinsCollected{});
                }
            }
        },
        [&](const Event::AllCoinsCollected&) {
            for (size_t door = 0; door < doors_open.size(); door++) {
                if (!doors_open[door]) {
                    publish(Event::DoorOpened{door});
                }
            }
        },
        [&](const Event::DoorOpened& door_opened) {
            doors_open[door_opened.door] = true;
            std::array<double, 2> at = levels[level].doors[door_opened.door].center();
            effect(EffectKind::Burst, at);
        },
        [&](const Event::SpringBounced& spring_bounced) {
            player.velocity[1] = SpringSpeed;
            player.grounded = false;
            player.standing = Standing::Still{};
            std::array<double, 2> at = levels[level].springs[spring_bounced.spring].center();
            effect(EffectKind::Bounce, at);
        },
        [&](const Event::CheckpointReached& checkpoint_reached) {
            reached[checkpoint_reached.checkpoint] = true;
            checkpoint = checkpoint_reached.checkpoint;
            std::array<double, 2> at = levels[level].checkpoints[checkpoint_reached.checkpoint].center();
            effect(EffectKind::Flag, at);
        },
        [&](const Event::Died&) {
            if (base::is<RunState::Running>(state_)) {
                effect(EffectKind::Poof, player.position);
                lives -= 1;
                if (lives > 0) {
                    publish(Event::Respawned{});
                } else {
                    publish(Event::OutOfLives{});
                }
            }
        },
        [&](const Event::Respawned&) {
            const Level& current_level = levels[level];
            std::array<double, 2> spawn = current_level.spawn;
            if (checkpoint) {
                Zone zone = current_level.checkpoints[*checkpoint];
                spawn = {zone.center()[0], zone.y};
            }
            player = Player::at(spawn);
            effect(EffectKind::Shimmer, player.position);
        },
        [&](const Event::OutOfLives&) { state_ = RunState::Failed{}; },
        [&](const Event::GoalReached&) {
            if (base::is<RunState::Running>(state_)) {
                publish(Event::RunFinished{simulated - started});
            }
        },
        [&](const Event::RunFinished& run_finished) {
            double seconds = run_finished.seconds;
            state_ = RunState::Finished{seconds};
            std::array<double, 2> at = levels[level].goal.center();
            effect(EffectKind::Confetti, at);
            std::optional<double> previous = progress.best[level];
            if (!previous || seconds < *previous) {
                publish(Event::RecordSet{seconds, previous});
            }
            size_t next = level + 1;
            if (next < levels.size() && next >= progress.unlocked) {
                publish(Event::LevelUnlocked{next});
            }
        },
        [&](const Event::RecordSet& record_set) {
            progress.record(level, record_set.seconds);
            publish(Event::SaveProgress{});
        },
        [&](const Event::LevelUnlocked& level_unlocked) {
            progress.unlock(level_unlocked.level);
            publish(Event::SaveProgress{});
        },
        [&](const Event::SaveProgress&) {
            // The first save in a cascade writes; the rest find the
            // progress clean and do nothing, so one cascade saves once.
            if (progress.dirty) {
                std::expected<std::string, std::string> saved = progress.save(save_path);
                if (saved) {
                    publish(Event::ProgressSaved{std::move(*saved)});
                } else {
                    publish(Event::SaveFailed{std::move(saved.error())});
                }
            }
        },
        [&](const Event::ProgressSaved& progress_saved) {
            status = std::format("Saved {}", progress_saved.file);
            std::array<double, 2> at = player.position;
            effect(EffectKind::SaveFlash, {at[0], at[1] + 1.5});
        },
        [&](Event::SaveFailed&& save_failed) { status = std::move(save_failed.error); });
}

void Game::effect(EffectKind kind, std::array<double, 2> at) {
    effects.push_back(Effect{
        .kind = kind,
        .at = at,
        .started = simulated,
    });
}

json::Value Game::state() const {
    std::optional<double> best = progress.best[level];
    std::string_view run_state = base::match(
        state_, [](const RunState::Running&) { return std::string_view("Running"); }, [](const RunState::Finished&) { return std::string_view("Finished"); },
        [](const RunState::Failed&) { return std::string_view("Failed"); });
    size_t shown = std::min<size_t>(log.size(), 6);
    std::vector<json::Value> events;
    events.reserve(shown);
    for (size_t index = 0; index < shown; index++) {
        events.push_back(json::Value(log[index].second));
    }
    return json::object({
        {"Level", current().name},
        {"Coins", coins},
        {"TotalCoins", total_coins()},
        {"Lives", lives},
        {"State", run_state},
        {"Time", run_seconds()},
        {"Best", best},
        {"DoorsOpen", std::ranges::all_of(doors_open, [](bool open) { return open; })},
        {"Position", json::Value::Array{{player.position[0], player.position[1]}}},
        {"Grounded", player.grounded},
        {"Unlocked", progress.unlocked},
        {"Status", status},
        {"Events", json::Value::Array{std::move(events)}},
    });
}

} // namespace platformer::game
