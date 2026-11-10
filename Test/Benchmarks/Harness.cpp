// The harness: its statistics of given times, its budget, its table, its
// JSON, and the command line.

#include "Base/Text.h"
#include "Benchmarks/Benchmarks.h"
#include "Testing/Testing.h"
#include "Json/Json.h"

#include <array>
#include <cmath>
#include <cstddef>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

using benchmarks::Budget;
using benchmarks::measure;
using benchmarks::Measurement;
using benchmarks::Options;
using benchmarks::parse_arguments;
using benchmarks::percentile;
using benchmarks::selected;
using benchmarks::table;

namespace {

// A member of an object, which must have it: what comparing get's result
// with Some(&expected) checks in Rust.
const json::Value& member(const json::Value& value, std::string_view name) {
    const json::Value* found = value.get(name);
    CHECK(found != nullptr, "{} isn't there", name);
    return *found;
}

} // namespace

TEST(the_statistics_are_of_the_times_per_unit) {
    Measurement measurement = Measurement::from_times("a", "frame", {{2.0, 4.0, 6.0, 8.0, 10.0}}, 2.0);
    CHECK_EQ(measurement.iterations, 5);
    CHECK_EQ(measurement.min_, 1.0);
    CHECK_EQ(measurement.median, 3.0);
    CHECK_EQ(measurement.mean, 3.0);
    CHECK_EQ(measurement.max_, 5.0);
    CHECK(std::abs(measurement.percentile_99 - 4.96) < 1.0e-9);
}

TEST(a_percentile_is_between_the_two_nearest_values) {
    CHECK_EQ(percentile({}, 0.5), 0.0);
    CHECK_EQ(percentile({{7.0}}, 0.99), 7.0);
    CHECK_EQ(percentile({{1.0, 3.0}}, 0.5), 2.0);
    CHECK_EQ(percentile({{1.0, 2.0, 3.0, 4.0}}, 1.0), 4.0);
}

TEST(the_budget_bounds_the_iterations) {
    {
        size_t calls = 0;
        Measurement measurement = measure("b", "call", Budget{.seconds = 10.0, .iterations = 7, .warmup = 3}, 1.0, [&] { calls += 1; });
        CHECK_EQ(calls, 10);
        CHECK_EQ(measurement.iterations, 7);
    }
    {
        size_t calls = 0;
        Measurement measurement = measure("c", "call", Budget::Once, 1.0, [&] { calls += 1; });
        CHECK_EQ(std::pair(calls, measurement.iterations), (std::pair<size_t, size_t>(1, 1)));
    }
    // The seconds stop it after the first iteration past them.
    Measurement measurement = measure("d", "call", Budget{.seconds = 0.0, .iterations = 100, .warmup = 0}, 1.0, [] {});
    CHECK_EQ(measurement.iterations, 1);
}

TEST(the_table_has_a_row_a_measurement_with_its_note) {
    Measurement measurement = Measurement::from_times("mesh/x", "subdivide", {{1.5}}, 1.0).with_note("6 faces");
    std::string text = table({{measurement}});
    std::vector<std::string_view> lines = base::lines(text);
    CHECK_EQ(lines.size(), 2);
    CHECK(lines[0].starts_with("Benchmark"));
    CHECK(lines[1].starts_with("mesh/x"));
    CHECK(lines[1].contains("1.5000"));
    CHECK(lines[1].ends_with("6 faces"));
}

TEST(a_measurement_s_json_has_every_statistic) {
    json::Value value = Measurement::from_times("a", "frame", {{1.0, 2.0}}, 1.0).to_json();
    CHECK_EQ(member(value, "Name"), json::Value("a"));
    CHECK_EQ(member(value, "Iterations"), json::Value(2.0));
    CHECK_EQ(member(value, "MinMs"), json::Value(1.0));
    CHECK_EQ(member(value, "MaxMs"), json::Value(2.0));
}

TEST(the_command_line_picks_the_groups_the_budget_and_the_output) {
    std::vector<std::string> arguments{"--filter", "mesh", "--quick", "--json", "out.json"};
    CHECK_EQ(parse_arguments(arguments).value(), (Options{
                                                     .filter = "mesh",
                                                     .budget = Budget::Quick,
                                                     .json = "out.json",
                                                     .list = false,
                                                 }));
    CHECK_EQ(parse_arguments({}).value().budget, Budget::Default);
    CHECK(!parse_arguments({{std::string("--filter")}}).has_value());
    CHECK(!parse_arguments({{std::string("--nope")}}).has_value());
    CHECK_EQ(selected("mesh").size(), 1);
    CHECK_EQ(selected("nothing").size(), 0);
    CHECK_EQ(selected(std::nullopt).size(), benchmarks::Groups.size());
}
