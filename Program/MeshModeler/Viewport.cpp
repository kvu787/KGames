#include "MeshModeler/Viewport.h"

#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "SimplePaint/Geometry.h"

// The shaders, which MeshModelerLibrary.vcxproj names: SimplePaintVertex,
// SimplePaintPixel, FlatVertex, FlatPixel, OverlayVertex and OverlayPixel.
#include "FlatPixel.h"
#include "FlatVertex.h"
#include "OverlayPixel.h"
#include "OverlayVertex.h"
#include "SimplePaintPixel.h"
#include "SimplePaintVertex.h"

#include <algorithm>
#include <initializer_list>
#include <limits>

namespace mesh_modeler::viewport {

using paint::MaximumPaints;
using simple_paint::direct3d12::materials_size;
using simple_paint::direct3d12::ObjectSize;
using simple_paint::direct3d12::Target;
using user_interface::Rect;

namespace {

constexpr size_t Alignment = 256;

// The flat shader's constants at b1.
struct FlatConstants {
    std::array<float, 4> light;
    std::array<std::array<float, 4>, MaximumPaints> colors;
};

// The least multiple of `multiple` that's at least the value:
// usize::next_multiple_of.
constexpr size_t next_multiple_of(size_t value, size_t multiple) {
    return (value + multiple - 1) / multiple * multiple;
}

// What the mesh's pipelines draw into.
Target mesh_target() {
    return Target{
        .format = graphics::TargetSrgbFormat,
        .depth_format = graphics::DepthFormat,
        .sample_count = 1,
        // Open meshes show their insides, which SimplePaint shades
        // as facing away.
        .cull_mode = D3D12_CULL_MODE_NONE,
    };
}

} // namespace

std::string_view name(Shader shader) {
    switch (shader) {
    case Shader::SimplePaint:
        return "SimplePaint";
    case Shader::Flat:
        return "Flat";
    }
    base::unreachable();
}

Viewport::Viewport(const graphics::Gpu& gpu) :
    paint(gpu.device.Get(), SimplePaintVertex, SimplePaintPixel, mesh_target()),
    flat(gpu.device.Get(), FlatVertex, FlatPixel, mesh_target()),
    overlay(overlay_pipeline(gpu.device.Get())),
    depth(gpu) {}

void Viewport::upload_geometry(const graphics::Gpu& gpu, const Scene& scene) {
    if (geometry_version == scene.version && geometry) {
        return;
    }
    size_t scene_vertex_bytes = scene.triangles.vertices.size() * sizeof(simple_paint::Vertex);
    size_t index_bytes = scene.triangles.indices.size() * sizeof(std::uint32_t);
    size_t needed = next_multiple_of(scene_vertex_bytes, Alignment) + std::max<size_t>(index_bytes, 4);
    if (!geometry || geometry->size < needed) {
        geometry = graphics::UploadBuffer(gpu.device.Get(), std::max<size_t>(needed * 3 / 2, size_t{1} << 16));
    }
    const graphics::UploadBuffer& buffer = *geometry;
    buffer.write(0, scene.triangles.vertices);
    buffer.write(next_multiple_of(scene_vertex_bytes, Alignment), scene.triangles.indices);
    vertex_bytes = scene_vertex_bytes;
    index_count = static_cast<std::uint32_t>(scene.triangles.indices.size());
    geometry_version = scene.version;
}

void Viewport::draw(const graphics::Gpu& gpu, const graphics::Frame& frame, const Scene& scene) {
    Rect view = scene.view.round();
    if (view.is_empty()) {
        return;
    }
    upload_geometry(gpu, scene);
    D3D12_CPU_DESCRIPTOR_HANDLE depth_view = depth.view(gpu, frame);

    // This frame's constants and overlay.
    FlatConstants flat_constants{
        .light = scene.light,
        .colors = scene.colors,
    };
    size_t transforms_at = 0;
    size_t materials_at = ObjectSize;
    size_t flat_at = materials_at + materials_size(MaximumPaints);
    size_t matrix_at = flat_at + next_multiple_of(sizeof(FlatConstants), Alignment);
    size_t overlay_at = matrix_at + Alignment;
    size_t overlay_bytes = next_multiple_of(scene.overlay.size() * sizeof(OverlayItem), Alignment);
    size_t xray_at = overlay_at + std::max(overlay_bytes, Alignment);
    size_t needed = xray_at + std::max(scene.xray.size() * sizeof(OverlayItem), Alignment);
    if (!frame_data || frame_data->size < needed) {
        frame_data = graphics::UploadBuffer(gpu.device.Get(), std::max<size_t>(needed * 3 / 2, size_t{1} << 16));
    }
    const graphics::UploadBuffer& data = *frame_data;
    data.write(transforms_at, {scene.transforms});
    data.write(materials_at, scene.materials);
    data.write(flat_at, {flat_constants});
    data.write(matrix_at, {scene.world_to_clip.rows});
    data.write(overlay_at, scene.overlay);
    data.write(xray_at, scene.xray);

    ID3D12GraphicsCommandList* commands = frame.list.Get();
    RECT rect{
        .left = base::saturating_cast<LONG>(view.x),
        .top = base::saturating_cast<LONG>(view.y),
        .right = base::saturating_cast<LONG>(view.right()),
        .bottom = base::saturating_cast<LONG>(view.bottom()),
    };
    D3D12_VIEWPORT viewport{
        .TopLeftX = view.x,
        .TopLeftY = view.y,
        .Width = view.width,
        .Height = view.height,
        .MinDepth = 0.0f,
        .MaxDepth = 1.0f,
    };
    commands->ClearRenderTargetView(frame.view, scene.background.data(), 1, &rect);
    commands->ClearDepthStencilView(depth_view, D3D12_CLEAR_FLAG_DEPTH, 1.0f, 0, 1, &rect);
    commands->RSSetViewports(1, &viewport);
    commands->RSSetScissorRects(1, &rect);

    // The mesh.
    if (index_count > 0 && geometry) {
        commands->OMSetRenderTargets(1, &frame.srgb_view, FALSE, &depth_view);
        const simple_paint::direct3d12::Pipeline& pipeline = scene.shader == Shader::SimplePaint ? paint : flat;
        size_t shading = scene.shader == Shader::SimplePaint ? materials_at : flat_at;
        pipeline.bind(commands);
        commands->SetGraphicsRootConstantBufferView(0, data.gpu_address(transforms_at));
        commands->SetGraphicsRootConstantBufferView(1, data.gpu_address(shading));
        D3D12_GPU_VIRTUAL_ADDRESS start = geometry->gpu_address(0);
        D3D12_VERTEX_BUFFER_VIEW vertex_view{
            .BufferLocation = start,
            .SizeInBytes = static_cast<UINT>(vertex_bytes),
            .StrideInBytes = static_cast<UINT>(sizeof(simple_paint::Vertex)),
        };
        commands->IASetVertexBuffers(0, 1, &vertex_view);
        D3D12_INDEX_BUFFER_VIEW index_view{
            .BufferLocation = geometry->gpu_address(next_multiple_of(vertex_bytes, Alignment)),
            .SizeInBytes = index_count * 4,
            .Format = DXGI_FORMAT_R32_UINT,
        };
        commands->IASetIndexBuffer(&index_view);
        commands->DrawIndexedInstanced(index_count, 1, 0, 0, 0);
    }

    // The overlay, tested against the mesh's depth, then the X-ray's.
    commands->OMSetRenderTargets(1, &frame.view, FALSE, &depth_view);
    commands->SetGraphicsRootSignature(overlay.root_signature.Get());
    commands->IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
    std::array<float, 4> constants{view.width, view.height, 0.0004f, 0.0f};
    struct Pass {
        ID3D12PipelineState* state;
        const std::vector<OverlayItem>& items;
        size_t at;
    };
    for (const Pass& pass : {Pass{overlay.tested.Get(), scene.overlay, overlay_at}, Pass{overlay.untested.Get(), scene.xray, xray_at}}) {
        if (pass.items.empty()) {
            continue;
        }
        commands->SetPipelineState(pass.state);
        commands->SetGraphicsRoot32BitConstants(0, 4, constants.data(), 0);
        commands->SetGraphicsRootConstantBufferView(1, data.gpu_address(matrix_at));
        commands->SetGraphicsRootShaderResourceView(2, data.gpu_address(pass.at));
        commands->DrawInstanced(4, static_cast<UINT>(pass.items.size()), 0, 0);
    }
}

Viewport::OverlayPipeline Viewport::overlay_pipeline(ID3D12Device* device) {
    OverlayPipeline pipeline;
    base::check(device->CreateRootSignature(0, OverlayVertex, sizeof(OverlayVertex), IID_PPV_ARGS(&pipeline.root_signature)));
    D3D12_RENDER_TARGET_BLEND_DESC blend{
        .BlendEnable = TRUE,
        .SrcBlend = D3D12_BLEND_ONE,
        .DestBlend = D3D12_BLEND_INV_SRC_ALPHA,
        .BlendOp = D3D12_BLEND_OP_ADD,
        .SrcBlendAlpha = D3D12_BLEND_ONE,
        .DestBlendAlpha = D3D12_BLEND_INV_SRC_ALPHA,
        .BlendOpAlpha = D3D12_BLEND_OP_ADD,
        .RenderTargetWriteMask = D3D12_COLOR_WRITE_ENABLE_ALL,
    };
    D3D12_GRAPHICS_PIPELINE_STATE_DESC description{
        // Borrow the root signature without touching its reference count.
        .pRootSignature = pipeline.root_signature.Get(),
        .VS = graphics::bytecode(OverlayVertex),
        .PS = graphics::bytecode(OverlayPixel),
        .SampleMask = std::numeric_limits<UINT>::max(),
        .RasterizerState =
            {
                .FillMode = D3D12_FILL_MODE_SOLID,
                .CullMode = D3D12_CULL_MODE_NONE,
                .DepthClipEnable = TRUE,
            },
        .DepthStencilState =
            {
                .DepthEnable = TRUE,
                .DepthWriteMask = D3D12_DEPTH_WRITE_MASK_ZERO,
                .DepthFunc = D3D12_COMPARISON_FUNC_LESS_EQUAL,
            },
        .PrimitiveTopologyType = D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        .NumRenderTargets = 1,
        .DSVFormat = graphics::DepthFormat,
        .SampleDesc =
            {
                .Count = 1,
                .Quality = 0,
            },
    };
    for (D3D12_RENDER_TARGET_BLEND_DESC& render_target : description.BlendState.RenderTarget) {
        render_target = blend;
    }
    description.RTVFormats[0] = graphics::TargetFormat;
    base::check(device->CreateGraphicsPipelineState(&description, IID_PPV_ARGS(&pipeline.tested)));
    description.DepthStencilState.DepthEnable = FALSE;
    base::check(device->CreateGraphicsPipelineState(&description, IID_PPV_ARGS(&pipeline.untested)));
    return pipeline;
}

} // namespace mesh_modeler::viewport
