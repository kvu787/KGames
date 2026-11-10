#include "Platformer/Events.h"

#include "Base/Panic.h"

#include <format>

namespace platformer::events {

std::string Event::label() const {
    return base::match(
        *this, [](const RunStarted&) { return std::string("RunStarted"); }, [](const CoinCollected& event) { return std::format("CoinCollected {}", event.coin); },
        [](const AllCoinsCollected&) { return std::string("AllCoinsCollected"); }, [](const DoorOpened& event) { return std::format("DoorOpened {}", event.door); },
        [](const SpringBounced& event) { return std::format("SpringBounced {}", event.spring); },
        [](const CheckpointReached& event) { return std::format("CheckpointReached {}", event.checkpoint); },
        [](const Died& event) {
            switch (event.cause) {
            case Cause::Spikes:
                return std::string("Died Spikes");
            case Cause::Fell:
                return std::string("Died Fell");
            }
            base::unreachable();
        },
        [](const Respawned&) { return std::string("Respawned"); }, [](const OutOfLives&) { return std::string("OutOfLives"); },
        [](const GoalReached&) { return std::string("GoalReached"); }, [](const RunFinished& event) { return std::format("RunFinished {:.3f}", event.seconds); },
        [](const RecordSet& event) { return std::format("RecordSet {:.3f}", event.seconds); },
        [](const LevelUnlocked& event) { return std::format("LevelUnlocked {}", event.level); }, [](const SaveProgress&) { return std::string("SaveProgress"); },
        [](const ProgressSaved& event) { return std::format("ProgressSaved {}", event.file); }, [](const SaveFailed& event) { return std::format("SaveFailed {}", event.error); });
}

} // namespace platformer::events
