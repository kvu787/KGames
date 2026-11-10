// Every group's benchmarks run once, and measure something: a finite time
// above zero for each, with its name and unit, and, for the memory groups,
// counts of the heap or of the GPU's memory.

#include "Benchmarks/Benchmarks.h"
#include "Testing/Testing.h"

#include <cmath>
#include <vector>

using benchmarks::Budget;
using benchmarks::Group;
using benchmarks::Groups;
using benchmarks::Measurement;
using benchmarks::MemoryGroup;
using benchmarks::MemoryGroups;
using benchmarks::Report;
using benchmarks::Usage;
using benchmarks::Video;

TEST(every_group_runs_once_and_measures_something) {
    for (const Group& group : Groups) {
        std::vector<Measurement> measurements = group.run(Budget::Once);
        CHECK(!measurements.empty(), "{} measured nothing", group.name);
        for (const Measurement& measurement : measurements) {
            CHECK_EQ(measurement.iterations, 1, "{}", measurement.name);
            CHECK(std::isfinite(measurement.min_) && measurement.min_ >= 0.0, "{} took {} ms", measurement.name, measurement.min_);
            CHECK(measurement.min_ <= measurement.max_, "{}", measurement.name);
            CHECK(!measurement.unit.empty(), "{}", measurement.name);
            CHECK(measurement.name.starts_with(group.name) || group.gpu, "{} isn't in {}", measurement.name, group.name);
        }
    }
}

TEST(every_memory_group_runs_once_and_counts_something) {
    for (const MemoryGroup& group : MemoryGroups) {
        Report report = group.run(Budget::Once);
        CHECK(!report.heap.empty() || !report.video.empty(), "{} counted nothing", group.name);
        for (const Usage& usage : report.heap) {
            CHECK(usage.name.starts_with("memory/"), "{} isn't in a memory group", usage.name);
            CHECK(usage.units >= 1.0, "{} counted {} units", usage.name, usage.units);
            CHECK(usage.allocations >= 0.0 && std::isfinite(usage.allocations), "{}", usage.name);
            CHECK(usage.bytes >= 0.0 && std::isfinite(usage.bytes), "{}", usage.name);
            // The peak is the most it ever held, so nothing is left above it.
            CHECK(usage.peak >= 0 && usage.peak >= usage.kept, "{} kept {} of a peak of {}", usage.name, usage.kept, usage.peak);
            CHECK(!usage.unit.empty(), "{}", usage.name);
        }
        for (const Video& video : report.video) {
            CHECK(video.name.starts_with("memory/gpu"), "{} isn't GPU memory", video.name);
            CHECK(group.gpu, "{} measured GPU memory", group.name);
        }
    }
}
