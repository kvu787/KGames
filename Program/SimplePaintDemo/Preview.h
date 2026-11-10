// Draws a UV sphere with SimplePaint, in an orthographic view, and copies
// the image into the frame at the preview card's view.
//
// Any view of a sphere shows every normal that faces the camera, each once,
// and SimplePaint colors a surface by its normal alone, so the sphere shows
// the whole material. It never needs to turn.
//
// The sphere is drawn with 4x multisampling into its own texture, so its edge
// is smooth, through an sRGB view, which encodes the shader's linear output
// once. The texture is resolved, which averages the samples in linear light,
// and the result is copied into the frame. The copy keeps the bytes, which
// are sRGB, as the frame's are.
//
// The resolved image is kept, and the sphere is drawn again only when the
// material changes. Every frame copies the image, as each frame starts
// cleared to the window's background. The image is as big as the view is in
// borderless fullscreen, its largest, around the card's color, so that it
// fills the view, and a smaller view shows its middle.

#pragma once

#include "Base/Windows.h"
#include "Graphics/Graphics.h"
#include "SimplePaint/Direct3D12.h"
#include "SimplePaint/Material.h"
#include "UserInterface/Drawing/Geometry.h"

#include <array>
#include <cstdint>
#include <d3d12.h>
#include <optional>
#include <string>
#include <wrl/client.h>

namespace simple_paint_demo::preview {

// The image's size in pixels: the preview card's view in borderless
// fullscreen. Maximized, the view is 71 pixels shorter.
inline constexpr std::uint32_t ImageWidth = 1152;
inline constexpr std::uint32_t ImageHeight = 1239;

class Preview {
public:
    explicit Preview(graphics::Gpu& gpu);
    Preview(const Preview&) = delete;
    Preview& operator=(const Preview&) = delete;
    Preview(Preview&&) = default;
    Preview& operator=(Preview&&) = default;

    // Copies the image of the sphere with this material, around `background`,
    // into the frame at `view`, which it fills. Records the sphere first if
    // the image has another material or background.
    void render(const graphics::Gpu& gpu, const graphics::Frame& frame, const simple_paint::GpuMaterial& material, user_interface::Color background, user_interface::Rect view);

private:
    struct Image {
        Microsoft::WRL::ComPtr<ID3D12Resource> multisampled;
        Microsoft::WRL::ComPtr<ID3D12Resource> resolved;
        // Around the sphere, in sRGB.
        user_interface::Color background;
    };

    // The multisampled texture, whose clear color is the background, and
    // the resolved one, which waits to be copied from.
    Image create_image(const graphics::Gpu& gpu, user_interface::Color background) const;

    // Records the sphere with this material into the resolved image, which
    // ends ready to copy from. The frame's own target is bound again by
    // whatever draws into it next.
    void draw_sphere(const graphics::Frame& frame, const simple_paint::GpuMaterial& material) const;

    // Copies the middle of the image into the frame, centered in the view,
    // and leaves the frame's target as it found it, ready to draw into.
    void copy(const graphics::Frame& frame, user_interface::Rect view) const;

    simple_paint::direct3d12::Pipeline pipeline;
    simple_paint::direct3d12::Mesh sphere;
    // The object's transforms, then the material, each at a constant
    // buffer's start. Only one frame is on the GPU at a time, and it's
    // finished before the next one is recorded, so they can change then.
    graphics::UploadBuffer constants;
    graphics::DescriptorHeap rtv_heap;
    // Made by the first frame, with the card's color around the sphere.
    std::optional<Image> image;
    // The material that the image has the sphere in, once it has one.
    std::optional<simple_paint::GpuMaterial> drawn;
};

// Where the image goes in a frame: the top left in the frame, and the part
// of the image.
struct CopyPlace {
    std::array<std::uint32_t, 2> destination;
    D3D12_BOX source;

    // Written out, as a D3D12_BOX has no == of its own.
    bool operator==(const CopyPlace& other) const {
        return destination == other.destination && source.left == other.source.left && source.top == other.source.top && source.front == other.source.front &&
               source.right == other.source.right && source.bottom == other.source.bottom && source.back == other.source.back;
    }
};

// How a failed check in a test shows a place.
std::string describe_value(const CopyPlace& place);

// The part of the image that fills the view, from its middle, inside the
// frame of the size. Nothing if none of it would show.
std::optional<CopyPlace> copy_place(user_interface::Rect view, std::array<std::uint32_t, 2> frame);

} // namespace simple_paint_demo::preview
