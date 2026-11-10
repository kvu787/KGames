// The test, as a program that Library/UserInterface runs: the sphere's
// motion, the card over the scene that F1 shows and hides, and the scene,
// which fills the window under the card.

#pragma once

#include "Base/Sum.h"
#include "Graphics/Graphics.h"
#include "MinimalSceneTest/Scene.h"
#include "MinimalSceneTest/Settings.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Core/Interface.h"
#include "Json/Json.h"

#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace minimal_scene_test::moving_sphere {

inline constexpr std::string_view Name = "Minimal Scene Test";
inline constexpr std::string_view Description = "A sphere moving at 8 units per second";

// What a frame's input means: F1, and the frame's time passing.
struct Message {
    struct ToggleText {
        bool operator==(const ToggleText&) const = default;
    };
    // Seconds since the last frame, which move the sphere.
    struct Tick {
        float seconds;
        bool operator==(const Tick&) const = default;
    };
    std::variant<ToggleText, Tick> value;

    template <base::AlternativeOf<Message> T> Message(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Message&) const = default;
};

class MovingSphere : public user_interface::Program<Message> {
public:
    settings::SceneSettings settings;
    scene::Motion motion;
    bool text_shown = true;

    explicit MovingSphere(settings::SceneSettings scene_settings);

    // With the settings from Settings.json.
    static MovingSphere start(user_interface::Startup& startup);

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

} // namespace minimal_scene_test::moving_sphere
