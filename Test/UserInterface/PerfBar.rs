// Tests what the PerfBar says: every label and value stays in the same
// columns whatever the stats are, starting, without input latency, or
// measuring it, with small numbers or large, and numbers that aren't known
// yet are dashes.

use present_timing::{Present, PresentMode};
use user_interface::clock;
use user_interface::performance::{Frame, Performance, Run, Summary};

fn ticks(milliseconds: f64) -> i64 {
    (milliseconds * clock::frequency() as f64 / 1000.0) as i64
}

// A frame a time apart from the last, for a second.
fn frames(performance: &mut Performance, apart: f64) {
    let count = (1.0 / apart) as usize;
    for index in 0..=count {
        performance.frame_started(index as f64 * apart);
    }
}

// Stats that measure input latency, from frames whose inputs reach the
// screen so many milliseconds later, a millisecond apart, near now.
fn measuring(apart: f64, latency: f64, with_input: bool) -> Summary {
    let base = clock::now() - ticks(500.0);
    let mut performance = Performance::given();
    frames(&mut performance, apart);
    let mut presents = Vec::new();
    for index in 0..20 {
        let input = base + ticks(index as f64);
        let frame = Frame {
            input: with_input.then_some(input),
            read: input,
            present: input,
        };
        performance.presented(frame);
        presents.push(Present {
            start: frame.present + 5,
            thread: 1,
            mode: PresentMode::IndependentFlip,
            tearing: true,
            screen: Some(input + ticks(latency)),
            flip_delay: 0,
        });
    }
    performance.known(presents);
    performance.update(1.0);
    performance.summary().clone()
}

fn starting() -> Summary {
    let mut performance = Performance::given();
    frames(&mut performance, 0.01);
    performance.update(1.0);
    performance.summary().clone()
}

fn without_latency(reason: &str) -> Summary {
    let mut performance = Performance::without_latency(reason);
    performance.update(0.0);
    performance.summary().clone()
}

// Each run's first column, its width, and whether it's a value.
fn columns(runs: &[Run]) -> Vec<(usize, usize, bool)> {
    let mut column = 0;
    runs.iter()
        .map(|run| {
            let width = run.text.chars().count();
            column += width;
            (column - width, width, run.value)
        })
        .collect()
}

fn labels(runs: &[Run]) -> Vec<&str> {
    runs.iter().filter(|run| !run.value).map(|run| run.text.as_str()).collect()
}

fn values(runs: &[Run]) -> Vec<&str> {
    runs.iter().filter(|run| run.value).map(|run| run.text.as_str()).collect()
}

#[test]
fn every_label_and_value_stays_in_its_columns() {
    let summaries = [
        without_latency("none in a test"),
        without_latency("needs the Performance Log Users group"),
        starting(),
        measuring(0.01, 10.0, true),
        measuring(0.0004, 5.0, true),
        measuring(0.25, 999.0, true),
        measuring(0.01, 10.0, false),
    ];
    let first = &summaries[0];
    for summary in &summaries {
        assert_eq!(columns(&summary.left), columns(&first.left), "{:?}", summary.left);
        assert_eq!(labels(&summary.left), labels(&first.left));
        assert_eq!(columns(&summary.right), columns(&first.right), "{:?}", summary.right);
        assert_eq!(labels(&summary.right), labels(&first.right));
    }
}

#[test]
fn numbers_are_right_aligned_and_unknown_ones_are_dashes() {
    let summary = measuring(0.01, 10.0, true);
    // Frames a second, and the latency's median.
    assert_eq!(values(&summary.left), ["  100.00", "  10.00"]);
    assert_eq!(values(&summary.right), ["Independent flip", "yes"]);
    assert!(summary.note.is_empty());

    // Without input latency, all of its numbers, and the present mode, are
    // dashes, and the note says why.
    let summary = without_latency("none in a test");
    for value in values(&summary.left).into_iter().chain(values(&summary.right)) {
        assert_eq!(value.trim(), "-", "{value:?}");
    }
    assert_eq!(summary.note, [
        Run { text: "Input to screen ".to_string(), value: false },
        Run { text: "none in a test".to_string(), value: true },
    ]);

    // Frames without input have no latency.
    let summary = measuring(0.01, 10.0, false);
    assert_eq!(values(&summary.left), ["  100.00", "      -"]);
}

#[test]
fn a_number_too_wide_for_its_field_fills_it() {
    let mut performance = Performance::without_latency("none in a test");
    performance.frame_started(0.0);
    performance.frame_started(0.000_001);
    performance.update(0.000_001);
    let summary = performance.summary();
    assert_eq!(values(&summary.left), ["########", "      -"]);
}
