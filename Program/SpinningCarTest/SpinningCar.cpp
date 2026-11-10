#include "SpinningCarTest/SpinningCar.h"

#include "SpinningCarTest/Controls.h"
#include "UserInterface/Widgets/InfoCard.h"

namespace spinning_car_test::spinning_car {

using graphics::Frame;
using graphics::Gpu;
using scene::car_placement;
using settings::SceneSettings;
using user_interface::Context;
using user_interface::InfoCard;
using user_interface::Startup;
using user_interface::Tree;
using user_interface::Ui;

SpinningCar::SpinningCar(SceneSettings scene_settings) : settings(std::move(scene_settings)), placement(car_placement(0.0, false)) {}

SpinningCar SpinningCar::start(Startup& startup) {
    return SpinningCar(SceneSettings::read(startup.settings));
}

std::string SpinningCar::title() const {
    return std::string(Name);
}

std::vector<Message> SpinningCar::input(const Ui& ui) const {
    std::vector<Message> messages;
    if (InfoCard::toggled(ui.input())) {
        messages.push_back(Message::ToggleText{});
    }
    if (controls::toggles_stationary(ui.input())) {
        messages.push_back(Message::ToggleStationary{});
    }
    messages.push_back(Message::Tick{ui.time()});
    return messages;
}

void SpinningCar::update(Message message, Context<Message>&) {
    base::match(
        message, [&](const Message::ToggleText&) { text_shown = !text_shown; }, [&](const Message::ToggleStationary&) { stationary = !stationary; },
        [&](const Message::Tick& tick) { placement = car_placement(tick.seconds, stationary); });
}

Tree<Message> SpinningCar::view(Ui& ui) const {
    Tree<Message> tree;
    if (text_shown) {
        std::string_view car = stationary ? "Stationary" : "Moving";
        tree.info_card(ui, InfoCard{
                               .title = Name,
                               .description = Description,
                               .rows = {{{"Car", car}}},
                               .keys = {{{"S", "Stationary car"}}},
                           });
    }
    return tree;
}

void SpinningCar::prepare(Gpu& gpu) {
    scene.emplace(gpu, settings);
}

void SpinningCar::render(Gpu& gpu, const Frame& frame) {
    if (scene) {
        scene->draw(gpu, frame, placement);
    }
}

json::Value SpinningCar::state() const {
    return json::object({
        {"CarPosition", placement.first},
        {"CarTurn", placement.second},
        {"Stationary", stationary},
        {"TextShown", text_shown},
    });
}

} // namespace spinning_car_test::spinning_car
