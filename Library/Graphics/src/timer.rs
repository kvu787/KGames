// Times spans of a frame's GPU work, with timestamp queries: how long the
// GPU took from one point in the frame's commands to another, like from
// before the interface's draw to after it, apart from what the CPU took to
// record and submit them. For benchmarks and tests, not for frames that the
// user sees.
//
//     let mut timer = GpuTimer::new(&gpu, 1)?;
//     let frame = offscreen.begin()?;
//     let span = timer.begin(&frame);
//     ... record the work ...
//     timer.end(&frame, span);
//     timer.resolve(&frame);
//     ... close, execute and signal ...
//     let milliseconds = timer.read(&gpu, fence)?[0];

use windows_api::*;
use windows_core::Result;

use crate::resources::create_buffer;
use crate::{Frame, Gpu};

// A span that `begin` started, to pass to `end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span(u32);

pub struct GpuTimer {
    heap: ID3D12QueryHeap,
    // Where the timestamps are resolved to, which the CPU reads.
    readback: ID3D12Resource,
    // The queue's timestamp ticks a second.
    frequency: u64,
    spans: u32,
    // How many spans this frame has begun.
    begun: u32,
    // Which of them it has ended. Resolving a query that was never ended is
    // undefined, and can hang the GPU or remove the device.
    ended: Vec<bool>,
}

impl GpuTimer {
    // A timer for up to `spans` spans a frame.
    pub fn new(gpu: &Gpu, spans: u32) -> Result<Self> {
        let spans = spans.max(1);
        unsafe {
            let description = D3D12_QUERY_HEAP_DESC {
                Type: D3D12_QUERY_HEAP_TYPE_TIMESTAMP,
                Count: spans * 2,
                NodeMask: 0,
            };
            let mut heap: Option<ID3D12QueryHeap> = None;
            gpu.device.CreateQueryHeap(&description, &mut heap)?;
            let heap = heap.ok_or_else(windows_core::Error::empty)?;
            let readback = create_buffer(
                &gpu.device,
                spans as u64 * 2 * size_of::<u64>() as u64,
                D3D12_HEAP_TYPE_READBACK,
                D3D12_RESOURCE_STATE_COPY_DEST,
            )?;
            let frequency = gpu.queue.GetTimestampFrequency()?;
            Ok(Self {
                heap,
                readback,
                frequency,
                spans,
                begun: 0,
                ended: vec![false; spans as usize],
            })
        }
    }

    // Starts a span. A timestamp is the time when the GPU has finished all of
    // the frame's work before it, so a span is the GPU's time for the work
    // between its begin and its end.
    pub fn begin(&mut self, frame: &Frame) -> Span {
        assert!(self.begun < self.spans, "more spans than the timer was made for");
        let span = Span(self.begun);
        self.begun += 1;
        unsafe { frame.list.EndQuery(&self.heap, D3D12_QUERY_TYPE_TIMESTAMP, span.0 * 2) };
        span
    }

    pub fn end(&mut self, frame: &Frame, span: Span) {
        assert!(span.0 < self.begun, "a span that this frame didn't begin");
        assert!(!self.ended[span.0 as usize], "a span ended twice");
        self.ended[span.0 as usize] = true;
        unsafe { frame.list.EndQuery(&self.heap, D3D12_QUERY_TYPE_TIMESTAMP, span.0 * 2 + 1) };
    }

    // Records the copy of the frame's timestamps to where the CPU reads
    // them, after every span's end.
    pub fn resolve(&self, frame: &Frame) {
        if self.begun == 0 {
            return;
        }
        assert!(
            self.ended[..self.begun as usize].iter().all(|ended| *ended),
            "a span was begun and never ended, so its end can't be resolved"
        );
        unsafe {
            frame.list.ResolveQueryData(&self.heap, D3D12_QUERY_TYPE_TIMESTAMP, 0, self.begun * 2, &self.readback, 0);
        }
    }

    // Each span's length in milliseconds, in the order they began, once the
    // GPU has signaled `fence`, which it does after the frame's commands.
    // Ready for the next frame's spans.
    pub fn read(&mut self, gpu: &Gpu, fence: u64) -> Result<Vec<f64>> {
        gpu.wait_for(fence)?;
        let count = self.begun as usize;
        self.begun = 0;
        self.ended.fill(false);
        if count == 0 {
            return Ok(Vec::new());
        }
        let mut stamps = vec![0u64; count * 2];
        unsafe {
            let mut mapped = std::ptr::null_mut();
            let range = D3D12_RANGE {
                Begin: 0,
                End: stamps.len() * size_of::<u64>(),
            };
            self.readback.Map(0, Some(&range), &mut mapped).ok()?;
            std::ptr::copy_nonoverlapping(mapped as *const u64, stamps.as_mut_ptr(), stamps.len());
            self.readback.Unmap(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }));
        }
        Ok(stamps
            .chunks(2)
            .map(|pair| pair[1].saturating_sub(pair[0]) as f64 * 1000.0 / self.frequency as f64)
            .collect())
    }
}
