// A depth buffer as big as the frame, which is made again when the frame's
// size changes, and cleared to the far plane for a frame.

#include "Graphics/Graphics.h"
#include "Testing/Testing.h"

#include <utility>

using graphics::AdapterChoice;
using graphics::DepthBuffer;
using graphics::Frame;
using graphics::Gpu;
using graphics::Image;
using graphics::Offscreen;

TEST(the_buffer_follows_the_frame_s_size) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    DepthBuffer depth(gpu);
    Offscreen small_(gpu, 64, 32, {});
    Frame frame = small_.begin();
    D3D12_CPU_DESCRIPTOR_HANDLE first = depth.clear(gpu, frame);
    small_.finish(gpu, std::move(frame));
    Offscreen large(gpu, 128, 96, {});
    frame = large.begin();
    // The same view, of a new buffer, which the frame's commands clear.
    D3D12_CPU_DESCRIPTOR_HANDLE second = depth.view(gpu, frame);
    CHECK_EQ(first.ptr, second.ptr);
    Image image = large.finish(gpu, std::move(frame));
    CHECK_EQ(std::pair(image.width, image.height), std::pair(128u, 96u));
}
