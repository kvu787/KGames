#include "MinimalSceneTest/Scene.h"

#include "Base/Panic.h"
#include "Base/Windows.h"
#include "Graphics/Error.h"
#include "Math/Math.h"
#include "Meshes/Meshes.h"

// SimplePaint's shaders, which MinimalSceneTestLibrary.vcxproj names:
// SimplePaintVertex and SimplePaintPixel.
#include "SimplePaintPixel.h"
#include "SimplePaintVertex.h"

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <expected>
#include <format>
#include <utility>

namespace minimal_scene_test::scene {

using graphics::Frame;
using graphics::Gpu;
using math::Matrix;
using settings::SceneSettings;
using simple_paint::GpuMaterial;
using simple_paint::Material;
using simple_paint::Parameters;
using simple_paint::direct3d12::materials_size;
using simple_paint::direct3d12::Mesh;
using simple_paint::direct3d12::ObjectSize;
using simple_paint::direct3d12::Target;
using simple_paint::orthographic::build_object_transforms;
using simple_paint::orthographic::make_projection;
using simple_paint::orthographic::ObjectTransforms;
using simple_paint::orthographic::Projection;

namespace {

// The sphere's units per second.
constexpr float Speed = 8.0f;

// The camera looks at the origin from 5 units away, 47.5 degrees above the
// horizon and 30 degrees from +Z toward +X: an orthographic view 5 units
// tall, from 1 to 20 units in front of it.
constexpr std::array<float, 3> CameraBackward{0.3377951f, 0.7372773f, 0.5850783f};
constexpr std::array<float, 3> CameraUp{-0.36863866f, 0.6755902f, -0.63850087f};
constexpr float CameraDistance = 5.0f;

// What SimplePaint rejected, as the failure that drawing gives.
graphics::Error invalid(const simple_paint::Error& error) {
    return graphics::Error(E_INVALIDARG, error.message);
}

// The camera's view and projection, for a frame of the size.
std::pair<Matrix, Projection> camera(float width, float height) {
    std::expected<Projection, simple_paint::Error> projection = make_projection(ViewHeight * width / height, ViewHeight, 1.0f, 20.0f);
    if (!projection) {
        throw invalid(projection.error());
    }
    std::array<float, 3> eye{CameraBackward[0] * CameraDistance, CameraBackward[1] * CameraDistance, CameraBackward[2] * CameraDistance};
    return {Matrix::look_at_rh(eye, {0.0f, 0.0f, 0.0f}, CameraUp), *projection};
}

// A mesh, copied to the GPU.
Mesh upload(Gpu& gpu, const meshes::Mesh& mesh, size_t material_count) {
    return Mesh(gpu, mesh.vertices, mesh.indices, static_cast<std::uint32_t>(material_count));
}

// A material's constants, as the GPU takes them.
GpuMaterial compile(const Parameters& parameters) {
    std::expected<Material, simple_paint::Error> material = Material::compile(parameters);
    if (!material) {
        throw invalid(material.error());
    }
    return material->constants();
}

} // namespace

void Motion::advance_frame(float seconds) {
    // std::fmin, which gives the longest frame for a time that isn't a
    // number, as f32::min does: std::min would give the NaN back.
    advance(std::fmin(seconds, LongestFrame));
}

void Motion::advance(float seconds) {
    position += direction * Speed * seconds;
    while (std::abs(position) > TravelDistance) {
        position = 2.0f * std::copysign(TravelDistance, position) - position;
        direction = -direction;
    }
}

std::array<float, 2> to_screen(std::array<float, 3> point, float width, float height) {
    try {
        auto [view, projection] = camera(width, height);
        std::array<float, 3> seen = view.transform_point(point);
        return {
            (1.0f + seen[0] * projection.scale[0]) * width / 2.0f,
            (1.0f - seen[1] * projection.scale[1]) * height / 2.0f,
        };
    } catch (const base::WindowsError& error) {
        base::panic(std::format("the camera fits any frame: {}", error.what()));
    }
}

std::array<float, 3> sphere_center(float x) {
    return {x, SphereRadius, 0.0f};
}

Scene::Scene(Gpu& gpu, const SceneSettings& settings) :
    pipeline(gpu.device.Get(), SimplePaintVertex, SimplePaintPixel,
             Target{
                 .format = graphics::TargetSrgbFormat,
                 .depth_format = graphics::DepthFormat,
                 .sample_count = 1,
                 .cull_mode = D3D12_CULL_MODE_NONE,
             }),
    meshes{
        upload(gpu, ::meshes::uv_sphere(64, 32, 0), MaterialCount),
        upload(gpu, ::meshes::plane({10.0f, 5.0f}, 1), MaterialCount),
        upload(gpu, ::meshes::cuboid({1.0f, 1.0f, 1.0f}, 2), MaterialCount),
    },
    constants(gpu.device.Get(), ObjectSize * ObjectCount + materials_size(MaterialCount)),
    materials{compile(settings.sphere), compile(settings.ground), compile(settings.cube)},
    depth(gpu) {}

void Scene::draw(const Gpu& gpu, const Frame& frame, float sphere_x) {
    auto [view, projection] = camera(static_cast<float>(frame.width), static_cast<float>(frame.height));
    std::array<Matrix, ObjectCount> worlds{
        Matrix::scaling(SphereRadius, SphereRadius, SphereRadius) * Matrix::translation(sphere_x, SphereRadius, 0.0f),
        Matrix::Identity,
        Matrix::translation(0.0f, 0.5f, -1.5f),
    };
    for (size_t index = 0; index < worlds.size(); index++) {
        std::expected<ObjectTransforms, simple_paint::Error> object = build_object_transforms(worlds[index] * view, projection);
        if (!object) {
            throw invalid(object.error());
        }
        constants.write(ObjectSize * index, {*object});
    }
    size_t materials_offset = ObjectSize * ObjectCount;
    constants.write(materials_offset, materials);

    frame.clear(Background);
    D3D12_CPU_DESCRIPTOR_HANDLE depth_view = depth.clear(gpu, frame);
    // SimplePaint's colors are linear, which the sRGB view encodes.
    frame.bind(true, depth_view);
    pipeline.bind(frame.list.Get());
    for (size_t index = 0; index < meshes.size(); index++) {
        meshes[index].draw(frame.list.Get(), constants.gpu_address(ObjectSize * index), constants.gpu_address(materials_offset));
    }
}

} // namespace minimal_scene_test::scene
