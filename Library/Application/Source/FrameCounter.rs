// Counts the frames drawn each second. It's timed with the frame's time from
// Window::begin_frame, which is the system clock, or a script's fixed clock,
// so that a scripted run shows the same rate every time.

#[derive(Default)]
pub struct FrameCounter {
    // The time, in seconds, of the frame that the current second started at.
    window_start: Option<f64>,
    // Frames since window_start.
    count: u32,
    // The frame rate over the last whole second, once there's been one.
    pub per_second: Option<f64>,
}

impl FrameCounter {
    // Call once per frame, with the frame's time in seconds.
    pub fn frame(&mut self, seconds: f64) {
        let Some(start) = self.window_start else {
            self.window_start = Some(seconds);
            return;
        };
        self.count += 1;
        let elapsed = seconds - start;
        if elapsed >= 1.0 {
            // The second ends at the first frame after it, so it runs a
            // little long. Dividing by its real length corrects for that.
            self.per_second = Some(self.count as f64 / elapsed);
            self.count = 0;
            self.window_start = Some(seconds);
        }
    }

    // Starts the count over after a stretch with no frames, like while the
    // window is minimized, so the stretch doesn't lower the next rate.
    pub fn pause(&mut self) {
        self.window_start = None;
        self.count = 0;
    }
}
