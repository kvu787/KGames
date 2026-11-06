// The memory harness: the counting allocator, the usage that it makes of the
// counts, how many runs a budget asks for, the tables, and the process's
// memory.
//
// The counters are the whole process's, so the tests that count take them in
// turn. Every allocation goes through `black_box`, so that the optimizer
// can't drop one whose block is never read.

use std::sync::Mutex;

use benchmarks::memory::{self, Counts, Video, counts, held, measure, process_memory, runs, start, stop, table, video_table};
use benchmarks::{Budget, Usage};

static COUNTERS: Mutex<()> = Mutex::new(());

#[test]
fn the_allocator_counts_the_bytes_it_hands_out_and_takes_back() {
    let _counters = COUNTERS.lock().unwrap();
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

#[test]
fn it_counts_nothing_until_a_benchmark_turns_it_on() {
    let _counters = COUNTERS.lock().unwrap();
    start();
    let quiet = stop();
    let block: Vec<u8> = std::hint::black_box(vec![7; 8192]);
    std::hint::black_box(&block);
    assert_eq!(counts(), quiet);
}

#[test]
fn a_measurement_is_of_one_run_of_the_work() {
    let _counters = COUNTERS.lock().unwrap();
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

#[test]
fn what_is_held_is_counted_while_it_is_alive() {
    let _counters = COUNTERS.lock().unwrap();
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

#[test]
fn the_budget_bounds_the_runs_and_one_run_is_enough_for_a_test() {
    assert_eq!(runs(Budget::ONCE, 200), (0, 1));
    assert_eq!(runs(Budget::DEFAULT, 200), (Budget::DEFAULT.warmup, 200));
    assert_eq!(runs(Budget::QUICK, 1000), (Budget::QUICK.warmup, Budget::QUICK.iterations));
}

#[test]
fn the_usage_of_counts_is_per_unit() {
    let counts = Counts {
        allocations: 20,
        frees: 18,
        bytes: 4000,
        live: 200,
        peak: 900,
    };
    let usage = Usage::from_counts("memory/test/counts", "frame", counts, 10.0).with_note("a note");
    assert_eq!((usage.allocations, usage.bytes), (2.0, 400.0));
    assert_eq!((usage.peak, usage.kept), (900, 200));
    let value = usage.to_json();
    assert_eq!(value.get("Name"), Some(&json::Value::from("memory/test/counts")));
    assert_eq!(value.get("AllocationsPerUnit"), Some(&json::Value::from(2.0)));
    assert_eq!(value.get("KeptBytes"), Some(&json::Value::from(200.0)));
    assert_eq!(value.get("Note"), Some(&json::Value::from("a note")));
}

#[test]
fn the_tables_have_a_row_a_measurement_with_its_note() {
    let usage = Usage::from_counts("memory/test/row", "frame", Counts { allocations: 3, frees: 3, bytes: 3072, live: 0, peak: 2048 }, 1.0);
    let text = table(&[usage.with_note("3 widgets")]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("Benchmark") && lines[0].contains("Peak KiB"));
    assert!(lines[1].starts_with("memory/test/row"));
    assert!(lines[1].ends_with("3 widgets"));

    let video = Video {
        name: "memory/gpu/test".to_string(),
        local: 2 * 1024 * 1024,
        non_local: 1024 * 1024,
        note: "a target".to_string(),
    };
    let text = video_table(&[video]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Local MiB") && lines[0].contains("Non-local MiB"));
    assert!(lines[1].contains("2.00") && lines[1].contains("1.00"));
    assert!(lines[1].ends_with("a target"));
}

#[test]
fn the_process_has_a_working_set_and_a_private_commit() {
    let process = process_memory();
    assert!(process.working_set > 0);
    assert!(process.peak_working_set >= process.working_set);
    assert!(process.private_commit > 0);
    let value = process.to_json();
    assert_eq!(value.get("WorkingSetBytes"), Some(&json::Value::from(process.working_set as f64)));
}

#[test]
fn a_report_holds_the_heap_rows_and_the_gpu_rows_together() {
    let mut report = memory::Report::of(vec![Usage::from_counts("memory/test/a", "frame", Counts::default(), 1.0)]);
    report.add(memory::Report {
        heap: vec![Usage::from_counts("memory/test/b", "frame", Counts::default(), 1.0)],
        video: vec![Video {
            name: "memory/gpu/b".to_string(),
            local: 1,
            non_local: 2,
            note: String::new(),
        }],
    });
    assert_eq!(report.heap.len(), 2);
    assert_eq!(report.video.len(), 1);
    assert_eq!(report.video[0].to_json().get("LocalBytes"), Some(&json::Value::from(1.0)));
}
