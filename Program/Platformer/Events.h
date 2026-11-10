// Everything that happens in the game, as data: the simulation publishes
// events as it discovers them, and Game::apply consumes each one, which may
// publish more. The queue drains to quiescence inside the substep that
// started it, so however deep the graph goes - the last coin opening the
// door, the goal ending the run, the record saving the file, the save
// flashing its light - all of it has happened by the end of the frame.
// An event carries the facts of its moment, like which coin or how many
// seconds; what a consumer needs beyond that, it reads from the game as it
// is when the event is applied, never from a snapshot.

#pragma once

#include "Base/Sum.h"

#include <cstddef>
#include <optional>
#include <string>
#include <utility>
#include <variant>

namespace platformer::events {

// A cascade that doesn't settle is a bug, not a wait.
inline constexpr size_t MostEvents = 1'000;

enum class Cause {
    Spikes,
    Fell,
};

struct Event {
    // A run began: a new game, a restart, or arriving in a level.
    struct RunStarted {
        bool operator==(const RunStarted&) const = default;
    };
    // The player touched a coin that wasn't collected yet.
    struct CoinCollected {
        size_t coin;
        bool operator==(const CoinCollected&) const = default;
    };
    // The last coin was collected, which is what opens doors.
    struct AllCoinsCollected {
        bool operator==(const AllCoinsCollected&) const = default;
    };
    struct DoorOpened {
        size_t door;
        bool operator==(const DoorOpened&) const = default;
    };
    // The player landed on a spring while falling.
    struct SpringBounced {
        size_t spring;
        bool operator==(const SpringBounced&) const = default;
    };
    // The player touched a checkpoint for the first time this run.
    struct CheckpointReached {
        size_t checkpoint;
        bool operator==(const CheckpointReached&) const = default;
    };
    struct Died {
        Cause cause;
        bool operator==(const Died&) const = default;
    };
    // After a death with lives left: back at the checkpoint, or the spawn.
    struct Respawned {
        bool operator==(const Respawned&) const = default;
    };
    // After a death without lives left: the run failed.
    struct OutOfLives {
        bool operator==(const OutOfLives&) const = default;
    };
    // The player touched the goal while the run was going.
    struct GoalReached {
        bool operator==(const GoalReached&) const = default;
    };
    // The run ended well, in this many seconds.
    struct RunFinished {
        double seconds;
        bool operator==(const RunFinished&) const = default;
    };
    // The run beat the level's best time, which was `previous`.
    struct RecordSet {
        double seconds;
        std::optional<double> previous;
        bool operator==(const RecordSet&) const = default;
    };
    // Finishing a level the first time unlocks the next.
    struct LevelUnlocked {
        size_t level;
        bool operator==(const LevelUnlocked&) const = default;
    };
    // Something made the progress worth saving. Consumers made it dirty;
    // the first SaveProgress writes the file, and any more in the same
    // cascade find it clean and do nothing, so one cascade saves once.
    struct SaveProgress {
        bool operator==(const SaveProgress&) const = default;
    };
    struct ProgressSaved {
        std::string file;
        bool operator==(const ProgressSaved&) const = default;
    };
    struct SaveFailed {
        std::string error;
        bool operator==(const SaveFailed&) const = default;
    };
    std::variant<RunStarted, CoinCollected, AllCoinsCollected, DoorOpened, SpringBounced, CheckpointReached, Died, Respawned, OutOfLives, GoalReached, RunFinished, RecordSet,
                 LevelUnlocked, SaveProgress, ProgressSaved, SaveFailed>
        value;

    template <base::AlternativeOf<Event> T> Event(T&& alternative) : value(std::forward<T>(alternative)) {}

    // A compact line for the log card and for scripts, like
    // "CoinCollected 2".
    std::string label() const;

    bool operator==(const Event&) const = default;
};

} // namespace platformer::events
