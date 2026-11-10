// FrameTimes' parts that don't need an ETW session, as a library that
// Test/FrameTimes tests: the statistics of a process's frame times, from the
// presents that Library/PresentTiming gives, the command line, and finding
// the process that it names.

#pragma once

#include "PresentTiming/PresentTiming.h"

#include <cstddef>
#include <cstdint>
#include <expected>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace frame_times {

inline constexpr std::string_view Usage = "FrameTimes <process name or ID> [--seconds N]";

// How long FrameTimes measures for, without --seconds.
inline constexpr double DefaultSeconds = 10.0;

// What the command line asks for.
struct Options {
    // The process's exe name, like AceCombat8.exe or AceCombat8, or its ID.
    std::string process;
    double seconds;

    bool operator==(const Options&) const = default;
};

// How a failed check in a test shows the options.
std::string describe_value(const Options& options);

std::expected<Options, std::string> parse_arguments(std::span<const std::string> arguments);

// A running process.
struct Process {
    std::uint32_t id;
    // Its exe's file name, like AceCombat8.exe.
    std::string name;

    bool operator==(const Process&) const = default;
};

// How a failed check in a test shows a process.
std::string describe_value(const Process& process);

// Every running process, from a snapshot of the system's.
std::expected<std::vector<Process>, std::string> processes();

// The process that the command line names: by its ID if it's a number, or
// else by its exe's name, regardless of case, with or without ".exe".
std::expected<Process, std::string> find_process(std::string_view query, std::span<const Process> processes);

// The time from each frame on the screen to the next, in milliseconds: the
// differences between the screen times of the presents that reached the
// screen, in order, on a performance counter of `frequency` ticks a second.
// Presents that never reached the screen don't count.
std::vector<double> frame_times(std::span<const present_timing::Present> presents, std::int64_t frequency);

// The arithmetic mean, or nothing of no values.
std::optional<double> mean(std::span<const double> values);

// The middle value, or the mean of the two middle values of an even count.
std::optional<double> median(std::span<const double> values);

// The nearest-rank percentile: the smallest value that at least `percent`
// percent of the values are at or under.
std::optional<double> percentile(std::span<const double> values, double percent);

std::optional<double> maximum(std::span<const double> values);

// How many frames took more than twice the median frame time: a frame that
// stayed on the screen for two of the usual frames or more, which shows as
// a hitch.
size_t stutters(std::span<const double> frame_times);

// The mode that most of the presents that reached the screen got there by,
// or nothing if none did. A tie goes to the mode that reached it first.
std::optional<present_timing::PresentMode> most_common_mode(std::span<const present_timing::Present> presents);

// The statistics of a process's frame times, in milliseconds.
struct Summary {
    // How many frame times there are: one fewer than the presents that
    // reached the screen.
    size_t frames;
    double frames_per_second;
    double mean;
    double median;
    double percentile_99;
    double maximum;
    // How many frames took more than twice the median.
    size_t stutters;
    // The mode that most of the presents reached the screen by.
    present_timing::PresentMode mode;

    bool operator==(const Summary&) const = default;
};

// How a failed check in a test shows a summary.
std::string describe_value(const Summary& summary);

// The statistics of these presents' frame times, or nothing if fewer than
// two of them reached the screen, which leaves no frame time.
std::optional<Summary> summarize(std::span<const present_timing::Present> presents, std::int64_t frequency);

// The summary as FrameTimes prints it, a line for each statistic.
std::string report(const Summary& summary);

} // namespace frame_times
