// What the presenter does once a frame's commands are on the GPU: it keeps
// the fence value that the GPU signals after them, which it waits for before
// it lets go of the frame's buffers, whether Present succeeds or not. A
// failed Present returns its error, and the main loop ends, so the value
// must be kept before then, or the buffers would go while the GPU still
// draws into them, and it waits for the value before it returns, as what the
// frame's commands use goes with the error. How it presents, with vertical sync or without. When the
// next frame can begin: only once the GPU has finished the last one. And how
// the main loop waits for that.

use std::time::{Duration, Instant};

use graphics::{AdapterChoice, Gpu, frame_can_begin, present_arguments, present_then_signal, wait_for_frame};
use windows_api_raw::{CloseHandle, CreateEventW, D3D12_FENCE_FLAG_NONE, ID3D12Fence, INFINITE};
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

// A frame that presents doesn't wait for the GPU: the next one does, once it
// can begin.
#[test]
fn a_presented_frame_s_fence_value_is_kept_and_returned() {
    let mut last_frame = 3;
    let fence = present_then_signal(&mut last_frame, || Ok(()), || Ok(4), |_| panic!("waited")).unwrap();
    assert_eq!((fence, last_frame), (4, 4));
}

// The main loop ends with the error, and lets go of what the frame's
// commands use, like a Readback's buffer, which the GPU may still be copying
// into, so the GPU must have finished them before the error returns.
#[test]
fn a_frame_whose_present_fails_is_still_waited_for() {
    let waited = std::cell::Cell::new(None);
    let mut last_frame = 3;
    let result = present_then_signal(
        &mut last_frame,
        || Err(Error::from(FAILED)),
        || Ok(4),
        |value| {
            waited.set(Some(value));
            Ok(())
        },
    );
    assert_eq!(result.unwrap_err().code(), FAILED);
    assert_eq!(last_frame, 4);
    assert_eq!(waited.get(), Some(4));
}

#[test]
fn the_fence_is_signaled_after_the_present() {
    let order = std::cell::RefCell::new(Vec::new());
    let mut last_frame = 0;
    let _ = present_then_signal(
        &mut last_frame,
        || {
            order.borrow_mut().push("present");
            Err(Error::from(FAILED))
        },
        || {
            order.borrow_mut().push("signal");
            Ok(1)
        },
        |_| {
            order.borrow_mut().push("wait");
            Ok(())
        },
    );
    assert_eq!(*order.borrow(), ["present", "signal", "wait"]);
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

// Lets the GPU go from another thread, after a while, for a test whose wait
// would otherwise last until the hold is dropped. GPU objects are
// free-threaded.
struct Release(ID3D12Fence);

unsafe impl Send for Release {}

impl Release {
    fn after(hold: &Hold, wait: Duration) {
        let release = Release(hold.0.clone());
        std::thread::spawn(move || {
            std::thread::sleep(wait);
            release.signal();
        });
    }

    fn signal(&self) {
        unsafe {
            let _ = self.0.Signal(1);
        }
    }
}

// The main loop's wait for a frame ends when its time passes, or a window
// message comes, even while the GPU is still on the last frame, like a long
// one at the GPU's idle priority, which RunLightly gives, behind a game's: a
// wait for the GPU alone would hold up the window's messages, like WM_CLOSE,
// and past about 5 seconds, Windows would take the window for hung. And it
// ends as soon as DXGI has room, while the GPU is busy. A thread lets the GPU
// go after 5 seconds, so that a wait that ignored its time fails the test
// instead of hanging it.
#[test]
fn the_wait_for_a_frame_ends_with_its_time_while_the_gpu_is_busy() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let hold = Hold::new(&gpu);
    let last_frame = gpu.signal().unwrap();
    Release::after(&hold, Duration::from_secs(5));
    let event = unsafe { CreateEventW(None, false, false, None) };
    assert!(!event.0.is_null());

    let start = Instant::now();
    let taken = wait_for_frame(&gpu, last_frame, event, None, 50).unwrap();
    let waited = start.elapsed();
    assert!(waited < Duration::from_secs(2), "{waited:?}");
    assert!(!taken);
    assert!(!gpu.is_complete(last_frame));

    // DXGI's waitable object, signaled, ends it at once, and its signal is
    // taken.
    let waitable = unsafe { CreateEventW(None, true, true, None) };
    let start = Instant::now();
    assert!(wait_for_frame(&gpu, last_frame, event, Some(waitable), INFINITE).unwrap());
    assert!(start.elapsed() < Duration::from_secs(2), "{:?}", start.elapsed());

    // Once the GPU has finished, there's nothing to wait for.
    drop(hold);
    gpu.wait_for(last_frame).unwrap();
    assert!(!wait_for_frame(&gpu, last_frame, event, None, INFINITE).unwrap());
    unsafe {
        let _ = CloseHandle(event);
        let _ = CloseHandle(waitable);
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
