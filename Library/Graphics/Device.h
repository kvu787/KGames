// The GPU: its adapter, device, command queue and fence, and a command list
// for setup work, like copying meshes into GPU memory.

#pragma once

#include "Base/Sum.h"
#include "Base/Windows.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <d3d12.h>
#include <d3d12sdklayers.h>
#include <dxgi1_6.h>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <wrl/client.h>

namespace graphics {

using Microsoft::WRL::ComPtr;

// Which GPU draws.
struct AdapterChoice {
    // The GPU that the monitor is an output of. On a laptop whose screen is
    // wired to the integrated GPU, drawing with the discrete one would add a
    // copy between the GPUs to every frame, and keep Windows from flipping
    // frames straight to the screen.
    struct Monitor {
        HMONITOR monitor;
    };
    // The fastest hardware GPU, or WARP if there's none, like for tests.
    struct HighPerformance {};
    // Microsoft's software rasterizer.
    struct Warp {};
    std::variant<Monitor, HighPerformance, Warp> value;

    template <base::AlternativeOf<AdapterChoice> T> AdapterChoice(T&& alternative) : value(std::forward<T>(alternative)) {}
};

// Where the GPU's scheduler puts the process's work among other processes'.
// Windows' job objects can't limit the GPU, so a process that must stay out
// of a game's way lowers its own priority: RunLightly asks for Idle through
// KGAMES_GPU_PRIORITY, which Gpu's constructor reads, for coding agents'
// tests and test runs while someone plays a game, whose frames then go first.
enum class GpuPriority {
    Idle,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
};

// How a failed check in a test shows a priority: its name, like "Idle".
std::string describe_value(GpuPriority priority);

// The priority of the name, like "Idle".
std::optional<GpuPriority> gpu_priority_from_name(std::string_view name);

// What KGAMES_GPU_PRIORITY asks for, or Normal when it's unset.
GpuPriority gpu_priority_from_environment();

// Sets where the GPU's scheduler puts this process's work, for every device
// it has and makes. Raising it above Normal needs a privilege that only
// administrators have.
void set_gpu_priority(GpuPriority priority);

// Where the GPU's scheduler puts this process's work now.
GpuPriority gpu_priority();

// What a GPU memory segment holds for this process: the bytes of it that the
// process has reserved, and the bytes that the GPU's scheduler is letting it
// have, which depends on what else is running.
struct VideoMemory {
    std::uint64_t used = 0;
    std::uint64_t budget = 0;

    bool operator==(const VideoMemory&) const = default;
};

class Gpu {
public:
    ComPtr<IDXGIFactory6> factory;
    ComPtr<IDXGIAdapter1> adapter;
    // Like "NVIDIA GeForce RTX 5090 Laptop GPU".
    std::string name;
    ComPtr<ID3D12Device> device;
    ComPtr<ID3D12CommandQueue> queue;
    // Whether presenting may tear, which needs both the display driver's and
    // Windows' support.
    bool tearing_supported = false;

    explicit Gpu(AdapterChoice choice);
    ~Gpu();
    Gpu(const Gpu&) = delete;
    Gpu& operator=(const Gpu&) = delete;
    Gpu(Gpu&& other) noexcept;
    Gpu& operator=(Gpu&& other) noexcept;

    // How much GPU memory this process has reserved, and how much the GPU's
    // scheduler is letting it have, for both of the GPU's segments: local,
    // which on a discrete GPU is its own memory, and non-local, which is
    // system memory that it reads across PCIe. Every committed resource
    // counts, and so does what the driver allocates for the process.
    std::array<VideoMemory, 2> video_memory() const;

    // The fence that `signal` has the queue signal, for what waits on its
    // own, like the presenter as it goes.
    ComPtr<ID3D12Fence> fence() const;

    // Has the queue signal the fence once the GPU finishes all the work
    // submitted so far, and returns the value that it will signal.
    std::uint64_t signal();

    // The newest value that the GPU has signaled.
    std::uint64_t completed() const;

    bool is_complete(std::uint64_t value) const;

    // Blocks until the GPU has signaled the value.
    void wait_for(std::uint64_t value) const;

    // Blocks until the GPU has finished everything submitted so far.
    void flush();

    // Records commands with `record`, which takes the command list as an
    // ID3D12GraphicsCommandList*, runs them, and waits for the GPU to finish
    // them, so that whatever they read, like an upload buffer, can go right
    // after. For setup, not for frames.
    template <class Record> void run(Record&& record) {
        base::check(setup_allocator->Reset());
        base::check(setup_list->Reset(setup_allocator.Get(), nullptr));
        std::forward<Record>(record)(setup_list.Get());
        base::check(setup_list->Close());
        ID3D12CommandList* lists[] = {setup_list.Get()};
        queue->ExecuteCommandLists(1, lists);
        flush();
        report_messages();
    }

    // Writes what the debug layer reported since the last call to standard
    // error, and returns how many were errors. Without the debug layer, it
    // has nothing to report.
    size_t report_messages() const;

private:
    ComPtr<ID3D12Fence> fence_;
    HANDLE fence_event = nullptr;
    std::uint64_t last_signal = 0;
    ComPtr<ID3D12CommandAllocator> setup_allocator;
    ComPtr<ID3D12GraphicsCommandList> setup_list;
    // With the debug layer, which only debug builds turn on.
    ComPtr<ID3D12InfoQueue> info_queue;
};

} // namespace graphics
