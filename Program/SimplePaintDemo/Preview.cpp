#include "SimplePaintDemo/Preview.h"

#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Graphics/Error.h"
#include "Math/Math.h"
#include "Meshes/Meshes.h"
#include "SimplePaint/Orthographic.h"

// The shaders, which SimplePaintDemoLibrary.vcxproj names: SimplePaintVertex
// and SimplePaintPixel.
#include "SimplePaintPixel.h"
#include "SimplePaintVertex.h"

#include <algorithm>
#include <expected>
#include <format>

namespace simple_paint_demo::preview {

using graphics::create_texture;
using graphics::Frame;
using graphics::Gpu;
using graphics::transition;
using graphics::UploadBuffer;
using math::Matrix;
using Microsoft::WRL::ComPtr;
using simple_paint::GpuMaterial;
using simple_paint::direct3d12::materials_size;
using simple_paint::direct3d12::Mesh;
using simple_paint::direct3d12::ObjectSize;
using simple_paint::direct3d12::Pipeline;
using simple_paint::direct3d12::Target;
using simple_paint::orthographic::build_object_transforms;
using simple_paint::orthographic::make_projection;
using simple_paint::orthographic::ObjectTransforms;
using simple_paint::orthographic::Projection;
using user_interface::Color;
using user_interface::Rect;

namespace {

constexpr std::uint32_t SampleCount = 4;
// Bytes stored as sRGB, in the frame's order, viewed as sRGB for drawing and
// resolving, and copied into the frame as they are.
constexpr DXGI_FORMAT StorageFormat = DXGI_FORMAT_B8G8R8A8_TYPELESS;
constexpr DXGI_FORMAT ViewFormat = DXGI_FORMAT_B8G8R8A8_UNORM_SRGB;
// Enough vertices that the edge looks round at this size, and the normals
// interpolate smoothly.
constexpr std::uint32_t Longitudes = 256;
constexpr std::uint32_t Latitudes = 128;
// How much of the image's width the sphere spans: its diameter is 2 of
// these 2.3.
constexpr float ViewSize = 2.3f;

graphics::Error invalid(const simple_paint::Error& error) {
    return graphics::Error(E_INVALIDARG, error.message);
}

// The sphere, copied to the GPU.
Mesh create_sphere(Gpu& gpu) {
    meshes::Mesh mesh = meshes::uv_sphere(Longitudes, Latitudes, 0);
    return Mesh(gpu, mesh.vertices, mesh.indices, 1);
}

// The constants, with the object's transforms in them, which never change.
UploadBuffer create_constants(ID3D12Device* device) {
    UploadBuffer constants(device, ObjectSize + materials_size(1));
    // The camera is 3 units in front of the sphere, looking at it.
    Matrix view = Matrix::look_at_rh({0.0f, 0.0f, 3.0f}, {0.0f, 0.0f, 0.0f}, {0.0f, 1.0f, 0.0f});
    float height = ViewSize * static_cast<float>(ImageHeight) / static_cast<float>(ImageWidth);
    std::expected<Projection, simple_paint::Error> projection = make_projection(ViewSize, height, 1.0f, 5.0f);
    if (!projection) {
        throw invalid(projection.error());
    }
    std::expected<ObjectTransforms, simple_paint::Error> object = build_object_transforms(view, *projection);
    if (!object) {
        throw invalid(object.error());
    }
    constants.write(0, {*object});
    return constants;
}

// An sRGB color as linear light, for clearing through an sRGB view, which
// encodes it back to the same bytes.
std::array<float, 4> linear(Color color) {
    auto [r, g, b] = color.to_linear();
    return {r, g, b, 1.0f};
}

} // namespace

Preview::Preview(Gpu& gpu) :
    pipeline(gpu.device.Get(), SimplePaintVertex, SimplePaintPixel,
             Target{
                 .format = ViewFormat,
                 .depth_format = std::nullopt,
                 .sample_count = SampleCount,
                 // The sphere is closed and convex, so hiding its back
                 // faces leaves only the front ones, without a depth buffer.
                 .cull_mode = D3D12_CULL_MODE_BACK,
             }),
    sphere(create_sphere(gpu)),
    constants(create_constants(gpu.device.Get())),
    rtv_heap(gpu.device.Get(), D3D12_DESCRIPTOR_HEAP_TYPE_RTV, 1, false) {}

void Preview::render(const Gpu& gpu, const Frame& frame, const GpuMaterial& material, Color background, Rect view) {
    if (!image || image->background != background) {
        image = create_image(gpu, background);
        drawn = std::nullopt;
    }
    if (drawn != material) {
        draw_sphere(frame, material);
        drawn = material;
    }
    copy(frame, view);
}

Preview::Image Preview::create_image(const Gpu& gpu, Color background) const {
    ID3D12Device* device = gpu.device.Get();
    D3D12_CLEAR_VALUE clear{.Format = ViewFormat};
    std::ranges::copy(linear(background), clear.Color);
    ComPtr<ID3D12Resource> multisampled =
        create_texture(device, ImageWidth, ImageHeight, StorageFormat, D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET, D3D12_RESOURCE_STATE_RENDER_TARGET, &clear, SampleCount);
    ComPtr<ID3D12Resource> resolved = create_texture(device, ImageWidth, ImageHeight, StorageFormat, D3D12_RESOURCE_FLAG_NONE, D3D12_RESOURCE_STATE_COPY_SOURCE, nullptr, 1);
    D3D12_RENDER_TARGET_VIEW_DESC description{
        .Format = ViewFormat,
        .ViewDimension = D3D12_RTV_DIMENSION_TEXTURE2DMS,
    };
    device->CreateRenderTargetView(multisampled.Get(), &description, rtv_heap.cpu(0));
    return Image{
        .multisampled = std::move(multisampled),
        .resolved = std::move(resolved),
        .background = background,
    };
}

void Preview::draw_sphere(const Frame& frame, const GpuMaterial& material) const {
    const Image& made = base::expect(image, "the image is made before the sphere is drawn");
    constants.write(ObjectSize, {material});
    ID3D12GraphicsCommandList* list = frame.list.Get();
    D3D12_CPU_DESCRIPTOR_HANDLE rtv = rtv_heap.cpu(0);
    list->OMSetRenderTargets(1, &rtv, FALSE, nullptr);
    D3D12_VIEWPORT viewport{
        .Width = static_cast<float>(ImageWidth),
        .Height = static_cast<float>(ImageHeight),
        .MaxDepth = 1.0f,
    };
    list->RSSetViewports(1, &viewport);
    D3D12_RECT scissor{
        .left = 0,
        .top = 0,
        .right = static_cast<LONG>(ImageWidth),
        .bottom = static_cast<LONG>(ImageHeight),
    };
    list->RSSetScissorRects(1, &scissor);
    list->ClearRenderTargetView(rtv, linear(made.background).data(), 0, nullptr);
    pipeline.bind(list);
    sphere.draw(list, constants.gpu_address(0), constants.gpu_address(ObjectSize));

    std::array<D3D12_RESOURCE_BARRIER, 2> to_resolve{
        transition(made.multisampled.Get(), D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_RESOLVE_SOURCE),
        transition(made.resolved.Get(), D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_RESOLVE_DEST),
    };
    list->ResourceBarrier(static_cast<UINT>(to_resolve.size()), to_resolve.data());
    list->ResolveSubresource(made.resolved.Get(), 0, made.multisampled.Get(), 0, ViewFormat);
    std::array<D3D12_RESOURCE_BARRIER, 2> resolved{
        transition(made.multisampled.Get(), D3D12_RESOURCE_STATE_RESOLVE_SOURCE, D3D12_RESOURCE_STATE_RENDER_TARGET),
        transition(made.resolved.Get(), D3D12_RESOURCE_STATE_RESOLVE_DEST, D3D12_RESOURCE_STATE_COPY_SOURCE),
    };
    list->ResourceBarrier(static_cast<UINT>(resolved.size()), resolved.data());
}

void Preview::copy(const Frame& frame, Rect view) const {
    const Image& made = base::expect(image, "the image is made before it's copied");
    std::optional<CopyPlace> place = copy_place(view, {frame.width, frame.height});
    if (!place) {
        return;
    }
    auto [x, y] = place->destination;
    ID3D12GraphicsCommandList* list = frame.list.Get();
    D3D12_RESOURCE_BARRIER to_copy = transition(frame.target.Get(), D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_COPY_DEST);
    list->ResourceBarrier(1, &to_copy);
    D3D12_TEXTURE_COPY_LOCATION destination{
        .pResource = frame.target.Get(),
        .Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
    };
    destination.SubresourceIndex = 0;
    D3D12_TEXTURE_COPY_LOCATION from{
        .pResource = made.resolved.Get(),
        .Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
    };
    from.SubresourceIndex = 0;
    list->CopyTextureRegion(&destination, x, y, 0, &from, &place->source);
    D3D12_RESOURCE_BARRIER copied = transition(frame.target.Get(), D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_RENDER_TARGET);
    list->ResourceBarrier(1, &copied);
}

std::string describe_value(const CopyPlace& place) {
    return std::format("CopyPlace {{ destination: [{}, {}], source: D3D12_BOX {{ left: {}, top: {}, front: {}, right: {}, bottom: {}, back: {} }} }}", place.destination[0],
                       place.destination[1], place.source.left, place.source.top, place.source.front, place.source.right, place.source.bottom, place.source.back);
}

std::optional<CopyPlace> copy_place(Rect view, std::array<std::uint32_t, 2> frame) {
    Rect rounded = view.round();
    std::int64_t view_x = base::saturating_cast<std::int64_t>(rounded.x);
    std::int64_t view_y = base::saturating_cast<std::int64_t>(rounded.y);
    std::int64_t view_width = base::saturating_cast<std::int64_t>(rounded.width);
    std::int64_t view_height = base::saturating_cast<std::int64_t>(rounded.height);
    std::int64_t image_width = static_cast<std::int64_t>(ImageWidth);
    std::int64_t image_height = static_cast<std::int64_t>(ImageHeight);
    // The image's middle at the view's middle, cut to the view and the frame.
    std::int64_t left = std::max(std::max(view_x + (view_width - image_width) / 2, view_x), std::int64_t{0});
    std::int64_t top = std::max(std::max(view_y + (view_height - image_height) / 2, view_y), std::int64_t{0});
    std::int64_t right = std::min(std::min(view_x + (view_width + image_width) / 2, base::saturating_cast<std::int64_t>(rounded.right())), static_cast<std::int64_t>(frame[0]));
    std::int64_t bottom = std::min(std::min(view_y + (view_height + image_height) / 2, base::saturating_cast<std::int64_t>(rounded.bottom())), static_cast<std::int64_t>(frame[1]));
    if (right <= left || bottom <= top) {
        return std::nullopt;
    }
    // Where the image's top left is in the frame.
    std::int64_t image_x = view_x + (view_width - image_width) / 2;
    std::int64_t image_y = view_y + (view_height - image_height) / 2;
    return CopyPlace{
        .destination = {static_cast<std::uint32_t>(left), static_cast<std::uint32_t>(top)},
        .source =
            D3D12_BOX{
                .left = static_cast<UINT>(left - image_x),
                .top = static_cast<UINT>(top - image_y),
                .front = 0,
                .right = static_cast<UINT>(right - image_x),
                .bottom = static_cast<UINT>(bottom - image_y),
                .back = 1,
            },
    };
}

} // namespace simple_paint_demo::preview
