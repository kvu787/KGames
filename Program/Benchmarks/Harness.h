// Times a body many times and keeps the statistics of its times: the
// minimum, which is what the work costs when nothing else is in the way,
// the median, the mean, the 99th percentile and the maximum, which say how
// steady it is. Each iteration is timed on its own, on the performance
// counter, so that a run under RunLightly, where the job's cap on CPU time
// stops the thread now and then, still finds the true cost in the minimum
// and the median, while the maximum shows the stops.

#pragma once

#include "Benchmarks/BlackBox.h"
#include "Json/Json.h"

#include <algorithm>
#include <chrono>
#include <cstddef>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace benchmarks {

// How long a benchmark runs: until the seconds pass, or the iterations are
// done, whichever is first, after the warm-up iterations, which aren't kept.
struct Budget {
    double seconds;
    size_t iterations;
    size_t warmup;

    // The default: up to half a second, or 2000 iterations.
    static const Budget Default;
    // A fifth of the default, for a quick look.
    static const Budget Quick;
    // One iteration, with no warm-up, for tests that check that a benchmark
    // runs at all.
    static const Budget Once;

    bool operator==(const Budget&) const = default;
};

inline constexpr Budget Budget::Default{.seconds = 0.5, .iterations = 2000, .warmup = 5};
inline constexpr Budget Budget::Quick{.seconds = 0.1, .iterations = 400, .warmup = 2};
inline constexpr Budget Budget::Once{.seconds = 0.0, .iterations = 1, .warmup = 0};

// The budget as Rust's {:?} writes it, like
// Budget { seconds: 0.5, iterations: 2000, warmup: 5 }, which the program
// prints, and a failed check in a test shows.
std::string describe_value(const Budget& budget);

// What a benchmark measured: its times in milliseconds, per unit of work,
// like a frame or a parse.
struct Measurement {
    std::string name;
    // What each time is of, like "frame" or "parse": a string literal's
    // text, which lasts as long as the program.
    std::string_view unit;
    size_t iterations;
    double min_;
    double median;
    double mean;
    double percentile_99;
    double max_;
    // What else the benchmark found out, like how many instances a frame
    // drew, for the table's last column.
    std::string note;

    // The statistics of times in milliseconds, each of so many units.
    static Measurement from_times(std::string_view name, std::string_view unit, std::span<const double> times, double units);

    Measurement&& with_note(std::string text) && {
        note = std::move(text);
        return std::move(*this);
    }

    json::Value to_json() const;

    bool operator==(const Measurement&) const = default;
};

// The value of sorted values at a fraction of the way through them,
// between the two nearest.
double percentile(std::span<const double> sorted, double fraction);

// The seconds since a time on the performance counter, which is what
// std::chrono::steady_clock reads: Instant::elapsed().as_secs_f64() in Rust,
// whole seconds and the rest of them added as it adds them.
inline double seconds_since(std::chrono::steady_clock::time_point start) {
    std::chrono::nanoseconds elapsed = std::max(std::chrono::steady_clock::now() - start, std::chrono::nanoseconds::zero());
    return static_cast<double>(elapsed.count() / 1'000'000'000) + static_cast<double>(elapsed.count() % 1'000'000'000) / 1.0e9;
}

// Times `body` within the budget, each call on its own. Each call does
// `units` units of work, like two frames, so that the times are per unit.
template <class Body> Measurement measure(std::string_view name, std::string_view unit, Budget budget, double units, Body&& body) {
    for (size_t iteration = 0; iteration < budget.warmup; iteration++) {
        body();
    }
    std::vector<double> times;
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    while (times.size() < std::max<size_t>(budget.iterations, 1)) {
        std::chrono::steady_clock::time_point before = std::chrono::steady_clock::now();
        body();
        times.push_back(seconds_since(before) * 1000.0);
        if (seconds_since(start) >= budget.seconds && !times.empty()) {
            break;
        }
    }
    return Measurement::from_times(name, unit, times, units);
}

// Like measure, for a body that times itself, like with the GPU's clock,
// and gives each call's milliseconds.
template <class Body> Measurement measure_timed(std::string_view name, std::string_view unit, Budget budget, double units, Body&& body) {
    for (size_t iteration = 0; iteration < budget.warmup; iteration++) {
        body();
    }
    std::vector<double> times;
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    while (times.size() < std::max<size_t>(budget.iterations, 1)) {
        times.push_back(body());
        if (seconds_since(start) >= budget.seconds && !times.empty()) {
            break;
        }
    }
    return Measurement::from_times(name, unit, times, units);
}

// The measurements as a table that reads in a terminal: milliseconds with
// four decimals, right-aligned.
std::string table(std::span<const Measurement> measurements);

} // namespace benchmarks
