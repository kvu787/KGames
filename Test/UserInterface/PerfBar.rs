// Tests what the PerfBar says: every label and value stays in the same
// columns whatever the stats are, starting, without input latency, or
// measuring it, with small numbers or large, in every present mode, with
// immediate flips or not, and numbers that aren't known yet are dashes.

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
    measuring_in(PresentMode::IndependentFlip, true, apart, latency, with_input)
}

// The same, with the frames reaching the screen in a mode, with immediate
// flips or at the display's refresh.
fn measuring_in(mode: PresentMode, immediate: bool, apart: f64, latency: f64, with_input: bool) -> Summary {
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
            mode,
            tearing: immediate,
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
        measuring_in(PresentMode::ComposedFlip, false, 0.01, 10.0, true),
    ];
    let first = &summaries[0];
    for summary in &summaries {
        assert_eq!(columns(&summary.stats), columns(&first.stats), "{:?}", summary.stats);
        assert_eq!(labels(&summary.stats), labels(&first.stats));
    }
}

#[test]
fn every_present_mode_shows_whole_in_the_same_columns() {
    let first = measuring_in(PresentMode::ALL[0], true, 0.01, 10.0, true);
    for mode in PresentMode::ALL {
        let summary = measuring_in(mode, true, 0.01, 10.0, true);
        assert_eq!(values(&summary.stats)[3].trim_end(), mode.name());
        assert_eq!(columns(&summary.stats), columns(&first.stats), "{:?}", summary.stats);
    }
}

#[test]
fn numbers_are_right_aligned_and_unknown_ones_are_dashes() {
    let summary = measuring(0.01, 10.0, true);
    // Frames a second, the latency's mean, the lookback, the mode and the
    // flip.
    assert_eq!(values(&summary.stats), ["  100.00", "  10.00", "1", "Independent flip ", "immediate "]);
    assert!(summary.note.is_empty());

    // Without input latency, all of its numbers, and the present mode, are
    // dashes, and the note says why.
    let summary = without_latency("none in a test");
    let stats = values(&summary.stats);
    for value in stats[..2].iter().chain(&stats[3..]) {
        assert_eq!(value.trim(), "-", "{value:?}");
    }
    assert_eq!(summary.note, [
        Run { text: " | Input latency = ".to_string(), value: false },
        Run { text: "none in a test".to_string(), value: true },
    ]);

    // Frames without input have no latency.
    let summary = measuring(0.01, 10.0, false);
    assert_eq!(values(&summary.stats)[..3], ["  100.00", "      -", "1"]);
}

#[test]
fn sections_are_separated_by_bars() {
    let summary = measuring(0.01, 10.0, true);
    let text = |runs: &[Run]| runs.iter().map(|run| run.text.as_str()).collect::<String>();
    assert_eq!(
        text(&summary.stats),
        "FPS =   100.00 | Input latency =   10.00 ms | Lookback = 1 s | Mode = Independent flip  | Flip = immediate "
    );
}

#[test]
fn the_flip_is_immediate_or_at_the_refresh() {
    let summary = measuring_in(PresentMode::IndependentFlip, true, 0.01, 10.0, true);
    assert!(summary.immediate);
    assert_eq!(values(&summary.stats)[4], "immediate ");
    let summary = measuring_in(PresentMode::ComposedFlip, false, 0.01, 10.0, true);
    assert!(!summary.immediate);
    assert_eq!(values(&summary.stats)[4], "at refresh");
}

#[test]
fn a_number_too_wide_for_its_field_fills_it() {
    let mut performance = Performance::without_latency("none in a test");
    performance.frame_started(0.0);
    performance.frame_started(0.000_001);
    performance.update(0.000_001);
    let summary = performance.summary();
    assert_eq!(values(&summary.stats)[..3], ["########", "      -", "1"]);
}
