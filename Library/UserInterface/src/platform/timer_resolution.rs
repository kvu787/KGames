// The resolution of the process's timers. Windows fires a process's timers,
// like a 1 ms wait's, on its clock's 15.6 ms tick, unless that process asks
// for finer ones with timeBeginPeriod. Another process's request doesn't
// change this one's.
//
// GameInput reads a gamepad in Xbox 360 mode, which Windows' XUSB driver
// runs, by polling it on a 1 ms timer, which the tick stretches. On the
// development machine, with GameInput 4.0.306, a GameSir controller in Xbox
// 360 mode, which reports 8,000 times a second, gave the only process reading
// GameInput 64 readings a second, 15.6 ms apart, and 950 a second, 1 ms
// apart, once that process had 1 ms timers. 0.5 ms ones gave no more. A
// second process starting to read GameInput dropped the first back to 64 a
// second, whatever its timers. So a program that reads the gamepad holds 1 ms
// timers while it reads input (see application/run.rs), so that a stick
// doesn't move in 15.6 ms steps on a screen that refreshes 144 times a
// second.
//
// Finer timers cost power, as the processor wakes more often, so programs
// that don't read the gamepad keep Windows' default.

use windows_api::{TIMERR_NOERROR, timeBeginPeriod, timeEndPeriod};

use crate::application::program::Options;

// 1 ms timers for the whole process, while it's held.
pub struct TimerResolution(());

impl TimerResolution {
    // 1 ms timers, or None if Windows refuses them.
    pub fn millisecond() -> Option<TimerResolution> {
        let result = unsafe { timeBeginPeriod(1) };
        (result.0 == TIMERR_NOERROR as u32).then_some(TimerResolution(()))
    }

    // What a program holds while it reads input with GameInput: 1 ms timers
    // if it reads the gamepad, and else Windows' default.
    pub fn for_program(options: &Options) -> Option<TimerResolution> {
        if options.gamepad { TimerResolution::millisecond() } else { None }
    }
}

impl Drop for TimerResolution {
    fn drop(&mut self) {
        unsafe {
            let _ = timeEndPeriod(1);
        }
    }
}
