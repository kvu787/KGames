// Windows' performance counter (QPC), the one clock that input, frames and
// the display's flips are all timed on, so that the time between any two of
// them is a subtraction. ETW's events carry QPC values, and GameInput's
// readings carry microseconds that input_devices.rs turns into QPC values.

use std::sync::OnceLock;

use windows_api::{QueryPerformanceCounter, QueryPerformanceFrequency};

// The counter now.
pub fn now() -> i64 {
    let mut value = 0;
    unsafe {
        let _ = QueryPerformanceCounter(&mut value);
    }
    value
}

// Ticks a second, which never changes while Windows runs.
pub fn frequency() -> i64 {
    static FREQUENCY: OnceLock<i64> = OnceLock::new();
    *FREQUENCY.get_or_init(|| {
        let mut value = 0;
        unsafe {
            let _ = QueryPerformanceFrequency(&mut value);
        }
        value.max(1)
    })
}

// A span of ticks in milliseconds.
pub fn milliseconds(ticks: i64) -> f64 {
    ticks as f64 * 1000.0 / frequency() as f64
}

// A span of ticks in seconds.
pub fn seconds(ticks: i64) -> f64 {
    ticks as f64 / frequency() as f64
}

// Ticks in a span of microseconds.
pub fn ticks_from_microseconds(microseconds: i64) -> i64 {
    (microseconds as i128 * frequency() as i128 / 1_000_000) as i64
}
