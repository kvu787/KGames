// The displays that Library/Displays finds: in physical pixels, with each
// one's own DPI, whatever DPI awareness the thread that asks has. Windows
// tells a thread that isn't DPI aware that every display is at 96 DPI, and
// scales its areas to match, so on a display that isn't at 100% scale, a
// program that asked before it made itself DPI aware would get them wrong.
// At 100%, both are the same anyway.

#include "Displays/Displays.h"
#include "Base/Windows.h"
#include "Testing/Testing.h"

#include <array>
#include <cstdint>
#include <future>
#include <string>
#include <thread>
#include <tuple>
#include <utility>
#include <vector>

namespace {

// A display's identifier, its area, its work area and its DPI.
using Seen = std::tuple<std::string, std::array<std::int32_t, 4>, std::array<std::int32_t, 4>, std::uint32_t>;

// The displays as a thread with this DPI awareness sees them.
std::vector<Seen> seen_with(DPI_AWARENESS_CONTEXT context) {
    // The task keeps what the thread throws, as a thread's panic is kept for
    // what joins it, so that it fails the test instead of stopping the tests.
    std::packaged_task<std::vector<Seen>()> task([context] {
        SetThreadDpiAwarenessContext(context);
        std::vector<Seen> seen;
        for (displays::Display& display : displays::enumerate()) {
            auto rect = [](RECT r) {
                return std::array<std::int32_t, 4>{r.left, r.top, r.right, r.bottom};
            };
            seen.emplace_back(std::move(display.identifier), rect(display.area), rect(display.work_area), display.dpi);
        }
        return seen;
    });
    std::future<std::vector<Seen>> result = task.get_future();
    std::thread thread(std::move(task));
    thread.join();
    return result.get();
}

} // namespace

TEST(a_thread_that_isnt_dpi_aware_sees_the_displays_as_they_are) {
    std::vector<Seen> aware = seen_with(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    CHECK(!aware.empty());
    CHECK_EQ(seen_with(DPI_AWARENESS_CONTEXT_UNAWARE), aware);
}
