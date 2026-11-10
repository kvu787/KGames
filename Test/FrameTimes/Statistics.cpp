// FrameTimes' statistics, over made-up presents: frame times are the
// differences between the screen times of the presents that reached the
// screen, and presents that didn't reach it don't count.

#include "FrameTimes/FrameTimes.h"
#include "PresentTiming/PresentTiming.h"
#include "Testing/Testing.h"

#include <cmath>
#include <cstdint>
#include <initializer_list>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

using frame_times::maximum;
using frame_times::mean;
using frame_times::median;
using frame_times::most_common_mode;
using frame_times::percentile;
using frame_times::report;
using frame_times::stutters;
using frame_times::summarize;
using frame_times::Summary;
using present_timing::Present;
using present_timing::PresentMode;

namespace {

// The performance counter's rate on Windows 10 and later.
constexpr std::int64_t Frequency = 10'000'000;
constexpr std::int64_t Millisecond = Frequency / 1000;

// A present that reached the screen at `screen` milliseconds.
Present shown(std::int64_t screen, PresentMode mode) {
    return Present{
        .start = (screen - 5) * Millisecond,
        .thread = 1,
        .mode = mode,
        .tearing = false,
        .screen = screen * Millisecond,
        .flip_delay = 0,
    };
}

// A present that started at `start` milliseconds and never reached the
// screen, like one that a newer one replaced.
Present missed(std::int64_t start, PresentMode mode) {
    return Present{
        .start = start * Millisecond,
        .thread = 1,
        .mode = mode,
        .tearing = false,
        .screen = std::nullopt,
        .flip_delay = 0,
    };
}

// Presents that reached the screen after each of these frame times, in
// milliseconds, from 0.
std::vector<Present> frames(std::span<const std::int64_t> times) {
    std::vector<Present> presents{shown(0, PresentMode::IndependentFlip)};
    std::int64_t screen = 0;
    for (std::int64_t time : times) {
        screen += time;
        presents.push_back(shown(screen, PresentMode::IndependentFlip));
    }
    return presents;
}

void check_close(double actual, double expected) {
    CHECK(std::abs(actual - expected) < 1e-9, "{} isn't {}", base::display(actual), base::display(expected));
}

void check_all_close(std::span<const double> actual, std::span<const double> expected) {
    CHECK_EQ(actual.size(), expected.size(), "{} isn't {}", testing::describe(actual), testing::describe(expected));
    for (size_t index = 0; index < actual.size(); index++) {
        check_close(actual[index], expected[index]);
    }
}

void check_all_close(std::span<const double> actual, std::initializer_list<double> expected) {
    check_all_close(actual, std::span<const double>(expected));
}

} // namespace

TEST(no_presents_have_no_statistics) {
    CHECK(frame_times::frame_times({}, Frequency).empty());
    CHECK_EQ(summarize({}, Frequency), std::nullopt);
    CHECK_EQ(most_common_mode({}), std::nullopt);
    CHECK_EQ(mean({}), std::nullopt);
    CHECK_EQ(median({}), std::nullopt);
    CHECK_EQ(percentile({}, 99.0), std::nullopt);
    CHECK_EQ(maximum({}), std::nullopt);
    CHECK_EQ(stutters({}), 0);
}

TEST(frame_times_are_the_differences_between_screen_times) {
    std::vector<Present> presents{
        shown(0, PresentMode::IndependentFlip),
        shown(16, PresentMode::IndependentFlip),
        shown(33, PresentMode::IndependentFlip),
        shown(50, PresentMode::IndependentFlip),
    };
    check_all_close(frame_times::frame_times(presents, Frequency), {16.0, 17.0, 17.0});
    // On another counter rate, the same ticks are other times.
    check_all_close(frame_times::frame_times(presents, 2 * Frequency), {8.0, 8.5, 8.5});
}

TEST(frame_times_follow_the_screen_times_order) {
    std::vector<Present> presents{
        shown(20, PresentMode::ComposedFlip),
        shown(0, PresentMode::ComposedFlip),
        shown(10, PresentMode::ComposedFlip),
    };
    check_all_close(frame_times::frame_times(presents, Frequency), {10.0, 10.0});
}

TEST(presents_that_never_reached_the_screen_dont_count) {
    std::vector<Present> presents{
        missed(-10, PresentMode::ComposedFlip),  shown(0, PresentMode::IndependentFlip), missed(3, PresentMode::ComposedFlip),    missed(4, PresentMode::ComposedFlip),
        shown(10, PresentMode::IndependentFlip), missed(13, PresentMode::ComposedFlip),  shown(20, PresentMode::IndependentFlip),
    };
    check_all_close(frame_times::frame_times(presents, Frequency), {10.0, 10.0});
    // Missed presents' modes don't count either, though there are more of
    // them.
    CHECK_EQ(most_common_mode(presents), std::optional(PresentMode::IndependentFlip));
    Summary summary = summarize(presents, Frequency).value();
    CHECK_EQ(summary.frames, 2);
    check_close(summary.mean, 10.0);
    CHECK_EQ(summary.mode, PresentMode::IndependentFlip);
}

TEST(fewer_than_two_presents_on_the_screen_have_no_frame_time) {
    std::vector<Present> none{missed(0, PresentMode::ComposedFlip), missed(10, PresentMode::ComposedFlip)};
    CHECK(frame_times::frame_times(none, Frequency).empty());
    CHECK_EQ(summarize(none, Frequency), std::nullopt);
    CHECK_EQ(most_common_mode(none), std::nullopt);

    std::vector<Present> one{missed(0, PresentMode::ComposedFlip), shown(10, PresentMode::ComposedFlip), missed(20, PresentMode::ComposedFlip)};
    CHECK(frame_times::frame_times(one, Frequency).empty());
    CHECK_EQ(summarize(one, Frequency), std::nullopt);
    CHECK_EQ(most_common_mode(one), std::optional(PresentMode::ComposedFlip));
}

TEST(steady_frames_with_two_hitches) {
    // 98 frames of 10 ms, then one of 30 ms and one of 50 ms.
    std::vector<std::int64_t> times(98, 10);
    times.push_back(30);
    times.push_back(50);
    Summary summary = summarize(frames(times), Frequency).value();
    CHECK_EQ(summary.frames, 100);
    check_close(summary.mean, 10.6);
    check_close(summary.frames_per_second, 1000.0 / 10.6);
    check_close(summary.median, 10.0);
    // The 99th of 100 sorted frame times.
    check_close(summary.percentile_99, 30.0);
    check_close(summary.maximum, 50.0);
    CHECK_EQ(summary.stutters, 2);
    CHECK_EQ(summary.mode, PresentMode::IndependentFlip);
}

TEST(median_of_an_even_count_is_halfway_between_the_middle_two) {
    CHECK_EQ(median(std::vector{3.0, 1.0, 2.0}), std::optional(2.0));
    CHECK_EQ(median(std::vector{4.0, 1.0, 3.0, 2.0}), std::optional(2.5));
    CHECK_EQ(median(std::vector{7.0}), std::optional(7.0));
}

TEST(percentile_is_the_nearest_rank) {
    std::vector<double> values;
    for (int value = 10; value >= 1; value--) {
        values.push_back(static_cast<double>(value));
    }
    CHECK_EQ(percentile(values, 99.0), std::optional(10.0));
    CHECK_EQ(percentile(values, 90.0), std::optional(9.0));
    CHECK_EQ(percentile(values, 50.0), std::optional(5.0));
    CHECK_EQ(percentile(values, 100.0), std::optional(10.0));
    CHECK_EQ(percentile(values, 0.0), std::optional(1.0));
    CHECK_EQ(percentile(std::vector{7.0}, 99.0), std::optional(7.0));
    CHECK_EQ(maximum(values), std::optional(10.0));
}

TEST(stutters_are_frames_over_twice_the_median) {
    // The median is 10 ms: 20 ms is twice it, which isn't over.
    CHECK_EQ(stutters(std::vector{10.0, 10.0, 10.0, 20.0, 21.0}), 1);
    CHECK_EQ(stutters(std::vector<double>(5, 10.0)), 0);
}

TEST(the_mode_seen_most_wins_and_a_tie_goes_to_the_first) {
    std::vector<Present> mostly_flips{
        shown(0, PresentMode::ComposedFlip),  shown(10, PresentMode::IndependentFlip), shown(20, PresentMode::IndependentFlip),
        shown(30, PresentMode::ComposedFlip), shown(40, PresentMode::IndependentFlip),
    };
    CHECK_EQ(most_common_mode(mostly_flips), std::optional(PresentMode::IndependentFlip));

    std::vector<Present> tie{
        shown(0, PresentMode::OverlayPlane),
        shown(10, PresentMode::ComposedFlip),
        shown(20, PresentMode::ComposedFlip),
        shown(30, PresentMode::OverlayPlane),
    };
    CHECK_EQ(most_common_mode(tie), std::optional(PresentMode::OverlayPlane));
}

TEST(frames_on_the_screen_at_once_have_no_rate) {
    // PresentTiming's correction for NVIDIA's flip delay can put a present
    // on the screen with the one before it.
    Summary summary = summarize(std::vector{shown(10, PresentMode::ComposedFlip), shown(10, PresentMode::ComposedFlip)}, Frequency).value();
    CHECK_EQ(summary.frames, 1);
    check_close(summary.mean, 0.0);
    check_close(summary.frames_per_second, 0.0);
}

TEST(the_report_has_a_line_for_each_statistic) {
    Summary summary{
        .frames = 100,
        .frames_per_second = 1000.0 / 10.6,
        .mean = 10.6,
        .median = 10.0,
        .percentile_99 = 30.0,
        .maximum = 50.0,
        .stutters = 2,
        .mode = PresentMode::IndependentFlip,
    };
    std::string text = report(summary);
    std::vector<std::string_view> lines = base::lines(text);
    CHECK_EQ(lines, (std::vector<std::string_view>{
                        "  Frames           100",
                        "  Frames a second  94.3",
                        "  Mean             10.60 ms",
                        "  Median           10.00 ms",
                        "  99th percentile  30.00 ms",
                        "  Maximum          50.00 ms",
                        "  Stutters         2, frames over twice the median (20.00 ms)",
                        "  Present mode     Independent flip",
                    }));
}
