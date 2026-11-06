# Benchmarks

Times KGames' hot paths, and prints their statistics: the programs' frames on the CPU and the
GPU, text, input, the PerfBar, meshes, models, JSON, math and the present tracker. It's a console
program, without a window. [Documentation/Performance.md](../../Documentation/Performance.md) is
the analysis made with it.

```powershell
Program\RunLightly\RunLightly.ps1 cargo build --release -p benchmarks
Program\RunLightly\RunLightly.ps1 target\release\Benchmarks.exe
target\release\Benchmarks.exe --filter mesh --quick
target\release\Benchmarks.exe --json target\Benchmarks\Benchmarks.json
target\release\Benchmarks.exe --list
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

## Testing

```powershell
Program\RunLightly\RunLightly.ps1 cargo test -p benchmarks_tests
```

The tests in `Test/Benchmarks` check the harness's statistics, budget, table, JSON and command
line, and run every group once.

## Layout

| Path                   | Contents                                                              |
| ---------------------- | --------------------------------------------------------------------- |
| `Source/Main.rs`       | Reads the command line, runs the groups, and prints the table         |
| `Source/Library.rs`    | The groups and the command line, as a library that the tests test     |
| `Source/Harness.rs`    | Times a body within a budget, and keeps the statistics                |
| `Source/Interface.rs`  | The programs' frames, on the CPU and the GPU, text, input, the PerfBar |
| `Source/Libraries.rs`  | Meshes, models, JSON, math and the present tracker                    |
