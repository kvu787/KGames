#include "Benchmarks/Memory.h"

#include "Base/Numbers.h"
#include "Base/Windows.h"

#include <Psapi.h>

#include <algorithm>
#include <atomic>
#include <cmath>
#include <format>
#include <iterator>

namespace benchmarks::memory {

namespace {

constexpr std::memory_order Relaxed = std::memory_order_relaxed;

// Whether the allocator counts. Off until `start`.
std::atomic<bool> counting{false};
std::atomic<std::uint64_t> allocations{0};
std::atomic<std::uint64_t> frees{0};
std::atomic<std::uint64_t> bytes_allocated{0};
std::atomic<std::int64_t> live{0};
std::atomic<std::int64_t> peak{0};

} // namespace

bool is_counting() {
    return counting.load(Relaxed);
}

void allocated(size_t bytes) {
    if (!counting.load(Relaxed)) {
        return;
    }
    allocations.fetch_add(1, Relaxed);
    bytes_allocated.fetch_add(static_cast<std::uint64_t>(bytes), Relaxed);
    std::int64_t now = live.fetch_add(static_cast<std::int64_t>(bytes), Relaxed) + static_cast<std::int64_t>(bytes);
    // fetch_max, which the standard library doesn't have yet.
    std::int64_t most = peak.load(Relaxed);
    while (most < now && !peak.compare_exchange_weak(most, now, Relaxed, Relaxed)) {}
}

void freed(size_t bytes) {
    if (!counting.load(Relaxed)) {
        return;
    }
    frees.fetch_add(1, Relaxed);
    live.fetch_sub(static_cast<std::int64_t>(bytes), Relaxed);
}

std::string describe_value(const Counts& counts) {
    return std::format("Counts {{ allocations: {}, frees: {}, bytes: {}, live: {}, peak: {} }}", counts.allocations, counts.frees, counts.bytes, counts.live, counts.peak);
}

void start() {
    counting.store(false, Relaxed);
    allocations.store(0, Relaxed);
    frees.store(0, Relaxed);
    bytes_allocated.store(0, Relaxed);
    live.store(0, Relaxed);
    peak.store(0, Relaxed);
    counting.store(true, Relaxed);
}

Counts stop() {
    Counts counted = counts();
    counting.store(false, Relaxed);
    return counted;
}

Counts counts() {
    return Counts{
        .allocations = allocations.load(Relaxed),
        .frees = frees.load(Relaxed),
        .bytes = bytes_allocated.load(Relaxed),
        .live = live.load(Relaxed),
        .peak = peak.load(Relaxed),
    };
}

Usage Usage::from_counts(std::string_view name, std::string_view unit, Counts counts, double units) {
    double divisor = std::fmax(units, 1.0);
    return Usage{
        .name = std::string(name),
        .unit = unit,
        .units = divisor,
        .allocations = static_cast<double>(counts.allocations) / divisor,
        .bytes = static_cast<double>(counts.bytes) / divisor,
        .peak = counts.peak,
        .kept = counts.live,
        .note = std::string(),
    };
}

json::Value Usage::to_json() const {
    return json::object({
        {"Name", json::Value(name)},
        {"Unit", json::Value(unit)},
        {"Units", json::Value(units)},
        {"AllocationsPerUnit", json::Value(allocations)},
        {"BytesPerUnit", json::Value(bytes)},
        {"PeakBytes", json::Value(static_cast<double>(peak))},
        {"KeptBytes", json::Value(static_cast<double>(kept))},
        {"Note", json::Value(note)},
    });
}

json::Value Video::to_json() const {
    return json::object({
        {"Name", json::Value(name)},
        {"LocalBytes", json::Value(static_cast<double>(local))},
        {"NonLocalBytes", json::Value(static_cast<double>(non_local))},
        {"Note", json::Value(note)},
    });
}

void Report::add(Report other) {
    heap.insert(heap.end(), std::make_move_iterator(other.heap.begin()), std::make_move_iterator(other.heap.end()));
    video.insert(video.end(), std::make_move_iterator(other.video.begin()), std::make_move_iterator(other.video.end()));
}

std::pair<size_t, size_t> runs(Budget budget, size_t most) {
    if (budget.iterations <= 1) {
        return {0, 1};
    }
    return {budget.warmup, std::max<size_t>(std::min(most, budget.iterations), 1)};
}

ProcessMemory process_memory() {
    PROCESS_MEMORY_COUNTERS_EX counters{};
    counters.cb = sizeof(PROCESS_MEMORY_COUNTERS_EX);
    BOOL ok = GetProcessMemoryInfo(GetCurrentProcess(), reinterpret_cast<PROCESS_MEMORY_COUNTERS*>(&counters), counters.cb);
    if (!ok) {
        return ProcessMemory{};
    }
    return ProcessMemory{
        .working_set = static_cast<std::uint64_t>(counters.WorkingSetSize),
        .peak_working_set = static_cast<std::uint64_t>(counters.PeakWorkingSetSize),
        .private_commit = static_cast<std::uint64_t>(counters.PrivateUsage),
    };
}

json::Value ProcessMemory::to_json() const {
    return json::object({
        {"WorkingSetBytes", json::Value(static_cast<double>(working_set))},
        {"PeakWorkingSetBytes", json::Value(static_cast<double>(peak_working_set))},
        {"PrivateCommitBytes", json::Value(static_cast<double>(private_commit))},
    });
}

std::string table(std::span<const Usage> usage) {
    size_t name_width = 4;
    size_t unit_width = 4;
    if (!usage.empty()) {
        name_width = 0;
        unit_width = 0;
        for (const Usage& row : usage) {
            name_width = std::max(name_width, row.name.size());
            unit_width = std::max(unit_width, row.unit.size());
        }
    }
    name_width = std::max<size_t>(name_width, 9);
    unit_width = std::max<size_t>(unit_width, 4);
    std::string text = std::format("{:<{}}  {:<{}}  {:>6}  {:>12}  {:>12}  {:>10}  {:>10}  Note\n", "Benchmark", name_width, "Unit", unit_width, "Units", "Allocations", "Bytes",
                                   "Peak KiB", "Kept KiB");
    for (const Usage& row : usage) {
        text += std::format("{:<{}}  {:<{}}  {:>6}  {:>12.1f}  {:>12.1f}  {:>10.1f}  {:>10.1f}  {}\n", row.name, name_width, row.unit, unit_width,
                            base::saturating_cast<std::uint64_t>(row.units), row.allocations, row.bytes, static_cast<double>(row.peak) / 1024.0,
                            static_cast<double>(row.kept) / 1024.0, row.note);
    }
    return text;
}

std::string video_table(std::span<const Video> video) {
    size_t name_width = 4;
    if (!video.empty()) {
        name_width = 0;
        for (const Video& row : video) {
            name_width = std::max(name_width, row.name.size());
        }
    }
    name_width = std::max<size_t>(name_width, 9);
    std::string text = std::format("{:<{}}  {:>10}  {:>14}  Note\n", "Benchmark", name_width, "Local MiB", "Non-local MiB");
    for (const Video& row : video) {
        text += std::format("{:<{}}  {:>10.2f}  {:>14.2f}  {}\n", row.name, name_width, static_cast<double>(row.local) / (1024.0 * 1024.0),
                            static_cast<double>(row.non_local) / (1024.0 * 1024.0), row.note);
    }
    return text;
}

} // namespace benchmarks::memory
