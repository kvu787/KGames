// The scene: the green ground and the red cube, in flat colors, then a car
// that moves back and forth along X at 8 units per second while it turns at
// 90 degrees per second, and a sphere that stays put, both painted with
// SimplePaint, all with one depth buffer. Every frame draws all of it.

#pragma once

#include "Graphics/Graphics.h"
#include "SimplePaint/SimplePaint.h"
#include "SpinningCarTest/Settings.h"
#include "UserInterface/Drawing/Geometry.h"

#include <array>
#include <cstddef>
#include <utility>

namespace spinning_car_test::scene {

// The camera's orthographic view is this many units tall.
inline constexpr float ViewHeight = 5.0f;
inline constexpr float SphereRadius = 0.4f;
// Where the sphere's middle is.
inline constexpr std::array<float, 3> SphereCenter{1.5f, SphereRadius, -1.5f};
// #4c4c4c, stored as it is.
inline constexpr std::array<float, 4> Background{76.0f / 255.0f, 76.0f / 255.0f, 76.0f / 255.0f, 1.0f};

// What only the scene's own code uses.
namespace scene_detail {

// The car's five materials, then the sphere's, as
// SIMPLE_PAINT_MATERIAL_COUNT in SpinningCarTestLibrary.vcxproj says.
inline constexpr size_t MaterialCount = 6;

} // namespace scene_detail

// Where the car is along X, and how far it has turned, `seconds` after the
// program started. It goes from the middle to +7, back to -7, and again, 28
// units a cycle. S makes it `stationary`.
std::pair<float, float> car_placement(double seconds, bool stationary);

// Where a point of the world shows in a frame of the size, in pixels from
// its top left corner.
std::array<float, 2> to_screen(std::array<float, 3> point, float width, float height);

// The rectangle that the car covers in a frame of the size, where
// car_placement put it.
user_interface::Rect car_on_screen(std::pair<float, float> placement, float width, float height);

class Scene {
public:
    Scene(graphics::Gpu& gpu, const settings::SceneSettings& settings);
    Scene(const Scene&) = delete;
    Scene& operator=(const Scene&) = delete;
    Scene(Scene&&) = default;
    Scene& operator=(Scene&&) = default;

    // Records the scene into the frame, over all of it, with the car where
    // car_placement put it.
    void draw(const graphics::Gpu& gpu, const graphics::Frame& frame, std::pair<float, float> placement);

private:
    simple_paint::direct3d12::Pipeline paint;
    simple_paint::direct3d12::Pipeline flat;
    simple_paint::direct3d12::Mesh car;
    simple_paint::direct3d12::Mesh sphere;
    simple_paint::direct3d12::Mesh ground;
    simple_paint::direct3d12::Mesh cube;
    // Compiled before the constants' buffer is made, so they're before it
    // here.
    std::array<simple_paint::GpuMaterial, scene_detail::MaterialCount> materials;
    // Each object's transforms, then the materials, then the flat shading,
    // each at a constant buffer's start. The GPU has finished with them when
    // a frame begins.
    graphics::UploadBuffer constants;
    graphics::DepthBuffer depth;
};

} // namespace spinning_car_test::scene
