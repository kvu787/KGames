#include "Benchmarks/Harness.h"

#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Base/Text.h"

#include <cmath>
#include <compare>
#include <format>

namespace benchmarks {

namespace {

// Whether a comes before b in the order of every double, NaNs too:
// f64::total_cmp.
bool total_less(double a, double b) {
    return std::strong_order(a, b) < 0;
}

} // namespace

std::string describe_value(const Budget& budget) {
    return std::format("Budget {{ seconds: {}, iterations: {}, warmup: {} }}", base::debug(budget.seconds), budget.iterations, budget.warmup);
}

Measurement Measurement::from_times(std::string_view name, std::string_view unit, std::span<const double> times, double units) {
    if (times.empty()) {
        base::panic(std::format("{} was never timed", name));
    }
    std::vector<double> sorted;
    sorted.reserve(times.size());
    for (double time : times) {
        sorted.push_back(time / units);
    }
    std::ranges::stable_sort(sorted, total_less);
    double sum = 0.0;
    for (double time : sorted) {
        sum += time;
    }
    return Measurement{
        .name = std::string(name),
        .unit = unit,
        .iterations = sorted.size(),
        .min_ = sorted[0],
        .median = percentile(sorted, 0.5),
        .mean = sum / static_cast<double>(sorted.size()),
        .percentile_99 = percentile(sorted, 0.99),
        .max_ = sorted[sorted.size() - 1],
        .note = std::string(),
    };
}

json::Value Measurement::to_json() const {
    return json::object({
        {"Name", json::Value(name)},
        {"Unit", json::Value(unit)},
        {"Iterations", json::Value(static_cast<double>(iterations))},
        {"MinMs", json::Value(min_)},
        {"MedianMs", json::Value(median)},
        {"MeanMs", json::Value(mean)},
        {"Percentile99Ms", json::Value(percentile_99)},
        {"MaxMs", json::Value(max_)},
        {"Note", json::Value(note)},
    });
}

double percentile(std::span<const double> sorted, double fraction) {
    if (sorted.empty()) {
        return 0.0;
    }
    double position = fraction * static_cast<double>(sorted.size() - 1);
    size_t below = base::saturating_cast<size_t>(std::floor(position));
    size_t above = base::saturating_cast<size_t>(std::ceil(position));
    double weight = position - static_cast<double>(below);
    return sorted[below] * (1.0 - weight) + sorted[above] * weight;
}

std::string table(std::span<const Measurement> measurements) {
    size_t name_width = 4;
    size_t unit_width = 4;
    if (!measurements.empty()) {
        name_width = 0;
        unit_width = 0;
        for (const Measurement& measurement : measurements) {
            name_width = std::max(name_width, measurement.name.size());
            unit_width = std::max(unit_width, measurement.unit.size());
        }
    }
    name_width = std::max<size_t>(name_width, 9);
    unit_width = std::max<size_t>(unit_width, 4);
    std::string text = std::format("{:<{}}  {:<{}}  {:>10}  {:>10}  {:>10}  {:>10}  {:>10}  {:>10}  Note\n", "Benchmark", name_width, "Unit", unit_width, "Iterations", "Min ms",
                                   "Median ms", "Mean ms", "99th ms", "Max ms");
    for (const Measurement& measurement : measurements) {
        text += std::format("{:<{}}  {:<{}}  {:>10}  {:>10.4f}  {:>10.4f}  {:>10.4f}  {:>10.4f}  {:>10.4f}  {}\n", measurement.name, name_width, measurement.unit, unit_width,
                            measurement.iterations, measurement.min_, measurement.median, measurement.mean, measurement.percentile_99, measurement.max_, measurement.note);
    }
    return text;
}

} // namespace benchmarks
