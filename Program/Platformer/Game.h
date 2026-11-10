// The game: a run through a level, simulated in fixed steps on its own
// clock, publishing an Event for everything that happens. Each substep
// moves the world, detects what the movement did - coins touched, springs
// landed on, spikes hit, the goal reached - and then drains the event queue
// to quiescence: every consumer runs, and whatever they publish runs too,
// before the next substep. So a frame's input produces the whole graph of
// its consequences inside that frame, however deep it goes, and the view
// only ever describes settled state.

#pragma once

#include "Base/Sum.h"
#include "Base/VecDeque.h"
#include "Platformer/Controls.h"
#include "Platformer/Events.h"
#include "Platformer/Level.h"
#include "Platformer/Physics.h"
#include "Platformer/Progress.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <filesystem>
#include <optional>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace platformer::game {

// Lives per run.
inline constexpr std::uint32_t Lives = 3;

// Where a run stands.
struct RunState {
    struct Running {
        bool operator==(const Running&) const = default;
    };
    // Finished, in this many seconds.
    struct Finished {
        double seconds;
        bool operator==(const Finished&) const = default;
    };
    // Out of lives.
    struct Failed {
        bool operator==(const Failed&) const = default;
    };
    std::variant<Running, Finished, Failed> value;

    template <base::AlternativeOf<RunState> T> RunState(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const RunState&) const = default;
};

// How a failed check in a test shows where a run stands.
std::string describe_value(const RunState& state);

// A moment drawn over the world for a while, like a coin's sparkle. State,
// not an action: spawned by an event's consumer, timed on the simulation's
// clock, pruned once over, drawn by the view.
enum class EffectKind {
    Sparkle,
    Burst,
    Bounce,
    Flag,
    Poof,
    Shimmer,
    Confetti,
    SaveFlash,
};

double life(EffectKind kind);

// How a failed check in a test shows a kind of effect.
std::string describe_value(EffectKind kind);

struct Effect {
    EffectKind kind;
    std::array<double, 2> at;
    double started;

    bool operator==(const Effect&) const = default;
};

class Game {
public:
    Game(std::vector<level::Level> levels, progress::Progress progress, std::filesystem::path save_path);
    Game(const Game&) = delete;
    Game& operator=(const Game&) = delete;
    Game(Game&&) = default;
    Game& operator=(Game&&) = default;

    const level::Level& current() const;

    size_t total_coins() const;

    // The run's clock: how long it took, or has taken so far.
    double run_seconds() const;

    // Starts the current level over: a fresh run on the same clock.
    void restart();

    // Goes to an unlocked level and starts a run there.
    void go_to_level(size_t target);

    // A frame: the frame's one-off controls, then fixed steps up to
    // `seconds`, each followed by its events, drained to quiescence.
    void update(const controls::Controls& controls, double seconds);

    // The game's state, for scripts and tests.
    json::Value state() const;

    std::vector<level::Level> levels;
    size_t level;
    progress::Progress progress;
    // The run.
    physics::Player player;
    std::vector<bool> collected;
    size_t coins;
    std::uint32_t lives;
    std::vector<bool> doors_open;
    std::vector<bool> reached;
    std::optional<size_t> checkpoint;
    RunState state_;
    // When the run started, on the simulation's clock.
    double started;
    // The simulation's clock, which only ever moves in steps of Step.
    double simulated;
    // The log of events that the Events card shows, newest first, each with
    // when it happened on the simulation's clock.
    base::VecDeque<std::pair<double, std::string>> log;
    std::vector<Effect> effects;
    // What the last save said, for the status bar.
    std::string status;
    bool show_cards;

private:
    // One fixed step: the platforms move, the player moves against them, and
    // what the movement did becomes events.
    void step(const controls::Controls& controls);

    void publish(events::Event event);

    // Runs every queued event, and everything they publish, now. A cascade
    // that doesn't settle within MostEvents in one frame is a bug in the
    // event graph, and stops the program rather than hiding.
    void drain();

    // One event's consequences: state changes, effects, and more events.
    // Facts ride in the event; everything else is read from the game as it
    // is now, never from a snapshot.
    void apply(events::Event event);

    void effect(EffectKind kind, std::array<double, 2> at);

    std::filesystem::path save_path;
    // The events: the queue that drains inside the frame, and how many
    // drained this frame, against the cap.
    base::VecDeque<events::Event> queue;
    size_t drained_this_frame;
};

} // namespace platformer::game
