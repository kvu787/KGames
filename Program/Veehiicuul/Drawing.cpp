#include "Veehiicuul/Drawing.h"

#include "Graphics/Error.h"
#include "Math/Math.h"
#include "Veehiicuul/Ground.h"

// The shaders, which VeehiicuulLibrary.vcxproj names: SimplePaintVertex,
// SimplePaintPixel, GroundVertex and GroundPixel.
#include "GroundPixel.h"
#include "GroundVertex.h"
#include "SimplePaintPixel.h"
#include "SimplePaintVertex.h"

#include <algorithm>
#include <cmath>
#include <expected>

namespace veehiicuul::drawing {

using graphics::Frame;
using graphics::Gpu;
using graphics::UploadBuffer;
using math::Matrix;
using models::Models;
using models::MostSlots;
using simple_paint::GpuMaterial;
using simple_paint::Material;
using simple_paint::Parameters;
using simple_paint::Vertex;
using simple_paint::direct3d12::ConstantBufferAlignment;
using simple_paint::direct3d12::materials_size;
using simple_paint::direct3d12::Mesh;
using simple_paint::direct3d12::ObjectSize;
using simple_paint::direct3d12::Pipeline;
using simple_paint::direct3d12::Target;
using simple_paint::orthographic::build_object_transforms;
using simple_paint::orthographic::make_projection;
using simple_paint::orthographic::ObjectTransforms;
using simple_paint::orthographic::Projection;
using Triangles = meshes::Mesh;

namespace {

// A model's materials, as SIMPLE_PAINT_MATERIAL_COUNT in
// VeehiicuulLibrary.vcxproj says.
constexpr size_t MaterialsSize = materials_size(MostSlots);
// The ground's colors, as Ground.hlsl takes them.
constexpr size_t ColorsSize = (sizeof(std::array<std::array<float, 4>, ground::ColorCount>) + ConstantBufferAlignment - 1) / ConstantBufferAlignment * ConstantBufferAlignment;
// The constants start with the ground's transforms and colors, then each
// model drawn's transforms and materials.
constexpr size_t ColorsOffset = ObjectSize;
constexpr size_t InstancesOffset = ColorsOffset + ColorsSize;
constexpr size_t InstanceSize = ObjectSize + MaterialsSize;

graphics::Error invalid(const simple_paint::Error& error) {
    return graphics::Error(E_INVALIDARG, error.message);
}

// An sRGB channel, as linear.
float linear(float channel) {
    if (channel <= 0.04045f) {
        return channel / 12.92f;
    }
    return std::pow((channel + 0.055f) / 1.055f, 2.4f);
}

// The models' linear colors, and the ground's, are encoded to sRGB,
// through the frame's sRGB view.
Target target(std::optional<DXGI_FORMAT> depth_format) {
    return Target{
        .format = graphics::TargetSrgbFormat,
        .depth_format = depth_format,
        .sample_count = 1,
        .cull_mode = D3D12_CULL_MODE_NONE,
    };
}

// Each model's mesh, copied to the GPU.
std::vector<Mesh> model_meshes(Gpu& gpu, const Models& game_models) {
    std::vector<Mesh> copied;
    copied.reserve(game_models.list.size());
    for (const models::Model& model : game_models.list) {
        copied.emplace_back(gpu, model.vertices, model.indices, static_cast<std::uint32_t>(MostSlots));
    }
    return copied;
}

} // namespace

Instance::Instance(size_t model_index, std::array<double, 2> at, double facing, double size, std::vector<Parameters> slot_materials) :
    model(model_index), position(at), heading(facing), scale(size), materials(std::move(slot_materials)) {}

void Drawing::Changing::set(ID3D12Device* device, const Triangles& mesh) {
    size_t mesh_vertex_bytes = mesh.vertices.size() * sizeof(Vertex);
    size_t size = mesh_vertex_bytes + mesh.indices.size() * sizeof(std::uint32_t);
    if (!buffer || buffer->size < size) {
        // Room to grow, so that painting doesn't make one each time.
        buffer = UploadBuffer(device, std::max(size * 2, size_t{65536}));
    }
    buffer->write(0, mesh.vertices);
    buffer->write(mesh_vertex_bytes, mesh.indices);
    vertex_bytes = mesh_vertex_bytes;
    index_count = static_cast<std::uint32_t>(mesh.indices.size());
}

void Drawing::Changing::draw(ID3D12GraphicsCommandList* list, D3D12_GPU_VIRTUAL_ADDRESS object, D3D12_GPU_VIRTUAL_ADDRESS colors) const {
    if (!buffer) {
        return;
    }
    if (index_count == 0) {
        return;
    }
    list->SetGraphicsRootConstantBufferView(0, object);
    list->SetGraphicsRootConstantBufferView(1, colors);
    D3D12_VERTEX_BUFFER_VIEW vertex_view{
        .BufferLocation = buffer->gpu_address(0),
        .SizeInBytes = static_cast<UINT>(vertex_bytes),
        .StrideInBytes = static_cast<UINT>(sizeof(Vertex)),
    };
    list->IASetVertexBuffers(0, 1, &vertex_view);
    D3D12_INDEX_BUFFER_VIEW index_view{
        .BufferLocation = buffer->gpu_address(vertex_bytes),
        .SizeInBytes = index_count * 4,
        .Format = DXGI_FORMAT_R32_UINT,
    };
    list->IASetIndexBuffer(&index_view);
    list->DrawIndexedInstanced(index_count, 1, 0, 0, 0);
}

Drawing::Drawing(Gpu& gpu, const Models& game_models) :
    paint(gpu.device.Get(), SimplePaintVertex, SimplePaintPixel, target(graphics::DepthFormat)),
    ground(gpu.device.Get(), GroundVertex, GroundPixel, target(std::nullopt)),
    models(model_meshes(gpu, game_models)),
    constants(gpu.device.Get(), InstancesOffset + InstanceSize * MostInstances),
    depth(gpu) {}

void Drawing::draw(const Gpu& gpu, const Frame& frame, const View& view) {
    ID3D12Device* device = gpu.device.Get();
    D3D12_CPU_DESCRIPTOR_HANDLE depth_view = depth.clear(gpu, frame);
    Matrix eye_view = view.eye.view();
    float aspect = static_cast<float>(frame.width) / static_cast<float>(frame.height);
    float height = static_cast<float>(view.eye.height);
    std::expected<Projection, simple_paint::Error> projection = make_projection(height * aspect, height, camera::Near, camera::Far);
    if (!projection) {
        throw invalid(projection.error());
    }
    auto transforms = [&](const Matrix& world) {
        std::expected<ObjectTransforms, simple_paint::Error> built = build_object_transforms(world * eye_view, *projection);
        if (!built) {
            throw invalid(built.error());
        }
        return *built;
    };

    ID3D12GraphicsCommandList* list = frame.list.Get();
    if (view.background) {
        std::array<float, 3> color = *view.background;
        frame.clear({color[0], color[1], color[2], 1.0f});
    }
    if (view.ground) {
        const Ground& flat = *view.ground;
        if (terrain_of != flat.terrain) {
            terrain.set(device, ground::terrain_mesh(flat.terrain));
            terrain_of = flat.terrain;
        }
        if (track_of != flat.track) {
            track.set(device, ground::track_mesh(flat.track, flat.shape));
            track_of = flat.track;
        }
        constants.write(0, {transforms(Matrix::Identity)});
        std::array<std::array<float, 3>, ground::ColorCount> srgb_colors = ground::colors(flat.terrain, flat.track);
        std::array<std::array<float, 4>, ground::ColorCount> colors{};
        for (size_t index = 0; index < colors.size(); index++) {
            colors[index] = {linear(srgb_colors[index][0]), linear(srgb_colors[index][1]), linear(srgb_colors[index][2]), 1.0f};
        }
        constants.write(ColorsOffset, colors);
        D3D12_GPU_VIRTUAL_ADDRESS object = constants.gpu_address(0);
        D3D12_GPU_VIRTUAL_ADDRESS colors_address = constants.gpu_address(ColorsOffset);
        frame.bind(true, std::nullopt);
        ground.bind(list);
        terrain.draw(list, object, colors_address);
        track.draw(list, object, colors_address);
    }

    frame.bind(true, depth_view);
    paint.bind(list);
    size_t shown = std::min(view.instances.size(), MostInstances);
    for (size_t index = 0; index < shown; index++) {
        const Instance& instance = view.instances[index];
        if (instance.model >= models.size()) {
            continue;
        }
        const Mesh& mesh = models[instance.model];
        float scale = static_cast<float>(instance.scale);
        float x = static_cast<float>(instance.position[0]);
        float z = static_cast<float>(instance.position[1]);
        Matrix world = Matrix::scaling(scale, scale, scale) * Matrix::rotation_y(static_cast<float>(instance.heading)) * Matrix::translation(x, 0.0f, z);
        size_t offset = InstancesOffset + InstanceSize * index;
        constants.write(offset, {transforms(world)});
        if (compiled.size() <= index) {
            compiled.emplace_back(std::vector<Parameters>(), std::array<GpuMaterial, MostSlots>{});
        }
        auto& [parameters, materials] = compiled[index];
        if (parameters != instance.materials) {
            for (size_t slot = 0; slot < std::min(materials.size(), instance.materials.size()); slot++) {
                std::expected<Material, simple_paint::Error> material = Material::compile(instance.materials[slot]);
                if (!material) {
                    throw invalid(material.error());
                }
                materials[slot] = material->constants();
            }
            parameters = instance.materials;
        }
        constants.write(offset + ObjectSize, materials);
        mesh.draw(list, constants.gpu_address(offset), constants.gpu_address(offset + ObjectSize));
    }
}

} // namespace veehiicuul::drawing
