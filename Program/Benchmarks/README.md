# Benchmarks

Times KGames' hot paths, and counts what they hold in memory: the programs' frames on the CPU and
the GPU, text, input, the PerfBar, meshes, models, JSON, math and the present tracker, and then
each program's heap, the caches, the mesh and file data, and the GPU's memory. It's a console
program, without a window. [Documentation/Performance.md](../../Documentation/Performance.md) and
[Documentation/Memory.md](../../Documentation/Memory.md) are the analyses made with it.

```powershell
Program\RunLightly\RunLightly.ps1 cargo build --release -p benchmarks
Program\RunLightly\RunLightly.ps1 target\release\benchmarks.exe
target\release\benchmarks.exe --filter mesh --quick
target\release\benchmarks.exe --json target\Benchmarks\Benchmarks.json
target\release\benchmarks.exe --list
```

| Argument          | What it does                                                               |
| ----------------- | -------------------------------------------------------------------------- |
| `--filter <text>` | Runs only the groups whose names have the text, like `mesh` or `gpu`       |
| `--quick`         | A fifth of the budget: up to 0.1 s or 400 iterations a benchmark           |
| `--json <path>`   | Writes the measurements, and what they were taken on, as JSON              |
| `--list`          | Lists the groups instead of running them                                   |

Each benchmark runs its body for up to half a second, or 2000 iterations, after a few warm-up
iterations, and times every iteration on its own, on the performance counter. The table has each
one's minimum, median, mean, 99th percentile and maximum, in milliseconds per unit of work, like a
frame or a parse, and a note with what else it found out, like how many instances a frame drew.

The minimum is what the work costs when nothing else is in the way, and the median is what it
usually costs. Run through [RunLightly](../RunLightly/README.md), as coding agents run it, the
benchmarks have the efficiency cores at efficient clocks, idle priority, and a cap on CPU time that
stops the thread now and then, so the maximum shows those stops, and every time is a few times what
the performance cores would take. Run it directly, from a terminal, for the performance cores'
times, when no game is running. The environment line says which: `Lightly` is whether it ran in
RunLightly's job.

## Groups

| Group            | What it times                                                                    |
| ---------------- | -------------------------------------------------------------------------------- |
| `ui`             | A frame's update of each GUI program, through `Headless`: steady, with nothing   |
|                  | changing; moving, with the pointer crossing the page; and the start, from        |
|                  | nothing to the first frame                                                       |
| `gpu`            | Drawing the frame with the GPU, into a target the size of the maximized window's |
|                  | client area, timed with `GpuTimer`'s timestamp queries; and what recording it    |
|                  | costs the CPU                                                                    |
| `text`           | DirectWrite through `Fonts`: a layout the cache has, one it doesn't, a wrapped   |
|                  | one, glyph quads, and rasterizing glyphs the atlas doesn't have                  |
| `input`          | The input reader turning a frame's readings into events: a 1000 Hz mouse at 60  |
|                  | frames a second, and a keyboard holding a key                                    |
| `perfbar`        | The PerfBar's bookkeeping a frame, at 240 frames a second                        |
| `mesh`           | Catmull-Clark subdivision of a cube and a torus, triangulation, topology, ray    |
|                  | casts, a UV sphere, and the edges                                                |
| `gltf`           | Reading Veehiicuul's models                                                      |
| `json`           | Parsing and writing a Veehiicuul scene                                           |
| `math`           | Matrix multiplication and transforming points                                    |
| `present-timing` | The tracker following a present through Windows' events                         |

The GPU is the fastest one, as the tests use, not the one a window's monitor is wired to. Each
`gpu` iteration waits for the GPU, so the CPU and GPU times are of the same frames.

## Memory groups

These count instead of timing, and print their own tables after the times: how many allocations
and bytes a frame or a run takes, the most the work held at once, and what it still held when it
was done, which is the footprint of what it built, or a leak for work that should keep nothing.

| Group              | What it counts                                                                 |
| ------------------ | ------------------------------------------------------------------------------ |
| `memory/frames`    | Each program's heap once it has started, what a frame churns through, and      |
|                    | what 1,500 frames leave behind                                                 |
| `memory/caches`    | The text layouts and their glyphs, what the purge gives back, and what the     |
|                    | interface keeps for widgets whose names change every frame                     |
| `memory/libraries` | A subdivided mesh, its topology and its triangles, a model, and a scene        |
| `memory/gpu`       | How much of the GPU's memory a program's drawing takes, from the device        |

The counts are of the Rust heap, from a global allocator in `src/memory.rs` that counts
nothing until a memory benchmark turns it on, so the timing benchmarks are unaffected. What
DirectWrite, Direct3D and the display driver allocate is in their own heaps, which the counts
can't see: the text benchmarks give what a layout adds to the process's private commit instead,
the GPU's memory comes from the device, and the last line says what the whole process holds.

## Testing

```powershell
Program\RunLightly\RunLightly.ps1 cargo test -p benchmarks_tests
```

The tests in `Test/Benchmarks` check the harness's statistics, budget, table, JSON and command
line, the counting allocator and what it measures, and run every group once. The allocator's
counts are the whole process's, so its tests, in `counting.rs`, and the one that runs every memory
group once, in `memory_groups.rs`, run without the test harness, one after another on the main
thread, where no other test or harness thread allocates while they count. A snapshot's peak is
never less than what it says is held, even while other threads allocate.

## Layout

| Path                | Contents                                                               |
| ------------------- | ---------------------------------------------------------------------- |
| `src/main.rs`       | Reads the command line, runs the groups, and prints the table          |
| `src/lib.rs`        | The groups and the command line, as a library that the tests test      |
| `src/harness.rs`    | Times a body within a budget, and keeps the statistics                 |
| `src/interface.rs`  | The programs' frames, on the CPU and the GPU, text, input, the PerfBar |
| `src/libraries.rs`  | Meshes, models, JSON, math and the present tracker                     |
| `src/memory.rs`     | The counting allocator, the usage it makes of the counts, and the      |
|                     | process's and the GPU's memory                                         |
| `src/footprints.rs` | What the programs and the libraries hold and churn                     |
