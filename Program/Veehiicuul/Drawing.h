// Draws the 3D view every frame, over the whole window, before the interface
// draws its bars and cards over it: the background's color, then what's flat
// on the ground, without a depth buffer, as each part goes over the ones
// before (the terrain, then the track), and then the models, painted with
// SimplePaint, with one: the car, the parked vehicles and the decorations.
//
// Each model is copied to the GPU once, at the start. The terrain and the
// track change as the editors change them, so their meshes are in upload
// buffers, which are written again when they change. One frame is on the
// GPU at a time, so by the time a frame is drawn, the GPU is done with what
// the last one read, and the buffers can be written again.

#pragma once

#include "Base/Windows.h"
#include "Graphics/Graphics.h"
#include "Meshes/Meshes.h"
#include "SimplePaint/SimplePaint.h"
#include "Veehiicuul/Camera.h"
#include "Veehiicuul/Models.h"
#include "Veehiicuul/Terrain.h"
#include "Veehiicuul/Track.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <d3d12.h>
#include <optional>
#include <span>
#include <utility>
#include <vector>

namespace veehiicuul::drawing {

// The most models a frame draws.
inline constexpr size_t MostInstances = 512;

// A model drawn somewhere: turned to `heading`, in radians from +Z toward
// +X, and scaled the same along every axis, with its slots' materials.
struct Instance {
    size_t model;
    std::array<double, 2> position;
    double heading;
    double scale;
    std::vector<simple_paint::Parameters> materials;

    Instance(size_t model_index, std::array<double, 2> at, double facing, double size, std::vector<simple_paint::Parameters> slot_materials);
    // An instance is made for a frame's view, and never copied.
    Instance(const Instance&) = delete;
    Instance& operator=(const Instance&) = delete;
    Instance(Instance&&) = default;
    Instance& operator=(Instance&&) = default;
};

// What's flat on the ground.
struct Ground {
    const terrain::Terrain& terrain;
    const track::Track& track;
    const track::Shape& shape;
};

// What a frame's 3D view shows.
struct View {
    camera::Eye eye;
    // As sRGB, or nothing to leave the frame as it's cleared, to the theme's
    // background.
    std::optional<std::array<float, 3>> background;
    std::optional<Ground> ground;
    std::span<const Instance> instances;
};

class Drawing {
public:
    // The pipelines, and the models' meshes, copied to the GPU.
    Drawing(graphics::Gpu& gpu, const models::Models& game_models);
    Drawing(const Drawing&) = delete;
    Drawing& operator=(const Drawing&) = delete;
    Drawing(Drawing&&) = default;
    Drawing& operator=(Drawing&&) = default;

    // Records the view's draws into the frame, over the whole of it, and
    // leaves its target in the render target state.
    void draw(const graphics::Gpu& gpu, const graphics::Frame& frame, const View& view);

private:
    // A mesh that changes, in an upload buffer, which the GPU reads where it is.
    struct Changing {
        std::optional<graphics::UploadBuffer> buffer;
        size_t vertex_bytes = 0;
        std::uint32_t index_count = 0;

        void set(ID3D12Device* device, const meshes::Mesh& mesh);

        void draw(ID3D12GraphicsCommandList* list, D3D12_GPU_VIRTUAL_ADDRESS object, D3D12_GPU_VIRTUAL_ADDRESS colors) const;
    };

    simple_paint::direct3d12::Pipeline paint;
    simple_paint::direct3d12::Pipeline ground;
    // Each model's mesh, in the models' order.
    std::vector<simple_paint::direct3d12::Mesh> models;
    Changing terrain;
    Changing track;
    // What the terrain's and the track's meshes were made from.
    std::optional<terrain::Terrain> terrain_of;
    std::optional<track::Track> track_of;
    graphics::UploadBuffer constants;
    // Each instance's materials as of the last frame, and as the GPU takes
    // them, so that they're compiled again only when they change.
    std::vector<std::pair<std::vector<simple_paint::Parameters>, std::array<simple_paint::GpuMaterial, models::MostSlots>>> compiled;
    graphics::DepthBuffer depth;
};

} // namespace veehiicuul::drawing
