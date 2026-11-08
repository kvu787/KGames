// WinUI's ProgressBar and ProgressRing: how far something has come, or that
// it's going on for a time no one knows, with their indeterminate
// animations' timing. Those move because their motion is what says that the
// work goes on and the program isn't frozen; apart from a text field's
// blinking caret, nothing else in the library animates. A determinate one
// shows its value as it is, without easing to it.

use crate::core::tree::{Draw, Widget};
use crate::design::controls::{
    PROGRESS_BAR_CURVE, PROGRESS_BAR_HEIGHT, PROGRESS_BAR_RADIUS, PROGRESS_BAR_TRACK_HEIGHT, PROGRESS_BAR_TRACK_RADIUS, PROGRESS_RING_CURVE,
    PROGRESS_RING_RADIUS, PROGRESS_RING_SIZE, PROGRESS_RING_STROKE,
};
use crate::drawing::geometry::Rect;

// A progress bar across its rectangle's middle: `Some` fraction from 0 to 1,
// or None while how long is unknown.
pub struct ProgressBar {
    value: Option<f32>,
}

impl ProgressBar {
    pub fn new(value: Option<f32>) -> Self {
        Self { value }
    }
}

impl<M: 'static> Widget<M> for ProgressBar {
    fn kind(&self) -> &'static str {
        "ProgressBar"
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let time = ctx.time();
        let mut painter = ctx.painter();
        let theme = painter.theme();
        let middle = rect.center().y;
        let bar = Rect::new(rect.x, (middle - PROGRESS_BAR_HEIGHT / 2.0).round(), rect.width, PROGRESS_BAR_HEIGHT);
        match self.value {
            Some(value) => {
                let track = Rect::new(rect.x, (middle - PROGRESS_BAR_TRACK_HEIGHT / 2.0).round(), rect.width, PROGRESS_BAR_TRACK_HEIGHT);
                painter.rounded(track, PROGRESS_BAR_TRACK_RADIUS, theme.control_strong_stroke);
                let width = rect.width * value.clamp(0.0, 1.0);
                if width > 0.0 {
                    painter.rounded(Rect::new(bar.x, bar.y, width, bar.height), PROGRESS_BAR_RADIUS, theme.accent);
                }
            }
            None => {
                // Two indicators chase each other across, every 2 seconds:
                // one 0.4 wide from -0.4 to 1.2 over 1.5 seconds, and one 0.6
                // wide from -0.9 to 0.996, from 0.75 seconds to 2.
                let cycle = (time % 2.0) as f32;
                let width = rect.width;
                let first = -0.4 + 1.6 * PROGRESS_BAR_CURVE.ease((cycle / 1.5).min(1.0));
                let second = if cycle < 0.75 { -0.9 } else { -0.9 + 1.896 * PROGRESS_BAR_CURVE.ease((cycle - 0.75) / 1.25) };
                painter.push_clip(Rect::new(rect.x, bar.y, rect.width, bar.height));
                for (offset, length) in [(first, 0.4), (second, 0.6)] {
                    let indicator = Rect::new(rect.x + width * offset, bar.y, width * length, bar.height);
                    painter.rounded(indicator, PROGRESS_BAR_RADIUS, theme.accent);
                }
                painter.pop_clip();
            }
        }
    }
}

// A progress ring, centered in its rectangle: `Some` fraction, or None
// while how long is unknown.
pub struct ProgressRing {
    value: Option<f32>,
    size: f32,
}

impl ProgressRing {
    pub fn new(value: Option<f32>) -> Self {
        Self {
            value,
            size: PROGRESS_RING_SIZE,
        }
    }

    // Its diameter.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl<M: 'static> Widget<M> for ProgressRing {
    fn kind(&self) -> &'static str {
        "ProgressRing"
    }

    fn draw(&self, ctx: &mut Draw) {
        let center = ctx.rect().center();
        let time = ctx.time();
        let radius = self.size * PROGRESS_RING_RADIUS;
        let stroke = self.size * PROGRESS_RING_STROKE;
        let mut painter = ctx.painter();
        let theme = painter.theme();
        match self.value {
            Some(value) => {
                // Its track is ProgressRingBackgroundThemeBrush, which is
                // transparent; the arc is all that shows.
                painter.arc(center, radius, stroke, 0.0, std::f32::consts::TAU * value.clamp(0.0, 1.0), theme.accent);
            }
            None => {
                // Over 2 seconds the ring turns 900 degrees; in the first
                // second its head runs half way round, and in the next its
                // tail catches up.
                let cycle = (time % 2.0) as f32;
                let (half, t) = if cycle < 1.0 { (0, cycle) } else { (1, cycle - 1.0) };
                let eased = PROGRESS_RING_CURVE.ease(t);
                let rotation = (450.0 * (half as f32 + eased)).to_radians();
                let (start, end) = if half == 0 { (0.0, 0.0001 + 0.4999 * eased) } else { (0.5 * eased, 0.5) };
                let tau = std::f32::consts::TAU;
                painter.arc(center, radius, stroke, rotation + start * tau, (end - start).max(0.0001) * tau, theme.accent);
            }
        }
    }
}
