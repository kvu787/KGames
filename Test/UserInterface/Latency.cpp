// Tests the input latency that the PerfBar shows: from when
// GameInput got a frame's newest input to when the display took the frame,
// with its parts, and how frames that never reach the screen pass their input
// on, as PresentMon's input latency does. The presents' fates are made up,
// on the performance counter's clock, near now.

#include "Testing/Testing.h"
#include "UserInterface/Application/Performance.h"
#include "UserInterface/Platform/Clock.h"

#include <cmath>

using present_timing::Present;
using present_timing::PresentMode;
using user_interface::performance::Frame;
using user_interface::performance::LatencyState;
using user_interface::performance::Performance;
using user_interface::performance::Summary;
namespace clock_ = user_interface::clock;

namespace {

std::int64_t ticks(double milliseconds) {
    return static_cast<std::int64_t>(milliseconds * static_cast<double>(clock_::frequency()) / 1000.0);
}

void close(double value, double expected) {
    CHECK(std::abs(value - expected) < 0.001, "{} isn't {}", value, expected);
}

// The present of the frame that called Present at a time, which DXGI's event
// times a little after, and when the display took it, if it did.
Present present(std::int64_t called, std::optional<std::int64_t> screen) {
    return Present{
        .start = called + 5,
        .thread = 1,
        .mode = PresentMode::IndependentFlip,
        .tearing = true,
        .screen = screen,
        .flip_delay = 0,
    };
}

} // namespace

TEST(a_frame_s_latency_runs_from_its_input_to_the_screen_in_three_parts) {
    std::int64_t base = clock_::now();
    Performance performance = Performance::given();
    Frame frame{
        .input = base,
        .read = base + ticks(1.0),
        .present = base + ticks(3.0),
    };
    performance.presented(frame);
    performance.known({present(frame.present, base + ticks(10.0))});
    performance.update(0.0);
    const Summary& summary = performance.summary();
    CHECK_EQ(summary.latency, LatencyState(LatencyState::Measuring{}));
    CHECK_EQ(summary.samples, 1u);
    close(summary.mean, 10.0);
    close(summary.waiting, 1.0);
    close(summary.building, 2.0);
    close(summary.presenting, 7.0);
    CHECK_EQ(summary.mode, PresentMode::IndependentFlip);
    CHECK(summary.immediate);
}

TEST(a_frame_that_never_reaches_the_screen_passes_its_input_to_the_next_that_does) {
    std::int64_t base = clock_::now();
    Performance performance = Performance::given();
    Frame dropped{
        .input = base,
        .read = base + ticks(1.0),
        .present = base + ticks(2.0),
    };
    Frame shown{
        .input = std::nullopt,
        .read = base + ticks(3.0),
        .present = base + ticks(4.0),
    };
    performance.presented(dropped);
    performance.presented(shown);
    performance.known({present(dropped.present, std::nullopt), present(shown.present, base + ticks(9.0))});
    performance.update(0.0);
    const Summary& summary = performance.summary();
    CHECK_EQ(summary.samples, 1u);
    close(summary.mean, 9.0);
    close(summary.presenting, 7.0);
}

TEST(a_frame_on_the_screen_with_input_of_its_own_takes_only_its_own) {
    std::int64_t base = clock_::now();
    Performance performance = Performance::given();
    Frame dropped{
        .input = base,
        .read = base + ticks(1.0),
        .present = base + ticks(2.0),
    };
    Frame shown{
        .input = base + ticks(2.5),
        .read = base + ticks(3.0),
        .present = base + ticks(4.0),
    };
    performance.presented(dropped);
    performance.presented(shown);
    performance.known({present(dropped.present, std::nullopt), present(shown.present, base + ticks(9.0))});
    performance.update(0.0);
    CHECK_EQ(performance.summary().samples, 1u);
    close(performance.summary().mean, 6.5);
}

TEST(frames_without_input_have_no_latency_but_count_on_the_screen) {
    std::int64_t base = clock_::now();
    Performance performance = Performance::given();
    std::vector<Present> presents;
    for (int index = 0; index < 11; index++) {
        std::int64_t called = base + ticks(static_cast<double>(index) * 10.0);
        performance.presented(Frame{
            .input = std::nullopt,
            .read = called - ticks(1.0),
            .present = called,
        });
        presents.push_back(present(called, called + ticks(5.0)));
    }
    performance.known(presents);
    performance.update(0.0);
    const Summary& summary = performance.summary();
    CHECK_EQ(summary.samples, 0u);
    close(summary.displayed_per_second, 100.0);
}

TEST(presents_that_aren_t_the_main_loop_s_are_left_alone) {
    std::int64_t base = clock_::now();
    Performance performance = Performance::given();
    Frame frame{
        .input = base,
        .read = base + ticks(1.0),
        .present = base + ticks(3.0),
    };
    performance.presented(frame);
    // A present that started before the frame's Present call.
    performance.known({present(base, base + ticks(4.0))});
    performance.update(0.0);
    CHECK_EQ(performance.summary().latency, LatencyState(LatencyState::Starting{}));
    performance.known({present(frame.present, base + ticks(10.0))});
    performance.update(1.0);
    CHECK_EQ(performance.summary().samples, 1u);
}

TEST(the_mean_and_99th_percentile_are_of_the_last_second) {
    std::int64_t base = clock_::now() - ticks(900.0);
    Performance performance = Performance::given();
    std::vector<Present> presents;
    // A frame that reached the screen more than a second ago, which doesn't
    // count.
    Frame old{
        .input = base - ticks(600.0),
        .read = base - ticks(600.0),
        .present = base - ticks(600.0),
    };
    performance.presented(old);
    presents.push_back(present(old.present, old.present + ticks(1.0)));
    for (int index = 0; index < 100; index++) {
        std::int64_t input = base + ticks(static_cast<double>(index) * 5.0);
        Frame frame{
            .input = input,
            .read = input,
            .present = input,
        };
        performance.presented(frame);
        presents.push_back(present(frame.present, input + ticks(static_cast<double>(index) + 1.0)));
    }
    performance.known(presents);
    performance.update(0.0);
    const Summary& summary = performance.summary();
    CHECK_EQ(summary.samples, 100u);
    close(summary.mean, 50.5);
    close(summary.percentile_99, 99.01);
}

TEST(presents_whose_events_come_more_than_a_second_late_still_count) {
    // Without PresentTiming's flushes, ETW hands over its events about
    // presents once a second, each 1 to 2 seconds after the frame, which the
    // main loop updates the stats for in the meantime.
    std::int64_t base = clock_::now() - ticks(1500.0);
    Performance performance = Performance::given();
    std::vector<Present> presents;
    for (int index = 0; index < 20; index++) {
        std::int64_t input = base + ticks(static_cast<double>(index) * 10.0);
        Frame frame{
            .input = input,
            .read = input,
            .present = input,
        };
        performance.presented(frame);
        presents.push_back(present(frame.present, input + ticks(8.0)));
    }
    performance.update(0.0);
    performance.known(presents);
    performance.update(1.0);
    const Summary& summary = performance.summary();
    CHECK_EQ(summary.latency, LatencyState(LatencyState::Measuring{}));
    CHECK_EQ(summary.samples, 20u);
    close(summary.mean, 8.0);
}

TEST(without_presents_there_is_no_latency_and_it_says_why) {
    Performance performance = Performance::without_latency("none in a test");
    performance.frame_started(0.0);
    performance.frame_started(0.01);
    performance.update(0.01);
    const Summary& summary = performance.summary();
    CHECK_EQ(summary.latency, LatencyState(LatencyState::Unavailable{"none in a test"}));
    close(summary.frames_per_second, 100.0);
}
