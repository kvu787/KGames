#include "Graphics/Presenter.h"

#include "Graphics/Error.h"
#include "Graphics/Graphics.h"

#include <algorithm>
#include <format>
#include <new>
#include <tuple>

#pragma comment(lib, "dxgi.lib")
#pragma comment(lib, "user32.lib")

namespace graphics {

namespace {

constexpr std::uint32_t BufferCount = 2;

std::uint32_t swap_chain_flags(const Gpu& gpu) {
    std::uint32_t flags = DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT;
    if (gpu.tearing_supported) {
        flags |= DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING;
    }
    return flags;
}

// The window's swap chain, with a maximum frame latency of 1.
ComPtr<IDXGISwapChain3> create_swap_chain(const Gpu& gpu, HWND window, std::uint32_t width, std::uint32_t height, std::uint32_t flags) {
    DXGI_SWAP_CHAIN_DESC1 description{
        .Width = width,
        .Height = height,
        .Format = TargetFormat,
        .SampleDesc = {.Count = 1, .Quality = 0},
        .BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT,
        .BufferCount = BufferCount,
        .SwapEffect = DXGI_SWAP_EFFECT_FLIP_DISCARD,
        .Flags = flags,
    };
    ComPtr<IDXGISwapChain1> created;
    base::check(gpu.factory->CreateSwapChainForHwnd(gpu.queue.Get(), window, &description, nullptr, nullptr, &created));
    ComPtr<IDXGISwapChain3> swap_chain;
    base::check(created.As(&swap_chain));
    // DXGI would otherwise watch the window's messages, and switch it
    // to fullscreen with SetFullscreenState on Alt+Enter, which
    // changes the display's mode, as Direct3D 12 has no exclusive
    // fullscreen. The programs handle their window themselves: F11
    // switches to borderless fullscreen.
    base::check(gpu.factory->MakeWindowAssociation(window, DXGI_MWA_NO_WINDOW_CHANGES | DXGI_MWA_NO_ALT_ENTER));
    base::check(swap_chain->SetMaximumFrameLatency(1));
    return swap_chain;
}

// An error, with why the device was removed, if it was, which
// DXGI_ERROR_DEVICE_REMOVED and DXGI_ERROR_DEVICE_RESET don't say.
Error with_removed_reason(const Gpu& gpu, HRESULT error) {
    HRESULT reason = gpu.device->GetDeviceRemovedReason();
    if (SUCCEEDED(reason)) {
        return Error(error);
    }
    return Error(error, std::format("{} The device was removed: {}", Error(error).message(), base::WindowsError(reason).what()));
}

} // namespace

Presenter::Presenter(const Gpu& gpu, HWND window, std::uint32_t width, std::uint32_t height, bool allow_tearing) :
    flags(swap_chain_flags(gpu)),
    tearing_(allow_tearing && gpu.tearing_supported),
    swap_chain(create_swap_chain(gpu, window, std::max(width, 1u), std::max(height, 1u), flags)),
    waitable(swap_chain->GetFrameLatencyWaitableObject()),
    views(gpu.device.Get(), D3D12_DESCRIPTOR_HEAP_TYPE_RTV, BufferCount * 2, false),
    width(std::max(width, 1u)),
    height(std::max(height, 1u)),
    fence(gpu.fence()) {
    std::tie(allocator, list) = command_list(gpu.device.Get());
    fence_event = CreateEventW(nullptr, FALSE, FALSE, nullptr);
    if (fence_event == nullptr) {
        throw base::last_error();
    }
    try {
        create_views(gpu);
    } catch (const base::WindowsError&) {
        // What the destructor does, which doesn't run for a presenter whose
        // constructor fails. No frame of its is on the GPU to wait for.
        CloseHandle(fence_event);
        CloseHandle(waitable);
        throw;
    }
}

Presenter::~Presenter() {
    // A presenter that was moved from holds nothing.
    if (!swap_chain) {
        return;
    }
    // The swap chain and its buffers go now, so the GPU must be done
    // with the last frame first, whichever way the main loop ended:
    // releasing what a frame still draws with makes its shaders fault,
    // which NVIDIA's driver logs, and which stutters every app on the
    // GPU.
    if (fence->GetCompletedValue() < last_frame && SUCCEEDED(fence->SetEventOnCompletion(last_frame, fence_event))) {
        WaitForSingleObject(fence_event, INFINITE);
    }
    CloseHandle(fence_event);
    CloseHandle(waitable);
}

Presenter::Presenter(Presenter&& other) noexcept :
    flags(other.flags),
    tearing_(other.tearing_),
    swap_chain(std::move(other.swap_chain)),
    waitable(std::exchange(other.waitable, nullptr)),
    buffers(std::move(other.buffers)),
    views(std::move(other.views)),
    allocator(std::move(other.allocator)),
    list(std::move(other.list)),
    width(other.width),
    height(other.height),
    fence(std::move(other.fence)),
    fence_event(std::exchange(other.fence_event, nullptr)),
    last_frame(other.last_frame),
    waitable_signaled(other.waitable_signaled),
    present_time_(other.present_time_),
    vsync_(other.vsync_) {}

Presenter& Presenter::operator=(Presenter&& other) noexcept {
    if (this != &other) {
        this->~Presenter();
        new (this) Presenter(std::move(other));
    }
    return *this;
}

bool Presenter::tearing() const {
    return tearing_ && !vsync_;
}

void Presenter::set_vsync(bool vsync) {
    vsync_ = vsync;
}

bool Presenter::vsync() const {
    return vsync_;
}

std::pair<std::uint32_t, std::uint32_t> Presenter::size() const {
    return {width, height};
}

void Presenter::resize(Gpu& gpu, std::uint32_t new_width, std::uint32_t new_height) {
    new_width = std::max(new_width, 1u);
    new_height = std::max(new_height, 1u);
    if (new_width == width && new_height == height) {
        return;
    }
    gpu.flush();
    buffers.clear();
    base::check(swap_chain->ResizeBuffers(BufferCount, new_width, new_height, TargetFormat, flags));
    width = new_width;
    height = new_height;
    create_views(gpu);
}

void Presenter::create_views(const Gpu& gpu) {
    for (std::uint32_t index = 0; index < BufferCount; index++) {
        ComPtr<ID3D12Resource> buffer;
        base::check(swap_chain->GetBuffer(index, IID_PPV_ARGS(&buffer)));
        create_target_views(gpu.device.Get(), buffer.Get(), views, static_cast<size_t>(index) * 2);
        buffers.push_back(std::move(buffer));
    }
}

bool Presenter::poll_ready(const Gpu& gpu) {
    if (!waitable_signaled) {
        waitable_signaled = WaitForSingleObject(waitable, 0) == WAIT_OBJECT_0;
    }
    return waitable_signaled && gpu.is_complete(last_frame);
}

void Presenter::wait_ready(const Gpu& gpu, std::uint32_t milliseconds) {
    gpu.wait_for(last_frame);
    if (!waitable_signaled) {
        DWORD result = MsgWaitForMultipleObjects(1, &waitable, FALSE, milliseconds, QS_ALLINPUT);
        waitable_signaled = result == WAIT_OBJECT_0;
    }
}

Frame Presenter::begin() {
    waitable_signaled = false;
    base::check(allocator->Reset());
    base::check(list->Reset(allocator.Get(), nullptr));
    size_t index = swap_chain->GetCurrentBackBufferIndex();
    ComPtr<ID3D12Resource> target = buffers[index];
    D3D12_RESOURCE_BARRIER barrier = transition(target.Get(), D3D12_RESOURCE_STATE_PRESENT, D3D12_RESOURCE_STATE_RENDER_TARGET);
    list->ResourceBarrier(1, &barrier);
    return Frame(list, std::move(target), views.cpu(index * 2), views.cpu(index * 2 + 1), width, height);
}

std::int64_t Presenter::present_time() const {
    return present_time_;
}

std::uint64_t Presenter::present(Gpu& gpu, Frame frame) {
    D3D12_RESOURCE_BARRIER barrier = transition(frame.target.Get(), D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_PRESENT);
    frame.list->ResourceBarrier(1, &barrier);
    base::check(frame.list->Close());
    execute(gpu.queue.Get(), frame.list.Get());
    UINT interval = 0;
    UINT present_flags = 0;
    if (vsync_) {
        interval = 1;
    } else if (tearing_) {
        present_flags = DXGI_PRESENT_ALLOW_TEARING;
    }
    LARGE_INTEGER counter{.QuadPart = present_time_};
    QueryPerformanceCounter(&counter);
    present_time_ = counter.QuadPart;
    // Present's DXGI_STATUS codes are successes, which change nothing
    // here. A flip model swap chain never returns DXGI_STATUS_OCCLUDED.
    auto present_frame = [&]() -> std::expected<void, HRESULT> {
        HRESULT result = swap_chain->Present(interval, present_flags);
        if (FAILED(result)) {
            return std::unexpected(result);
        }
        return {};
    };
    std::expected<std::uint64_t, HRESULT> fence_value = present_then_signal(last_frame, present_frame, [&] { return gpu.signal(); });
    if (!fence_value) {
        throw with_removed_reason(gpu, fence_value.error());
    }
    return *fence_value;
}

} // namespace graphics
