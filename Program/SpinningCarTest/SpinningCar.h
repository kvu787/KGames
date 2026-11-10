// The test, as a program that Library/UserInterface runs: where the car is,
// what S and F1 do, the card over the scene, and the scene, which fills the
// window under the card. The car keeps time while the window is minimized.

#pragma once

#include "Base/Sum.h"
#include "Graphics/Graphics.h"
#include "SpinningCarTest/Scene.h"
#include "SpinningCarTest/Settings.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Core/Interface.h"
#include "Json/Json.h"

#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace spinning_car_test::spinning_car {

inline constexpr std::string_view Name = "Spinning Car Test";
inline constexpr std::string_view Description = "A car moving at 8 units and turning at 90 degrees per second";

// What a frame's input means: F1, S, and the frame's time, which places the
// car. The tick comes after the toggles, so the frame that presses S places
// the car with the new state.
struct Message {
    struct ToggleText {
        bool operator==(const ToggleText&) const = default;
    };
    struct ToggleStationary {
        bool operator==(const ToggleStationary&) const = default;
    };
    // Seconds since the program started.
    struct Tick {
        double seconds;
        bool operator==(const Tick&) const = default;
    };
    std::variant<ToggleText, ToggleStationary, Tick> value;

    template <base::AlternativeOf<Message> T> Message(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Message&) const = default;
};

class SpinningCar : public user_interface::Program<Message> {
public:
    settings::SceneSettings settings;
    bool stationary = false;
    // Where the car is along X, and how far it has turned.
    std::pair<float, float> placement;
    bool text_shown = true;

    explicit SpinningCar(settings::SceneSettings scene_settings);

    // With the settings from Settings.json.
    static SpinningCar start(user_interface::Startup& startup);

    std::string title() const;

    std::vector<Message> input(const user_interface::Ui& ui) const;

    void update(Message message, user_interface::Context<Message>& ui);

    user_interface::Tree<Message> view(user_interface::Ui& ui) const;

    void prepare(graphics::Gpu& gpu);

    void render(graphics::Gpu& gpu, const graphics::Frame& frame);

    json::Value state() const;

private:
    // Made once there's a GPU, before the first frame.
    std::optional<scene::Scene> scene;
};

} // namespace spinning_car_test::spinning_car
