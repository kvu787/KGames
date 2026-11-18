// The counting allocator, and the usage that `measure` and `held` make of
// its counts.
//
// The counters are the whole process's, so these tests run without the test
// harness, one after another on the main thread, where nothing else allocates
// while they count, but the threads that a test starts itself. Under libtest,
// other tests run on threads of their own, and its main thread allocates as
// it starts and finishes them, which the counts would take in. Every
// allocation goes through `black_box`, so that the optimizer can't drop one
// whose block is never read.

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use benchmarks::Budget;
use benchmarks::memory::{counts, held, measure, start, stop};

const TESTS: [(&str, fn()); 5] = [
    ("the_allocator_counts_the_bytes_it_hands_out_and_takes_back", the_allocator_counts_the_bytes_it_hands_out_and_takes_back),
    ("it_counts_nothing_until_a_benchmark_turns_it_on", it_counts_nothing_until_a_benchmark_turns_it_on),
    ("a_measurement_is_of_one_run_of_the_work", a_measurement_is_of_one_run_of_the_work),
    ("what_is_held_is_counted_while_it_is_alive", what_is_held_is_counted_while_it_is_alive),
    ("the_peak_is_never_less_than_what_is_held_while_other_threads_allocate", the_peak_is_never_less_than_what_is_held_while_other_threads_allocate),
];

fn main() {
    common::run(&TESTS);
}

fn the_allocator_counts_the_bytes_it_hands_out_and_takes_back() {
    start();
    let block: Vec<u8> = std::hint::black_box(vec![7; 4096]);
    let after = counts();
    assert!(after.allocations >= 1, "{after:?}");
    assert!(after.bytes >= 4096, "{after:?}");
    assert!(after.live >= 4096, "{after:?}");
    drop(block);
    let done = stop();
    assert!(done.frees >= 1, "{done:?}");
    assert!(done.live <= 0, "{done:?}");
    assert!(done.peak >= 4096, "{done:?}");
}

fn it_counts_nothing_until_a_benchmark_turns_it_on() {
    start();
    let quiet = stop();
    let block: Vec<u8> = std::hint::black_box(vec![7; 8192]);
    std::hint::black_box(&block);
    assert_eq!(counts(), quiet);
}

fn a_measurement_is_of_one_run_of_the_work() {
    let mut runs_done = 0;
    let usage = measure("memory/test/vector", "run", Budget { seconds: 0.0, iterations: 100, warmup: 2 }, 4, || {
        runs_done += 1;
        let block: Vec<u64> = std::hint::black_box(vec![1; 128]);
        std::hint::black_box(&block);
    });
    // Two warm-up runs that aren't counted, and four that are.
    assert_eq!(runs_done, 6);
    assert_eq!(usage.units, 4.0);
    assert_eq!(usage.allocations, 1.0);
    assert_eq!(usage.bytes, 1024.0);
    // It frees what it allocates, and never holds more than one at a time.
    assert_eq!(usage.kept, 0);
    assert_eq!(usage.peak, 1024);
}

fn what_is_held_is_counted_while_it_is_alive() {
    let (usage, held_block) = held("memory/test/held", "block", || std::hint::black_box(vec![0u8; 2048]));
    assert_eq!(held_block.len(), 2048);
    std::hint::black_box(&held_block);
    assert_eq!(usage.units, 1.0);
    assert_eq!(usage.kept, 2048);
    assert_eq!(usage.peak, 2048);
    // Counting is off again, so the drop isn't counted.
    let before = counts();
    drop(held_block);
    assert_eq!(counts(), before);
}

// The counters are the whole process's, so other threads' allocations count
// too, like those of work that spreads over threads. Each allocation adds to
// what's held, then raises the peak, so a snapshot between the two would hold
// more than its peak. Here other threads keep what they allocate, so that each
// allocation is a new peak, while this one takes snapshots.
fn the_peak_is_never_less_than_what_is_held_while_other_threads_allocate() {
    const THREADS: usize = 4;
    const BLOCKS: usize = 200_000;
    start();
    let finished = Arc::new(AtomicUsize::new(0));
    let threads: Vec<_> = (0..THREADS)
        .map(|_| {
            let finished = finished.clone();
            std::thread::spawn(move || {
                let mut kept = Vec::with_capacity(BLOCKS);
                for _ in 0..BLOCKS {
                    kept.push(std::hint::black_box(Box::new([0u8; 64])));
                }
                finished.fetch_add(1, Ordering::Relaxed);
                kept
            })
        })
        .collect();
    let mut snapshots = 0u64;
    while finished.load(Ordering::Relaxed) < THREADS {
        let snapshot = counts();
        assert!(snapshot.peak >= snapshot.live, "after {snapshots} snapshots: {snapshot:?}");
        snapshots += 1;
    }
    let kept: Vec<_> = threads.into_iter().map(|thread| thread.join().unwrap()).collect();
    let done = stop();
    assert!(done.peak >= done.live && done.live >= (THREADS * BLOCKS * 64) as i64, "{done:?}");
    drop(kept);
}
