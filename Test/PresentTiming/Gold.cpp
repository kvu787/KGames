// Tests that read_file gives what PresentMon 2.6.1 writes for the gold
// traces in its repository (Tests/Gold), without running PresentMon: the
// gold CSVs beside them are what its own tests require its output to match,
// row for row. Every present in them, of every process, has the same mode,
// tearing and screen time. The traces aren't in this repository, so the
// test checks nothing unless it's asked for, with PRESENTMON_GOLD naming a
// folder that has test_case_0.etl to test_case_5.etl and their gold CSVs,
// test_case_0.csv to test_case_5.csv:
//
//   $env:PRESENTMON_GOLD = 'C:\PresentMon\Tests\Gold'
//   Program\RunTests\RunTests.ps1 PresentTimingTests --filter gold_traces
//
// It also tests, without traces, that read_file takes only a trace whose
// events are timed on the performance counter.

#include "Base/Files.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Base/Windows.h"
#include "PresentTiming/PresentTiming.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <format>
#include <map>
#include <optional>
#include <print>
#include <ranges>
#include <string>
#include <string_view>
#include <unordered_map>
#include <utility>
#include <vector>

using present_timing::file_clock;
using present_timing::Present;
using present_timing::PresentMode;
using present_timing::read_file;

namespace {

constexpr size_t Cases = 6;

// What PresentMon's CSV calls each mode.
std::string_view presentmon_name(PresentMode mode) {
    switch (mode) {
    case PresentMode::Unknown:
        return "Other";
    case PresentMode::LegacyFlip:
        return "Hardware: Legacy Flip";
    case PresentMode::LegacyCopy:
        return "Hardware: Legacy Copy to front buffer";
    case PresentMode::IndependentFlip:
        return "Hardware: Independent Flip";
    case PresentMode::OverlayPlane:
        return "Hardware Composed: Independent Flip";
    case PresentMode::ComposedFlip:
        return "Composed: Flip";
    case PresentMode::ComposedGpuCopy:
        return "Composed: Copy with GPU GDI";
    case PresentMode::ComposedCpuCopy:
        return "Composed: Copy with CPU GDI";
    }
    base::unreachable();
}

// A row of PresentMon's CSV.
struct Row {
    std::uint32_t process;
    std::string application;
    std::string mode;
    bool tearing;
    std::int64_t start;
    // In milliseconds, or nothing if it never reached the screen.
    std::optional<double> until_displayed;
};

std::vector<Row> read_csv(const std::filesystem::path& path) {
    std::expected<std::string, std::string> read = base::read_to_string(path);
    if (!read) {
        base::panic(std::format("Couldn't read {}: {}", base::display(path), read.error()));
    }
    std::string_view text = *read;
    // A byte order mark, U+FEFF, as UTF-8.
    while (text.starts_with("\xEF\xBB\xBF")) {
        text.remove_prefix(3);
    }
    std::vector<std::string_view> lines = base::lines(text);
    if (lines.empty()) {
        base::panic(std::format("{} has no header", base::display(path)));
    }
    std::vector<std::string_view> header = base::split(lines[0], ',');
    auto column = [&](std::string_view name) {
        auto known = std::ranges::find(header, name);
        if (known == header.end()) {
            base::panic(std::format("No {} column", name));
        }
        return static_cast<size_t>(known - header.begin());
    };
    size_t application = column("Application");
    size_t process = column("ProcessID");
    size_t tearing = column("AllowsTearing");
    size_t mode = column("PresentMode");
    size_t start = column("TimeInQPC");
    size_t until_displayed = column("MsUntilDisplayed");
    std::vector<Row> rows;
    for (std::string_view line : lines | std::views::drop(1)) {
        std::vector<std::string_view> fields = base::split(line, ',');
        rows.push_back(Row{
            .process = static_cast<std::uint32_t>(base::parse_u64(fields[process]).value()),
            .application = std::string(fields[application]),
            .mode = std::string(fields[mode]),
            .tearing = fields[tearing] == "1",
            .start = base::parse_i64(fields[start]).value(),
            .until_displayed = base::parse_f64(fields[until_displayed]),
        });
    }
    return rows;
}

// The environment variable's value, or nothing if it isn't set.
std::optional<std::string> environment_variable(std::string_view name) {
    std::wstring wide = base::widen(name);
    DWORD size = GetEnvironmentVariableW(wide.c_str(), nullptr, 0);
    if (size == 0) {
        return std::nullopt;
    }
    std::wstring value(size, L'\0');
    value.resize(GetEnvironmentVariableW(wide.c_str(), value.data(), size));
    return base::narrow(value);
}

} // namespace

// It needs PresentMon's gold traces, in the folder that PRESENTMON_GOLD
// names. Without the variable it checks nothing, and says so: what an ignored
// test is in Rust.
TEST(presents_are_the_same_as_presentmons_on_its_gold_traces) {
    std::optional<std::string> gold = environment_variable("PRESENTMON_GOLD");
    if (!gold) {
        std::println("ignored: needs PresentMon's gold traces, in the folder that PRESENTMON_GOLD names");
        return;
    }
    std::filesystem::path folder = base::path(*gold);
    std::vector<std::string> mismatches;
    // How many presents of each mode were the same.
    std::map<std::string, size_t> same;
    for (size_t case_ = 0; case_ < Cases; case_++) {
        std::vector<Row> rows = read_csv(folder / std::format("test_case_{}.csv", case_));
        std::vector<std::uint32_t> processes;
        for (const Row& row : rows) {
            processes.push_back(row.process);
        }
        std::ranges::stable_sort(processes);
        auto repeated = std::ranges::unique(processes);
        processes.erase(repeated.begin(), repeated.end());
        for (std::uint32_t process : processes) {
            std::pair<std::vector<Present>, std::int64_t> read = read_file(folder / std::format("test_case_{}.etl", case_), process).value();
            const std::vector<Present>& presents = read.first;
            std::int64_t frequency = read.second;
            std::unordered_map<std::int64_t, const Present*> by_start;
            for (const Present& present : presents) {
                by_start.insert_or_assign(present.start, &present);
            }
            for (const Row& row : rows) {
                if (row.process != process) {
                    continue;
                }
                std::string place = std::format("test_case_{}, {} {}, present at {}", case_, row.application, process, row.start);
                auto found = by_start.find(row.start);
                if (found == by_start.end()) {
                    mismatches.push_back(std::format("{}: missing, PresentMon has {}", place, row.mode));
                    continue;
                }
                const Present& present = *found->second;
                std::optional<double> until_displayed;
                if (present.screen) {
                    until_displayed = static_cast<double>(*present.screen - present.start) * 1000.0 / static_cast<double>(frequency);
                }
                std::string_view mode = presentmon_name(present.mode);
                bool times_same = false;
                if (until_displayed && row.until_displayed) {
                    times_same = std::abs(*until_displayed - *row.until_displayed) < 1e-6;
                } else if (!until_displayed && !row.until_displayed) {
                    times_same = true;
                }
                if (mode != row.mode || present.tearing != row.tearing || !times_same) {
                    mismatches.push_back(std::format("{}: {}, tearing {}, {} ms, but PresentMon has {}, tearing {}, {} ms", place, mode, present.tearing,
                                                     testing::describe(until_displayed), row.mode, row.tearing, testing::describe(row.until_displayed)));
                } else {
                    same[row.mode] += 1;
                }
            }
        }
    }
    for (const auto& [mode, count] : same) {
        std::println("{:5} the same: {}", count, mode);
    }
    for (const std::string& mismatch : mismatches) {
        std::println("{}", mismatch);
    }
    CHECK(mismatches.empty(), "{} presents differ from PresentMon's", mismatches.size());
}

// A trace file's header says which clock its events are timed on, in
// ReservedFlags: 1 for the performance counter, which read_file's times are
// on, 2 for the system time, and 3 for the CPU's cycles, which it refuses.
TEST(only_a_trace_timed_on_the_performance_counter_is_read) {
    CHECK_EQ(file_clock(1, 10'000'000).value(), 10'000'000);
    CHECK(!file_clock(1, 0).has_value());
    CHECK(!file_clock(2, 10'000'000).has_value());
    CHECK(!file_clock(3, 10'000'000).has_value());
}
