// Tests what the PerfBar says: every label and value stays in the same
// columns whatever the stats are, starting, without input latency, or
// measuring it, with small numbers or large, in every present mode, with
// immediate flips or not, and numbers that aren't known yet are dashes.

#include "Base/Text.h"
#include "Testing/Testing.h"
#include "UserInterface/Application/Performance.h"
#include "UserInterface/Platform/Clock.h"

#include <span>
#include <tuple>

using present_timing::Present;
using present_timing::PresentMode;
using user_interface::performance::Frame;
using user_interface::performance::Performance;
using user_interface::performance::Run;
using user_interface::performance::Summary;
namespace clock_ = user_interface::clock;

namespace {

std::int64_t ticks(double milliseconds) {
    return static_cast<std::int64_t>(milliseconds * static_cast<double>(clock_::frequency()) / 1000.0);
}

// A frame a time apart from the last, for a second.
void frames(Performance& performance, double apart) {
    size_t count = static_cast<size_t>(1.0 / apart);
    for (size_t index = 0; index <= count; index++) {
        performance.frame_started(static_cast<double>(index) * apart);
    }
}

// The same, with the frames reaching the screen in a mode, with immediate
// flips or at the display's refresh.
Summary measuring_in(PresentMode mode, bool immediate, double apart, double latency, bool with_input) {
    std::int64_t base = clock_::now() - ticks(500.0);
    Performance performance = Performance::given();
    frames(performance, apart);
    std::vector<Present> presents;
    for (int index = 0; index < 20; index++) {
        std::int64_t input = base + ticks(static_cast<double>(index));
        Frame frame{
            .input = with_input ? std::optional<std::int64_t>(input) : std::nullopt,
            .read = input,
            .present = input,
        };
        performance.presented(frame);
        presents.push_back(Present{
            .start = frame.present + 5,
            .thread = 1,
            .mode = mode,
            .tearing = immediate,
            .screen = input + ticks(latency),
            .flip_delay = 0,
        });
    }
    performance.known(presents);
    performance.update(1.0);
    return performance.summary();
}

// Stats that measure input latency, from frames whose inputs reach the
// screen so many milliseconds later, a millisecond apart, near now.
Summary measuring(double apart, double latency, bool with_input) {
    return measuring_in(PresentMode::IndependentFlip, true, apart, latency, with_input);
}

Summary starting() {
    Performance performance = Performance::given();
    frames(performance, 0.01);
    performance.update(1.0);
    return performance.summary();
}

Summary without_latency(std::string_view reason) {
    Performance performance = Performance::without_latency(reason);
    performance.update(0.0);
    return performance.summary();
}

// Each run's first column, its width, and whether it's a value.
std::vector<std::tuple<size_t, size_t, bool>> columns(std::span<const Run> runs) {
    size_t column = 0;
    std::vector<std::tuple<size_t, size_t, bool>> made;
    for (const Run& run : runs) {
        size_t width = base::char_count(run.text);
        column += width;
        made.emplace_back(column - width, width, run.value);
    }
    return made;
}

std::vector<std::string_view> labels(std::span<const Run> runs) {
    std::vector<std::string_view> made;
    for (const Run& run : runs) {
        if (!run.value) {
            made.push_back(run.text);
        }
    }
    return made;
}

std::vector<std::string_view> values(std::span<const Run> runs) {
    std::vector<std::string_view> made;
    for (const Run& run : runs) {
        if (run.value) {
            made.push_back(run.text);
        }
    }
    return made;
}

std::string_view trim_end(std::string_view text) {
    while (!text.empty() && text.back() == ' ') {
        text.remove_suffix(1);
    }
    return text;
}

} // namespace

TEST(every_label_and_value_stays_in_its_columns) {
    std::vector<Summary> summaries{
        without_latency("none in a test"),
        without_latency("needs the Performance Log Users group"),
        starting(),
        measuring(0.01, 10.0, true),
        measuring(0.0004, 5.0, true),
        measuring(0.25, 999.0, true),
        measuring(0.01, 10.0, false),
        measuring_in(PresentMode::ComposedFlip, false, 0.01, 10.0, true),
    };
    const Summary& first = summaries[0];
    for (const Summary& summary : summaries) {
        CHECK_EQ(columns(summary.stats), columns(first.stats), "{}", testing::describe(summary.stats));
        CHECK_EQ(labels(summary.stats), labels(first.stats));
    }
}

TEST(every_present_mode_shows_whole_in_the_same_columns) {
    Summary first = measuring_in(present_timing::AllPresentModes[0], true, 0.01, 10.0, true);
    for (PresentMode mode : present_timing::AllPresentModes) {
        Summary summary = measuring_in(mode, true, 0.01, 10.0, true);
        CHECK_EQ(trim_end(values(summary.stats)[3]), present_timing::name(mode));
        CHECK_EQ(columns(summary.stats), columns(first.stats), "{}", testing::describe(summary.stats));
    }
}

TEST(numbers_are_right_aligned_and_unknown_ones_are_dashes) {
    Summary summary = measuring(0.01, 10.0, true);
    // Frames a second, the latency's mean, the lookback, the mode and the
    // flip.
    CHECK_EQ(values(summary.stats), (std::vector<std::string_view>{"  100.00", "  10.00", "1", "Independent flip ", "immediate "}));
    CHECK(summary.note.empty());

    // Without input latency, all of its numbers, and the present mode, are
    // dashes, and the note says why.
    summary = without_latency("none in a test");
    std::vector<std::string_view> stats = values(summary.stats);
    for (size_t index = 0; index < stats.size(); index++) {
        if (index == 2) {
            continue;
        }
        CHECK_EQ(base::trim(stats[index]), "-", "{}", stats[index]);
    }
    CHECK_EQ(summary.note, (std::vector<Run>{
                               Run{.text = " | Input latency = ", .value = false},
                               Run{.text = "none in a test", .value = true},
                           }));

    // Frames without input have no latency.
    summary = measuring(0.01, 10.0, false);
    std::vector<std::string_view> without = values(summary.stats);
    without.resize(3);
    CHECK_EQ(without, (std::vector<std::string_view>{"  100.00", "      -", "1"}));
}

TEST(sections_are_separated_by_bars) {
    Summary summary = measuring(0.01, 10.0, true);
    std::string text;
    for (const Run& run : summary.stats) {
        text += run.text;
    }
    CHECK_EQ(text, "FPS =   100.00 | Input latency =   10.00 ms | Lookback = 1 s | Mode = Independent flip  | Flip = immediate ");
}

TEST(the_flip_is_immediate_or_at_the_refresh) {
    Summary summary = measuring_in(PresentMode::IndependentFlip, true, 0.01, 10.0, true);
    CHECK(summary.immediate);
    CHECK_EQ(values(summary.stats)[4], "immediate ");
    summary = measuring_in(PresentMode::ComposedFlip, false, 0.01, 10.0, true);
    CHECK(!summary.immediate);
    CHECK_EQ(values(summary.stats)[4], "at refresh");
}

TEST(a_number_too_wide_for_its_field_fills_it) {
    Performance performance = Performance::without_latency("none in a test");
    performance.frame_started(0.0);
    performance.frame_started(0.000001);
    performance.update(0.000001);
    const Summary& summary = performance.summary();
    std::vector<std::string_view> stats = values(summary.stats);
    stats.resize(3);
    CHECK_EQ(stats, (std::vector<std::string_view>{"########", "      -", "1"}));
}
