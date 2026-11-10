#include "MinimalSceneTest/MovingSphere.h"

#include "UserInterface/Widgets/InfoCard.h"

namespace minimal_scene_test::moving_sphere {

using graphics::Frame;
using graphics::Gpu;
using settings::SceneSettings;
using user_interface::Context;
using user_interface::InfoCard;
using user_interface::Startup;
using user_interface::Tree;
using user_interface::Ui;

MovingSphere::MovingSphere(SceneSettings scene_settings) : settings(std::move(scene_settings)) {}

MovingSphere MovingSphere::start(Startup& startup) {
    return MovingSphere(SceneSettings::read(startup.settings));
}

std::string MovingSphere::title() const {
    return std::string(Name);
}

std::vector<Message> MovingSphere::input(const Ui& ui) const {
    std::vector<Message> messages;
    if (InfoCard::toggled(ui.input())) {
        messages.push_back(Message::ToggleText{});
    }
    messages.push_back(Message::Tick{ui.delta()});
    return messages;
}

void MovingSphere::update(Message message, Context<Message>&) {
    base::match(message, [&](const Message::ToggleText&) { text_shown = !text_shown; }, [&](const Message::Tick& tick) { motion.advance_frame(tick.seconds); });
}

Tree<Message> MovingSphere::view(Ui& ui) const {
    Tree<Message> tree;
    if (text_shown) {
        tree.info_card(ui, InfoCard{
                               .title = Name,
                               .description = Description,
                               .rows = {},
                               .keys = {},
                           });
    }
    return tree;
}

void MovingSphere::prepare(Gpu& gpu) {
    scene.emplace(gpu, settings);
}

void MovingSphere::render(Gpu& gpu, const Frame& frame) {
    if (scene) {
        scene->draw(gpu, frame, motion.position);
    }
}

json::Value MovingSphere::state() const {
    return json::object({
        {"SpherePosition", motion.position},
        {"SphereDirection", motion.direction},
        {"TextShown", text_shown},
    });
}

} // namespace minimal_scene_test::moving_sphere
