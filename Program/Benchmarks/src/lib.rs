// Benchmarks of KGames' hot paths, as a library that Test/Benchmarks tests:
// the harness, the groups of benchmarks, and the command line. README.md
// describes them.
//
// - harness: times a body many times and keeps the statistics.
// - interface: the programs' frames on the CPU and the GPU, text, input and
//   the PerfBar.
// - libraries: meshes, models, JSON, math and the present tracker.
// - memory: counts the heap, and reads the GPU's and the process's memory.
// - footprints: what the programs and the libraries hold and churn.

pub mod footprints;
pub mod harness;
pub mod interface;
pub mod libraries;
pub mod memory;

pub use harness::{Budget, Measurement, measure, measure_timed, percentile, table};
pub use memory::{ProcessMemory, Report, Usage, Video, process_memory};

// The heap counts of every benchmark that asks for them, from one allocator
// for the whole program. It counts nothing until a memory benchmark turns it
// on, so the timing benchmarks pay only a relaxed load an allocation.
#[global_allocator]
static ALLOCATOR: memory::Counting = memory::Counting;

pub const USAGE: &str = "Benchmarks [--filter <text>] [--quick] [--json <path>] [--list]";

// A group of benchmarks, run together because they share a setup, like a
// GPU.
#[derive(Clone, Copy)]
pub struct Group {
    pub name: &'static str,
    // Whether its benchmarks draw with the GPU.
    pub gpu: bool,
    pub run: fn(Budget) -> Vec<Measurement>,
}

pub const GROUPS: [Group; 10] = [
    Group { name: "ui", gpu: false, run: interface::frames },
    Group { name: "gpu", gpu: true, run: interface::draws },
    Group { name: "text", gpu: false, run: interface::text },
    Group { name: "input", gpu: false, run: interface::input },
    Group { name: "perfbar", gpu: false, run: interface::perf_bar },
    Group { name: "mesh", gpu: false, run: libraries::meshes },
    Group { name: "gltf", gpu: false, run: libraries::models },
    Group { name: "json", gpu: false, run: libraries::json },
    Group { name: "math", gpu: false, run: libraries::math },
    Group { name: "present-timing", gpu: false, run: libraries::present_timing },
];

// A group of memory benchmarks, which count what's held and churned
// instead of timing it.
#[derive(Clone, Copy)]
pub struct MemoryGroup {
    pub name: &'static str,
    // Whether its benchmarks need a GPU.
    pub gpu: bool,
    pub run: fn(Budget) -> Report,
}

pub const MEMORY_GROUPS: [MemoryGroup; 4] = [
    MemoryGroup { name: "memory/frames", gpu: false, run: footprints::frames },
    MemoryGroup { name: "memory/caches", gpu: false, run: footprints::caches },
    MemoryGroup { name: "memory/libraries", gpu: false, run: footprints::libraries },
    MemoryGroup { name: "memory/gpu", gpu: true, run: footprints::video },
];

// What the command line asks for.
#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    // Only the groups whose names have the text, if there is any.
    pub filter: Option<String>,
    pub budget: Budget,
    // Where to write the measurements as JSON, if anywhere.
    pub json: Option<String>,
    // Only list the groups.
    pub list: bool,
}

pub fn parse_arguments(arguments: &[String]) -> Result<Options, String> {
    let mut options = Options {
        filter: None,
        budget: Budget::DEFAULT,
        json: None,
        list: false,
    };
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--filter" => options.filter = Some(arguments.next().ok_or("--filter needs text after it.")?.clone()),
            "--json" => options.json = Some(arguments.next().ok_or("--json needs a path after it.")?.clone()),
            "--quick" => options.budget = Budget::QUICK,
            "--list" => options.list = true,
            other => return Err(format!("Unknown argument: {other}.")),
        }
    }
    Ok(options)
}

// The groups that the filter keeps.
pub fn selected(filter: Option<&str>) -> Vec<Group> {
    GROUPS.iter().copied().filter(|group| filter.is_none_or(|text| group.name.contains(text))).collect()
}

// The memory groups that the filter keeps.
pub fn selected_memory(filter: Option<&str>) -> Vec<MemoryGroup> {
    MEMORY_GROUPS.iter().copied().filter(|group| filter.is_none_or(|text| group.name.contains(text))).collect()
}

// What the measurements were taken on, for the report's header.
pub fn environment() -> json::Value {
    let gpu = graphics::Gpu::new(graphics::AdapterChoice::HighPerformance).map(|gpu| gpu.name.clone()).unwrap_or_else(|error| format!("none: {error}"));
    json::object([
        ("Gpu", json::Value::from(gpu.as_str())),
        ("Lightly", json::Value::from(std::env::var("KGAMES_LIGHT_JOB").is_ok())),
        ("Profile", json::Value::from(if cfg!(debug_assertions) { "debug" } else { "release" })),
        ("Cores", json::Value::from(std::thread::available_parallelism().map_or(0.0, |cores| cores.get() as f64))),
    ])
}

// The measurements, the memory, and the environment as one JSON object.
pub fn report(environment: &json::Value, measurements: &[Measurement], memory: &Report, process: ProcessMemory) -> json::Value {
    json::object([
        ("Environment", environment.clone()),
        ("Measurements", json::Value::Array(measurements.iter().map(Measurement::to_json).collect())),
        ("Memory", json::Value::Array(memory.heap.iter().map(Usage::to_json).collect())),
        ("VideoMemory", json::Value::Array(memory.video.iter().map(Video::to_json).collect())),
        ("Process", process.to_json()),
    ])
}
