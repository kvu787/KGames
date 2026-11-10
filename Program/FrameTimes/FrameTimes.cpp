#include "FrameTimes/FrameTimes.h"

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Base/Windows.h"

#include <TlHelp32.h>

#include <algorithm>
#include <cmath>
#include <compare>
#include <format>
#include <iterator>
#include <utility>

namespace frame_times {

using present_timing::Present;
using present_timing::PresentMode;

namespace {

// A whole number that fits 32 bits: str::parse::<u32>.
std::optional<std::uint32_t> parse_u32(std::string_view text) {
    std::optional<std::uint64_t> number = base::parse_u64(text);
    if (!number || *number > UINT32_MAX) {
        return std::nullopt;
    }
    return static_cast<std::uint32_t>(*number);
}

// The text in lower case, of every alphabet, as Windows maps it:
// str::to_lowercase.
std::string to_lowercase(std::string_view text) {
    std::wstring wide = base::widen(text);
    if (wide.empty()) {
        return {};
    }
    std::wstring lower(wide.size(), L'\0');
    int length =
        LCMapStringEx(LOCALE_NAME_INVARIANT, LCMAP_LOWERCASE, wide.data(), static_cast<int>(wide.size()), lower.data(), static_cast<int>(lower.size()), nullptr, nullptr, 0);
    if (length <= 0) {
        return base::to_ascii_lowercase(text);
    }
    lower.resize(static_cast<size_t>(length));
    return base::narrow(lower);
}

// Whether a comes before b in the order of every double, NaNs too:
// f64::total_cmp.
bool total_less(double a, double b) {
    return std::strong_order(a, b) < 0;
}

std::vector<double> sorted(std::span<const double> values) {
    std::vector<double> sorted(values.begin(), values.end());
    std::ranges::stable_sort(sorted, total_less);
    return sorted;
}

} // namespace

std::string describe_value(const Options& options) {
    return std::format("Options {{ process: {}, seconds: {} }}", base::debug(options.process), base::debug(options.seconds));
}

std::expected<Options, std::string> parse_arguments(std::span<const std::string> arguments) {
    std::optional<std::string> process;
    double seconds = DefaultSeconds;
    for (auto argument = arguments.begin(); argument != arguments.end(); ++argument) {
        if (*argument == "--seconds") {
            ++argument;
            if (argument == arguments.end()) {
                return std::unexpected(std::string("--seconds needs a number of seconds after it."));
            }
            const std::string& value = *argument;
            std::optional<double> number = base::parse_f64(value);
            if (!number || !std::isfinite(*number) || !(*number > 0.0)) {
                return std::unexpected(std::format("--seconds needs a number of seconds above 0, not {}.", value));
            }
            seconds = *number;
        } else if (argument->starts_with("--")) {
            return std::unexpected(std::format("Unknown option: {}. The only one is --seconds.", *argument));
        } else if (process) {
            return std::unexpected(std::format("Name one process, not both {} and {}.", *process, *argument));
        } else {
            process = *argument;
        }
    }
    if (!process) {
        return std::unexpected(std::string("Name the process to measure, by its exe's name or its ID."));
    }
    return Options{
        .process = std::move(*process),
        .seconds = seconds,
    };
}

std::string describe_value(const Process& process) {
    return std::format("Process {{ id: {}, name: {} }}", process.id, base::debug(process.name));
}

std::expected<std::vector<Process>, std::string> processes() {
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snapshot == INVALID_HANDLE_VALUE) {
        return std::unexpected(std::format("Couldn't list the processes: {}", base::os_error(GetLastError())));
    }
    PROCESSENTRY32W entry{};
    entry.dwSize = sizeof(PROCESSENTRY32W);
    std::vector<Process> list;
    bool more = Process32FirstW(snapshot, &entry) != FALSE;
    while (more) {
        std::wstring_view name(entry.szExeFile, std::size(entry.szExeFile));
        size_t length = std::min(name.find(L'\0'), name.size());
        list.push_back(Process{
            .id = entry.th32ProcessID,
            .name = base::narrow(name.substr(0, length)),
        });
        more = Process32NextW(snapshot, &entry) != FALSE;
    }
    CloseHandle(snapshot);
    return list;
}

std::expected<Process, std::string> find_process(std::string_view query, std::span<const Process> processes) {
    if (std::optional<std::uint32_t> id = parse_u32(query)) {
        auto process = std::ranges::find(processes, *id, &Process::id);
        if (process == processes.end()) {
            return std::unexpected(std::format("No process has the ID {}.", *id));
        }
        return *process;
    }
    std::string wanted = to_lowercase(query);
    std::vector<const Process*> found;
    for (const Process& process : processes) {
        std::string name = to_lowercase(process.name);
        if (name == wanted || (name.ends_with(".exe") && std::string_view(name).substr(0, name.size() - 4) == wanted)) {
            found.push_back(&process);
        }
    }
    if (found.empty()) {
        return std::unexpected(std::format("No process is named {}.", query));
    }
    if (found.size() == 1) {
        return *found[0];
    }
    std::string ids;
    for (const Process* process : found) {
        if (!ids.empty()) {
            ids += ", ";
        }
        ids += std::to_string(process->id);
    }
    return std::unexpected(std::format("{} processes are named {}: {}. Name the one to measure by its ID.", found.size(), found[0]->name, ids));
}

std::vector<double> frame_times(std::span<const Present> presents, std::int64_t frequency) {
    std::vector<std::int64_t> screens;
    for (const Present& present : presents) {
        if (present.screen) {
            screens.push_back(*present.screen);
        }
    }
    std::ranges::sort(screens);
    std::vector<double> times;
    times.reserve(base::saturating_sub(screens.size(), size_t{1}));
    for (size_t index = 1; index < screens.size(); index++) {
        times.push_back(static_cast<double>(screens[index] - screens[index - 1]) * 1000.0 / static_cast<double>(frequency));
    }
    return times;
}

std::optional<double> mean(std::span<const double> values) {
    if (values.empty()) {
        return std::nullopt;
    }
    // From -0.0, as Rust's sum is, which only shows in a sum of -0.0s.
    double sum = -0.0;
    for (double value : values) {
        sum += value;
    }
    return sum / static_cast<double>(values.size());
}

std::optional<double> median(std::span<const double> values) {
    if (values.empty()) {
        return std::nullopt;
    }
    std::vector<double> in_order = sorted(values);
    size_t middle = in_order.size() / 2;
    if (in_order.size() % 2 == 1) {
        return in_order[middle];
    }
    return (in_order[middle - 1] + in_order[middle]) / 2.0;
}

std::optional<double> percentile(std::span<const double> values, double percent) {
    if (values.empty()) {
        return std::nullopt;
    }
    std::vector<double> in_order = sorted(values);
    // Multiplying first keeps whole ranks whole, like 99% of 100 values.
    size_t rank = base::saturating_cast<size_t>(std::ceil(percent * static_cast<double>(in_order.size()) / 100.0));
    return in_order[std::clamp(rank, size_t{1}, in_order.size()) - 1];
}

std::optional<double> maximum(std::span<const double> values) {
    std::optional<double> most;
    for (double value : values) {
        if (!most || !total_less(value, *most)) {
            most = value;
        }
    }
    return most;
}

size_t stutters(std::span<const double> frame_times) {
    std::optional<double> middle = median(frame_times);
    if (!middle) {
        return 0;
    }
    return static_cast<size_t>(std::ranges::count_if(frame_times, [&](double time) { return time > 2.0 * *middle; }));
}

std::optional<PresentMode> most_common_mode(std::span<const Present> presents) {
    std::vector<std::pair<PresentMode, size_t>> counts;
    for (const Present& present : presents) {
        if (!present.screen) {
            continue;
        }
        auto known = std::ranges::find(counts, present.mode, &std::pair<PresentMode, size_t>::first);
        if (known != counts.end()) {
            known->second += 1;
        } else {
            counts.emplace_back(present.mode, 1);
        }
    }
    std::optional<std::pair<PresentMode, size_t>> most;
    for (const auto& [mode, count] : counts) {
        if (!most || count > most->second) {
            most = std::pair(mode, count);
        }
    }
    if (!most) {
        return std::nullopt;
    }
    return most->first;
}

std::string describe_value(const Summary& summary) {
    return std::format("Summary {{ frames: {}, frames_per_second: {}, mean: {}, median: {}, percentile_99: {}, maximum: {}, stutters: {}, mode: {} }}", summary.frames,
                       base::debug(summary.frames_per_second), base::debug(summary.mean), base::debug(summary.median), base::debug(summary.percentile_99),
                       base::debug(summary.maximum), summary.stutters, present_timing::describe_value(summary.mode));
}

std::optional<Summary> summarize(std::span<const Present> presents, std::int64_t frequency) {
    std::vector<double> times = frame_times(presents, frequency);
    std::optional<double> mean_time = mean(times);
    if (!mean_time) {
        return std::nullopt;
    }
    // The frames over the time from the first on the screen to the last.
    double frames_per_second = *mean_time > 0.0 ? 1000.0 / *mean_time : 0.0;
    std::optional<double> median_time = median(times);
    if (!median_time) {
        return std::nullopt;
    }
    std::optional<double> percentile_99 = percentile(times, 99.0);
    if (!percentile_99) {
        return std::nullopt;
    }
    std::optional<double> longest = maximum(times);
    if (!longest) {
        return std::nullopt;
    }
    size_t stutter_count = stutters(times);
    std::optional<PresentMode> mode = most_common_mode(presents);
    if (!mode) {
        return std::nullopt;
    }
    return Summary{
        .frames = times.size(),
        .frames_per_second = frames_per_second,
        .mean = *mean_time,
        .median = *median_time,
        .percentile_99 = *percentile_99,
        .maximum = *longest,
        .stutters = stutter_count,
        .mode = *mode,
    };
}

std::string report(const Summary& summary) {
    std::pair<std::string_view, std::string> lines[] = {
        {"Frames", std::to_string(summary.frames)},
        {"Frames a second", std::format("{:.1f}", summary.frames_per_second)},
        {"Mean", std::format("{:.2f} ms", summary.mean)},
        {"Median", std::format("{:.2f} ms", summary.median)},
        {"99th percentile", std::format("{:.2f} ms", summary.percentile_99)},
        {"Maximum", std::format("{:.2f} ms", summary.maximum)},
        {"Stutters", std::format("{}, frames over twice the median ({:.2f} ms)", summary.stutters, 2.0 * summary.median)},
        {"Present mode", std::string(present_timing::name(summary.mode))},
    };
    std::string text;
    for (const auto& [label, value] : lines) {
        text += std::format("  {:<17}{}\n", label, value);
    }
    return text;
}

} // namespace frame_times
