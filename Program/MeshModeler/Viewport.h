// Draws the view: the mesh, painted with SimplePaint or flat-lit, into the
// view's part of the frame through its sRGB view, with a depth buffer; then
// the overlay over it, the cage's edges and vertices, selected faces and the
// floor's grid, through the frame's plain view, tested against the mesh's
// depth, and the overlay that shows through the mesh, untested.
//
// The mesh's triangles go into an upload buffer only when they change; the
// constants and the overlay go into another every frame. The main loop starts
// a frame only once the GPU has finished the last, so rewriting them before
// drawing is safe.

#pragma once

#include "Base/Windows.h"
#include "Graphics/Graphics.h"
#include "Math/Math.h"
#include "MeshModeler/Paint.h"
#include "PolygonMesh/PolygonMesh.h"
#include "SimplePaint/Direct3D12.h"
#include "SimplePaint/Material.h"
#include "SimplePaint/Orthographic.h"
#include "UserInterface/Drawing/Geometry.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <d3d12.h>
#include <optional>
#include <string_view>
#include <vector>
#include <wrl/client.h>

namespace mesh_modeler::viewport {

enum class Shader {
    SimplePaint,
    Flat,
};

std::string_view name(Shader shader);

// What the overlay draws, laid out as Overlay.hlsl's Item: a line from a to
// b, a dot at a, or a triangle, `width` pixels wide, in a premultiplied sRGB
// color.
struct OverlayItem {
    std::array<float, 3> a{};
    float width = 0.0f;
    std::array<float, 3> b{};
    std::uint32_t color = 0;
    std::array<float, 3> c{};
    std::uint32_t kind = 0;
};

static_assert(sizeof(OverlayItem) == 48);

inline constexpr std::uint32_t KindLine = 0;
inline constexpr std::uint32_t KindPoint = 1;
inline constexpr std::uint32_t KindTriangle = 2;

// What a frame of the view draws, which the program updates as it builds the
// interface.
struct Scene {
    // Where the view is in the window.
    user_interface::Rect view = user_interface::Rect::Zero;
    // Its background, sRGB.
    std::array<float, 4> background{};
    simple_paint::orthographic::ObjectTransforms transforms;
    math::Matrix world_to_clip = math::Matrix::Identity;
    Shader shader = Shader::SimplePaint;
    // The mesh's triangles, and the document's version that they're of.
    polygon_mesh::Triangles triangles;
    std::uint64_t version = 0;
    std::array<simple_paint::GpuMaterial, paint::MaximumPaints> materials{};
    // The flat shader's: the paints' linear colors, and the light toward the
    // camera's upper left, with how much of the color is ambient as w.
    std::array<std::array<float, 4>, paint::MaximumPaints> colors{};
    std::array<float, 4> light{};
    std::vector<OverlayItem> overlay;
    std::vector<OverlayItem> xray;

    Scene() = default;
    Scene(const Scene&) = delete;
    Scene& operator=(const Scene&) = delete;
    Scene(Scene&&) = default;
    Scene& operator=(Scene&&) = default;
};

class Viewport {
public:
    explicit Viewport(const graphics::Gpu& gpu);
    Viewport(const Viewport&) = delete;
    Viewport& operator=(const Viewport&) = delete;
    Viewport(Viewport&&) = default;
    Viewport& operator=(Viewport&&) = default;

    // Draws the scene into its part of the frame.
    void draw(const graphics::Gpu& gpu, const graphics::Frame& frame, const Scene& scene);

private:
    struct OverlayPipeline {
        Microsoft::WRL::ComPtr<ID3D12RootSignature> root_signature;
        Microsoft::WRL::ComPtr<ID3D12PipelineState> tested;
        Microsoft::WRL::ComPtr<ID3D12PipelineState> untested;
    };

    static OverlayPipeline overlay_pipeline(ID3D12Device* device);

    // Copies the mesh's triangles into the geometry buffer, if they changed.
    void upload_geometry(const graphics::Gpu& gpu, const Scene& scene);

    simple_paint::direct3d12::Pipeline paint;
    simple_paint::direct3d12::Pipeline flat;
    OverlayPipeline overlay;
    graphics::DepthBuffer depth;
    std::optional<graphics::UploadBuffer> geometry;
    std::uint64_t geometry_version = 0;
    std::uint32_t index_count = 0;
    size_t vertex_bytes = 0;
    std::optional<graphics::UploadBuffer> frame_data;
};

} // namespace mesh_modeler::viewport
