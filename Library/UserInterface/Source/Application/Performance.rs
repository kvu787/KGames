// What the PerfBar along the top of every program's window measures:
// how fast frames come, how fast the display shows them, and the input
// latency, from when GameInput got the input that a frame took to when the
// display took that frame, all on the performance counter's clock:
//
//     input --- read ----------- present ------------- screen
//       waiting      building          presenting
//
// - input: the GameInput reading's time, when Windows got the device's report.
// - read: when the main loop read the frame's input, right before the frame.
// - present: right before the frame's Present call.
// - screen: when the display took the frame, from Windows' own events about
//   presents (Library/PresentTiming), as PresentMon times it, so that the
//   numbers compare with PresentMon's.
//
// A frame's input is the newest reading it took, as PresentMon's is the
// newest input that a frame took. A frame that never reaches the screen,
// like one that a newer frame replaced, passes its input on to the next frame
// that does, unless that one has input of its own.

use std::collections::VecDeque;

use present_timing::{Present, PresentMode, PresentTiming, StartError};

use crate::platform::clock;

// The stats are of this many seconds before now.
pub const WINDOW_SECONDS: f64 = 2.0;
// How often what the PerfBar says changes, so that it can be read.
const REFRESH_SECONDS: f64 = 0.1;
// How far before its Present call's start a frame's own present time can be,
// to be the same present.
const MATCH_SECONDS: f64 = 0.005;

// The PerfBar's fields' widths, in characters: frames a second and times in
// milliseconds, both with two decimals, and the present mode, as wide as the
// longest mode's name, "Independent flip".
const RATE_WIDTH: usize = 8;
const MILLISECONDS_WIDTH: usize = 7;
const MODE_WIDTH: usize = 16;
const YES_NO_WIDTH: usize = 3;
// The gap between the bar's groups, like the frames' and the input
// latency's.
pub(crate) const GAP: &str = "    ";

// A frame that the main loop presented, with the times of its input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    // When the frame's newest input reading came, if it took any.
    pub input: Option<i64>,
    pub read: i64,
    pub present: i64,
}

// One frame's input latency, in milliseconds, and its parts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    // When the display took the frame, as the counter's ticks.
    pub screen: i64,
    pub total: f64,
    pub waiting: f64,
    pub building: f64,
    pub presenting: f64,
}

// What the PerfBar says about input latency.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum LatencyState {
    // Before the first frame's present is known.
    #[default]
    Starting,
    // Why the presents' times aren't known, like in a scripted run.
    Unavailable(String),
    Measuring,
}

// What the PerfBar shows, which changes 10 times a second.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Summary {
    pub frames_per_second: f64,
    // Milliseconds, the mean and the longest.
    pub frame_time: f64,
    pub longest_frame: f64,
    pub latency: LatencyState,
    // Frames a second that reached the screen.
    pub displayed_per_second: f64,
    // The input latency's samples, their median and 99th percentile, and the
    // median of each part.
    pub samples: usize,
    pub median: f64,
    pub percentile_99: f64,
    pub waiting: f64,
    pub building: f64,
    pub presenting: f64,
    // How the newest frame on the screen got there.
    pub mode: PresentMode,
    pub tearing: bool,
    // What the PerfBar says, made here when the summary is, 10 times a
    // second, not every frame: the runs along its left, those along its
    // right, which the bar ends with whether vertical sync is on, and why
    // there's no input latency, if there isn't, between them.
    pub left: Vec<Run>,
    pub right: Vec<Run>,
    pub note: Vec<Run>,
}

// A run of the PerfBar's text, which the bar sets in Cascadia Mono, whose
// characters are all as wide: a label, or a value. Every run is as many
// characters wide whatever the stats are, numbers right-aligned in their
// fields and a dash for one that isn't known, so that every label and value
// stays in the same columns.
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub text: String,
    pub value: bool,
}

// Runs, one after another, with a label's words joined into one run.
#[derive(Default)]
struct Runs(Vec<Run>);

impl Runs {
    fn label(&mut self, text: &str) {
        match self.0.last_mut() {
            Some(run) if !run.value => run.text.push_str(text),
            _ => self.0.push(Run { text: text.to_string(), value: false }),
        }
    }

    fn value(&mut self, text: String) {
        self.0.push(Run { text, value: true });
    }
}

// A number with so many decimals, right-aligned in a field so many
// characters wide, or a dash where its last digit would go if it isn't
// known. One too wide for its field fills it with #, as a spreadsheet's
// cell does, instead of pushing what comes after it out of place.
fn number(value: Option<f64>, decimals: usize, width: usize) -> String {
    let text = match value {
        Some(value) => format!("{value:>width$.decimals$}"),
        None => format!("{:>width$}", "-"),
    };
    if text.len() > width { "#".repeat(width) } else { text }
}

// Words, left-aligned in a field so many characters wide, cut short if
// they're wider, or a dash if they aren't known.
fn words(value: Option<&str>, width: usize) -> String {
    format!("{:<width$.width$}", value.unwrap_or("-"))
}

impl Summary {
    // Words the PerfBar.
    fn word(&mut self) {
        let frames = self.frames_per_second > 0.0;
        let measuring = self.latency == LatencyState::Measuring;
        let latency = measuring && self.samples > 0;
        let rate = |value: f64, known: bool| number(known.then_some(value), 2, RATE_WIDTH);
        let time = |value: f64, known: bool| number(known.then_some(value), 2, MILLISECONDS_WIDTH);

        let mut left = Runs::default();
        left.label("FPS ");
        left.value(rate(self.frames_per_second, frames));
        left.label(GAP);
        left.label("Input to screen  median ");
        left.value(time(self.median, latency));
        left.label(" ms");
        self.left = left.0;

        let mut right = Runs::default();
        right.label("Mode ");
        right.value(words(measuring.then_some(self.mode.name()), MODE_WIDTH));
        right.label("  Tearing ");
        right.value(words(measuring.then_some(if self.tearing { "yes" } else { "no" }), YES_NO_WIDTH));
        self.right = right.0;

        let mut note = Runs::default();
        let reason = match &self.latency {
            LatencyState::Unavailable(reason) => Some(reason.as_str()),
            LatencyState::Starting => Some("starting"),
            LatencyState::Measuring => None,
        };
        if let Some(reason) = reason {
            note.label("Input to screen ");
            note.value(reason.to_string());
        }
        self.note = note.0;
    }
}

// Where the presents' fates come from.
enum Source {
    Timing(PresentTiming),
    // From `known`, like a test's.
    Given,
    // Nowhere, and why.
    Unavailable(String),
}

pub struct Performance {
    source: Source,
    // The frames' start times on the frames' clock, in seconds.
    frame_times: VecDeque<f64>,
    // Frames whose presents aren't known yet, oldest first.
    presented: VecDeque<Frame>,
    // The input of frames that didn't reach the screen, for the next one that
    // does.
    carried: Option<Frame>,
    samples: VecDeque<Sample>,
    displayed: VecDeque<i64>,
    mode: PresentMode,
    tearing: bool,
    known: bool,
    summary: Summary,
    refreshed: f64,
}

impl Performance {
    // Input latency for the process's presents, from an ETW session named
    // after the program.
    pub fn measure(name: &str) -> Performance {
        let source = match PresentTiming::start(name, std::process::id()) {
            Ok(timing) => Source::Timing(timing),
            Err(StartError::AccessDenied) => Source::Unavailable("needs the Performance Log Users group".to_string()),
            Err(StartError::Failed(message)) => Source::Unavailable(format!("unavailable: {message}")),
        };
        Performance::new(source)
    }

    // Without input latency, saying why, like in a scripted run.
    pub fn without_latency(reason: &str) -> Performance {
        Performance::new(Source::Unavailable(reason.to_string()))
    }

    // Input latency for presents that `known` gives, like a test's.
    pub fn given() -> Performance {
        Performance::new(Source::Given)
    }

    fn new(source: Source) -> Performance {
        Performance {
            source,
            frame_times: VecDeque::new(),
            presented: VecDeque::new(),
            carried: None,
            samples: VecDeque::new(),
            displayed: VecDeque::new(),
            mode: PresentMode::Unknown,
            tearing: false,
            known: false,
            summary: Summary::default(),
            refreshed: f64::NEG_INFINITY,
        }
    }

    // A frame starts, at a time on the frames' clock, in seconds.
    pub fn frame_started(&mut self, time: f64) {
        self.frame_times.push_back(time);
        while self.frame_times.front().is_some_and(|start| *start < time - 1.0) {
            self.frame_times.pop_front();
        }
    }

    // The main loop presented a frame.
    pub fn presented(&mut self, frame: Frame) {
        if !matches!(self.source, Source::Unavailable(_)) {
            self.presented.push_back(frame);
        }
    }

    // Presents whose fates are known, in the order they started.
    pub fn known(&mut self, presents: Vec<Present>) {
        for present in presents {
            self.present(present);
        }
    }

    // Takes what's known about presents, and refreshes what the PerfBar
    // says, 10 times a second, at a time on the frames' clock, in seconds.
    pub fn update(&mut self, time: f64) {
        if let Source::Timing(timing) = &mut self.source {
            let presents = timing.take();
            self.known(presents);
        }
        let now = clock::now();
        let window = (WINDOW_SECONDS * clock::frequency() as f64) as i64;
        while self.samples.front().is_some_and(|sample| sample.screen < now - window) {
            self.samples.pop_front();
        }
        // The frames on the screen, over the window before the newest one
        // known, as Windows' events about them come late.
        if let Some(newest) = self.displayed.back().copied() {
            while self.displayed.front().is_some_and(|screen| *screen < newest - window) {
                self.displayed.pop_front();
            }
        }
        // Frames whose presents never came, like if ETW lost an event.
        while self.presented.front().is_some_and(|frame| frame.present < now - window) {
            self.presented.pop_front();
        }
        if time - self.refreshed >= REFRESH_SECONDS || time < self.refreshed {
            self.refreshed = time;
            self.summary = self.summarize();
        }
    }

    pub fn summary(&self) -> &Summary {
        &self.summary
    }

    // A present whose fate is known: which frame it was, and that frame's
    // input latency, if it reached the screen.
    fn present(&mut self, present: Present) {
        let slack = (MATCH_SECONDS * clock::frequency() as f64) as i64;
        // Frames before this present that no present came for.
        while self.presented.front().is_some_and(|frame| frame.present < present.start - slack) {
            self.presented.pop_front();
        }
        let Some(frame) = self.presented.front().copied() else {
            return;
        };
        if frame.present > present.start {
            // Not one of the main loop's presents.
            return;
        }
        self.presented.pop_front();
        self.known = true;
        let Some(screen) = present.screen else {
            if frame.input.is_some() {
                self.carried = Some(frame);
            }
            return;
        };
        self.displayed.push_back(screen);
        self.mode = present.mode;
        self.tearing = present.tearing;
        let source = if frame.input.is_some() { Some(frame) } else { self.carried };
        self.carried = None;
        if let Some(source) = source
            && let Some(input) = source.input
        {
            self.samples.push_back(Sample {
                screen,
                total: clock::milliseconds(screen - input),
                waiting: clock::milliseconds(source.read - input),
                building: clock::milliseconds(source.present - source.read),
                presenting: clock::milliseconds(screen - source.present),
            });
        }
    }

    fn summarize(&self) -> Summary {
        let mut summary = Summary::default();
        if let (Some(first), Some(last)) = (self.frame_times.front(), self.frame_times.back()) {
            let count = self.frame_times.len() - 1;
            if count > 0 && last > first {
                summary.frames_per_second = count as f64 / (last - first);
                summary.frame_time = (last - first) * 1000.0 / count as f64;
                summary.longest_frame = self
                    .frame_times
                    .iter()
                    .zip(self.frame_times.iter().skip(1))
                    .map(|(before, after)| (after - before) * 1000.0)
                    .fold(0.0, f64::max);
            }
        }
        summary.latency = match &self.source {
            Source::Unavailable(reason) => LatencyState::Unavailable(reason.clone()),
            _ if !self.known => LatencyState::Starting,
            _ => LatencyState::Measuring,
        };
        if let (Some(first), Some(last)) = (self.displayed.front(), self.displayed.back())
            && last > first
        {
            summary.displayed_per_second = (self.displayed.len() - 1) as f64 / clock::seconds(last - first);
        }
        summary.mode = self.mode;
        summary.tearing = self.tearing;
        summary.samples = self.samples.len();
        if !self.samples.is_empty() {
            let mut totals: Vec<f64> = self.samples.iter().map(|sample| sample.total).collect();
            summary.median = median(&mut totals);
            summary.percentile_99 = percentile(&totals, 0.99);
            summary.waiting = median(&mut self.samples.iter().map(|sample| sample.waiting).collect::<Vec<_>>());
            summary.building = median(&mut self.samples.iter().map(|sample| sample.building).collect::<Vec<_>>());
            summary.presenting = median(&mut self.samples.iter().map(|sample| sample.presenting).collect::<Vec<_>>());
        }
        summary.word();
        summary
    }
}

// Sorts the values, and gives their median.
fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    percentile(values, 0.5)
}

// The value of sorted values at a fraction of the way through them, between
// the two nearest.
fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let position = fraction * (sorted.len() - 1) as f64;
    let below = position.floor() as usize;
    let above = position.ceil() as usize;
    let weight = position - below as f64;
    sorted[below] * (1.0 - weight) + sorted[above] * weight
}
