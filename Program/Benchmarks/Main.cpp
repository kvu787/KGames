// Runs the benchmarks and prints their tables, and writes them as JSON if
// asked. It has no window. See README.md.
//
//     Benchmarks [--filter <text>] [--quick] [--json <path>] [--list]

#include "Base/Files.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Benchmarks/Benchmarks.h"

#include <fcntl.h>
#include <io.h>

#include <cstdio>
#include <expected>
#include <format>
#include <iterator>
#include <optional>
#include <print>
#include <string>
#include <string_view>
#include <vector>

using benchmarks::environment;
using benchmarks::Measurement;
using benchmarks::parse_arguments;
using benchmarks::process_memory;
using benchmarks::ProcessMemory;
using benchmarks::Report;
using benchmarks::report;
using benchmarks::selected;
using benchmarks::selected_memory;
using benchmarks::table;
using benchmarks::Usage_;

namespace {

std::expected<void, std::string> run() {
    std::vector<std::string> arguments = base::arguments();
    std::expected<benchmarks::Options, std::string> options = parse_arguments(arguments);
    if (!options) {
        return std::unexpected(std::format("{}\nUsage: {}", options.error(), Usage_));
    }
    std::optional<std::string_view> filter = options->filter ? std::optional<std::string_view>(*options->filter) : std::nullopt;
    std::vector<benchmarks::Group> groups = selected(filter);
    std::vector<benchmarks::MemoryGroup> memory_groups = selected_memory(filter);
    if (groups.empty() && memory_groups.empty()) {
        return std::unexpected(std::format("No group's name has {}. Use --list to see them.", base::debug(filter.value_or(std::string_view()))));
    }
    if (options->list) {
        for (const benchmarks::Group& group : groups) {
            std::println("{}{}", group.name, group.gpu ? " (GPU)" : "");
        }
        for (const benchmarks::MemoryGroup& group : memory_groups) {
            std::println("{}{}", group.name, group.gpu ? " (GPU)" : "");
        }
        return {};
    }
    json::Value environment_value = environment();
    std::println("Environment: {}", environment_value.to_json());
    std::println("Budget: {}", describe_value(options->budget));
    // Rust hands on each line as it ends. C keeps what's written to a file
    // or a pipe until there's a lot of it, or the program ends, so the lines
    // are handed on here, before the groups say on standard error that they
    // run.
    std::fflush(stdout);
    std::vector<Measurement> measurements;
    for (const benchmarks::Group& group : groups) {
        std::println(stderr, "Running {}...", group.name);
        std::vector<Measurement> measured = group.run(options->budget);
        measurements.insert(measurements.end(), std::make_move_iterator(measured.begin()), std::make_move_iterator(measured.end()));
    }
    Report memory;
    for (const benchmarks::MemoryGroup& group : memory_groups) {
        std::println(stderr, "Running {}...", group.name);
        memory.add(group.run(options->budget));
    }
    ProcessMemory process = process_memory();
    std::println();
    if (!measurements.empty()) {
        std::print("{}", table(measurements));
    }
    if (!memory.heap.empty()) {
        std::println();
        std::print("{}", benchmarks::memory::table(memory.heap));
    }
    if (!memory.video.empty()) {
        std::println();
        std::print("{}", benchmarks::memory::video_table(memory.video));
    }
    std::println("\nThis process: {:.1f} MiB of working set, {:.1f} MiB at its peak, {:.1f} MiB of private commit.", static_cast<double>(process.working_set) / (1024.0 * 1024.0),
                 static_cast<double>(process.peak_working_set) / (1024.0 * 1024.0), static_cast<double>(process.private_commit) / (1024.0 * 1024.0));
    if (options->json) {
        const std::string& path = *options->json;
        std::expected<void, std::string> written = base::write(base::path(path), report(environment_value, measurements, memory, process).to_json() + "\n");
        if (!written) {
            return std::unexpected(std::format("Couldn't write {}: {}", path, written.error()));
        }
        std::println("Wrote {}.", path);
    }
    return {};
}

} // namespace

int main() {
    base::fail_quietly();
    // Text goes out as it's written, as Rust's does: without this, a line
    // that's written to a file or a pipe ends with a carriage return and a
    // line feed, not a line feed alone.
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
    std::expected<void, std::string> result = run();
    if (!result) {
        std::println(stderr, "Benchmarks: {}", result.error());
        return 1;
    }
    return 0;
}
