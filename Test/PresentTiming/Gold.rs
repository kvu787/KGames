// Tests that read_file gives what PresentMon 2.6.1 writes for the gold
// traces in its repository (Tests/Gold), without running PresentMon: the
// gold CSVs beside them are what its own tests require its output to match,
// row for row. Every present in them, of every process, has the same mode,
// tearing and screen time. The traces aren't in this repository, so the
// test is ignored unless it's asked for, with PRESENTMON_GOLD naming a
// folder that has test_case_0.etl to test_case_5.etl and their gold CSVs,
// test_case_0.csv to test_case_5.csv:
//
//   $env:PRESENTMON_GOLD = 'C:\PresentMon\Tests\Gold'
//   cargo test -p present_timing_tests --test Gold -- --ignored --nocapture

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use present_timing::{PresentMode, read_file};

const CASES: usize = 6;

// What PresentMon's CSV calls each mode.
fn presentmon_name(mode: PresentMode) -> &'static str {
    match mode {
        PresentMode::Unknown => "Other",
        PresentMode::LegacyFlip => "Hardware: Legacy Flip",
        PresentMode::LegacyCopy => "Hardware: Legacy Copy to front buffer",
        PresentMode::IndependentFlip => "Hardware: Independent Flip",
        PresentMode::OverlayPlane => "Hardware Composed: Independent Flip",
        PresentMode::ComposedFlip => "Composed: Flip",
        PresentMode::ComposedGpuCopy => "Composed: Copy with GPU GDI",
        PresentMode::ComposedCpuCopy => "Composed: Copy with CPU GDI",
    }
}

// A row of PresentMon's CSV.
struct Row {
    process: u32,
    application: String,
    mode: String,
    tearing: bool,
    start: i64,
    // In milliseconds, or None if it never reached the screen.
    until_displayed: Option<f64>,
}

fn read_csv(path: &Path) -> Vec<Row> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("Couldn't read {}: {error}", path.display()));
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let column = |name: &str| header.iter().position(|known| *known == name).unwrap_or_else(|| panic!("No {name} column"));
    let (application, process, tearing, mode, start, until_displayed) = (
        column("Application"),
        column("ProcessID"),
        column("AllowsTearing"),
        column("PresentMode"),
        column("TimeInQPC"),
        column("MsUntilDisplayed"),
    );
    lines
        .map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            Row {
                process: fields[process].parse().unwrap(),
                application: fields[application].to_string(),
                mode: fields[mode].to_string(),
                tearing: fields[tearing] == "1",
                start: fields[start].parse().unwrap(),
                until_displayed: fields[until_displayed].parse().ok(),
            }
        })
        .collect()
}

#[test]
#[ignore = "needs PresentMon's gold traces, in the folder that PRESENTMON_GOLD names"]
fn presents_are_the_same_as_presentmons_on_its_gold_traces() {
    let folder = std::env::var("PRESENTMON_GOLD").expect("PRESENTMON_GOLD names the folder with PresentMon's gold traces");
    let folder = Path::new(&folder);
    let mut mismatches = Vec::new();
    // How many presents of each mode were the same.
    let mut same: BTreeMap<String, usize> = BTreeMap::new();
    for case in 0..CASES {
        let rows = read_csv(&folder.join(format!("test_case_{case}.csv")));
        let mut processes: Vec<u32> = rows.iter().map(|row| row.process).collect();
        processes.sort();
        processes.dedup();
        for process in processes {
            let (presents, frequency) = read_file(&folder.join(format!("test_case_{case}.etl")), process).unwrap();
            let by_start: HashMap<i64, _> = presents.iter().map(|present| (present.start, present)).collect();
            for row in rows.iter().filter(|row| row.process == process) {
                let place = format!("test_case_{case}, {} {process}, present at {}", row.application, row.start);
                let Some(present) = by_start.get(&row.start) else {
                    mismatches.push(format!("{place}: missing, PresentMon has {}", row.mode));
                    continue;
                };
                let until_displayed =
                    present.screen.map(|screen| (screen - present.start) as f64 * 1000.0 / frequency as f64);
                let mode = presentmon_name(present.mode);
                let times_same = match (until_displayed, row.until_displayed) {
                    (Some(ours), Some(theirs)) => (ours - theirs).abs() < 1e-6,
                    (None, None) => true,
                    _ => false,
                };
                if mode != row.mode || present.tearing != row.tearing || !times_same {
                    mismatches.push(format!(
                        "{place}: {mode}, tearing {}, {until_displayed:?} ms, but PresentMon has {}, tearing {}, {:?} ms",
                        present.tearing, row.mode, row.tearing, row.until_displayed
                    ));
                } else {
                    *same.entry(row.mode.clone()).or_default() += 1;
                }
            }
        }
    }
    for (mode, count) in &same {
        println!("{count:5} the same: {mode}");
    }
    for mismatch in &mismatches {
        println!("{mismatch}");
    }
    assert!(mismatches.is_empty(), "{} presents differ from PresentMon's", mismatches.len());
}
