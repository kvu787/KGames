// When each of a process's presents reached the screen, from Windows' own
// events about them, which a real-time ETW session reads as they happen, by
// the rules that PresentMon follows, so that the times are the same as its.
// See README.md.

#[path = "Decoder.rs"]
mod decoder;
#[path = "Events.rs"]
pub mod events;
#[path = "Session.rs"]
mod session;
#[path = "Tracker.rs"]
pub mod tracker;

use std::sync::{Arc, Mutex};

use windows_api::QueryPerformanceFrequency;

use crate::decoder::Decoder;
use crate::session::Session;
use crate::tracker::Tracker;

// How a present got to the screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PresentMode {
    #[default]
    Unknown,
    // The display took the swap chain's buffer itself, like in borderless
    // fullscreen.
    IndependentFlip,
    // The same, on one of the display's overlay planes, over other windows.
    OverlayPlane,
    // The desktop window manager drew it into its own frame.
    Composed,
}

impl PresentMode {
    pub fn name(self) -> &'static str {
        match self {
            PresentMode::Unknown => "Unknown",
            PresentMode::IndependentFlip => "Independent flip",
            PresentMode::OverlayPlane => "Overlay plane",
            PresentMode::Composed => "Composed",
        }
    }
}

// A present, once it's known whether it reached the screen, and when.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Present {
    // When the Present call started, as the performance counter's ticks.
    pub start: i64,
    // The thread that called Present.
    pub thread: u32,
    pub mode: PresentMode,
    // Whether its flip could tear, reaching the screen mid-refresh.
    pub tearing: bool,
    // When the display took it, as the performance counter's ticks, or None
    // if it never reached the screen, like when a newer one replaced it.
    pub screen: Option<i64>,
    // How much later than the kernel's flip NVIDIA's driver said it would
    // flip, which the screen time has in it, in ticks.
    pub flip_delay: i64,
}

// Why presents' times aren't known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartError {
    // Reading ETW's events needs the Performance Log Users group, or an
    // elevated administrator.
    AccessDenied,
    Failed(String),
}

impl std::fmt::Display for StartError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StartError::AccessDenied => write!(formatter, "reading ETW's events needs the Performance Log Users group"),
            StartError::Failed(message) => write!(formatter, "{message}"),
        }
    }
}

// Reads the presents of one process, until it's dropped.
pub struct PresentTiming {
    _session: Session,
    presents: Arc<Mutex<Vec<Present>>>,
    corrector: Corrector,
}

impl PresentTiming {
    // Starts reading the process's presents, in an ETW session named after
    // the program and this process, after stopping sessions of programs
    // whose processes are gone.
    pub fn start(name: &str, process: u32) -> Result<PresentTiming, StartError> {
        let presents = Arc::new(Mutex::new(Vec::new()));
        let output = presents.clone();
        let mut decoder = Decoder::default();
        let mut tracker = Tracker::new(process);
        let session = Session::start(
            name,
            Box::new(move |record| {
                if let Some(event) = events::decode(&mut decoder, record) {
                    tracker.handle(&event);
                    let known = tracker.take();
                    if !known.is_empty() {
                        output.lock().unwrap().extend(known);
                    }
                }
            }),
        )?;
        Ok(PresentTiming {
            _session: session,
            presents,
            corrector: Corrector::default(),
        })
    }

    // The presents whose fate became known since the last call, in the order
    // they started.
    pub fn take(&mut self) -> Vec<Present> {
        let presents = std::mem::take(&mut *self.presents.lock().unwrap());
        self.corrector.correct(presents)
    }
}

// The process's presents in a trace file that has these providers' events,
// like one that xperf or WPR saved, with the performance counter's
// frequency on the computer that saved it, to compare with what PresentMon
// says of the same file.
pub fn read_file(path: &std::path::Path, process: u32) -> Result<(Vec<Present>, i64), String> {
    let presents = Arc::new(Mutex::new(Vec::new()));
    let output = presents.clone();
    let mut decoder = Decoder::default();
    let mut tracker = Tracker::new(process);
    let frequency = session::read_file(
        path,
        Box::new(move |record| {
            if let Some(event) = events::decode(&mut decoder, record) {
                tracker.handle(&event);
                output.lock().unwrap().extend(tracker.take());
            }
        }),
    )?;
    let presents = std::mem::take(&mut *presents.lock().unwrap());
    Ok((Corrector::default().correct(presents), frequency))
}

// What PresentMon's console does to the presents that it reads, before it
// times them: in the order they started, a present that NVIDIA's flip delay
// would put on the screen before the one before it goes on the screen with
// that one.
#[derive(Default)]
pub struct Corrector {
    last_screen: i64,
    last_flip_delay: i64,
    last_start: i64,
}

impl Corrector {
    pub fn correct(&mut self, mut presents: Vec<Present>) -> Vec<Present> {
        presents.sort_by_key(|present| present.start);
        for present in &mut presents {
            let Some(screen) = present.screen else {
                continue;
            };
            if present.start == self.last_start {
                continue;
            }
            let mut screen = screen;
            if self.last_flip_delay != 0 && self.last_screen > screen {
                present.flip_delay += self.last_screen - screen;
                screen = self.last_screen;
                present.screen = Some(screen);
            }
            self.last_screen = screen;
            self.last_flip_delay = present.flip_delay;
            self.last_start = present.start;
        }
        presents
    }
}

// The performance counter's ticks a second.
pub fn frequency() -> i64 {
    static FREQUENCY: std::sync::OnceLock<i64> = std::sync::OnceLock::new();
    *FREQUENCY.get_or_init(|| {
        let mut value = 0;
        unsafe {
            let _ = QueryPerformanceFrequency(&mut value);
        }
        value.max(1)
    })
}
