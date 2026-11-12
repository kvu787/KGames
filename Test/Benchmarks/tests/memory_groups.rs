// Every memory group runs once, and counts something: the heap, or the GPU's
// memory.
//
// The allocator's counts are the whole process's, so this runs without the
// test harness, alone on the main thread, like counting.rs. Under libtest,
// other tests run on threads of their own, and its main thread allocates as
// it starts and finishes them, which a group's counts would take in.

mod common;

use benchmarks::{Budget, MEMORY_GROUPS};

fn main() {
    common::run(&[("every_memory_group_runs_once_and_counts_something", every_memory_group_runs_once_and_counts_something)]);
}

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
