// Benchmarks of KGames' hot paths, as a library that Test/Benchmarks tests:
// the harness, the groups of benchmarks, and the command line. README.md
// describes them.
//
// - Harness.h: times a body many times and keeps the statistics.
// - Interface.h: the programs' frames on the CPU and the GPU, text, input and
//   the PerfBar.
// - Libraries.h: meshes, models, JSON, math and the present tracker.
// - Memory.h: counts the heap, and reads the GPU's and the process's memory.
// - Footprints.h: what the programs and the libraries hold and churn.
//
// The heap counts of every benchmark that asks for them are from one
// allocator for the whole program, which is Allocator.cpp's.

#pragma once

#include "Benchmarks/BlackBox.h"
#include "Benchmarks/Footprints.h"
#include "Benchmarks/Harness.h"
#include "Benchmarks/Interface.h"
#include "Benchmarks/Libraries.h"
#include "Benchmarks/Memory.h"
#include "Json/Json.h"

#include <array>
#include <expected>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace benchmarks {

using memory::process_memory;
using memory::ProcessMemory;
using memory::Report;
using memory::Usage;
using memory::Video;

// The command line. With an underscore, as Usage is what a memory benchmark
// measured.
inline constexpr std::string_view Usage_ = "Benchmarks [--filter <text>] [--quick] [--json <path>] [--list]";

// A group of benchmarks, run together because they share a setup, like a
// GPU.
struct Group {
    std::string_view name;
    // Whether its benchmarks draw with the GPU.
    bool gpu;
    std::vector<Measurement> (*run)(Budget);
};

inline constexpr auto Groups = std::to_array<Group>({
    {"ui", false, interface_::frames},
    {"gpu", true, interface_::draws},
    {"text", false, interface_::text},
    {"input", false, interface_::input},
    {"perfbar", false, interface_::perf_bar},
    {"mesh", false, libraries::meshes},
    {"gltf", false, libraries::models},
    {"json", false, libraries::json},
    {"math", false, libraries::math},
    {"present-timing", false, libraries::present_timing},
});

// A group of memory benchmarks, which count what's held and churned
// instead of timing it.
struct MemoryGroup {
    std::string_view name;
    // Whether its benchmarks need a GPU.
    bool gpu;
    Report (*run)(Budget);
};

inline constexpr auto MemoryGroups = std::to_array<MemoryGroup>({
    {"memory/frames", false, footprints::frames},
    {"memory/caches", false, footprints::caches},
    {"memory/libraries", false, footprints::libraries},
    {"memory/gpu", true, footprints::video},
});

// What the command line asks for.
struct Options {
    // Only the groups whose names have the text, if there is any.
    std::optional<std::string> filter;
    Budget budget;
    // Where to write the measurements as JSON, if anywhere.
    std::optional<std::string> json;
    // Only list the groups.
    bool list;

    bool operator==(const Options&) const = default;
};

// The options as Rust's {:?} writes them, which a failed check in a test
// shows.
std::string describe_value(const Options& options);

std::expected<Options, std::string> parse_arguments(std::span<const std::string> arguments);

// The groups that the filter keeps.
std::vector<Group> selected(std::optional<std::string_view> filter);

// The memory groups that the filter keeps.
std::vector<MemoryGroup> selected_memory(std::optional<std::string_view> filter);

// What the measurements were taken on, for the report's header: the GPU,
// whether the program ran in RunLightly's job, the build, the processors,
// and what compiled it, and whether with the standard library's checks, for
// comparing with the same benchmarks in another language or another build.
json::Value environment();

// The measurements, the memory, and the environment as one JSON object.
json::Value report(const json::Value& environment, std::span<const Measurement> measurements, const Report& memory, ProcessMemory process);

} // namespace benchmarks
