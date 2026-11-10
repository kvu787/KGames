// A window's swap chain, paced for the lowest input latency:
//
// - It has a frame latency waitable object and a maximum frame latency of 1,
//   and only one frame is on the GPU at a time, so no frame waits behind
//   another in DXGI's queue.
// - `poll_ready` says, without blocking, whether the GPU has finished the
//   previous frame and DXGI can take another. The main loop polls it while it
//   handles window messages, and reads input only once it's ready, so no input
//   ages during the wait.
// - With vertical sync on, as every program starts, frames wait for the
//   screen's refresh, and never tear, so a program draws no faster than the
//   screen shows.
// - With it off, frames present without waiting for vertical sync. With
//   tearing allowed, a frame can reach the screen mid-refresh, when Windows
//   flips it straight there, like in borderless fullscreen.

#pragma once

#include "Base/Windows.h"
#include "Graphics/Device.h"
#include "Graphics/Frame.h"
#include "Graphics/Resources.h"

#include <cstdint>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <expected>
#include <utility>
#include <vector>
#include <wrl/client.h>

namespace graphics {

class Presenter {
public:
    // With `allow_tearing` false, frames still don't wait for vertical sync,
    // but the screen takes the newest one at each refresh, so none tear.
    Presenter(const Gpu& gpu, HWND window, std::uint32_t width, std::uint32_t height, bool allow_tearing);
    ~Presenter();
    Presenter(const Presenter&) = delete;
    Presenter& operator=(const Presenter&) = delete;
    Presenter(Presenter&& other) noexcept;
    Presenter& operator=(Presenter&& other) noexcept;

    // Whether frames may tear.
    bool tearing() const;

    // Makes frames wait for vertical sync, which paces them at the display's
    // refresh rate instead of as fast as the GPU goes, so that they don't
    // keep the GPU busy. They don't tear then. Every program starts with it.
    void set_vsync(bool vsync);

    // Whether frames wait for vertical sync.
    bool vsync() const;

    std::pair<std::uint32_t, std::uint32_t> size() const;

    // Makes the back buffers the window's new size. Waits for the GPU.
    void resize(Gpu& gpu, std::uint32_t new_width, std::uint32_t new_height);

    // Whether a frame can begin now: the GPU has finished the last one, and
    // DXGI's queue has room. Never blocks.
    bool poll_ready(const Gpu& gpu);

    // Waits, without spinning, until a frame can begin, a window message
    // comes, or the milliseconds pass. With vertical sync, a frame can't
    // begin before the last one has gone to the screen anyway, so waiting
    // instead of polling costs only the thread's wake-up.
    void wait_ready(const Gpu& gpu, std::uint32_t milliseconds);

    // Begins a frame in the back buffer, once poll_ready says so. The back
    // buffer is in the render target state.
    Frame begin();

    // The performance counter right before the last frame's Present call,
    // which input latency measures to and from.
    std::int64_t present_time() const;

    // Submits the frame's commands and presents it. Returns the fence value
    // that the GPU signals once it has drawn the frame, which the presenter
    // waits for before it lets go of the frame's buffers, even when Present
    // fails.
    std::uint64_t present(Gpu& gpu, Frame frame);

private:
    void create_views(const Gpu& gpu);

    // First, since the swap chain is made with them.
    std::uint32_t flags;
    bool tearing_;
    ComPtr<IDXGISwapChain3> swap_chain;
    HANDLE waitable;
    std::vector<ComPtr<ID3D12Resource>> buffers;
    DescriptorHeap views;
    ComPtr<ID3D12CommandAllocator> allocator;
    ComPtr<ID3D12GraphicsCommandList> list;
    std::uint32_t width;
    std::uint32_t height;
    // The GPU's fence, and an event to wait on it with, so that the swap
    // chain and its buffers go only once the GPU is done with the last
    // frame, however the main loop ends.
    ComPtr<ID3D12Fence> fence;
    HANDLE fence_event = nullptr;
    // What the fence reaches once the GPU finishes the last frame presented.
    std::uint64_t last_frame = 0;
    // Whether the waitable object was signaled since the last frame began.
    // Waiting on it takes the signal, so it's remembered until the frame.
    bool waitable_signaled = false;
    // The performance counter right before the last Present call.
    std::int64_t present_time_ = 0;
    // Whether frames wait for vertical sync, which `set_vsync` switches.
    bool vsync_ = false;
};

// Presents, then has the GPU signal the fence after the frame's commands,
// which are on the GPU already, and keeps the value in `last_frame`, whether
// the present succeeded or not, so that whatever lets go of the frame's
// resources waits for the value however the frame ended. Returns the value,
// or the present's error.
//
// `present` returns a std::expected<void, HRESULT>, and `signal` the value,
// as a std::uint64_t.
template <class Present, class Signal> std::expected<std::uint64_t, HRESULT> present_then_signal(std::uint64_t& last_frame, Present&& present, Signal&& signal) {
    std::expected<void, HRESULT> presented = std::forward<Present>(present)();
    last_frame = std::forward<Signal>(signal)();
    return presented.transform([&] { return last_frame; });
}

} // namespace graphics
