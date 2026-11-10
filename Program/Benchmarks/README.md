# Benchmarks

Times KGames' hot paths, and counts what they hold in memory: the programs' frames on the CPU and
the GPU, text, input, the PerfBar, meshes, models, JSON, math and the present tracker, and then
each program's heap, the caches, the mesh and file data, and the GPU's memory. It's a console
program, without a window. [Documentation/Performance.md](../../Documentation/Performance.md) and
[Documentation/Memory.md](../../Documentation/Memory.md) are analyses made with it.

```powershell
Program\RunLightly\RunLightly.ps1 Program\Build\Build.ps1 --release Benchmarks
Program\RunLightly\RunLightly.ps1 target\Release\Benchmarks.exe
target\Release\Benchmarks.exe --filter mesh --quick
target\Release\Benchmarks.exe --json target\Benchmarks\Benchmarks.json
target\Release\Benchmarks.exe --list
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

The environment line also says what built the program, for comparing its numbers with another
build's: `Language`, `Compiler`, which is MSVC's version, `Profile`, and `Hardened`, which is
whether the standard library checks every index. `Program\Build\Build.ps1 --release --hardened
Benchmarks` builds that one, as `target\ReleaseHardened\Benchmarks.exe`.

A benchmark's result goes through `black_box`, in `BlackBox.h`, so that the optimizer, which sees
the whole program in Release, can't drop the work that made it.

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
| `input`          | The input reader turning a frame's readings into events: a 1000 Hz mouse at 60   |
|                  | frames a second, and a keyboard holding a key                                    |
| `perfbar`        | The PerfBar's bookkeeping a frame, at 240 frames a second                        |
| `mesh`           | Catmull-Clark subdivision of a cube and a torus, triangulation, topology, ray    |
|                  | casts, a UV sphere, and the edges                                                |
| `gltf`           | Reading Veehiicuul's models                                                      |
| `json`           | Parsing and writing a Veehiicuul scene                                           |
| `math`           | Matrix multiplication and transforming points                                    |
| `present-timing` | The tracker following a present through Windows' events                          |

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

The counts are of the C++ heap: everything that the program and the libraries get with `new`, the
standard library's containers and strings among it. They're from the global `operator new` and
`operator delete`, which `Allocator.cpp` replaces with ones that count nothing until a memory
benchmark turns them on, so the timing benchmarks are unaffected: an allocation there costs them
one load of a flag.

What they count:

- A block is the bytes that were asked for, when it's allocated and again when it's freed. At
  most frees the compiler says how many they were, and at the rest, like the `delete[]` of an
  array of numbers, the C runtime's heap does, with `_msize`.
- That's what the heap is asked for, which isn't always what the code asked a container for: MSVC's
  standard library asks for 39 bytes more than a vector or a string of 4,096 bytes or more holds,
  to align it itself.
- A vector that grows allocates its new block before it frees its old one, so both are in the most
  held at once.
- In a Debug build, every container also allocates a proxy for its checked iterators, so the counts
  mean something in Release.

What they don't see:

- What the C runtime allocates for itself with `malloc`, like a file's buffer.
- What DirectWrite, Direct3D and the display driver allocate, which is in their own heaps: the text
  benchmarks give what a layout adds to the process's private commit instead, the GPU's memory
  comes from the device, and the last line says what the whole process holds.

## Testing

```powershell
Program\RunLightly\RunLightly.ps1 Program\RunTests\RunTests.ps1 BenchmarksTests
```

The tests in `Test/Benchmarks` check the harness's statistics, budget, table, JSON and command
line, the counting allocator and what it measures, in every form of `operator new` and `operator
delete`, and run every group once. The allocator's counts are the whole process's, so its tests,
in `Counting.cpp`, are one test that runs them one after another, with counting off whenever the
test harness itself allocates.

## Layout

| Path                              | Contents                                                        |
| --------------------------------- | --------------------------------------------------------------- |
| `Main.cpp`                        | Reads the command line, runs the groups, and prints the tables  |
| `Benchmarks.h`, `Benchmarks.cpp`  | The groups and the command line, as a library that the tests    |
|                                   | test                                                            |
| `Harness.h`, `Harness.cpp`        | Times a body within a budget, and keeps the statistics          |
| `BlackBox.h`                      | `black_box`, which keeps a benchmark's work from being          |
|                                   | optimized away                                                  |
| `Interface.h`, `Interface.cpp`    | The programs' frames, on the CPU and the GPU, text, input, the  |
|                                   | PerfBar                                                         |
| `Libraries.h`, `Libraries.cpp`    | Meshes, models, JSON, math and the present tracker              |
| `Memory.h`, `Memory.cpp`          | What the allocator counts, the usage made of the counts, and    |
|                                   | the process's and the GPU's memory                              |
| `Allocator.cpp`                   | The counting `operator new` and `operator delete`, which        |
|                                   | `Benchmarks.exe` and its tests each compile as their own, since |
|                                   | a library's wouldn't replace the C runtime's                    |
| `Footprints.h`, `Footprints.cpp`  | What the programs and the libraries hold and churn              |
