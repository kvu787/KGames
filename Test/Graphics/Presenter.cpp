// What the presenter does once a frame's commands are on the GPU: it keeps
// the fence value that the GPU signals after them, which it waits for before
// it lets go of the frame's buffers, whether Present succeeds or not. A
// failed Present returns its error, and the main loop ends, so the value
// must be kept before then, or the buffers would go while the GPU still
// draws into them.

#include "Graphics/Graphics.h"
#include "Testing/Testing.h"

#include <cstdint>
#include <expected>
#include <string_view>
#include <utility>
#include <vector>

using graphics::present_then_signal;

namespace {

constexpr HRESULT Failed = static_cast<HRESULT>(0x887A0001);

} // namespace

TEST(a_presented_frame_s_fence_value_is_kept_and_returned) {
    std::uint64_t last_frame = 3;
    std::uint64_t fence = present_then_signal(last_frame, [] { return std::expected<void, HRESULT>(); }, [] { return std::uint64_t{4}; }).value();
    CHECK_EQ(std::pair(fence, last_frame), std::pair(std::uint64_t{4}, std::uint64_t{4}));
}

TEST(a_frame_whose_present_fails_is_still_waited_for) {
    std::uint64_t last_frame = 3;
    std::expected<std::uint64_t, HRESULT> result =
        present_then_signal(last_frame, [] { return std::expected<void, HRESULT>(std::unexpected(Failed)); }, [] { return std::uint64_t{4}; });
    CHECK(!result.has_value());
    CHECK_EQ(result.error(), Failed);
    CHECK_EQ(last_frame, 4);
}

TEST(the_fence_is_signaled_after_the_present) {
    std::vector<std::string_view> order;
    std::uint64_t last_frame = 0;
    std::expected<std::uint64_t, HRESULT> result = present_then_signal(
        last_frame,
        [&] {
            order.push_back("present");
            return std::expected<void, HRESULT>();
        },
        [&] {
            order.push_back("signal");
            return std::uint64_t{1};
        });
    CHECK(result.has_value());
    CHECK_EQ(order, (std::vector<std::string_view>{"present", "signal"}));
}
