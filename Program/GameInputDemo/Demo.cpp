#include "GameInputDemo/Demo.h"

#include "Base/Numbers.h"
#include "Base/Vec.h"
#include "GameInputDemo/Cards.h"
#include "GameInputDemo/Devices.h"
#include "GameInputDemo/GameInputDemo.h"
#include "GameInputDemo/GameInputSource.h"
#include "GameInputDemo/ScriptedSource.h"
#include "GameInputDemo/Settings.h"
#include "UserInterface/Testing/Script.h"

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <format>
#include <memory>
#include <string_view>

namespace game_input_demo {

namespace {

std::uint64_t microseconds(double seconds) {
    return base::saturating_cast<std::uint64_t>(std::round(seconds * 1'000'000.0));
}

// The input that the demo starts with: from GameInput, or from a script's
// devices and readings in a scripted run.
std::expected<Input, std::string> starting_input(user_interface::Startup& startup) {
    DemoSettings settings = DemoSettings::read(startup.settings);
    std::unique_ptr<InputSource> source;
    if (startup.scripted) {
        source = std::make_unique<ScriptedSource>();
    } else if (startup.devices != nullptr) {
        source = std::make_unique<GameInputSource>(startup.devices);
    } else {
        return std::unexpected(std::string("GameInput isn't available"));
    }
    return std::expected<Input, std::string>(std::in_place, std::move(source), settings.rumble_dead_zone);
}

} // namespace

Demo::Demo(user_interface::Startup& startup) : Demo(starting_input(startup)) {}

Demo::Demo(std::expected<Input, std::string> input) : input_(std::move(input)), title_(Name) {}

Demo Demo::with_input(std::expected<Input, std::string> input) {
    return Demo(std::move(input));
}

Demo Demo::scripted(float rumble_dead_zone) {
    return Demo::with_input(std::expected<Input, std::string>(std::in_place, std::make_unique<ScriptedSource>(), rumble_dead_zone));
}

std::string Demo::title() const {
    return title_;
}

std::vector<Message> Demo::input(const user_interface::Ui&) const {
    return base::vec(Message(Message::Tick{}));
}

void Demo::update(Message message, user_interface::Context<Message>& ui) {
    base::match(
        message,
        [&](Message::Listen& picked) {
            if (input_) {
                // The pick applies before the frame's Tick advances the
                // clock, so bring the clock to the frame first, and the
                // log line carries this frame's time.
                input_->align_clock(ui.time());
                input_->pick_listen(std::move(picked.listen));
            }
        },
        [&](Message::Tick&) {
            last_time = ui.time();
            last_delta = static_cast<double>(ui.delta());
            const user_interface::Environment& environment = ui.environment();
            if (!environment.gpu.empty() && title_tearing != environment.tearing) {
                title_tearing = environment.tearing;
                std::string_view tearing = environment.tearing ? "frames may tear" : "frames don't tear";
                title_ = std::format("{} - {} - {}", Name, environment.gpu, tearing);
            }
            if (input_) {
                if (ScriptedSource* scripted_source = input_->scripted()) {
                    scripted_source->now_ = std::max(scripted_source->now_, microseconds(ui.time()));
                }
                bool focused = ui.window_focused();
                // A frame can both lose focus and get it back, like when
                // focus goes to another window and back between two
                // frames. Losing focus first drops what came meanwhile,
                // stops the motors, and makes getting it back count as
                // getting focus.
                if (focused && ui.input().focus_lost) {
                    input_->update(false);
                }
                input_->update(focused);
            }
        });
}

user_interface::Tree<Message> Demo::view(user_interface::Ui& ui) const {
    return game_input_demo::view(ui, input_);
}

json::Value Demo::state() const {
    if (input_) {
        return input_->state();
    } else {
        return json::Value::Object{};
    }
}

std::expected<void, std::string> Demo::script(const json::Value& step) {
    if (!input_) {
        return std::unexpected(std::format("GameInput isn't available: {}", input_.error()));
    }
    ScriptedSource* scripted_source = input_->scripted();
    if (scripted_source == nullptr) {
        return std::unexpected(std::string("the input doesn't come from a script"));
    }
    // The step comes before its frame's update, so its readings are at
    // the time of that frame: the last one's, and the time between
    // frames, which a script keeps the same.
    double frame_time = last_delta > 0.0 ? last_delta : user_interface::DefaultFrameTime;
    scripted_source->now_ = std::max(scripted_source->now_, microseconds(last_time + frame_time));
    return scripted_source->apply(step);
}

} // namespace game_input_demo
