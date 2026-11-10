// What the PerfBar along the top of every program's window measures:
// how fast frames come, how fast the display shows them, and the input
// latency, from when GameInput got the input that a frame took to when the
// display took that frame, all on the performance counter's clock:
//
//     input --- read ----------- present ------------- screen
//       waiting      building          presenting
//
// - input: the GameInput reading's time, when Windows got the device's report.
// - read: when the main loop read the frame's input, right before the frame.
// - present: right before the frame's Present call.
// - screen: when the display took the frame, from Windows' own events about
//   presents (Library/PresentTiming), as PresentMon times it, so that the
//   numbers compare with PresentMon's.
//
// A frame's input is the newest reading it took, as PresentMon's is the
// newest input that a frame took. A frame that never reaches the screen,
// like one that a newer frame replaced, passes its input on to the next frame
// that does, unless that one has input of its own.

#pragma once

#include "Base/Sum.h"
#include "Base/VecDeque.h"
#include "PresentTiming/PresentTiming.h"

#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace user_interface::performance {

// The stats, the frames' and the input latency's, are of this many seconds:
// the frames' before now, and the input latency's before the newest frame
// known on the screen.
inline constexpr double WindowSeconds = 1.0;

// The library's own: what separates the bar's sections, like the frames' and
// the input latency's.
inline constexpr std::string_view Separator = " | ";

// A frame that the main loop presented, with the times of its input.
struct Frame {
    // When the frame's newest input reading came, if it took any.
    std::optional<std::int64_t> input;
    std::int64_t read;
    std::int64_t present;

    bool operator==(const Frame&) const = default;
};

// One frame's input latency, in milliseconds, and its parts.
struct Sample {
    // When the display took the frame, as the counter's ticks.
    std::int64_t screen;
    double total;
    double waiting;
    double building;
    double presenting;

    bool operator==(const Sample&) const = default;
};

// What the PerfBar says about input latency.
struct LatencyState {
    // Before the first frame's present is known.
    struct Starting {
        bool operator==(const Starting&) const = default;
    };
    // Why the presents' times aren't known, like in a scripted run.
    struct Unavailable {
        std::string reason;
        bool operator==(const Unavailable&) const = default;
    };
    struct Measuring {
        bool operator==(const Measuring&) const = default;
    };
    std::variant<Starting, Unavailable, Measuring> value;

    // Starting.
    LatencyState() = default;
    template <base::AlternativeOf<LatencyState> T> LatencyState(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const LatencyState&) const = default;
};

// How a failed check in a test shows a latency state.
std::string describe_value(const LatencyState& state);

// A run of the PerfBar's text, which the bar sets in Cascadia Mono, whose
// characters are all as wide: a label, or a value. Every run is as many
// characters wide whatever the stats are, numbers right-aligned in their
// fields and a dash for one that isn't known, so that every label and value
// stays in the same columns.
struct Run {
    std::string text;
    bool value;

    bool operator==(const Run&) const = default;
};

// How a failed check in a test shows a run.
std::string describe_value(const Run& run);

// What the PerfBar shows, which changes 10 times a second.
struct Summary {
    double frames_per_second = 0.0;
    // Milliseconds, the mean and the longest.
    double frame_time = 0.0;
    double longest_frame = 0.0;
    LatencyState latency;
    // Frames a second that reached the screen.
    double displayed_per_second = 0.0;
    // The input latency's samples, their mean and 99th percentile, and the
    // mean of each part.
    size_t samples = 0;
    double mean = 0.0;
    double percentile_99 = 0.0;
    double waiting = 0.0;
    double building = 0.0;
    double presenting = 0.0;
    // How the newest frame on the screen got there, and whether the display
    // took it at once, instead of at its next refresh.
    present_timing::PresentMode mode = present_timing::PresentMode::Unknown;
    bool immediate = false;
    // What the PerfBar says, made here when the summary is, 10 times a
    // second, not every frame: the runs that it starts with, at its left,
    // which it follows with whether vertical sync is on, and why there's no
    // input latency, if there isn't, after that, at its end, so that the
    // stats before it stay in their columns whether it's there or not.
    std::vector<Run> stats;
    std::vector<Run> note;

    bool operator==(const Summary&) const = default;
};

class Performance {
public:
    // Input latency for the process's presents, from an ETW session named
    // after the program.
    static Performance measure(std::string_view name);

    // Without input latency, saying why, like in a scripted run.
    static Performance without_latency(std::string_view reason);

    // Input latency for presents that `known` gives, like a test's.
    static Performance given();

    Performance(const Performance&) = delete;
    Performance& operator=(const Performance&) = delete;
    Performance(Performance&&) = default;
    Performance& operator=(Performance&&) = default;

    // A frame starts, at a time on the frames' clock, in seconds.
    void frame_started(double time);

    // The main loop presented a frame.
    void presented(Frame frame);

    // Presents whose fates are known, in the order they started.
    void known(std::vector<present_timing::Present> presents);

    // Takes what's known about presents, and refreshes what the PerfBar
    // says, 10 times a second, at a time on the frames' clock, in seconds.
    void update(double time);

    const Summary& summary() const;

private:
    // Where the presents' fates come from.
    struct Source {
        struct Timing {
            present_timing::PresentTiming timing;
        };
        // From `known`, like a test's.
        struct Given {};
        // Nowhere, and why.
        struct Unavailable {
            std::string reason;
        };
        std::variant<Timing, Given, Unavailable> value;

        template <base::AlternativeOf<Source> T> Source(T&& alternative) : value(std::forward<T>(alternative)) {}
    };

    explicit Performance(Source source);

    // A present whose fate is known: which frame it was, and that frame's
    // input latency, if it reached the screen.
    void present(const present_timing::Present& present);

    Summary summarize() const;

    Source source;
    // The frames' start times on the frames' clock, in seconds.
    base::VecDeque<double> frame_times;
    // Frames whose presents aren't known yet, oldest first.
    base::VecDeque<Frame> presented_;
    // The input of frames that didn't reach the screen, for the next one that
    // does.
    std::optional<Frame> carried;
    base::VecDeque<Sample> samples;
    base::VecDeque<std::int64_t> displayed;
    present_timing::PresentMode mode = present_timing::PresentMode::Unknown;
    bool immediate = false;
    bool known_ = false;
    Summary summary_;
    double refreshed;
};

} // namespace user_interface::performance
