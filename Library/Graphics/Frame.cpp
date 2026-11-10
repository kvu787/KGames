#include "Graphics/Frame.h"

#include "Graphics/Graphics.h"

#include <tuple>
#include <utility>

namespace graphics {

Frame::Frame(ComPtr<ID3D12GraphicsCommandList> list, ComPtr<ID3D12Resource> target, D3D12_CPU_DESCRIPTOR_HANDLE view, D3D12_CPU_DESCRIPTOR_HANDLE srgb_view, std::uint32_t width,
             std::uint32_t height) :
    list(std::move(list)), target(std::move(target)), view(view), srgb_view(srgb_view), width(width), height(height) {}

void Frame::bind(bool srgb, std::optional<D3D12_CPU_DESCRIPTOR_HANDLE> depth) const {
    D3D12_CPU_DESCRIPTOR_HANDLE target_view = srgb ? srgb_view : view;
    list->OMSetRenderTargets(1, &target_view, FALSE, depth ? &*depth : nullptr);
    D3D12_VIEWPORT whole = viewport();
    list->RSSetViewports(1, &whole);
    D3D12_RECT scissor{
        .left = 0,
        .top = 0,
        .right = static_cast<LONG>(width),
        .bottom = static_cast<LONG>(height),
    };
    list->RSSetScissorRects(1, &scissor);
}

D3D12_VIEWPORT Frame::viewport() const {
    return D3D12_VIEWPORT{
        .TopLeftX = 0.0f,
        .TopLeftY = 0.0f,
        .Width = static_cast<float>(width),
        .Height = static_cast<float>(height),
        .MinDepth = 0.0f,
        .MaxDepth = 1.0f,
    };
}

void Frame::clear(std::array<float, 4> color) const {
    list->ClearRenderTargetView(view, color.data(), 0, nullptr);
}

std::array<std::uint8_t, 4> Image::pixel(std::uint32_t x, std::uint32_t y) const {
    size_t at = (y * width + x) * 4;
    return {pixels[at + 2], pixels[at + 1], pixels[at], pixels[at + 3]};
}

Readback::Readback(ComPtr<ID3D12Resource> buffer, D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint, std::uint32_t width, std::uint32_t height) :
    buffer(std::move(buffer)), footprint(footprint), width(width), height(height) {}

Readback Readback::record(ID3D12Device* device, const Frame& frame) {
    D3D12_RESOURCE_DESC description = frame.target->GetDesc();
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
    std::uint64_t total = 0;
    device->GetCopyableFootprints(&description, 0, 1, 0, &footprint, nullptr, nullptr, &total);
    ComPtr<ID3D12Resource> buffer = create_buffer(device, total, D3D12_HEAP_TYPE_READBACK, D3D12_RESOURCE_STATE_COPY_DEST);
    ID3D12GraphicsCommandList* list = frame.list.Get();
    D3D12_RESOURCE_BARRIER to_source = transition(frame.target.Get(), D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_COPY_SOURCE);
    list->ResourceBarrier(1, &to_source);
    D3D12_TEXTURE_COPY_LOCATION destination{
        .pResource = buffer.Get(),
        .Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
        .PlacedFootprint = footprint,
    };
    D3D12_TEXTURE_COPY_LOCATION source{
        .pResource = frame.target.Get(),
        .Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
        .SubresourceIndex = 0,
    };
    list->CopyTextureRegion(&destination, 0, 0, 0, &source, nullptr);
    D3D12_RESOURCE_BARRIER to_target = transition(frame.target.Get(), D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_RENDER_TARGET);
    list->ResourceBarrier(1, &to_target);
    return Readback(std::move(buffer), footprint, frame.width, frame.height);
}

Image Readback::read(const Gpu& gpu, std::uint64_t fence) const {
    gpu.wait_for(fence);
    void* mapped = nullptr;
    base::check(buffer->Map(0, nullptr, &mapped));
    size_t pitch = footprint.Footprint.RowPitch;
    size_t row = static_cast<size_t>(width) * 4;
    std::vector<std::uint8_t> pixels;
    pixels.reserve(row * static_cast<size_t>(height));
    for (size_t y = 0; y < static_cast<size_t>(height); y++) {
        const std::uint8_t* start = static_cast<const std::uint8_t*>(mapped) + static_cast<size_t>(footprint.Offset) + y * pitch;
        pixels.insert(pixels.end(), start, start + row);
    }
    D3D12_RANGE written{.Begin = 0, .End = 0};
    buffer->Unmap(0, &written);
    return Image{
        .width = width,
        .height = height,
        .pixels = std::move(pixels),
    };
}

namespace {

// An offscreen's texture, which frames clear to `clear`.
ComPtr<ID3D12Resource> create_target(const Gpu& gpu, std::uint32_t width, std::uint32_t height, std::array<float, 4> clear) {
    D3D12_CLEAR_VALUE clear_value{
        .Format = TargetFormat,
        .Color = {clear[0], clear[1], clear[2], clear[3]},
    };
    // Typeless, so that it can have both of a frame's views, which
    // read it as UNORM and as sRGB. A swap chain's buffers can always
    // be viewed in another format of their group, but a fully typed
    // texture only on GPUs that support casting fully typed formats.
    return create_texture(gpu.device.Get(), width, height, DXGI_FORMAT_B8G8R8A8_TYPELESS, D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET, D3D12_RESOURCE_STATE_RENDER_TARGET, &clear_value,
                          1);
}

} // namespace

Offscreen::Offscreen(const Gpu& gpu, std::uint32_t width, std::uint32_t height, std::array<float, 4> clear) :
    texture(create_target(gpu, width, height, clear)), views(gpu.device.Get(), D3D12_DESCRIPTOR_HEAP_TYPE_RTV, 2, false), width(width), height(height) {
    create_target_views(gpu.device.Get(), texture.Get(), views, 0);
    std::tie(allocator, list) = command_list(gpu.device.Get());
}

Frame Offscreen::begin() {
    base::check(allocator->Reset());
    base::check(list->Reset(allocator.Get(), nullptr));
    return Frame(list, texture, views.cpu(0), views.cpu(1), width, height);
}

Image Offscreen::finish(Gpu& gpu, Frame frame) {
    Readback readback = Readback::record(gpu.device.Get(), frame);
    std::uint64_t fence = submit(gpu, std::move(frame));
    // The debug layer's messages are written whether or not the read fails.
    std::optional<Image> image;
    try {
        image = readback.read(gpu, fence);
    } catch (const base::WindowsError&) {
        gpu.report_messages();
        throw;
    }
    gpu.report_messages();
    return std::move(*image);
}

std::uint64_t Offscreen::submit(Gpu& gpu, Frame frame) {
    base::check(frame.list->Close());
    execute(gpu.queue.Get(), frame.list.Get());
    return gpu.signal();
}

} // namespace graphics
