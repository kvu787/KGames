// Times a body many times and keeps the statistics of its times: the
// minimum, which is what the work costs when nothing else is in the way,
// the median, the mean, the 99th percentile and the maximum, which say how
// steady it is. Each iteration is timed on its own, on the performance
// counter, so that a run under RunLightly, where the job's cap on CPU time
// stops the thread now and then, still finds the true cost in the minimum
// and the median, while the maximum shows the stops.

use std::time::Instant;

// How long a benchmark runs: until the seconds pass, or the iterations are
// done, whichever is first, after the warm-up iterations, which aren't kept.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Budget {
    pub seconds: f64,
    pub iterations: usize,
    pub warmup: usize,
}

impl Budget {
    // The default: up to half a second, or 2000 iterations.
    pub const DEFAULT: Budget = Budget { seconds: 0.5, iterations: 2000, warmup: 5 };
    // A fifth of the default, for a quick look.
    pub const QUICK: Budget = Budget { seconds: 0.1, iterations: 400, warmup: 2 };
    // One iteration, with no warm-up, for tests that check that a benchmark
    // runs at all.
    pub const ONCE: Budget = Budget { seconds: 0.0, iterations: 1, warmup: 0 };
}

// What a benchmark measured: its times in milliseconds, per unit of work,
// like a frame or a parse.
#[derive(Clone, Debug, PartialEq)]
pub struct Measurement {
    pub name: String,
    // What each time is of, like "frame" or "parse".
    pub unit: &'static str,
    pub iterations: usize,
    pub min: f64,
    pub median: f64,
    pub mean: f64,
    pub percentile_99: f64,
    pub max: f64,
    // What else the benchmark found out, like how many instances a frame
    // drew, for the table's last column.
    pub note: String,
}

impl Measurement {
    // The statistics of times in milliseconds, each of so many units.
    pub fn from_times(name: &str, unit: &'static str, times: &[f64], units: f64) -> Measurement {
        assert!(!times.is_empty(), "{name} was never timed");
        let mut sorted: Vec<f64> = times.iter().map(|time| time / units).collect();
        sorted.sort_by(f64::total_cmp);
        Measurement {
            name: name.to_string(),
            unit,
            iterations: sorted.len(),
            min: sorted[0],
            median: percentile(&sorted, 0.5),
            mean: sorted.iter().sum::<f64>() / sorted.len() as f64,
            percentile_99: percentile(&sorted, 0.99),
            max: sorted[sorted.len() - 1],
            note: String::new(),
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Measurement {
        self.note = note.into();
        self
    }

    pub fn to_json(&self) -> json::Value {
        json::object([
            ("Name", json::Value::from(self.name.as_str())),
            ("Unit", json::Value::from(self.unit)),
            ("Iterations", json::Value::from(self.iterations as f64)),
            ("MinMs", json::Value::from(self.min)),
            ("MedianMs", json::Value::from(self.median)),
            ("MeanMs", json::Value::from(self.mean)),
            ("Percentile99Ms", json::Value::from(self.percentile_99)),
            ("MaxMs", json::Value::from(self.max)),
            ("Note", json::Value::from(self.note.as_str())),
        ])
    }
}

// The value of sorted values at a fraction of the way through them,
// between the two nearest.
pub fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let position = fraction * (sorted.len() - 1) as f64;
    let below = position.floor() as usize;
    let above = position.ceil() as usize;
    let weight = position - below as f64;
    sorted[below] * (1.0 - weight) + sorted[above] * weight
}

// Times `body` within the budget, each call on its own. Each call does
// `units` units of work, like two frames, so that the times are per unit.
pub fn measure(name: &str, unit: &'static str, budget: Budget, units: f64, mut body: impl FnMut()) -> Measurement {
    for _ in 0..budget.warmup {
        body();
    }
    let mut times = Vec::new();
    let start = Instant::now();
    while times.len() < budget.iterations.max(1) {
        let before = Instant::now();
        body();
        times.push(before.elapsed().as_secs_f64() * 1000.0);
        if start.elapsed().as_secs_f64() >= budget.seconds && !times.is_empty() {
            break;
        }
    }
    Measurement::from_times(name, unit, &times, units)
}

// Like measure, for a body that times itself, like with the GPU's clock,
// and gives each call's milliseconds.
pub fn measure_timed(name: &str, unit: &'static str, budget: Budget, units: f64, mut body: impl FnMut() -> f64) -> Measurement {
    for _ in 0..budget.warmup {
        body();
    }
    let mut times = Vec::new();
    let start = Instant::now();
    while times.len() < budget.iterations.max(1) {
        times.push(body());
        if start.elapsed().as_secs_f64() >= budget.seconds && !times.is_empty() {
            break;
        }
    }
    Measurement::from_times(name, unit, &times, units)
}

// The measurements as a table that reads in a terminal: milliseconds with
// four decimals, right-aligned.
pub fn table(measurements: &[Measurement]) -> String {
    let name_width = measurements.iter().map(|m| m.name.len()).max().unwrap_or(4).max(9);
    let unit_width = measurements.iter().map(|m| m.unit.len()).max().unwrap_or(4).max(4);
    let mut text = format!(
        "{:<name_width$}  {:<unit_width$}  {:>10}  {:>10}  {:>10}  {:>10}  {:>10}  {:>10}  Note\n",
        "Benchmark", "Unit", "Iterations", "Min ms", "Median ms", "Mean ms", "99th ms", "Max ms"
    );
    for measurement in measurements {
        text += &format!(
            "{:<name_width$}  {:<unit_width$}  {:>10}  {:>10.4}  {:>10.4}  {:>10.4}  {:>10.4}  {:>10.4}  {}\n",
            measurement.name,
            measurement.unit,
            measurement.iterations,
            measurement.min,
            measurement.median,
            measurement.mean,
            measurement.percentile_99,
            measurement.max,
            measurement.note
        );
    }
    text
}
