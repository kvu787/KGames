// What the presenter does once a frame's commands are on the GPU: it keeps
// the fence value that the GPU signals after them, which it waits for before
// it lets go of the frame's buffers, whether Present succeeds or not. A
// failed Present returns its error, and the main loop ends, so the value
// must be kept before then, or the buffers would go while the GPU still
// draws into them. How it presents, with vertical sync or without. And how
// it waits on several presenters at once.

use std::time::{Duration, Instant};

use graphics::{present_arguments, present_then_signal, wait_any};
use windows_api::{CloseHandle, CreateEventW, HANDLE};
use windows_core::{Error, HRESULT, PCWSTR};

const FAILED: HRESULT = HRESULT(0x887A0001_u32 as i32);

// DXGI_PRESENT_ALLOW_TEARING.
const ALLOW_TEARING: u32 = 0x200;

// Vertical sync on never tears, and off always may: there's no third way, of
// frames that don't wait and don't tear either.
#[test]
fn frames_wait_for_vertical_sync_or_may_tear() {
    assert_eq!(present_arguments(true), (1, 0));
    assert_eq!(present_arguments(false), (0, ALLOW_TEARING));
}

#[test]
fn a_presented_frame_s_fence_value_is_kept_and_returned() {
    let mut last_frame = 3;
    let fence = present_then_signal(&mut last_frame, || Ok(()), || Ok(4)).unwrap();
    assert_eq!((fence, last_frame), (4, 4));
}

#[test]
fn a_frame_whose_present_fails_is_still_waited_for() {
    let mut last_frame = 3;
    let result = present_then_signal(&mut last_frame, || Err(Error::from(FAILED)), || Ok(4));
    assert_eq!(result.unwrap_err().code(), FAILED);
    assert_eq!(last_frame, 4);
}

#[test]
fn the_fence_is_signaled_after_the_present() {
    let order = std::cell::RefCell::new(Vec::new());
    let mut last_frame = 0;
    present_then_signal(
        &mut last_frame,
        || {
            order.borrow_mut().push("present");
            Ok(())
        },
        || {
            order.borrow_mut().push("signal");
            Ok(1)
        },
    )
    .unwrap();
    assert_eq!(*order.borrow(), ["present", "signal"]);
}

// An event that resets as a wait takes its signal, like a swap chain's frame
// latency waitable object, which is a semaphore.
fn event(signaled: bool) -> HANDLE {
    let event = unsafe { CreateEventW(None, false, signaled, PCWSTR::null()) };
    assert!(!event.0.is_null());
    event
}

// The overlays wait on each display's presenter at once: a display that
// stops taking frames, like one that's asleep, never signals its waitable,
// and holds up none of the others, wherever it is in the list.
#[test]
fn a_wait_on_several_presenters_ends_with_whichever_is_ready() {
    let asleep = event(false);
    let ready = event(true);
    let start = Instant::now();
    assert_eq!(wait_any(&[asleep, ready], 5_000), Some(1));
    assert!(start.elapsed() < Duration::from_secs(1), "{:?}", start.elapsed());
    // The wait took the signal, which the presenter keeps until its frame.
    assert_eq!(wait_any(&[asleep, ready], 0), None);
    // Nothing ready ends the wait when the time passes.
    let start = Instant::now();
    assert_eq!(wait_any(&[asleep], 20), None);
    assert!(start.elapsed() >= Duration::from_millis(15), "{:?}", start.elapsed());
    unsafe {
        let _ = CloseHandle(asleep);
        let _ = CloseHandle(ready);
    }
}
