#include "Graphics/Depth.h"

#include "Graphics/Graphics.h"

#include <utility>

namespace graphics {

DepthBuffer::DepthBuffer(const Gpu& gpu) : views(gpu.device.Get(), D3D12_DESCRIPTOR_HEAP_TYPE_DSV, 1, false) {}

D3D12_CPU_DESCRIPTOR_HANDLE DepthBuffer::view(const Gpu& gpu, const Frame& frame) {
    D3D12_CPU_DESCRIPTOR_HANDLE handle = views.cpu(0);
    bool fits = texture && std::get<1>(*texture) == frame.width && std::get<2>(*texture) == frame.height;
    if (!fits) {
        D3D12_CLEAR_VALUE clear_value{
            .Format = DepthFormat,
            .DepthStencil = {.Depth = 1.0f, .Stencil = 0},
        };
        ComPtr<ID3D12Resource> made =
            create_texture(gpu.device.Get(), frame.width, frame.height, DepthFormat, D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL, D3D12_RESOURCE_STATE_DEPTH_WRITE, &clear_value, 1);
        gpu.device->CreateDepthStencilView(made.Get(), nullptr, handle);
        texture = std::tuple(std::move(made), frame.width, frame.height);
    }
    return handle;
}

D3D12_CPU_DESCRIPTOR_HANDLE DepthBuffer::clear(const Gpu& gpu, const Frame& frame) {
    D3D12_CPU_DESCRIPTOR_HANDLE handle = view(gpu, frame);
    frame.list->ClearDepthStencilView(handle, D3D12_CLEAR_FLAG_DEPTH, 1.0f, 0, 0, nullptr);
    return handle;
}

} // namespace graphics
