// What the presenter does once a frame's commands are on the GPU: it keeps
// the fence value that the GPU signals after them, which it waits for before
// it lets go of the frame's buffers, whether Present succeeds or not. A
// failed Present returns its error, and the main loop ends, so the value
// must be kept before then, or the buffers would go while the GPU still
// draws into them.

use graphics::present_then_signal;
use windows_core::{Error, HRESULT};

const FAILED: HRESULT = HRESULT(0x887A0001_u32 as i32);

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
