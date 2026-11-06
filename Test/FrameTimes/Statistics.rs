// FrameTimes' statistics, over made-up presents: frame times are the
// differences between the screen times of the presents that reached the
// screen, and presents that didn't reach it don't count.

use frame_times::{
    Summary, frame_times, maximum, mean, median, most_common_mode, percentile, report, stutters, summarize,
};
use present_timing::{Present, PresentMode};

// The performance counter's rate on Windows 10 and later.
const FREQUENCY: i64 = 10_000_000;
const MILLISECOND: i64 = FREQUENCY / 1000;

// A present that reached the screen at `screen` milliseconds.
fn shown(screen: i64, mode: PresentMode) -> Present {
    Present {
        start: (screen - 5) * MILLISECOND,
        thread: 1,
        mode,
        tearing: false,
        screen: Some(screen * MILLISECOND),
        flip_delay: 0,
    }
}

// A present that started at `start` milliseconds and never reached the
// screen, like one that a newer one replaced.
fn missed(start: i64, mode: PresentMode) -> Present {
    Present {
        start: start * MILLISECOND,
        thread: 1,
        mode,
        tearing: false,
        screen: None,
        flip_delay: 0,
    }
}

// Presents that reached the screen after each of these frame times, in
// milliseconds, from 0.
fn frames(times: &[i64]) -> Vec<Present> {
    let mut presents = vec![shown(0, PresentMode::IndependentFlip)];
    let mut screen = 0;
    for time in times {
        screen += time;
        presents.push(shown(screen, PresentMode::IndependentFlip));
    }
    presents
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} isn't {expected}");
}

fn assert_all_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len(), "{actual:?} isn't {expected:?}");
    for (actual, expected) in actual.iter().zip(expected) {
        assert_close(*actual, *expected);
    }
}

#[test]
fn no_presents_have_no_statistics() {
    assert!(frame_times(&[], FREQUENCY).is_empty());
    assert_eq!(summarize(&[], FREQUENCY), None);
    assert_eq!(most_common_mode(&[]), None);
    assert_eq!(mean(&[]), None);
    assert_eq!(median(&[]), None);
    assert_eq!(percentile(&[], 99.0), None);
    assert_eq!(maximum(&[]), None);
    assert_eq!(stutters(&[]), 0);
}

#[test]
fn frame_times_are_the_differences_between_screen_times() {
    let presents = [
        shown(0, PresentMode::IndependentFlip),
        shown(16, PresentMode::IndependentFlip),
        shown(33, PresentMode::IndependentFlip),
        shown(50, PresentMode::IndependentFlip),
    ];
    assert_all_close(&frame_times(&presents, FREQUENCY), &[16.0, 17.0, 17.0]);
    // On another counter rate, the same ticks are other times.
    assert_all_close(&frame_times(&presents, 2 * FREQUENCY), &[8.0, 8.5, 8.5]);
}

#[test]
fn frame_times_follow_the_screen_times_order() {
    let presents = [
        shown(20, PresentMode::Composed),
        shown(0, PresentMode::Composed),
        shown(10, PresentMode::Composed),
    ];
    assert_all_close(&frame_times(&presents, FREQUENCY), &[10.0, 10.0]);
}

#[test]
fn presents_that_never_reached_the_screen_dont_count() {
    let presents = [
        missed(-10, PresentMode::Composed),
        shown(0, PresentMode::IndependentFlip),
        missed(3, PresentMode::Composed),
        missed(4, PresentMode::Composed),
        shown(10, PresentMode::IndependentFlip),
        missed(13, PresentMode::Composed),
        shown(20, PresentMode::IndependentFlip),
    ];
    assert_all_close(&frame_times(&presents, FREQUENCY), &[10.0, 10.0]);
    // Missed presents' modes don't count either, though there are more of
    // them.
    assert_eq!(most_common_mode(&presents), Some(PresentMode::IndependentFlip));
    let summary = summarize(&presents, FREQUENCY).unwrap();
    assert_eq!(summary.frames, 2);
    assert_close(summary.mean, 10.0);
    assert_eq!(summary.mode, PresentMode::IndependentFlip);
}

#[test]
fn fewer_than_two_presents_on_the_screen_have_no_frame_time() {
    let none = [missed(0, PresentMode::Composed), missed(10, PresentMode::Composed)];
    assert!(frame_times(&none, FREQUENCY).is_empty());
    assert_eq!(summarize(&none, FREQUENCY), None);
    assert_eq!(most_common_mode(&none), None);

    let one = [missed(0, PresentMode::Composed), shown(10, PresentMode::Composed), missed(20, PresentMode::Composed)];
    assert!(frame_times(&one, FREQUENCY).is_empty());
    assert_eq!(summarize(&one, FREQUENCY), None);
    assert_eq!(most_common_mode(&one), Some(PresentMode::Composed));
}

#[test]
fn steady_frames_with_two_hitches() {
    // 98 frames of 10 ms, then one of 30 ms and one of 50 ms.
    let mut times = vec![10; 98];
    times.extend([30, 50]);
    let summary = summarize(&frames(&times), FREQUENCY).unwrap();
    assert_eq!(summary.frames, 100);
    assert_close(summary.mean, 10.6);
    assert_close(summary.frames_per_second, 1000.0 / 10.6);
    assert_close(summary.median, 10.0);
    // The 99th of 100 sorted frame times.
    assert_close(summary.percentile_99, 30.0);
    assert_close(summary.maximum, 50.0);
    assert_eq!(summary.stutters, 2);
    assert_eq!(summary.mode, PresentMode::IndependentFlip);
}

#[test]
fn median_of_an_even_count_is_halfway_between_the_middle_two() {
    assert_eq!(median(&[3.0, 1.0, 2.0]), Some(2.0));
    assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), Some(2.5));
    assert_eq!(median(&[7.0]), Some(7.0));
}

#[test]
fn percentile_is_the_nearest_rank() {
    let values: Vec<f64> = (1..=10).rev().map(f64::from).collect();
    assert_eq!(percentile(&values, 99.0), Some(10.0));
    assert_eq!(percentile(&values, 90.0), Some(9.0));
    assert_eq!(percentile(&values, 50.0), Some(5.0));
    assert_eq!(percentile(&values, 100.0), Some(10.0));
    assert_eq!(percentile(&values, 0.0), Some(1.0));
    assert_eq!(percentile(&[7.0], 99.0), Some(7.0));
    assert_eq!(maximum(&values), Some(10.0));
}

#[test]
fn stutters_are_frames_over_twice_the_median() {
    // The median is 10 ms: 20 ms is twice it, which isn't over.
    assert_eq!(stutters(&[10.0, 10.0, 10.0, 20.0, 21.0]), 1);
    assert_eq!(stutters(&[10.0; 5]), 0);
}

#[test]
fn the_mode_seen_most_wins_and_a_tie_goes_to_the_first() {
    let mostly_flips = [
        shown(0, PresentMode::Composed),
        shown(10, PresentMode::IndependentFlip),
        shown(20, PresentMode::IndependentFlip),
        shown(30, PresentMode::Composed),
        shown(40, PresentMode::IndependentFlip),
    ];
    assert_eq!(most_common_mode(&mostly_flips), Some(PresentMode::IndependentFlip));

    let tie = [
        shown(0, PresentMode::OverlayPlane),
        shown(10, PresentMode::Composed),
        shown(20, PresentMode::Composed),
        shown(30, PresentMode::OverlayPlane),
    ];
    assert_eq!(most_common_mode(&tie), Some(PresentMode::OverlayPlane));
}

#[test]
fn frames_on_the_screen_at_once_have_no_rate() {
    // PresentTiming's correction for NVIDIA's flip delay can put a present
    // on the screen with the one before it.
    let summary = summarize(&[shown(10, PresentMode::Composed), shown(10, PresentMode::Composed)], FREQUENCY).unwrap();
    assert_eq!(summary.frames, 1);
    assert_close(summary.mean, 0.0);
    assert_close(summary.frames_per_second, 0.0);
}

#[test]
fn the_report_has_a_line_for_each_statistic() {
    let summary = Summary {
        frames: 100,
        frames_per_second: 1000.0 / 10.6,
        mean: 10.6,
        median: 10.0,
        percentile_99: 30.0,
        maximum: 50.0,
        stutters: 2,
        mode: PresentMode::IndependentFlip,
    };
    let text = report(&summary);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        lines,
        [
            "  Frames           100",
            "  Frames a second  94.3",
            "  Mean             10.60 ms",
            "  Median           10.00 ms",
            "  99th percentile  30.00 ms",
            "  Maximum          50.00 ms",
            "  Stutters         2, frames over twice the median (20.00 ms)",
            "  Present mode     Independent flip",
        ]
    );
}
