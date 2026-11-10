// Times spans of a frame's GPU work, with timestamp queries: how long the
// GPU took from one point in the frame's commands to another, like from
// before the interface's draw to after it, apart from what the CPU took to
// record and submit them. For benchmarks and tests, not for frames that the
// user sees.
//
//     GpuTimer timer(gpu, 1);
//     Frame frame = offscreen.begin();
//     Span span = timer.begin(frame);
//     ... record the work ...
//     timer.end(frame, span);
//     timer.resolve(frame);
//     ... close, execute and signal ...
//     double milliseconds = timer.read(gpu, fence)[0];

#pragma once

#include "Base/Windows.h"
#include "Graphics/Device.h"
#include "Graphics/Frame.h"

#include <cstdint>
#include <d3d12.h>
#include <vector>
#include <wrl/client.h>

namespace graphics {

// A span that `begin` started, to pass to `end`.
class Span {
public:
    bool operator==(const Span&) const = default;

private:
    friend class GpuTimer;

    explicit Span(std::uint32_t index) : index(index) {}

    std::uint32_t index;
};

class GpuTimer {
public:
    // A timer for up to `spans` spans a frame.
    GpuTimer(const Gpu& gpu, std::uint32_t spans);
    GpuTimer(const GpuTimer&) = delete;
    GpuTimer& operator=(const GpuTimer&) = delete;
    GpuTimer(GpuTimer&&) = default;
    GpuTimer& operator=(GpuTimer&&) = default;

    // Starts a span. A timestamp is the time when the GPU has finished all of
    // the frame's work before it, so a span is the GPU's time for the work
    // between its begin and its end.
    Span begin(const Frame& frame);

    void end(const Frame& frame, Span span);

    // Records the copy of the frame's timestamps to where the CPU reads
    // them, after every span's end.
    void resolve(const Frame& frame) const;

    // Each span's length in milliseconds, in the order they began, once the
    // GPU has signaled `fence`, which it does after the frame's commands.
    // Ready for the next frame's spans.
    std::vector<double> read(const Gpu& gpu, std::uint64_t fence);

private:
    ComPtr<ID3D12QueryHeap> heap;
    // Where the timestamps are resolved to, which the CPU reads.
    ComPtr<ID3D12Resource> readback;
    // The queue's timestamp ticks a second.
    std::uint64_t frequency = 0;
    std::uint32_t spans;
    // How many spans this frame has begun.
    std::uint32_t begun = 0;
    // Which of them it has ended. Resolving a query that was never ended is
    // undefined, and can hang the GPU or remove the device.
    std::vector<bool> ended;
};

} // namespace graphics
