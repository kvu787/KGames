// The platformer as a Program, which Library/UserInterface's main loop runs.
// A frame's input is one message: the controls, which the game turns into
// fixed steps and events, drained to quiescence before the frame's view.
// The app bar goes between the unlocked levels and starts a run over, and
// the cards show the run and the events as they happen.

#pragma once

#include "Base/Sum.h"
#include "Platformer/Controls.h"
#include "Platformer/Game.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Core/Interface.h"
#include "Json/Json.h"

#include <cstddef>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace platformer::program {

// What a frame's input means.
struct Message {
    // Once a frame: the frame's controls, which drive the simulation to the
    // frame's time.
    struct Tick {
        controls::Controls controls;
    };
    // The app bar: a level to go to, and starting the run over.
    struct GoLevel {
        size_t level;
    };
    struct Restart {};
    std::variant<Tick, GoLevel, Restart> value;

    template <base::AlternativeOf<Message> T> Message(T&& alternative) : value(std::forward<T>(alternative)) {}
};

class Platformer : public user_interface::Program<Message> {
public:
    game::Game game;

    explicit Platformer(game::Game game);

    // The game as it starts: with the progress from Save.json in the data
    // folder, which is the program's folder, or a script's output folder, so
    // scripted runs leave the player's progress alone.
    static Platformer start(user_interface::Startup& startup);

    std::string title() const;

    std::vector<Message> input(const user_interface::Ui& ui) const;

    void update(Message message, user_interface::Context<Message>& ui);

    user_interface::Tree<Message> view(user_interface::Ui& ui) const;

    json::Value state() const;
};

} // namespace platformer::program
