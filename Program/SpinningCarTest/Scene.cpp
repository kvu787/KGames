#include "SpinningCarTest/Scene.h"

#include "Base/Panic.h"
#include "Base/Windows.h"
#include "Graphics/Error.h"
#include "Math/Math.h"
#include "Meshes/Meshes.h"

// The shaders, which SpinningCarTestLibrary.vcxproj names: SimplePaintVertex,
// SimplePaintPixel, FlatShadingVertex and FlatShadingPixel.
#include "FlatShadingPixel.h"
#include "FlatShadingVertex.h"
#include "SimplePaintPixel.h"
#include "SimplePaintVertex.h"

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <expected>
#include <format>
#include <limits>
#include <numbers>

namespace spinning_car_test::scene {

using graphics::Frame;
using graphics::Gpu;
using math::Matrix;
using scene_detail::MaterialCount;
using settings::SceneSettings;
using simple_paint::GpuMaterial;
using simple_paint::Material;
using simple_paint::Parameters;
using simple_paint::direct3d12::ConstantBufferAlignment;
using simple_paint::direct3d12::materials_size;
using simple_paint::direct3d12::Mesh;
using simple_paint::direct3d12::ObjectSize;
using simple_paint::direct3d12::Target;
using simple_paint::orthographic::build_object_transforms;
using simple_paint::orthographic::make_projection;
using simple_paint::orthographic::ObjectTransforms;
using simple_paint::orthographic::Projection;
using user_interface::Rect;
namespace car_mesh = meshes::car;

namespace {

// The sphere's material, after the car's five.
constexpr std::uint32_t SphereMaterial = car_mesh::MaterialCount;
// The ground's color, then the cube's, as FlatShading.hlsl's MaterialCount
// says.
constexpr size_t FlatMaterialCount = 2;
// The car, the sphere, the ground and the cube.
constexpr size_t ObjectCount = 4;

// What FlatShading.hlsl binds at b1.
struct Shading {
    // Toward the light, in view space. W isn't used.
    std::array<float, 4> light;
    std::array<std::array<float, 4>, FlatMaterialCount> colors;
};

// Where the constants have the materials, after each object's transforms,
// and the flat shading, after the materials.
constexpr size_t MaterialsOffset = ObjectSize * ObjectCount;
constexpr size_t ShadingOffset = MaterialsOffset + materials_size(MaterialCount);
constexpr size_t ShadingSize = (sizeof(Shading) + ConstantBufferAlignment - 1) / ConstantBufferAlignment * ConstantBufferAlignment;
// The camera looks down at the origin from 45 degrees above the horizon, and
// 45 degrees from +Z toward +X, 5 units away, at a view 5 units tall.
constexpr float CameraPitch = std::numbers::pi_v<float> / 4.0f;
constexpr float CameraYaw = std::numbers::pi_v<float> / 4.0f;
constexpr float CameraDistance = 5.0f;
constexpr float CarScale = 0.25f;
// Units per second, and how far from the middle the car turns back.
constexpr double Speed = 8.0;
constexpr double TravelDistance = 7.0;
// Radians per second.
constexpr double TurnSpeed = std::numbers::pi / 2.0;
// Along X and Z.
constexpr std::array<float, 2> GroundSize{10.0f, 5.0f};
constexpr float CubeSize = 1.0f;
// sRGB, like the C++ scene's.
constexpr std::array<float, 4> GroundColor{0.08900002f, 0.89f, 0.48950002f, 1.0f};
constexpr std::array<float, 4> CubeColor{0.88f, 0.1672f, 0.17907982f, 1.0f};
// Toward the light, which is above, between +X and +Z, as a unit vector.
constexpr std::array<float, 3> Light{0.5f, std::numbers::sqrt2_v<float> / 2.0f, 0.5f};

// What SimplePaint rejected, as the failure that drawing gives.
graphics::Error invalid(const simple_paint::Error& error) {
    return graphics::Error(E_INVALIDARG, error.message);
}

Matrix car_world(std::pair<float, float> placement) {
    auto [position, turn] = placement;
    return Matrix::scaling(CarScale, CarScale, CarScale) * Matrix::rotation_y(turn) * Matrix::translation(position, 0.0f, 0.0f);
}

// The camera's view and projection, for a frame of the size.
std::pair<Matrix, Projection> camera(float width, float height) {
    std::array<float, 3> eye{
        CameraDistance * std::cos(CameraPitch) * std::sin(CameraYaw),
        CameraDistance * std::sin(CameraPitch),
        CameraDistance * std::cos(CameraPitch) * std::cos(CameraYaw),
    };
    Matrix view = Matrix::look_at_rh(eye, {0.0f, 0.0f, 0.0f}, {0.0f, 1.0f, 0.0f});
    std::expected<Projection, simple_paint::Error> projection = make_projection(ViewHeight * width / height, ViewHeight, 1.0f, 20.0f);
    if (!projection) {
        throw invalid(projection.error());
    }
    return {view, *projection};
}

// What a pipeline of the scene draws into, in the format.
Target target(DXGI_FORMAT format) {
    return Target{
        .format = format,
        .depth_format = graphics::DepthFormat,
        .sample_count = 1,
        .cull_mode = D3D12_CULL_MODE_NONE,
    };
}

// A mesh, copied to the GPU.
Mesh upload(Gpu& gpu, const meshes::Mesh& mesh, size_t material_count) {
    return Mesh(gpu, mesh.vertices, mesh.indices, static_cast<std::uint32_t>(material_count));
}

// The materials' constants, as the GPU takes them.
std::array<GpuMaterial, MaterialCount> compile(const std::array<Parameters, MaterialCount>& parameters) {
    std::array<GpuMaterial, MaterialCount> materials{};
    for (size_t index = 0; index < materials.size(); index++) {
        std::expected<Material, simple_paint::Error> material = Material::compile(parameters[index]);
        if (!material) {
            throw invalid(material.error());
        }
        materials[index] = material->constants();
    }
    return materials;
}

} // namespace

std::pair<float, float> car_placement(double seconds, bool stationary) {
    if (stationary) {
        // The mesh faces +Z. Facing +X puts the headlights at the lower
        // right in the fixed 45-degree camera.
        return {0.0f, std::numbers::pi_v<float> / 2.0f};
    }
    double phase = std::fmod(seconds * Speed + TravelDistance, TravelDistance * 4.0);
    double position = TravelDistance - std::abs(phase - TravelDistance * 2.0);
    double turn = std::fmod(seconds * TurnSpeed, 2.0 * std::numbers::pi);
    return {static_cast<float>(position), static_cast<float>(turn)};
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

Rect car_on_screen(std::pair<float, float> placement, float width, float height) {
    Matrix world = car_world(placement);
    float left = std::numeric_limits<float>::max();
    float top = std::numeric_limits<float>::max();
    float right = std::numeric_limits<float>::lowest();
    float bottom = std::numeric_limits<float>::lowest();
    for (const simple_paint::Vertex& vertex : car_mesh::Vertices) {
        std::array<float, 2> at = to_screen(world.transform_point(vertex.position), width, height);
        left = std::min(left, at[0]);
        top = std::min(top, at[1]);
        right = std::max(right, at[0]);
        bottom = std::max(bottom, at[1]);
    }
    return Rect(left, top, right - left, bottom - top);
}

Scene::Scene(Gpu& gpu, const SceneSettings& settings) :
    paint(gpu.device.Get(), SimplePaintVertex, SimplePaintPixel, target(graphics::TargetSrgbFormat)),
    // The flat colors are already sRGB, so they're stored as they are.
    flat(gpu.device.Get(), FlatShadingVertex, FlatShadingPixel, target(graphics::TargetFormat)),
    car(gpu, car_mesh::Vertices, car_mesh::Indices, static_cast<std::uint32_t>(MaterialCount)),
    sphere(upload(gpu, meshes::uv_sphere(settings.sphere_resolution[0], settings.sphere_resolution[1], SphereMaterial), MaterialCount)),
    ground(upload(gpu, meshes::plane(GroundSize, 0), FlatMaterialCount)),
    cube(upload(gpu, meshes::cuboid({CubeSize, CubeSize, CubeSize}, 1), FlatMaterialCount)),
    materials(compile(settings.materials)),
    constants(gpu.device.Get(), ShadingOffset + ShadingSize),
    depth(gpu) {}

void Scene::draw(const Gpu& gpu, const Frame& frame, std::pair<float, float> placement) {
    auto [view, projection] = camera(static_cast<float>(frame.width), static_cast<float>(frame.height));
    std::array<Matrix, ObjectCount> worlds{
        car_world(placement),
        Matrix::scaling(SphereRadius, SphereRadius, SphereRadius) * Matrix::translation(SphereCenter[0], SphereCenter[1], SphereCenter[2]),
        Matrix::Identity,
        Matrix::translation(0.0f, CubeSize * 0.5f, -1.5f),
    };
    for (size_t index = 0; index < worlds.size(); index++) {
        std::expected<ObjectTransforms, simple_paint::Error> object = build_object_transforms(worlds[index] * view, projection);
        if (!object) {
            throw invalid(object.error());
        }
        constants.write(ObjectSize * index, {*object});
    }
    constants.write(MaterialsOffset, materials);
    std::array<float, 3> toward = view.transform_direction(Light);
    Shading shading{
        .light = {toward[0], toward[1], toward[2], 0.0f},
        .colors = {GroundColor, CubeColor},
    };
    constants.write(ShadingOffset, {shading});

    frame.clear(Background);
    D3D12_CPU_DESCRIPTOR_HANDLE depth_view = depth.clear(gpu, frame);
    ID3D12GraphicsCommandList* list = frame.list.Get();
    auto object = [&](size_t index) {
        return constants.gpu_address(ObjectSize * index);
    };
    // The ground and the cube, in their flat colors.
    frame.bind(false, depth_view);
    flat.bind(list);
    D3D12_GPU_VIRTUAL_ADDRESS shading_address = constants.gpu_address(ShadingOffset);
    ground.draw(list, object(2), shading_address);
    cube.draw(list, object(3), shading_address);
    // The car and the sphere, whose linear colors are encoded to sRGB, in
    // front of or behind them.
    frame.bind(true, depth_view);
    paint.bind(list);
    D3D12_GPU_VIRTUAL_ADDRESS materials_address = constants.gpu_address(MaterialsOffset);
    car.draw(list, object(0), materials_address);
    sphere.draw(list, object(1), materials_address);
}

} // namespace spinning_car_test::scene
