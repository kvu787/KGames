// The displays that Library/Displays finds: in physical pixels, with each
// one's own DPI, whatever DPI awareness the thread that asks has. Windows
// tells a thread that isn't DPI aware that every display is at 96 DPI, and
// scales its areas to match, so on a display that isn't at 100% scale, a
// program that asked before it made itself DPI aware would get them wrong.
// At 100%, both are the same anyway.

use windows_api::{DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetThreadDpiAwarenessContext};

// DPI_AWARENESS_CONTEXT_UNAWARE.
const UNAWARE: isize = -1;

// The displays as a thread with this DPI awareness sees them.
fn seen_with(context: isize) -> Vec<(String, [i32; 4], [i32; 4], u32)> {
    std::thread::spawn(move || {
        unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT(context as _)) };
        displays::enumerate()
            .unwrap()
            .into_iter()
            .map(|display| {
                let rect = |r: windows_api::RECT| [r.left, r.top, r.right, r.bottom];
                (display.identifier, rect(display.area), rect(display.work_area), display.dpi)
            })
            .collect()
    })
    .join()
    .unwrap()
}

#[test]
fn a_thread_that_isnt_dpi_aware_sees_the_displays_as_they_are() {
    let aware = seen_with(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2.0 as isize);
    assert!(!aware.is_empty());
    assert_eq!(seen_with(UNAWARE), aware);
}
