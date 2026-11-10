#include "GameInputDemo/Settings.h"

#include "Base/Text.h"

#include <expected>
#include <format>
#include <string>

namespace game_input_demo {

DemoSettings DemoSettings::read(user_interface::Settings& settings) {
    // Less than 1, because the strength grows from the dead zone's edge
    // to the stick's.
    double dead_zone = settings.number("RumbleTestInnerDeadZone", 0.1, [](double number) -> std::expected<void, std::string> {
        if (number >= 0.0 && number < 1.0) {
            return {};
        } else {
            return std::unexpected(std::format("must be a number from 0 to less than 1, not {}", base::display(number)));
        }
    });
    return DemoSettings{.rumble_dead_zone = static_cast<float>(dead_zone)};
}

} // namespace game_input_demo
