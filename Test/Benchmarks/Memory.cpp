// The memory harness: how many runs a budget asks for, the usage that it
// makes of the counts, the tables, and the process's memory. Counting.cpp
// tests the counting allocator, and the measurements that turn it on.

#include "Base/Text.h"
#include "Benchmarks/Benchmarks.h"
#include "Testing/Testing.h"
#include "Json/Json.h"

#include <cstddef>
#include <cstdint>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

using benchmarks::Budget;
using benchmarks::Usage;
using benchmarks::memory::Counts;
using benchmarks::memory::process_memory;
using benchmarks::memory::ProcessMemory;
using benchmarks::memory::Report;
using benchmarks::memory::runs;
using benchmarks::memory::table;
using benchmarks::memory::Video;
using benchmarks::memory::video_table;

namespace {

// A member of an object, which must have it: what comparing get's result
// with Some(&expected) checks in Rust.
const json::Value& member(const json::Value& value, std::string_view name) {
    const json::Value* found = value.get(name);
    CHECK(found != nullptr, "{} isn't there", name);
    return *found;
}

} // namespace

TEST(the_budget_bounds_the_runs_and_one_run_is_enough_for_a_test) {
    CHECK_EQ(runs(Budget::Once, 200), (std::pair<size_t, size_t>(0, 1)));
    CHECK_EQ(runs(Budget::Default, 200), (std::pair<size_t, size_t>(Budget::Default.warmup, 200)));
    CHECK_EQ(runs(Budget::Quick, 1000), (std::pair<size_t, size_t>(Budget::Quick.warmup, Budget::Quick.iterations)));
}

TEST(the_usage_of_counts_is_per_unit) {
    Counts counted{
        .allocations = 20,
        .frees = 18,
        .bytes = 4000,
        .live = 200,
        .peak = 900,
    };
    Usage usage = Usage::from_counts("memory/test/counts", "frame", counted, 10.0).with_note("a note");
    CHECK_EQ(std::pair(usage.allocations, usage.bytes), std::pair(2.0, 400.0));
    CHECK_EQ(std::pair(usage.peak, usage.kept), (std::pair<std::int64_t, std::int64_t>(900, 200)));
    json::Value value = usage.to_json();
    CHECK_EQ(member(value, "Name"), json::Value("memory/test/counts"));
    CHECK_EQ(member(value, "AllocationsPerUnit"), json::Value(2.0));
    CHECK_EQ(member(value, "KeptBytes"), json::Value(200.0));
    CHECK_EQ(member(value, "Note"), json::Value("a note"));
}

TEST(the_tables_have_a_row_a_measurement_with_its_note) {
    Usage usage = Usage::from_counts("memory/test/row", "frame", Counts{.allocations = 3, .frees = 3, .bytes = 3072, .live = 0, .peak = 2048}, 1.0);
    {
        std::string text = table({{std::move(usage).with_note("3 widgets")}});
        std::vector<std::string_view> lines = base::lines(text);
        CHECK_EQ(lines.size(), 2);
        CHECK(lines[0].starts_with("Benchmark") && lines[0].contains("Peak KiB"));
        CHECK(lines[1].starts_with("memory/test/row"));
        CHECK(lines[1].ends_with("3 widgets"));
    }

    Video video{
        .name = "memory/gpu/test",
        .local = 2 * 1024 * 1024,
        .non_local = 1024 * 1024,
        .note = "a target",
    };
    std::string text = video_table({{video}});
    std::vector<std::string_view> lines = base::lines(text);
    CHECK_EQ(lines.size(), 2);
    CHECK(lines[0].contains("Local MiB") && lines[0].contains("Non-local MiB"));
    CHECK(lines[1].contains("2.00") && lines[1].contains("1.00"));
    CHECK(lines[1].ends_with("a target"));
}

TEST(the_process_has_a_working_set_and_a_private_commit) {
    ProcessMemory process = process_memory();
    CHECK(process.working_set > 0);
    CHECK(process.peak_working_set >= process.working_set);
    CHECK(process.private_commit > 0);
    json::Value value = process.to_json();
    CHECK_EQ(member(value, "WorkingSetBytes"), json::Value(static_cast<double>(process.working_set)));
}

TEST(a_report_holds_the_heap_rows_and_the_gpu_rows_together) {
    Report report = Report::of({Usage::from_counts("memory/test/a", "frame", Counts{}, 1.0)});
    report.add(Report{
        .heap = {Usage::from_counts("memory/test/b", "frame", Counts{}, 1.0)},
        .video = {Video{
            .name = "memory/gpu/b",
            .local = 1,
            .non_local = 2,
            .note = std::string(),
        }},
    });
    CHECK_EQ(report.heap.size(), 2);
    CHECK_EQ(report.video.size(), 1);
    CHECK_EQ(member(report.video[0].to_json(), "LocalBytes"), json::Value(1.0));
}
