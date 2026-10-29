// Counts the frames drawn each second. It's timed with the system clock
// rather than GameInput's timestamps, so it works without GameInput too.

use std::time::{Duration, Instant};

#[derive(Default)]
pub struct FrameCounter {
    // The frame that the current second started at.
    window_start: Option<Instant>,
    // Frames since window_start.
    count: u32,
    // The frame rate over the last whole second, once there's been one.
    pub per_second: Option<f64>,
}

impl FrameCounter {
    // Call once per frame.
    pub fn frame(&mut self) {
        let now = Instant::now();
        let Some(start) = self.window_start else {
            self.window_start = Some(now);
            return;
        };
        self.count += 1;
        let elapsed = now - start;
        if elapsed >= Duration::from_secs(1) {
            // The second ends at the first frame after it, so it runs a
            // little long. Dividing by its real length corrects for that.
            self.per_second = Some(self.count as f64 / elapsed.as_secs_f64());
            self.count = 0;
            self.window_start = Some(now);
        }
    }

    // Starts the count over after a stretch with no frames, like while the
    // window is minimized, so the stretch doesn't lower the next rate.
    pub fn pause(&mut self) {
        self.window_start = None;
        self.count = 0;
    }
}
