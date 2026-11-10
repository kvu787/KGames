// A depth buffer as big as the frame, for 3D views. It's made again when the
// frame's size changes, like between the two window setups' heights, which
// is safe because the GPU has finished the last frame by the time the next
// one is drawn.

#pragma once

#include "Base/Windows.h"
#include "Graphics/Device.h"
#include "Graphics/Frame.h"
#include "Graphics/Resources.h"

#include <cstdint>
#include <d3d12.h>
#include <optional>
#include <tuple>
#include <wrl/client.h>

namespace graphics {

class DepthBuffer {
public:
    explicit DepthBuffer(const Gpu& gpu);
    DepthBuffer(const DepthBuffer&) = delete;
    DepthBuffer& operator=(const DepthBuffer&) = delete;
    DepthBuffer(DepthBuffer&&) = default;
    DepthBuffer& operator=(DepthBuffer&&) = default;

    // The view of a depth buffer as big as the frame, made first if there's
    // none yet or the frame's size changed, with whatever it held.
    D3D12_CPU_DESCRIPTOR_HANDLE view(const Gpu& gpu, const Frame& frame);

    // The view, cleared to the far plane, for a frame that draws the whole
    // of it.
    D3D12_CPU_DESCRIPTOR_HANDLE clear(const Gpu& gpu, const Frame& frame);

private:
    // The buffer, and the frame size that it was made for.
    std::optional<std::tuple<ComPtr<ID3D12Resource>, std::uint32_t, std::uint32_t>> texture;
    DescriptorHeap views;
};

} // namespace graphics
