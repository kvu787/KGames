#include "Benchmarks/Benchmarks.h"

#include "Base/Text.h"
#include "Base/Windows.h"
#include "Graphics/Graphics.h"

#include <format>
#include <utility>

namespace benchmarks {

namespace {

// An optional text as Rust's {:?} writes it, like Some("mesh") or None.
std::string debug(const std::optional<std::string>& text) {
    return text ? std::format("Some({})", base::debug(std::string_view(*text))) : std::string("None");
}

// The compiler's version, like MSVC 19.51.36260, which is cl.exe's own
// first three numbers.
std::string compiler() {
    return std::format("MSVC {}.{:02}.{}", _MSC_FULL_VER / 10'000'000, _MSC_FULL_VER / 100'000 % 100, _MSC_FULL_VER % 100'000);
}

} // namespace

std::string describe_value(const Options& options) {
    return std::format("Options {{ filter: {}, budget: {}, json: {}, list: {} }}", debug(options.filter), describe_value(options.budget), debug(options.json), options.list);
}

std::expected<Options, std::string> parse_arguments(std::span<const std::string> arguments) {
    Options options{
        .filter = std::nullopt,
        .budget = Budget::Default,
        .json = std::nullopt,
        .list = false,
    };
    for (auto argument = arguments.begin(); argument != arguments.end(); ++argument) {
        if (*argument == "--filter") {
            ++argument;
            if (argument == arguments.end()) {
                return std::unexpected(std::string("--filter needs text after it."));
            }
            options.filter = *argument;
        } else if (*argument == "--json") {
            ++argument;
            if (argument == arguments.end()) {
                return std::unexpected(std::string("--json needs a path after it."));
            }
            options.json = *argument;
        } else if (*argument == "--quick") {
            options.budget = Budget::Quick;
        } else if (*argument == "--list") {
            options.list = true;
        } else {
            return std::unexpected(std::format("Unknown argument: {}.", *argument));
        }
    }
    return options;
}

std::vector<Group> selected(std::optional<std::string_view> filter) {
    std::vector<Group> groups;
    for (const Group& group : Groups) {
        if (!filter || group.name.contains(*filter)) {
            groups.push_back(group);
        }
    }
    return groups;
}

std::vector<MemoryGroup> selected_memory(std::optional<std::string_view> filter) {
    std::vector<MemoryGroup> groups;
    for (const MemoryGroup& group : MemoryGroups) {
        if (!filter || group.name.contains(*filter)) {
            groups.push_back(group);
        }
    }
    return groups;
}

json::Value environment() {
    std::string gpu;
    try {
        gpu = graphics::Gpu(graphics::AdapterChoice::HighPerformance{}).name;
    } catch (const base::WindowsError& error) {
        gpu = std::format("none: {}", error.what());
    }
    // The processors that Windows schedules threads on, which is what Rust's
    // std::thread::available_parallelism reads.
    SYSTEM_INFO system{};
    GetSystemInfo(&system);
#ifdef NDEBUG
    std::string_view profile = "release";
#else
    std::string_view profile = "debug";
#endif
    return json::object({
        {"Gpu", json::Value(gpu)},
        {"Lightly", json::Value(base::environment_variable("KGAMES_LIGHT_JOB").has_value())},
        {"Profile", json::Value(profile)},
        {"Cores", json::Value(static_cast<double>(system.dwNumberOfProcessors))},
        {"Language", json::Value("C++")},
        {"Compiler", json::Value(compiler())},
    });
}

json::Value report(const json::Value& environment, std::span<const Measurement> measurements, const Report& memory, ProcessMemory process) {
    json::Value::Array measured;
    measured.values.reserve(measurements.size());
    for (const Measurement& measurement : measurements) {
        measured.values.push_back(measurement.to_json());
    }
    json::Value::Array heap;
    heap.values.reserve(memory.heap.size());
    for (const Usage& usage : memory.heap) {
        heap.values.push_back(usage.to_json());
    }
    json::Value::Array video;
    video.values.reserve(memory.video.size());
    for (const Video& row : memory.video) {
        video.values.push_back(row.to_json());
    }
    return json::object({
        {"Environment", environment},
        {"Measurements", json::Value(std::move(measured))},
        {"Memory", json::Value(std::move(heap))},
        {"VideoMemory", json::Value(std::move(video))},
        {"Process", process.to_json()},
    });
}

} // namespace benchmarks
