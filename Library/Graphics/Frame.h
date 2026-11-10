// A frame being drawn, into a swap chain's back buffer or an offscreen
// texture, and what reads its pixels back.

#pragma once

#include "Base/Windows.h"
#include "Graphics/Device.h"
#include "Graphics/Resources.h"

#include <array>
#include <cstdint>
#include <d3d12.h>
#include <optional>
#include <vector>
#include <wrl/client.h>

namespace graphics {

// What a frame's commands draw into. The target is in the render target state
// while the frame is being recorded, and must be left in it.
struct Frame {
    ComPtr<ID3D12GraphicsCommandList> list;
    ComPtr<ID3D12Resource> target;
    // Stores colors as they are, for colors that are sRGB already.
    D3D12_CPU_DESCRIPTOR_HANDLE view;
    // Encodes linear colors to sRGB.
    D3D12_CPU_DESCRIPTOR_HANDLE srgb_view;
    std::uint32_t width;
    std::uint32_t height;

    Frame(ComPtr<ID3D12GraphicsCommandList> list, ComPtr<ID3D12Resource> target, D3D12_CPU_DESCRIPTOR_HANDLE view, D3D12_CPU_DESCRIPTOR_HANDLE srgb_view, std::uint32_t width,
          std::uint32_t height);
    // A frame is handed on, to what presents or finishes it, and never copied.
    Frame(const Frame&) = delete;
    Frame& operator=(const Frame&) = delete;
    Frame(Frame&&) = default;
    Frame& operator=(Frame&&) = default;

    // Draws into the whole target, through one of its views, with an
    // optional depth buffer.
    void bind(bool srgb, std::optional<D3D12_CPU_DESCRIPTOR_HANDLE> depth) const;

    D3D12_VIEWPORT viewport() const;

    void clear(std::array<float, 4> color) const;
};

// Pixels, top row first, 4 bytes each, in the target's order: blue, green,
// red and alpha.
struct Image {
    std::uint32_t width = 0;
    std::uint32_t height = 0;
    std::vector<std::uint8_t> pixels;

    // Red, green, blue and alpha of a pixel.
    std::array<std::uint8_t, 4> pixel(std::uint32_t x, std::uint32_t y) const;
};

// A copy of a frame's target into memory that the CPU can read, once the GPU
// has made it.
class Readback {
public:
    Readback(const Readback&) = delete;
    Readback& operator=(const Readback&) = delete;
    Readback(Readback&&) = default;
    Readback& operator=(Readback&&) = default;

    // Records the copy into the frame's commands, and leaves the target in the
    // render target state.
    static Readback record(ID3D12Device* device, const Frame& frame);

    // The pixels, once the GPU has signaled `fence`, which it does after the
    // frame's commands.
    Image read(const Gpu& gpu, std::uint64_t fence) const;

private:
    Readback(ComPtr<ID3D12Resource> buffer, D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint, std::uint32_t width, std::uint32_t height);

    ComPtr<ID3D12Resource> buffer;
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint;
    std::uint32_t width;
    std::uint32_t height;
};

// A texture to draw frames into without a window, for tests and for
// drawing a program's frames headless.
class Offscreen {
public:
    ComPtr<ID3D12Resource> texture;

    // Frames clear it to `clear`, which the GPU clears to fastest.
    Offscreen(const Gpu& gpu, std::uint32_t width, std::uint32_t height, std::array<float, 4> clear);
    Offscreen(const Offscreen&) = delete;
    Offscreen& operator=(const Offscreen&) = delete;
    Offscreen(Offscreen&&) = default;
    Offscreen& operator=(Offscreen&&) = default;

    Frame begin();

    // Runs the frame's commands, waits for them, and returns what they drew.
    Image finish(Gpu& gpu, Frame frame);

    // Runs the frame's commands without reading anything back, like for
    // timing them, and returns the fence value that the GPU signals once
    // it has finished them.
    std::uint64_t submit(Gpu& gpu, Frame frame);

private:
    DescriptorHeap views;
    ComPtr<ID3D12CommandAllocator> allocator;
    ComPtr<ID3D12GraphicsCommandList> list;
    std::uint32_t width;
    std::uint32_t height;
};

} // namespace graphics
