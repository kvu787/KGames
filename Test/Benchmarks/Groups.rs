// Every group's benchmarks run once, and measure something: a finite time
// above zero for each, with its name and unit, and, for the memory groups,
// counts of the heap or of the GPU's memory.

use benchmarks::{Budget, GROUPS, MEMORY_GROUPS};

#[test]
fn every_group_runs_once_and_measures_something() {
    for group in GROUPS {
        let measurements = (group.run)(Budget::ONCE);
        assert!(!measurements.is_empty(), "{} measured nothing", group.name);
        for measurement in measurements {
            assert_eq!(measurement.iterations, 1, "{}", measurement.name);
            assert!(measurement.min.is_finite() && measurement.min >= 0.0, "{} took {} ms", measurement.name, measurement.min);
            assert!(measurement.min <= measurement.max, "{}", measurement.name);
            assert!(!measurement.unit.is_empty(), "{}", measurement.name);
            assert!(measurement.name.starts_with(group.name) || group.gpu, "{} isn't in {}", measurement.name, group.name);
        }
    }
}

#[test]
fn every_memory_group_runs_once_and_counts_something() {
    for group in MEMORY_GROUPS {
        let report = (group.run)(Budget::ONCE);
        assert!(!report.heap.is_empty() || !report.video.is_empty(), "{} counted nothing", group.name);
        for usage in &report.heap {
            assert!(usage.name.starts_with("memory/"), "{} isn't in a memory group", usage.name);
            assert!(usage.units >= 1.0, "{} counted {} units", usage.name, usage.units);
            assert!(usage.allocations >= 0.0 && usage.allocations.is_finite(), "{}", usage.name);
            assert!(usage.bytes >= 0.0 && usage.bytes.is_finite(), "{}", usage.name);
            // The peak is the most it ever held, so nothing is left above it.
            assert!(usage.peak >= 0 && usage.peak >= usage.kept, "{} kept {} of a peak of {}", usage.name, usage.kept, usage.peak);
            assert!(!usage.unit.is_empty(), "{}", usage.name);
        }
        for video in &report.video {
            assert!(video.name.starts_with("memory/gpu"), "{} isn't GPU memory", video.name);
            assert!(group.gpu, "{} measured GPU memory", group.name);
        }
    }
}
