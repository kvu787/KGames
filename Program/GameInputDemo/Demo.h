// The demo, as a Program that Library/UserInterface's main loop runs: what
// it does each frame, before Cards.h describes what it read.
//
// The demo reads its input from the devices that Library/UserInterface reads
// with GameInput, or from a script's Program steps in a scripted run, its
// shortcuts F1 and F2 too. F10 and F11 are the library's, from the same
// readings. Without GameInput the window still opens, and the library's
// notice under the app bar says why there's no input.

#pragma once

#include "Base/Sum.h"
#include "GameInputDemo/Input.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "Json/Json.h"

#include <expected>
#include <optional>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace game_input_demo {

// What a frame means to the demo.
struct Message {
    // Once a frame: the title, and the frame's readings.
    struct Tick {};
    // The combo box picked which gamepad to listen to.
    struct Listen {
        game_input_demo::Listen listen;
    };
    std::variant<Tick, Listen> value;

    template <base::AlternativeOf<Message> T> Message(T&& alternative) : value(std::forward<T>(alternative)) {}
};

class Demo : public user_interface::Program<Message> {
public:
    // The input, or why GameInput isn't available.
    std::expected<Input, std::string> input_;

    // The demo as the main loop starts it: on GameInput, or on a script's
    // devices and readings in a scripted run.
    explicit Demo(user_interface::Startup& startup);

    // The demo with this input, like a test's.
    static Demo with_input(std::expected<Input, std::string> input);

    // Input from a test's devices and readings, with the rumble test's dead
    // zone.
    static Demo scripted(float rumble_dead_zone);

    std::string title() const;

    std::vector<Message> input(const user_interface::Ui& ui) const;

    void update(Message message, user_interface::Context<Message>& ui);

    user_interface::Tree<Message> view(user_interface::Ui& ui) const;

    // What the cards show, for scripts to save and check. The main loop adds
    // the interface's, like whether the combo box's list is open, and which
    // device's tooltip shows.
    json::Value state() const;

    // Connects and disconnects a script's devices, and gives their readings.
    std::expected<void, std::string> script(const json::Value& step);

private:
    explicit Demo(std::expected<Input, std::string> input);

    // The window's title, with the GPU that draws, and whether frames may
    // tear, once a frame knows them. F10 changes whether they may, so the
    // title follows it: `title_tearing` is what the title says.
    std::string title_;
    std::optional<bool> title_tearing;
    // The last frame's time, and the time between it and the one before, in
    // seconds, which give a script's readings the time of the frame that
    // they're for.
    double last_time = 0.0;
    double last_delta = 0.0;
};

} // namespace game_input_demo
