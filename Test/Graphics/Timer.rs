// The GPU timer: its spans measure the GPU's time between two points in a
// frame's commands, a span around a clear of a large target is longer than
// one around nothing, it's ready for the next frame after it's read, and it
// won't resolve a span that was begun and never ended.

use graphics::{AdapterChoice, Gpu, GpuTimer, Offscreen};

// Draws a frame that clears the target `clears` times, with a span around
// the clears and one around nothing, and gives the two spans' milliseconds.
fn time(gpu: &mut Gpu, timer: &mut GpuTimer, target: &mut Offscreen, clears: u32) -> (f64, f64) {
    let frame = target.begin().unwrap();
    let busy = timer.begin(&frame);
    for index in 0..clears {
        unsafe { frame.clear([index as f32 / clears as f32, 0.5, 0.25, 1.0]) };
    }
    timer.end(&frame, busy);
    let idle = timer.begin(&frame);
    timer.end(&frame, idle);
    timer.resolve(&frame);
    let fence = target.submit(gpu, frame).unwrap();
    let spans = timer.read(gpu, fence).unwrap();
    assert_eq!(spans.len(), 2);
    (spans[0], spans[1])
}

#[test]
fn a_span_around_work_is_longer_than_one_around_nothing() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut timer = GpuTimer::new(&gpu, 2).unwrap();
    let mut target = Offscreen::new(&gpu, 2560, 1440, [0.0; 4]).unwrap();
    // A frame or two to warm the GPU up, then the one that's checked.
    time(&mut gpu, &mut timer, &mut target, 64);
    let (busy, idle) = time(&mut gpu, &mut timer, &mut target, 64);
    assert!(busy.is_finite() && busy > 0.0, "the busy span is {busy} ms");
    assert!(idle >= 0.0 && idle < busy, "an idle span of {idle} ms isn't shorter than the busy span of {busy} ms");
    // Short enough to be plausible even on WARP: 64 clears of 2560 x 1440.
    assert!(busy < 1000.0, "the busy span is {busy} ms");
}

#[test]
fn a_frame_with_no_spans_reads_as_none() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut timer = GpuTimer::new(&gpu, 1).unwrap();
    let mut target = Offscreen::new(&gpu, 16, 16, [0.0; 4]).unwrap();
    let frame = target.begin().unwrap();
    timer.resolve(&frame);
    target.finish(&mut gpu, frame).unwrap();
    let fence = gpu.signal().unwrap();
    assert_eq!(timer.read(&gpu, fence).unwrap(), Vec::<f64>::new());
}

// Resolving a query that was never ended is undefined, and can hang the GPU,
// so a span that's begun and not ended stops the program at the resolve.
#[test]
#[should_panic(expected = "never ended")]
fn resolving_a_span_that_never_ended_panics() {
    let gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut timer = GpuTimer::new(&gpu, 2).unwrap();
    let mut target = Offscreen::new(&gpu, 16, 16, [0.0; 4]).unwrap();
    let frame = target.begin().unwrap();
    let ended = timer.begin(&frame);
    timer.end(&frame, ended);
    let _never_ended = timer.begin(&frame);
    timer.resolve(&frame);
}
