// What the presenter does once a frame's commands are on the GPU: it keeps
// the fence value that the GPU signals after them, which it waits for before
// it lets go of the frame's buffers, whether Present succeeds or not. A
// failed Present returns its error, and the main loop ends, so the value
// must be kept before then, or the buffers would go while the GPU still
// draws into them. How it presents, with vertical sync or without. And when
// the next frame can begin: only once the GPU has finished the last one.

use graphics::{AdapterChoice, Gpu, frame_can_begin, present_arguments, present_then_signal};
use windows_api::{D3D12_FENCE_FLAG_NONE, ID3D12Fence};
use windows_core::{Error, HRESULT};

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

// Holds the GPU's queue at a wait on a fence that only the CPU signals, so
// that nothing submitted after it finishes until the hold is dropped. A
// failed assertion drops it too, or the queue would wait forever, and the
// Gpu's drop, which waits for the queue, with it.
struct Hold(ID3D12Fence);

impl Hold {
    fn new(gpu: &Gpu) -> Hold {
        unsafe {
            let fence: ID3D12Fence = gpu.device.CreateFence(0, D3D12_FENCE_FLAG_NONE).unwrap();
            gpu.queue.Wait(&fence, 1).ok().unwrap();
            Hold(fence)
        }
    }
}

impl Drop for Hold {
    fn drop(&mut self) {
        unsafe {
            let _ = self.0.Signal(1);
        }
    }
}

// A frame begins only once the GPU has finished the last one, whatever
// DXGI's queue says: as soon as it begins, its drawing rewrites what the last
// frame's commands read, like the interface's upload buffer. The hold keeps
// the GPU on the last frame for as long as the test needs.
#[test]
fn a_frame_begins_only_once_the_gpu_has_finished_the_last() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let hold = Hold::new(&gpu);
    let last_frame = gpu.signal().unwrap();
    assert!(!frame_can_begin(true, &gpu, last_frame));

    // The GPU finishes the last frame.
    drop(hold);
    gpu.wait_for(last_frame).unwrap();
    assert!(frame_can_begin(true, &gpu, last_frame));
    // And DXGI's queue must have room too.
    assert!(!frame_can_begin(false, &gpu, last_frame));
}
