# Performance

Where the programs' time goes, measured on 2026-11-06 at commit `2fb2672` with
[Benchmarks](../Program/Benchmarks/README.md), [RunInBackground](../Program/RunInBackground/README.md)
and the PerfBar, and read from the code. The short of it:

1. **Frames are cheap.** A frame's update takes 0.02 to 0.18 ms of CPU, recording its drawing 0.01
   to 0.02 ms, and drawing it 0.02 to 0.16 ms of GPU, at 2560 x 1369. The programs draw 700 to
   1700 frames a second without vertical sync, even throttled. At 120 Hz, a frame uses about 4% of
   its 8.3 ms. Nothing in the interface needs optimizing for frame rate.
2. **With vertical sync on, input waits most of a refresh.** The main loop reads input the moment
   DXGI can take a frame, which is right after the last frame went to the screen, builds the frame
   in about 0.3 ms, and then the frame waits for the next refresh. So of the input latency, about
   8 ms at 120 Hz is waiting in DXGI's queue, and under 1 ms is work. Pacing the frame to begin
   just before the refresh instead would cut the latency by up to 7 ms at 120 Hz. This is the one
   large win left, and it's a design change, not a hot path. See [Input latency](#input-latency).
3. **MeshModeler's per-edit work scales badly.** Triangulating 18,432 quads with smooth normals
   takes 25 ms, building their topology 15 ms, and subdividing a 1,152-face torus to that many
   4 ms. These run once an edit, not every frame, and the starting cube is tiny, but a model of a
   few thousand faces at 3 or 4 levels would hitch for hundreds of milliseconds an edit. The
   topology is built up to three times an edit, in collections that allocate for each vertex and
   edge. See [Meshes](#meshes).
4. **The rest is already fast:** text layouts hit their cache in 30 to 50 ns, the input reader turns
   a 1000 Hz mouse's frame of readings into events in 0.6 µs, the PerfBar's bookkeeping takes 0.1
   µs, a scene's JSON parses in 61 µs, and a model reads in 72 to 341 µs. A few small
   allocations a frame remain, listed under [Small things](#small-things), none worth more than a
   line.

## How it was measured

| What                 | Value                                                                       |
| -------------------- | --------------------------------------------------------------------------- |
| CPU                  | Intel Core Ultra 9 275HX, 8 performance and 16 efficiency cores, 24 threads |
| GPU                  | NVIDIA GeForce RTX 5090 Laptop GPU, driver 32.0.16.2114                     |
| Memory               | 64 GB                                                                       |
| Display for programs | CVVNY2, PA278QGV, 2560 x 1440 at 120 Hz, 100% scale, HDMI                   |
| Toolchain            | rustc 1.99.1, cargo 1.99.1, release profile                                 |
| Windows              | 11 Pro 10.0.26300                                                           |

Everything ran through [RunLightly](../Program/RunLightly/README.md), as coding agents must: on the
efficiency cores, at efficient clocks, at idle priority, with a third of the CPU's time, and with
the GPU's work after other processes'. So every CPU time here is a few times what a performance
core would take, the maximums include the job's stops, and the GPU's maximums include its
scheduler giving the GPU to something else for a frame. The minimums and medians are the costs
themselves. Run `benchmarks.exe` from a terminal, with no game running, for the performance cores'
numbers. The comparisons between benchmarks hold either way.

The benchmarks time each iteration on the performance counter, and the GPU's spans with
`GpuTimer`'s timestamp queries, so the GPU times are the GPU's own, apart from the CPU's recording
and submitting. The `ui` benchmarks run a program's frames through `Headless`, which runs the same
`update` as the window, without a window. The programs' frame rates come from running each program
with `RunInBackground --no-vsync` for 6 seconds and reading its PerfBar.

To reproduce:

```powershell
Program\RunLightly\RunLightly.ps1 cargo build --release -p benchmarks
Program\RunLightly\RunLightly.ps1 target\release\benchmarks.exe --json target\Benchmarks\Benchmarks.json
Program\RunInBackground\RunInBackground.ps1 UiShowcase --no-vsync --seconds 6
```

## Frames

A frame's update of each program, through `Headless`, with nothing changing (steady) and with the
pointer crossing the page every frame (moving), and the first frame from nothing (start).
Milliseconds, the median of up to 2000 iterations, with the minimum in parentheses.

| Program                   | Steady        | Moving        | Start       | Targets | Instances |
| ------------------------- | ------------- | ------------- | ----------- | ------: | --------: |
| UiShowcase                | 0.184 (0.172) | 0.391 (0.357) | 3.49 (2.79) |     100 |     2,027 |
| SimplePaintDemo           | 0.158 (0.153) |               |             |      25 |     2,644 |
| MeshModeler               | 0.109 (0.104) |               |             |      68 |       978 |
| MeshModeler, 4 levels     | 0.111 (0.105) | 0.232 (0.211) |             |      68 |       994 |
| GameInputDemo, 2 gamepads | 0.072 (0.069) |               |             |      13 |     1,105 |
| Veehiicuul                | 0.064 (0.061) |               | 3.72 (3.13) |      16 |       622 |
| MinimalSceneTest          | 0.015 (0.015) |               |             |       2 |       235 |
| SpinningCarTest           | 0.018 (0.017) |               |             |       2 |       276 |

Moving is per frame, of two a move. The cost follows the instances, about 70 ns an instance, plus
the widgets' own logic: UiShowcase's 2,027 instances are its 100 widgets' shapes, borders, shadows
and glyphs. A moving pointer doubles it, as what's hovered changes what every widget under the
pointer draws, and the tooltip's timer runs. The start is DirectWrite's factory and font
collection, the first layouts and the first frame, once.

Drawing the frame into a 2560 x 1369 target, on the GPU, and recording it on the CPU, which writes
the instances into the upload buffer and records the one draw call. Milliseconds, median (minimum).

| Program               | GPU draw      | CPU record    | Instances |
| --------------------- | ------------- | ------------- | --------: |
| UiShowcase            | 0.105 (0.097) | 0.012 (0.010) |     2,022 |
| SimplePaintDemo       | 0.159 (0.150) | 0.022 (0.016) |     2,640 |
| MeshModeler, 4 levels | 0.144 (0.138) | 0.012 (0.009) |       990 |
| GameInputDemo         | 0.104 (0.094) | 0.009 (0.006) |     1,099 |
| Veehiicuul            | 0.044 (0.042) | 0.017 (0.011) |       618 |
| MinimalSceneTest      | 0.020 (0.012) | 0.007 (0.005) |       213 |
| SpinningCarTest       | 0.021 (0.020) | 0.008 (0.006) |       254 |

The GPU draw is the program's 3D and the interface's one instanced draw together. Its cost is
fill: every card's background, border and shadow is a quad the card's size whose pixel shader runs
for every pixel, and the shadows integrate a Gaussian in a 6-step loop. SimplePaintDemo and
MeshModeler are the costliest because of their 3D views, and the full-window cards with elevation
under everything. At 0.16 ms it's 2% of a 120 Hz refresh; nothing to do here. The GPU's maximums
of about 7 ms in some rows are the scheduler giving the GPU to another process for a frame, which
RunLightly asks for.

The programs themselves, in the window, without vertical sync, under RunLightly:

| Program          | Frames a second | Milliseconds a frame |
| ---------------- | --------------: | -------------------: |
| UiShowcase       |             722 |                 1.39 |
| MeshModeler      |             785 |                 1.27 |
| SimplePaintDemo  |             776 |                 1.29 |
| Veehiicuul       |           1,671 |                 0.60 |
| MinimalSceneTest |           1,678 |                 0.60 |

The update, the recording and the GPU draw of UiShowcase add up to 0.3 ms of its 1.39, so about
1 ms a frame, and the whole 0.6 ms of the lightest programs, is the loop itself: the message pump,
`Present`, the fence's signal and the wait for the GPU to finish before the next frame begins,
which the loop does on purpose so that no frame waits behind another. With vertical sync, the
loop waits in `MsgWaitForMultipleObjects` instead of spinning, and UiShowcase showed 115 frames a
second on the 120 Hz display, which is the throttled job missing a refresh now and then.

## Input latency

The PerfBar measures the input latency from GameInput's reading to the display's flip, in three
parts: waiting, from the reading to the frame reading it; building, to the `Present` call; and
presenting, from `Present` to the screen. The benchmarks can't measure it, as there's no input
without a user, so this part is read from the code, with the frame costs above.

With vertical sync on, as every program starts:

1. The presenter has a frame latency waitable object and a maximum frame latency of 1, so the
   object signals once the last frame has left DXGI's queue, which with vertical sync is when the
   display took it at a refresh.
2. The main loop reads input and builds the frame right then, in about 0.3 ms, and presents it.
3. The frame then waits in the queue until the next refresh: at 120 Hz, about 8 ms.

So the input that a frame shows was read about 8.3 ms before the display takes it, of which 0.3 ms
is work. Input that comes during the wait is a frame older still. The way to cut this is to begin
the frame late instead of early: wait until the next refresh is a frame's work away, then read
input, build and present. The frame still makes the refresh, and its input is 0.5 to 1 ms old
instead of 8. This is what frame limiters like Special K's and NVIDIA Reflex do for games.

What's needed:

- When the next refresh is. [Library/PresentTiming](../Library/PresentTiming/README.md) already
  gives each frame's screen time on the performance counter, and the refreshes are their period
  apart, so the loop can predict the next one from the last few, and `DwmGetCompositionTimingInfo`
  or `IDXGIOutput::WaitForVBlank` can give it too.
- How long a frame takes: the PerfBar's building time, which the loop has, plus the GPU draw,
  plus a margin for the job's and Windows' jitter, say 1 to 2 ms at first.
- To wait precisely: `MsgWaitForMultipleObjects` with a timeout to within a millisecond, then a
  short spin on the counter, as the loop spins without vertical sync now.

The PerfBar's presenting time is the number to watch: it should fall from about a refresh to a
margin. Without vertical sync, the loop spins and presents as soon as it can, and with tearing in
borderless fullscreen a frame flips straight to the screen, so there's nothing to gain there.

## Text

| Benchmark                           | Median (minimum) ms | Per     |
| ----------------------------------- | ------------------: | ------- |
| Layout that the cache has, body     |   0.00004 (0.00003) | measure |
| Layout that the cache has, caption  |   0.00004 (0.00003) | measure |
| New layout, one line                |     0.0101 (0.0091) | layout  |
| New layout, wrapped at 400 px       |     0.0294 (0.0265) | layout  |
| Glyph quads from a cached layout    |   0.00003 (0.00002) | quad    |
| Rasterizing a glyph the atlas lacks |     0.0122 (0.0008) | glyph   |

The cache is what makes frames cheap: a frame that shows the same text as the last asks DirectWrite
for nothing, at about 40 ns a label, and a new label costs 10 µs once. Widgets whose text changes
every frame, like the PerfBar's numbers 10 times a second or a slider's value as it's dragged, lay
out anew each time, which is 10 µs each: fine. Layouts unused for 600 frames go, and the 2048 x
2048 atlas starts over when it's full, which rasterizes the visible glyphs again on one frame, at
about 12 µs each.

## Input and the PerfBar

| Benchmark                                                 | Median (minimum) ms |
| --------------------------------------------------------- | ------------------: |
| Input reader, 17 mouse readings and a held key, one frame |     0.0006 (0.0005) |
| PerfBar bookkeeping, one frame at 240 frames a second     |     0.0001 (0.0000) |
| Present tracker, one present's 9 events                   |     0.0010 (0.0009) |

A 1000 Hz mouse at 60 frames a second is 17 readings a frame, which the reader sorts and turns
into events in under a microsecond. The tracker, which follows every present in the process through
Windows' events on PresentTiming's thread, costs a microsecond a present.

## Meshes

Library/PolygonMesh, for MeshModeler. Milliseconds, median (minimum).

| Benchmark                         | Median (minimum) | Size             |
| --------------------------------- | ---------------: | ---------------- |
| Catmull-Clark, cube, 1 level      |    0.004 (0.004) | 24 faces out     |
| Catmull-Clark, cube, 2 levels     |    0.018 (0.018) | 96 faces out     |
| Catmull-Clark, cube, 3 levels     |    0.071 (0.069) | 384 faces out    |
| Catmull-Clark, cube, 4 levels     |    0.300 (0.282) | 1,536 faces out  |
| Catmull-Clark, torus, 1 level     |    0.723 (0.670) | 4,608 faces out  |
| Catmull-Clark, torus, 2 levels    |      4.12 (3.68) | 18,432 faces out |
| Triangulate, smooth normals       |      25.3 (21.9) | 18,432 quads     |
| Triangulate, flat normals         |       10.9 (9.9) | 18,432 quads     |
| Topology                          |      14.7 (12.8) | 18,432 quads     |
| Edges, as a sorted set            |      1.63 (1.51) | 18,432 quads     |
| Ray cast through the triangles    |    0.517 (0.515) | 36,864 triangles |
| Ray cast through the mesh's faces |      2.82 (2.72) | 18,432 quads     |
| UV sphere, 64 x 32                |    0.080 (0.077) |                  |

Subdivision is about 0.2 µs a face out, which is fine. Triangulation and the topology are the
costs, at about 1.4 µs and 0.8 µs a quad, and they're paid together: MeshModeler subdivides and
triangulates once an edit, and the smooth triangulation builds the topology again inside
`smooth_fans`, after each subdivision level built its own. For the torus at 2 levels that's about
45 ms an edit, which a user feels as a hitch while dragging a vertex, and it grows with the faces,
so a 2,000-face model at 4 levels, 512,000 faces, would take over a second an edit.

Why it's slow, from the code:

- `Topology::new` collects the edges into a `BTreeSet`, then indexes them in a `HashMap` with the
  standard library's SipHash, and builds a `Vec<Vec<usize>>` for each vertex's edges and faces and
  each edge's faces: about 100,000 small allocations for the torus. Sorting the edges in a `Vec`
  and finding each with a binary search, or a hand-written FNV hash as `Ui::id` and `Fonts` use,
  and one flat array with an offset a vertex, as a compressed sparse row, would take most of it.
- `triangulate` calls `face_triangles` for each face, which returns a `Vec` a face, and `topology`
  again for smooth normals. Taking a `&Topology` that the caller built, and writing triangles into
  a buffer, would halve the smooth triangulation.
- `subdivide` builds a `Topology` a level, and `vertex_point` collects three `Vec`s a vertex. The
  levels could share the edge lists, since each level's topology is the last's, refined.
- Ray casts through the faces take 2.8 ms because each face is triangulated on the way. The
  triangles' own ray cast, which MeshModeler uses every frame for what's under the pointer, is 0.5
  ms for 36,864 triangles, a brute-force walk that's fine until a model is much larger, when a
  bounding volume hierarchy would be the next step.

None of this matters for the models MeshModeler has today. It will for the first real one.

## Files

| Benchmark                | Median (minimum) ms | Size                     |
| ------------------------ | ------------------: | ------------------------ |
| glTF, SlopeCar.glb       |       0.072 (0.070) | 60 KB, 1,884 triangles   |
| glTF, BigCone.glb        |       0.341 (0.329) | 384 KB, 21,376 triangles |
| JSON, parse Ribeye scene |       0.061 (0.059) | 19 KB                    |
| JSON, write Ribeye scene |       0.103 (0.102) | 19 KB                    |

All loaded at start, once. Veehiicuul's start of 3.7 ms includes them.

## Small things

Allocations and lookups a frame that the benchmarks show are too small to measure, listed so
that no one looks for them again:

- `InputDevices::reading` clones the whole `Device`, with its three `String`s, to read its `id`,
  once a reading: 17 clones a frame with a 1000 Hz mouse at 60 frames a second, a few microseconds.
  `device` could give the index or the id alone.
- `InputReader::frame` clones each `Reading`, with a keyboard's `Vec` of keys, into the sorted
  list. The sort could be of indices.
- `Painter::text_in` measures the text and then draws it, two lookups of the layout cache, each
  hashing the text: about 80 ns a label, twice.
- The main loop asks `program.title()` for a new `String` a frame, to see whether it changed.
- `Ui::state`, which scripts save at a capture, checks each target's name against every one before
  it: quadratic in the targets, only at captures.
- `Fonts::begin_frame` purges old layouts on every 600th frame.

## Building and testing

| What                                                    | Time         |
| ------------------------------------------------------- | ------------ |
| Clean release build of the workspace, 28 crates, 8 jobs | 13.1 s       |
| Release build with nothing changed                      | 1.9 s        |
| `cargo test --workspace --release`, each GPU test       | 0.3 to 0.6 s |
| `cargo test --workspace --release`, each other test     | under 0.1 s  |
| The programs' exes, release                             | 12.1 MB      |

The only dependency outside the repository is `windows-core`, so a clean build is seconds, even
on the efficiency cores. The generated bindings in Library/WindowsApi, 9,185 lines, are the
largest crate.

## What to do, in order

1. **Pace frames to the refresh with vertical sync on**, in `Library/UserInterface`'s main loop and
   `Library/Graphics`' presenter, and check it with the PerfBar's presenting time. Worth about 7
   ms of input latency at 120 Hz, which is more than everything else here together.
2. **Build the topology once an edit in MeshModeler and reuse it**, and flatten its collections,
   when a model that's larger than a cube is modeled. Worth tens to hundreds of milliseconds an
   edit then.
3. Run `Benchmarks` after changing a hot path, and compare with `target\Benchmarks\Benchmarks.json`
   from before, with the same `Lightly` in the environment line.
