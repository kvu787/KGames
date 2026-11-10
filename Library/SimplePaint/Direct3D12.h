// SimplePaint on the GPU, for hosts that draw with Library/Graphics: the
// pipeline of SimplePaint.hlsl's adapter, meshes in GPU memory, and the
// sizes of the constant buffers that the adapter binds. The host compiles
// the adapter with its material count (see Usage.md), owns the constant
// buffers, and keeps them from changing while the GPU reads them. A host's
// own shader that takes the same vertices and binds b0 and b1 the same way,
// like SpinningCarTest's FlatShading.hlsl, draws with them too.

#pragma once

#include "Base/Windows.h"
#include "Graphics/Graphics.h"
#include "SimplePaint/Geometry.h"
#include "SimplePaint/Material.h"
#include "SimplePaint/Orthographic.h"

#include <cstddef>
#include <cstdint>
#include <d3d12.h>
#include <optional>
#include <span>
#include <wrl/client.h>

namespace simple_paint::direct3d12 {

// Where constant buffers may start.
inline constexpr size_t ConstantBufferAlignment = 256;
// The bytes an object's transforms take at b0, rounded up to a whole
// constant buffer.
inline constexpr size_t ObjectSize = (sizeof(orthographic::ObjectTransforms) + ConstantBufferAlignment - 1) / ConstantBufferAlignment * ConstantBufferAlignment;

// The bytes that `count` materials take at b1. They're packed 80 bytes
// apart, and the buffer is rounded up to a whole constant buffer.
constexpr size_t materials_size(size_t count) {
    return (count * sizeof(GpuMaterial) + ConstantBufferAlignment - 1) / ConstantBufferAlignment * ConstantBufferAlignment;
}

// What a pipeline draws into, and how.
struct Target {
    // SimplePaint's shader outputs linear colors, so for it, this should be
    // an sRGB format, which encodes them once for display.
    DXGI_FORMAT format{};
    // With a depth buffer, nearer surfaces hide farther ones.
    std::optional<DXGI_FORMAT> depth_format;
    std::uint32_t sample_count{};
    // Meshes are wound counterclockwise when seen from their front, the
    // right-handed convention, so D3D12_CULL_MODE_BACK hides their insides.
    D3D12_CULL_MODE cull_mode{};
};

class Pipeline {
public:
    // From the adapter's two stages, compiled with the same material count,
    // or another shader's that binds like it. The vertex stage holds the
    // root signature.
    Pipeline(ID3D12Device* device, std::span<const std::uint8_t> vertex_shader, std::span<const std::uint8_t> pixel_shader, const Target& target);
    Pipeline(const Pipeline&) = delete;
    Pipeline& operator=(const Pipeline&) = delete;
    Pipeline(Pipeline&&) = default;
    Pipeline& operator=(Pipeline&&) = default;

    // Sets the pipeline, its root signature, and the triangle list topology.
    void bind(ID3D12GraphicsCommandList* list) const;

private:
    Microsoft::WRL::ComPtr<ID3D12RootSignature> root_signature;
    Microsoft::WRL::ComPtr<ID3D12PipelineState> state;
};

// An indexed triangle mesh in the GPU's own memory. Meshes never change, so
// it's copied there once, instead of the GPU reading it from an upload heap
// in system memory, across PCIe, every frame.
class Mesh {
public:
    std::uint32_t index_count = 0;

    // Validates the mesh with validate_mesh for `material_count` materials
    // first, and fails with its reason. The copy waits for the GPU, so it's
    // for setup, not for frames.
    Mesh(graphics::Gpu& gpu, std::span<const Vertex> vertices, std::span<const std::uint32_t> indices, std::uint32_t material_count);
    Mesh(const Mesh&) = delete;
    Mesh& operator=(const Mesh&) = delete;
    Mesh(Mesh&&) = default;
    Mesh& operator=(Mesh&&) = default;

    // Draws the mesh with the pipeline that's bound, with its object
    // transforms and its materials at these addresses, each at the start of
    // a constant buffer.
    void draw(ID3D12GraphicsCommandList* list, D3D12_GPU_VIRTUAL_ADDRESS object, D3D12_GPU_VIRTUAL_ADDRESS materials) const;

private:
    Microsoft::WRL::ComPtr<ID3D12Resource> buffer;
    D3D12_VERTEX_BUFFER_VIEW vertex_view{};
    D3D12_INDEX_BUFFER_VIEW index_view{};
};

} // namespace simple_paint::direct3d12
