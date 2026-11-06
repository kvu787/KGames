// Tests the input latency that the PerfBar shows: from when
// GameInput got a frame's newest input to when the display took the frame,
// with its parts, and how frames that never reach the screen pass their input
// on, as PresentMon's input latency does. The presents' fates are made up,
// on the performance counter's clock, near now.

use present_timing::{Present, PresentMode};
use user_interface::clock;
use user_interface::performance::{Frame, LatencyState, Performance};

fn ticks(milliseconds: f64) -> i64 {
    (milliseconds * clock::frequency() as f64 / 1000.0) as i64
}

fn close(value: f64, expected: f64) {
    assert!((value - expected).abs() < 0.001, "{value} isn't {expected}");
}

// The present of the frame that called Present at a time, which DXGI's event
// times a little after, and when the display took it, if it did.
fn present(called: i64, screen: Option<i64>) -> Present {
    Present {
        start: called + 5,
        thread: 1,
        mode: PresentMode::IndependentFlip,
        tearing: true,
        screen,
        flip_delay: 0,
    }
}

#[test]
fn a_frame_s_latency_runs_from_its_input_to_the_screen_in_three_parts() {
    let base = clock::now();
    let mut performance = Performance::given();
    let frame = Frame {
        input: Some(base),
        read: base + ticks(1.0),
        present: base + ticks(3.0),
    };
    performance.presented(frame);
    performance.known(vec![present(frame.present, Some(base + ticks(10.0)))]);
    performance.update(0.0);
    let summary = performance.summary();
    assert_eq!(summary.latency, LatencyState::Measuring);
    assert_eq!(summary.samples, 1);
    close(summary.median, 10.0);
    close(summary.waiting, 1.0);
    close(summary.building, 2.0);
    close(summary.presenting, 7.0);
    assert_eq!(summary.mode, PresentMode::IndependentFlip);
    assert!(summary.tearing);
}

#[test]
fn a_frame_that_never_reaches_the_screen_passes_its_input_to_the_next_that_does() {
    let base = clock::now();
    let mut performance = Performance::given();
    let dropped = Frame {
        input: Some(base),
        read: base + ticks(1.0),
        present: base + ticks(2.0),
    };
    let shown = Frame {
        input: None,
        read: base + ticks(3.0),
        present: base + ticks(4.0),
    };
    performance.presented(dropped);
    performance.presented(shown);
    performance.known(vec![present(dropped.present, None), present(shown.present, Some(base + ticks(9.0)))]);
    performance.update(0.0);
    let summary = performance.summary();
    assert_eq!(summary.samples, 1);
    close(summary.median, 9.0);
    close(summary.presenting, 7.0);
}

#[test]
fn a_frame_on_the_screen_with_input_of_its_own_takes_only_its_own() {
    let base = clock::now();
    let mut performance = Performance::given();
    let dropped = Frame {
        input: Some(base),
        read: base + ticks(1.0),
        present: base + ticks(2.0),
    };
    let shown = Frame {
        input: Some(base + ticks(2.5)),
        read: base + ticks(3.0),
        present: base + ticks(4.0),
    };
    performance.presented(dropped);
    performance.presented(shown);
    performance.known(vec![present(dropped.present, None), present(shown.present, Some(base + ticks(9.0)))]);
    performance.update(0.0);
    assert_eq!(performance.summary().samples, 1);
    close(performance.summary().median, 6.5);
}

#[test]
fn frames_without_input_have_no_latency_but_count_on_the_screen() {
    let base = clock::now();
    let mut performance = Performance::given();
    let mut presents = Vec::new();
    for index in 0..11 {
        let called = base + ticks(index as f64 * 10.0);
        performance.presented(Frame {
            input: None,
            read: called - ticks(1.0),
            present: called,
        });
        presents.push(present(called, Some(called + ticks(5.0))));
    }
    performance.known(presents);
    performance.update(0.0);
    let summary = performance.summary();
    assert_eq!(summary.samples, 0);
    close(summary.displayed_per_second, 100.0);
}

#[test]
fn presents_that_aren_t_the_main_loop_s_are_left_alone() {
    let base = clock::now();
    let mut performance = Performance::given();
    let frame = Frame {
        input: Some(base),
        read: base + ticks(1.0),
        present: base + ticks(3.0),
    };
    performance.presented(frame);
    // A present that started before the frame's Present call.
    performance.known(vec![present(base, Some(base + ticks(4.0)))]);
    performance.update(0.0);
    assert_eq!(performance.summary().latency, LatencyState::Starting);
    performance.known(vec![present(frame.present, Some(base + ticks(10.0)))]);
    performance.update(1.0);
    assert_eq!(performance.summary().samples, 1);
}

#[test]
fn the_median_and_99th_percentile_are_of_the_last_two_seconds() {
    let base = clock::now() - ticks(1000.0);
    let mut performance = Performance::given();
    let mut presents = Vec::new();
    for index in 0..100 {
        let input = base + ticks(index as f64 * 5.0);
        let frame = Frame {
            input: Some(input),
            read: input,
            present: input,
        };
        performance.presented(frame);
        presents.push(present(frame.present, Some(input + ticks(index as f64 + 1.0))));
    }
    performance.known(presents);
    performance.update(0.0);
    let summary = performance.summary();
    assert_eq!(summary.samples, 100);
    close(summary.median, 50.5);
    close(summary.percentile_99, 99.01);
}

#[test]
fn without_presents_there_is_no_latency_and_it_says_why() {
    let mut performance = Performance::without_latency("none in a test");
    performance.frame_started(0.0);
    performance.frame_started(0.01);
    performance.update(0.01);
    let summary = performance.summary();
    assert_eq!(summary.latency, LatencyState::Unavailable("none in a test".to_string()));
    close(summary.frames_per_second, 100.0);
}
