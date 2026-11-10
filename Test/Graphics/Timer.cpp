// The GPU timer: its spans measure the GPU's time between two points in a
// frame's commands, a span around a clear of a large target is longer than
// one around nothing, it's ready for the next frame after it's read, and it
// won't resolve a span that was begun and never ended.

#include "Base/Panic.h"
#include "Base/Text.h"
#include "Graphics/Graphics.h"
#include "Testing/Testing.h"

#include <cmath>
#include <cstdint>
#include <string_view>
#include <utility>
#include <vector>

using graphics::AdapterChoice;
using graphics::Frame;
using graphics::Gpu;
using graphics::GpuTimer;
using graphics::Offscreen;
using graphics::Span;

namespace {

// Draws a frame that clears the target `clears` times, with a span around
// the clears and one around nothing, and gives the two spans' milliseconds.
std::pair<double, double> time(Gpu& gpu, GpuTimer& timer, Offscreen& target, std::uint32_t clears) {
    Frame frame = target.begin();
    Span busy = timer.begin(frame);
    for (std::uint32_t index = 0; index < clears; index++) {
        frame.clear({static_cast<float>(index) / static_cast<float>(clears), 0.5f, 0.25f, 1.0f});
    }
    timer.end(frame, busy);
    Span idle = timer.begin(frame);
    timer.end(frame, idle);
    timer.resolve(frame);
    std::uint64_t fence = target.submit(gpu, std::move(frame));
    std::vector<double> spans = timer.read(gpu, fence);
    CHECK_EQ(spans.size(), 2);
    return {spans[0], spans[1]};
}

} // namespace

TEST(a_span_around_work_is_longer_than_one_around_nothing) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    GpuTimer timer(gpu, 2);
    Offscreen target(gpu, 2560, 1440, {});
    // A frame or two to warm the GPU up, then the one that's checked.
    time(gpu, timer, target, 64);
    auto [busy, idle] = time(gpu, timer, target, 64);
    CHECK(std::isfinite(busy) && busy > 0.0, "the busy span is {} ms", base::display(busy));
    CHECK(idle >= 0.0 && idle < busy, "an idle span of {} ms isn't shorter than the busy span of {} ms", base::display(idle), base::display(busy));
    // Short enough to be plausible even on WARP: 64 clears of 2560 x 1440.
    CHECK(busy < 1000.0, "the busy span is {} ms", base::display(busy));
}

TEST(a_frame_with_no_spans_reads_as_none) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    GpuTimer timer(gpu, 1);
    Offscreen target(gpu, 16, 16, {});
    Frame frame = target.begin();
    timer.resolve(frame);
    target.finish(gpu, std::move(frame));
    std::uint64_t fence = gpu.signal();
    CHECK_EQ(timer.read(gpu, fence), std::vector<double>());
}

// Resolving a query that was never ended is undefined, and can hang the GPU,
// so a span that's begun and not ended stops the program at the resolve.
TEST(resolving_a_span_that_never_ended_panics) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    GpuTimer timer(gpu, 2);
    Offscreen target(gpu, 16, 16, {});
    Frame frame = target.begin();
    Span ended = timer.begin(frame);
    timer.end(frame, ended);
    [[maybe_unused]] Span never_ended = timer.begin(frame);
    try {
        timer.resolve(frame);
    } catch (const base::Panic& panic) {
        CHECK(std::string_view(panic.what()).contains("never ended"), "{}", panic.what());
        return;
    }
    testing::fail("resolving didn't panic");
}
