// The test's own settings, after the ones every program has (see
// Library/UserInterface), in Settings.json in the program's folder: the
// SimplePaint material of each object, in Library/PaintSettings' format,
// which SimplePaintDemo saves, so a material can be tuned there and copied
// here.
//
// By default, the ground and the cube are lit, and the sphere is a constant
// color, which SimplePaint paints with equal Dark and Light Points.

#pragma once

#include "SimplePaint/Material.h"
#include "UserInterface/Application/Settings.h"

#include <array>

namespace minimal_scene_test::settings {

// What only the settings' own code uses.
namespace settings_detail {

// A lit material, painted with its color on faces that face up, as if lit
// from above, and darker sides.
constexpr simple_paint::Parameters lit(std::array<double, 3> base_color_srgb) {
    return simple_paint::Parameters{
        .base_color_srgb = base_color_srgb,
        .brightness = 0.5,
        .shift = 0.0,
        .rotation_degrees = 0.0,
        .dark_point = 0.05,
        .light_point = 0.7,
    };
}

} // namespace settings_detail

struct SceneSettings {
    // Blue is the largest that SimplePaint's sRGB channels accept,
    // 1 - 1/1024.
    simple_paint::Parameters sphere{
        .base_color_srgb = {0.17000002, 0.47433347, 0.9990234375},
        .brightness = 0.5,
        .shift = 0.0,
        .rotation_degrees = 0.0,
        .dark_point = 0.5,
        .light_point = 0.5,
    };
    simple_paint::Parameters ground = settings_detail::lit({0.08900002, 0.89, 0.48950002});
    simple_paint::Parameters cube = settings_detail::lit({0.88, 0.1672, 0.17907982});

    // Reads the materials, "Sphere", "Ground" and "Cube". One that's missing
    // or wrong, like one that SimplePaint rejects, keeps its default, and
    // the settings say what's wrong with it.
    static SceneSettings read(user_interface::Settings& settings);

    bool operator==(const SceneSettings&) const = default;
};

} // namespace minimal_scene_test::settings
