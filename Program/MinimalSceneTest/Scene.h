// The scene, drawn with SimplePaint: a sphere that moves back and forth along
// X, above a ground plane, past a cube, seen by an orthographic camera from
// above and to the side.

#pragma once

#include "Graphics/Graphics.h"
#include "MinimalSceneTest/Settings.h"
#include "SimplePaint/SimplePaint.h"

#include <array>
#include <cstddef>

namespace minimal_scene_test::scene {

// How far from the middle the sphere turns back.
inline constexpr float TravelDistance = 7.0f;
// Of a frame's time, at most this many seconds count, so that the sphere
// doesn't jump ahead when frames stopped for a while, like while the window
// was minimized, or showed what's wrong with its setup instead of the scene.
inline constexpr float LongestFrame = 0.1f;
// Its mesh has 64 vertices around each ring, and 32 bands from pole to pole.
inline constexpr float SphereRadius = 0.5f;

// The camera's orthographic view is this many units tall.
inline constexpr float ViewHeight = 5.0f;

// #4c4c4c, stored as it is.
inline constexpr std::array<float, 4> Background{76.0f / 255.0f, 76.0f / 255.0f, 76.0f / 255.0f, 1.0f};

// Where the sphere is along X, and which way it's moving: +1 or -1.
struct Motion {
    // The sphere starts in the middle, moving toward +X.
    float position = 0.0f;
    float direction = 1.0f;

    // Moves the sphere by a frame's time since the last frame, of which
    // LongestFrame at most counts.
    void advance_frame(float seconds);

    // Moves the sphere by `seconds`. Past the travel distance, it bounces
    // back by as much as it went past, as many times as the time takes it,
    // so that no time takes it off its path.
    void advance(float seconds);

    bool operator==(const Motion&) const = default;
};

// Where a point of the world shows in a frame of the size, in pixels from
// its top left corner.
std::array<float, 2> to_screen(std::array<float, 3> point, float width, float height);

// Where the sphere's middle is, with it at `x` along X.
std::array<float, 3> sphere_center(float x);

class Scene {
public:
    Scene(graphics::Gpu& gpu, const settings::SceneSettings& settings);
    Scene(const Scene&) = delete;
    Scene& operator=(const Scene&) = delete;
    Scene(Scene&&) = default;
    Scene& operator=(Scene&&) = default;

    // Records the scene into the frame, over all of it, with the sphere at
    // `sphere_x` along X.
    void draw(const graphics::Gpu& gpu, const graphics::Frame& frame, float sphere_x);

private:
    // The shaders' SIMPLE_PAINT_MATERIAL_COUNT, from
    // MinimalSceneTestLibrary.vcxproj: the sphere's, the ground's and the
    // cube's, in that order.
    static constexpr size_t MaterialCount = 3;
    static constexpr size_t ObjectCount = 3;

    simple_paint::direct3d12::Pipeline pipeline;
    // The sphere, the ground and the cube.
    std::array<simple_paint::direct3d12::Mesh, ObjectCount> meshes;
    // Each object's transforms, then the materials, each at a constant
    // buffer's start. The GPU has finished with them when a frame begins.
    graphics::UploadBuffer constants;
    std::array<simple_paint::GpuMaterial, MaterialCount> materials;
    graphics::DepthBuffer depth;
};

} // namespace minimal_scene_test::scene
