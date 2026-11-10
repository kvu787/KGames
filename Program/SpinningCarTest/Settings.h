// The test's own settings, after the ones every program has (see
// Library/UserInterface), in Settings.json in the program's folder: the
// SimplePaint material of each part of the car and of the sphere, in
// Library/PaintSettings' format, and the sphere's resolution.

#pragma once

#include "SimplePaint/Material.h"
#include "UserInterface/Application/Settings.h"

#include <array>
#include <cstdint>
#include <string_view>

namespace spinning_car_test::settings {

// The car's materials, in its mesh's order, then the sphere's.
inline constexpr std::array<std::string_view, 6> MaterialNames{
    "SimplePaintShader_Axles", "SimplePaintShader_Body", "SimplePaintShader_Cabin", "SimplePaintShader_Headlights", "SimplePaintShader_Wheels", "SimplePaintShader_Sphere",
};

// What only the settings' own code uses.
namespace settings_detail {

constexpr simple_paint::Parameters material(std::array<double, 3> base_color_srgb, double brightness, double shift, double rotation_degrees, double dark_point,
                                            double light_point) {
    return simple_paint::Parameters{
        .base_color_srgb = base_color_srgb,
        .brightness = brightness,
        .shift = shift,
        .rotation_degrees = rotation_degrees,
        .dark_point = dark_point,
        .light_point = light_point,
    };
}

} // namespace settings_detail

struct SceneSettings {
    std::array<simple_paint::Parameters, 6> materials{
        settings_detail::material({0.2159017, 0.21590173, 0.2159017}, 0.5, 0.0, 0.0, 0.0, 1.0),
        settings_detail::material({0.103999995, 0.3592001, 0.8}, 0.20996094, 0.4140625, 35.600586, 0.0, 1.0),
        settings_detail::material({0.37600002, 0.5314668, 0.8}, 0.26464844, 0.36132813, 28.935547, 0.2000000095, 1.0),
        settings_detail::material({0.98, 0.74676, 0.15680003}, 0.48242188, 0.6484375, 49.7666, 0.0, 0.8999766052025),
        settings_detail::material({0.27022192, 0.27022192, 0.27022186}, 0.41796875, 0.2841797, 224.49902, 0.12890625, 0.78222656),
        settings_detail::material({0.107, 0.223, 0.578}, 0.126, 0.0, 0.0, 0.0, 1.0),
    };
    // Vertices around each of the sphere's rings, and bands from pole to pole.
    std::array<std::uint32_t, 2> sphere_resolution{64, 32};

    // Reads the materials, then "Sphere", with the sphere's resolution, like
    // {"UResolution": 64, "VResolution": 32}. One that's missing or wrong,
    // like a material that SimplePaint rejects, keeps its default, and the
    // settings say what's wrong with it.
    static SceneSettings read(user_interface::Settings& settings);

    bool operator==(const SceneSettings&) const = default;
};

} // namespace spinning_car_test::settings
