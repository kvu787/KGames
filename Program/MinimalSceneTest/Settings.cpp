#include "MinimalSceneTest/Settings.h"

#include "PaintSettings/PaintSettings.h"
#include "Json/Json.h"

#include <expected>
#include <string>
#include <string_view>
#include <utility>

namespace minimal_scene_test::settings {

using simple_paint::Parameters;

SceneSettings SceneSettings::read(user_interface::Settings& settings) {
    SceneSettings defaults;
    auto material = [&](std::string_view name, const Parameters& default_) {
        return settings.value(name, paint_settings::to_value(default_), [&](const json::Value& value) {
            // The settings go on after what's wrong, so it ends without
            // the sentence's period.
            return paint_settings::parse(value, default_).transform_error([](std::string error) {
                while (error.ends_with('.')) {
                    error.pop_back();
                }
                return error;
            });
        });
    };
    return SceneSettings{
        .sphere = material("Sphere", defaults.sphere),
        .ground = material("Ground", defaults.ground),
        .cube = material("Cube", defaults.cube),
    };
}

} // namespace minimal_scene_test::settings
