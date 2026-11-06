// What the programs and the libraries keep in memory, and what they churn
// through: the Rust heap, counted by a global allocator of our own; the GPU's
// memory, which the device reports; and the process's working set and private
// commit, which Windows reports.
//
// The allocator counts nothing until a memory benchmark turns it on, so the
// timing benchmarks pay only a relaxed load an allocation. It counts the Rust
// heap alone: what DirectWrite, Direct3D and the display driver allocate for
// the process is in their own heaps, which the working set and the GPU's
// memory show instead.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering::Relaxed};

use windows_api::*;

use crate::harness::Budget;

// Whether the allocator counts. Off until `start`.
static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static FREES: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
static LIVE: AtomicI64 = AtomicI64::new(0);
static PEAK: AtomicI64 = AtomicI64::new(0);

// The system allocator, counting what it hands out and takes back.
pub struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            allocated(layout.size());
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            allocated(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        freed(layout.size());
        unsafe { System.dealloc(pointer, layout) }
    }

    // One allocation of the new size, and the old size let go of, whether
    // the heap grew the block where it was or moved it.
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let moved = unsafe { System.realloc(pointer, layout, new_size) };
        if !moved.is_null() {
            freed(layout.size());
            allocated(new_size);
        }
        moved
    }
}

fn allocated(bytes: usize) {
    if !COUNTING.load(Relaxed) {
        return;
    }
    ALLOCATIONS.fetch_add(1, Relaxed);
    BYTES.fetch_add(bytes as u64, Relaxed);
    let live = LIVE.fetch_add(bytes as i64, Relaxed) + bytes as i64;
    PEAK.fetch_max(live, Relaxed);
}

fn freed(bytes: usize) {
    if !COUNTING.load(Relaxed) {
        return;
    }
    FREES.fetch_add(1, Relaxed);
    LIVE.fetch_sub(bytes as i64, Relaxed);
}

// What the heap has done since counting started.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub allocations: u64,
    pub frees: u64,
    pub bytes: u64,
    // Bytes allocated and not yet freed, from where counting started, so it
    // can be negative when work frees what was there before it.
    pub live: i64,
    // The most that `live` ever was.
    pub peak: i64,
}

// Starts counting, from nothing. Every measurement here begins with one, so
// that `live` is from where the work began.
pub fn start() {
    COUNTING.store(false, Relaxed);
    ALLOCATIONS.store(0, Relaxed);
    FREES.store(0, Relaxed);
    BYTES.store(0, Relaxed);
    LIVE.store(0, Relaxed);
    PEAK.store(0, Relaxed);
    COUNTING.store(true, Relaxed);
}

// Stops counting, and gives what the heap did.
pub fn stop() -> Counts {
    let counts = counts();
    COUNTING.store(false, Relaxed);
    counts
}

pub fn counts() -> Counts {
    Counts {
        allocations: ALLOCATIONS.load(Relaxed),
        frees: FREES.load(Relaxed),
        bytes: BYTES.load(Relaxed),
        live: LIVE.load(Relaxed),
        peak: PEAK.load(Relaxed),
    }
}

// What a memory benchmark measured.
#[derive(Clone, Debug, PartialEq)]
pub struct Usage {
    pub name: String,
    // What the counts are of, like "frame" or "subdivide".
    pub unit: &'static str,
    pub units: f64,
    // Allocations and bytes allocated, per unit.
    pub allocations: f64,
    pub bytes: f64,
    // The most the work held at once, from where it started.
    pub peak: i64,
    // What it still held when it was done: the footprint of what it built,
    // or, for repeated work that keeps nothing, a leak.
    pub kept: i64,
    pub note: String,
}

impl Usage {
    pub fn from_counts(name: &str, unit: &'static str, counts: Counts, units: f64) -> Usage {
        let units = units.max(1.0);
        Usage {
            name: name.to_string(),
            unit,
            units,
            allocations: counts.allocations as f64 / units,
            bytes: counts.bytes as f64 / units,
            peak: counts.peak,
            kept: counts.live,
            note: String::new(),
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Usage {
        self.note = note.into();
        self
    }

    pub fn to_json(&self) -> json::Value {
        json::object([
            ("Name", json::Value::from(self.name.as_str())),
            ("Unit", json::Value::from(self.unit)),
            ("Units", json::Value::from(self.units)),
            ("AllocationsPerUnit", json::Value::from(self.allocations)),
            ("BytesPerUnit", json::Value::from(self.bytes)),
            ("PeakBytes", json::Value::from(self.peak as f64)),
            ("KeptBytes", json::Value::from(self.kept as f64)),
            ("Note", json::Value::from(self.note.as_str())),
        ])
    }
}

// How much GPU memory a program's drawing takes, from the device.
#[derive(Clone, Debug, PartialEq)]
pub struct Video {
    pub name: String,
    // Bytes of the GPU's own memory, and of system memory that it reads
    // across PCIe, that the process has reserved.
    pub local: u64,
    pub non_local: u64,
    pub note: String,
}

impl Video {
    pub fn to_json(&self) -> json::Value {
        json::object([
            ("Name", json::Value::from(self.name.as_str())),
            ("LocalBytes", json::Value::from(self.local as f64)),
            ("NonLocalBytes", json::Value::from(self.non_local as f64)),
            ("Note", json::Value::from(self.note.as_str())),
        ])
    }
}

// What a memory group measured: the heap, and the GPU's memory.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub heap: Vec<Usage>,
    pub video: Vec<Video>,
}

impl Report {
    pub fn of(heap: Vec<Usage>) -> Report {
        Report { heap, video: Vec::new() }
    }

    pub fn add(&mut self, other: Report) {
        self.heap.extend(other.heap);
        self.video.extend(other.video);
    }
}

// Counts what `body` does to the heap over `repeat` runs, after `warmup`
// runs that aren't counted, and gives the counts per run. Allocation counts
// don't vary the way times do, so there's no need for statistics: the one
// number is the cost, as long as the warm-up has filled the caches that the
// work hits.
pub fn measure(name: &str, unit: &'static str, budget: Budget, most: usize, mut body: impl FnMut()) -> Usage {
    let (warmup, repeat) = runs(budget, most);
    for _ in 0..warmup {
        body();
    }
    start();
    for _ in 0..repeat {
        body();
    }
    let counts = stop();
    Usage::from_counts(name, unit, counts, repeat as f64)
}

// Counts what making one thing costs, and how much heap it holds while it's
// alive, which is its footprint. The value goes back to the caller, so that
// nothing of it is freed before it's counted.
pub fn held<T>(name: &str, unit: &'static str, make: impl FnOnce() -> T) -> (Usage, T) {
    start();
    let value = make();
    let counts = counts();
    COUNTING.store(false, Relaxed);
    (Usage::from_counts(name, unit, counts, 1.0), value)
}

// How many warm-up runs and counted runs the budget asks for, at most
// `most` counted. One run and no warm-up for Budget::ONCE, which tests use.
pub fn runs(budget: Budget, most: usize) -> (usize, usize) {
    if budget.iterations <= 1 {
        return (0, 1);
    }
    (budget.warmup, most.min(budget.iterations).max(1))
}

// The process's memory, as Windows counts it: the physical memory it has
// now, the most it ever had, and the private commit, which is everything it
// has asked for that isn't shared with another process, paged in or not.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProcessMemory {
    pub working_set: u64,
    pub peak_working_set: u64,
    pub private_commit: u64,
}

pub fn process_memory() -> ProcessMemory {
    let mut counters = PROCESS_MEMORY_COUNTERS_EX {
        cb: size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
        ..Default::default()
    };
    let ok = unsafe {
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut counters as *mut PROCESS_MEMORY_COUNTERS_EX as *mut PROCESS_MEMORY_COUNTERS,
            counters.cb,
        )
    };
    if !ok.as_bool() {
        return ProcessMemory::default();
    }
    ProcessMemory {
        working_set: counters.WorkingSetSize as u64,
        peak_working_set: counters.PeakWorkingSetSize as u64,
        private_commit: counters.PrivateUsage as u64,
    }
}

impl ProcessMemory {
    pub fn to_json(&self) -> json::Value {
        json::object([
            ("WorkingSetBytes", json::Value::from(self.working_set as f64)),
            ("PeakWorkingSetBytes", json::Value::from(self.peak_working_set as f64)),
            ("PrivateCommitBytes", json::Value::from(self.private_commit as f64)),
        ])
    }
}

// The heap measurements as a table that reads in a terminal: allocations and
// bytes a unit, and the peak and kept bytes, in kibibytes.
pub fn table(usage: &[Usage]) -> String {
    let name_width = usage.iter().map(|row| row.name.len()).max().unwrap_or(4).max(9);
    let unit_width = usage.iter().map(|row| row.unit.len()).max().unwrap_or(4).max(4);
    let mut text = format!(
        "{:<name_width$}  {:<unit_width$}  {:>6}  {:>12}  {:>12}  {:>10}  {:>10}  Note\n",
        "Benchmark", "Unit", "Units", "Allocations", "Bytes", "Peak KiB", "Kept KiB"
    );
    for row in usage {
        text += &format!(
            "{:<name_width$}  {:<unit_width$}  {:>6}  {:>12.1}  {:>12.1}  {:>10.1}  {:>10.1}  {}\n",
            row.name,
            row.unit,
            row.units as u64,
            row.allocations,
            row.bytes,
            row.peak as f64 / 1024.0,
            row.kept as f64 / 1024.0,
            row.note
        );
    }
    text
}

// The GPU memory measurements as a table, in mebibytes.
pub fn video_table(video: &[Video]) -> String {
    let name_width = video.iter().map(|row| row.name.len()).max().unwrap_or(4).max(9);
    let mut text = format!(
        "{:<name_width$}  {:>10}  {:>14}  Note\n",
        "Benchmark", "Local MiB", "Non-local MiB"
    );
    for row in video {
        text += &format!(
            "{:<name_width$}  {:>10.2}  {:>14.2}  {}\n",
            row.name,
            row.local as f64 / (1024.0 * 1024.0),
            row.non_local as f64 / (1024.0 * 1024.0),
            row.note
        );
    }
    text
}
