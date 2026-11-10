// What the programs and the libraries keep in memory, and what they churn
// through: the C++ heap, counted by a global operator new and operator delete
// of our own; the GPU's memory, which the device reports; and the process's
// working set and private commit, which Windows reports.
//
// The allocator counts nothing until a memory benchmark turns it on, so the
// timing benchmarks pay only a relaxed load an allocation. It counts the C++
// heap alone, which is everything that the program and the libraries get
// with new, the standard library's containers and strings too. What the C
// runtime allocates for itself with malloc, and what DirectWrite, Direct3D
// and the display driver allocate for the process, is in heaps that it
// doesn't see, which the working set and the GPU's memory show instead.
//
// The operators themselves are in Allocator.cpp, which isn't part of this
// library: Benchmarks.exe and BenchmarksTests.exe each compile it as their
// own, as it says. They tell this file of every block, with `allocated` and
// `freed`.

#pragma once

#include "Benchmarks/Harness.h"
#include "Json/Json.h"

#include <cstddef>
#include <cstdint>
#include <span>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <vector>

namespace benchmarks::memory {

// Whether the allocator counts. Off until `start`. The operators ask before
// they find out the size of a block that they weren't told the size of.
bool is_counting();

// What the operators call for each block that they hand out, with the bytes
// that were asked for, and for each that they take back.
void allocated(size_t bytes);
void freed(size_t bytes);

// What the heap has done since counting started.
struct Counts {
    std::uint64_t allocations = 0;
    std::uint64_t frees = 0;
    std::uint64_t bytes = 0;
    // Bytes allocated and not yet freed, from where counting started, so it
    // can be negative when work frees what was there before it.
    std::int64_t live = 0;
    // The most that `live` ever was.
    std::int64_t peak = 0;

    bool operator==(const Counts&) const = default;
};

// The counts as Rust's {:?} writes them, which a failed check in a test
// shows.
std::string describe_value(const Counts& counts);

// Starts counting, from nothing. Every measurement here begins with one, so
// that `live` is from where the work began.
void start();

// Stops counting, and gives what the heap did.
Counts stop();

Counts counts();

// What a memory benchmark measured.
struct Usage {
    std::string name;
    // What the counts are of, like "frame" or "subdivide": a string
    // literal's text, which lasts as long as the program.
    std::string_view unit;
    double units;
    // Allocations and bytes allocated, per unit.
    double allocations;
    double bytes;
    // The most the work held at once, from where it started.
    std::int64_t peak;
    // What it still held when it was done: the footprint of what it built,
    // or, for repeated work that keeps nothing, a leak.
    std::int64_t kept;
    std::string note;

    static Usage from_counts(std::string_view name, std::string_view unit, Counts counts, double units);

    Usage&& with_note(std::string text) && {
        note = std::move(text);
        return std::move(*this);
    }

    json::Value to_json() const;

    bool operator==(const Usage&) const = default;
};

// How much GPU memory a program's drawing takes, from the device.
struct Video {
    std::string name;
    // Bytes of the GPU's own memory, and of system memory that it reads
    // across PCIe, that the process has reserved.
    std::uint64_t local;
    std::uint64_t non_local;
    std::string note;

    json::Value to_json() const;

    bool operator==(const Video&) const = default;
};

// What a memory group measured: the heap, and the GPU's memory.
struct Report {
    std::vector<Usage> heap;
    std::vector<Video> video;

    static Report of(std::vector<Usage> heap) {
        return Report{.heap = std::move(heap), .video = {}};
    }

    void add(Report other);

    bool operator==(const Report&) const = default;
};

// How many warm-up runs and counted runs the budget asks for, at most
// `most` counted. One run and no warm-up for Budget::Once, which tests use.
std::pair<size_t, size_t> runs(Budget budget, size_t most);

// Counts what `body` does to the heap over `repeat` runs, after `warmup`
// runs that aren't counted, and gives the counts per run. Allocation counts
// don't vary the way times do, so there's no need for statistics: the one
// number is the cost, as long as the warm-up has filled the caches that the
// work hits.
template <class Body> Usage measure(std::string_view name, std::string_view unit, Budget budget, size_t most, Body&& body) {
    auto [warmup, repeat] = runs(budget, most);
    for (size_t run = 0; run < warmup; run++) {
        body();
    }
    start();
    for (size_t run = 0; run < repeat; run++) {
        body();
    }
    Counts counted = stop();
    return Usage::from_counts(name, unit, counted, static_cast<double>(repeat));
}

// Counts what making one thing costs, and how much heap it holds while it's
// alive, which is its footprint. The value goes back to the caller, so that
// nothing of it is freed before it's counted.
template <class Make> auto held(std::string_view name, std::string_view unit, Make&& make) -> std::pair<Usage, std::invoke_result_t<Make>> {
    start();
    std::invoke_result_t<Make> value = std::forward<Make>(make)();
    // Counting is off from here, so the usage's own text isn't counted.
    Counts counted = stop();
    return {Usage::from_counts(name, unit, counted, 1.0), std::move(value)};
}

// The process's memory, as Windows counts it: the physical memory it has
// now, the most it ever had, and the private commit, which is everything it
// has asked for that isn't shared with another process, paged in or not.
struct ProcessMemory {
    std::uint64_t working_set = 0;
    std::uint64_t peak_working_set = 0;
    std::uint64_t private_commit = 0;

    json::Value to_json() const;

    bool operator==(const ProcessMemory&) const = default;
};

ProcessMemory process_memory();

// The heap measurements as a table that reads in a terminal: allocations and
// bytes a unit, and the peak and kept bytes, in kibibytes.
std::string table(std::span<const Usage> usage);

// The GPU memory measurements as a table, in mebibytes.
std::string video_table(std::span<const Video> video);

} // namespace benchmarks::memory
