// A program that reads the gamepad holds 1 ms timers while it reads input,
// because GameInput polls gamepads in Xbox 360 mode on a timer, which
// Windows' default resolution stretches to 15.6 ms: 64 readings a second
// instead of about 1,000 (see Library/UserInterface's
// platform/timer_resolution.rs). Cargo makes a test binary of each file in
// tests, and the resolution is the whole process's, so nothing else here
// changes it while this measures it.

use std::time::Instant;

use user_interface::{Options, TimerResolution};
use windows_api::{GetCurrentProcess, WaitForSingleObject};

// How long a 1 ms wait takes, in milliseconds, at the middle of 20. A wait
// with a timeout fires on the process's timers, unlike std::thread::sleep,
// which uses a high-resolution timer whatever the process's resolution.
fn millisecond_wait() -> f64 {
    let mut waits: Vec<f64> = (0..20)
        .map(|_| {
            let start = Instant::now();
            // The process is signaled only once it ends, so this times out.
            unsafe { WaitForSingleObject(GetCurrentProcess(), 1) };
            start.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    waits.sort_by(f64::total_cmp);
    waits[waits.len() / 2]
}

#[test]
fn programs_that_read_the_gamepad_hold_1_ms_timers() {
    // Windows' default, which a program that doesn't read the gamepad keeps.
    let without = TimerResolution::for_program(&Options::new("Test", "."));
    assert!(without.is_none());
    let wait = millisecond_wait();
    assert!(wait > 10.0, "a 1 ms wait took {wait:.2} ms, not the default 15.6 ms tick");

    let held = TimerResolution::for_program(&Options::new("Test", ".").gamepad());
    assert!(held.is_some(), "Windows refused 1 ms timers");
    let wait = millisecond_wait();
    assert!(wait < 5.0, "a 1 ms wait took {wait:.2} ms with 1 ms timers held");

    // Let go of, the process is back to the default.
    drop(held);
    let wait = millisecond_wait();
    assert!(wait > 10.0, "a 1 ms wait took {wait:.2} ms after letting go of 1 ms timers");
}
