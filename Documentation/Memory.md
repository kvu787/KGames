# Memory

> These are measurements of KGames as it was written in Rust, which it was until it was ported to
> C++. The toolchain, the commands to reproduce them and the names of files are that version's, and
> "the Rust heap" is the program's own heap. [CppVersusRust.md](CppVersusRust.md) has the heap's
> numbers for the C++ and the Rust side by side. What's said here about a window's memory, the
> GPU's and the driver's holds for both. Today, `Program\Build\Build.ps1 --release Benchmarks`
> builds `target\Release\Benchmarks.exe`.

Where the programs' memory goes, measured on 2026-11-06 at commit `5f03bcb` with
[Benchmarks](../Program/Benchmarks/README.md)' memory groups,
[RunInBackground](../Program/RunInBackground/README.md), Windows' process and GPU counters, and
read from the code. [Performance.md](Performance.md) is the same for time. The short of it:

1. **A window costs about 80 MiB of system memory and 60 to 90 MiB of the GPU's, and almost none
   of it is ours.** Each program's own heap is under 1 MiB once it has started: UiShowcase's is
   496 KiB, Veehiicuul's 983 KiB, MinimalSceneTest's 75 KiB. The rest is Windows' DLLs, Direct3D's
   device, the display driver and DirectWrite, which every window pays whatever it draws. See
   [The whole process](#the-whole-process).
2. **Nothing leaks.** 1,500 frames of UiShowcase, after the first 800, left the heap 0.4 KiB
   *smaller* than they found it, and never held more than 5 KiB above it. A program left running
   kept the same 83.6 MiB of working set for three minutes. The caches that could grow are purged
   or bounded, except the one below. See [What grows](#what-grows).
3. **Frames churn, though: 692 allocations and 15 KB a frame in UiShowcase, about 7 allocations a
   widget**, nearly all of them strings of widgets' names, built again every frame and freed at
   the end of it. Freed in the same frame, they cost no memory; what they cost is time, and they
   buy little, since each one exists to be hashed into an id, or read by a script that usually
   isn't running. See [What a frame churns](#what-a-frame-churns).
4. **The GPU's memory is pictures, and they're all as big as they must be**: the swap chain's two
   2560 x 1369 buffers, 26.7 MiB; a 3D program's depth buffer, 13.4 MiB; SimplePaintDemo's 4x
   multisampled preview, 27.2 MiB, which holds the largest single resource in the repo and is the
   size of the card it fills. The one that isn't earned is the glyph atlas: 4 MiB in every
   program, whose glyphs would fit in a few percent of it. See
   [The GPU's memory](#the-gpus-memory).
5. **Meshes are the only data that scales, and they scale badly.** A `Vec<u32>` for each face
   costs a quad 60 bytes where its four indices are 16, the topology 358 bytes a quad, and
   building either tens of thousands of small allocations: 95,541 for an 18,432-quad topology.
   MeshModeler's undo history keeps 200 whole copies of the mesh, so a model of a few thousand
   faces would hold tens of megabytes of history, and a large one hundreds. See
   [Meshes](#meshes).

## How it was measured

| What                 | Value                                                                       |
| -------------------- | --------------------------------------------------------------------------- |
| CPU                  | Intel Core Ultra 9 275HX, 8 performance and 16 efficiency cores, 24 threads |
| GPU                  | NVIDIA GeForce RTX 5090 Laptop GPU, driver 32.0.16.2114, 23,369 MiB budget  |
| Memory               | 64 GB                                                                       |
| Display for programs | CVVNY2, PA278QGV, 2560 x 1440 at 120 Hz, 100% scale, HDMI                   |
| Toolchain            | rustc 1.99.1, cargo 1.99.1, release profile                                 |
| Windows              | 11 Pro 10.0.26300                                                           |

The heap counts come from a global allocator in Benchmarks that counts allocations, frees, bytes
and the live bytes' peak. It counts nothing until a memory benchmark turns it on, so the timing
benchmarks are unaffected: every one of them, in the same run, came out at or below
[Performance.md](Performance.md)'s time, by up to 4%, and UiShowcase's steady frame had the same
0.172 ms minimum.

It counts the Rust heap, and only that. DirectWrite's text layouts, Direct3D's resources, the
display driver's allocations and GameInput's are in their own heaps, which it can't see. Those
show up in three other numbers, all of which are here too: the process's working set and private
commit, from `GetProcessMemoryInfo`; the GPU's memory that the process has reserved, from
`IDXGIAdapter3::QueryVideoMemoryInfo`; and Windows' `GPU Process Memory` counters for a program
running in its own window.

Everything ran through [RunLightly](../Program/RunLightly/README.md), as coding agents must,
which changes times but not what's allocated. The benchmarks' programs run through `Headless`:
the same `update` as a real window, with the same widgets, but no window, no GameInput and no
present timing, so the per-frame counts leave out what a real run's input path allocates, which
[What a frame churns](#what-a-frame-churns) says.

To reproduce:

```powershell
Program\RunLightly\RunLightly.ps1 cargo build --release -p benchmarks
Program\RunLightly\RunLightly.ps1 target\release\benchmarks.exe --filter memory --json target\Benchmarks\Memory.json
Program\RunInBackground\RunInBackground.ps1 UiShowcase --keep-open
```

and, for a program that RunInBackground left running, with its process ID:

```powershell
Get-Process -Id <id> | Select-Object WorkingSet64, PeakWorkingSet64, PrivateMemorySize64
```

```powershell
(Get-Counter "\GPU Process Memory(pid_<id>*)\Dedicated Usage").CounterSamples.CookedValue
```

## The whole process

Each program in a maximized window on the 2560 x 1440 display, with vertical sync, eighteen
seconds after it opened, under RunLightly. Mebibytes.

| Program          | Working set | Private commit | GPU, dedicated | GPU, shared |
| ---------------- | ----------: | -------------: | -------------: | ----------: |
| Veehiicuul       |        83.8 |          133.8 |           76.7 |        29.2 |
| UiShowcase       |        83.6 |          118.8 |           61.8 |        29.4 |
| SimplePaintDemo  |        82.3 |          147.7 |           91.6 |        28.6 |
| MeshModeler      |        82.2 |          129.4 |           76.1 |        28.1 |
| MinimalSceneTest |        80.1 |          130.7 |           79.9 |        27.7 |
| GameInputDemo    |        79.9 |          115.2 |           61.8 |        27.9 |

The working set is the physical memory the process has; the private commit is everything it has
asked for that isn't shared, paged in or not; dedicated is the GPU's own memory, and shared is
system memory that the GPU reads across PCIe. Each program also has 16 threads, of which ours are
the main one and [PresentTiming](../Library/PresentTiming/README.md)'s two; the rest are
Direct3D's and the driver's.

Every program lands within four mebibytes of 80, whatever it does, because the floor is what a
window with Direct3D 12 and DirectWrite costs on Windows. Each has the same 66 modules mapped
into it, 284 MiB of them, of which the display driver's two are 204 MiB: `nvgpucomp64.dll` at 113
MiB and `nvwgf2umx.dll` at 91 MiB. Only the pages they touch are in the working set, and that,
with the device, the fonts and Windows' own, is the 80 MiB. The programs' own data, which the
next sections count, is under 1 MiB of it. There's nothing to win here without giving up Direct3D
or DirectWrite.

The differences between the GPU's numbers are the programs' own pictures, and they come to within
a few mebibytes of what the next section computes: SimplePaintDemo's 91.6 is 61.8 plus its 27.2
MiB preview, MinimalSceneTest's 79.9 is 61.8 plus a 13.4 MiB depth buffer, and MeshModeler's 76.1
and Veehiicuul's 76.7 are 61.8 plus the depth buffer, their buffers and their meshes. The 61.8
that they share is the swap chain's 26.7 MiB, the glyph atlas's 4 MiB, and 31 MiB of device and
driver state, of which an idle device with no window is 15.6.

## The GPU's memory

What each thing costs, from its format and size in the code. The two window setups differ only in
height, 1369 maximized and 1440 in borderless fullscreen, so each picture differs by 5%.

| What                                              | Maximized | Fullscreen | Where     |
| ------------------------------------------------- | --------: | ---------: | --------- |
| Swap chain, 2 buffers, BGRA8                      |  26.7 MiB |   28.1 MiB | Local     |
| Depth buffer, D32, for a 3D view                  |  13.4 MiB |   14.1 MiB | Local     |
| Glyph atlas, 2048 x 2048, R8                      |   4.0 MiB |    4.0 MiB | Local     |
| Interface upload buffer, at least                 |   1.0 MiB |    1.0 MiB | Non-local |
| SimplePaintDemo's preview, 1152 x 1239, 4x and 1x |  27.2 MiB |   27.2 MiB | Local     |
| Veehiicuul's constants, 512 instances of 1,536 B  |   0.8 MiB |    0.8 MiB | Non-local |
| Veehiicuul's models, SlopeCar and BigCone         |   0.4 MiB |    0.4 MiB | Local     |
| MeshModeler's geometry, a cube at 4 levels        |   0.3 MiB |    0.3 MiB | Non-local |

And what the device reports a program's drawing adding, over an idle device with a 2560 x 1369
target already made, from `memory/gpu`. Mebibytes.

| Program          | Local | Non-local | What it is                                             |
| ---------------- | ----: | --------: | ------------------------------------------------------ |
| UiShowcase       |  9.88 |      5.02 | The atlas, the upload buffer, and the driver's first   |
| SimplePaintDemo  | 33.75 |      1.82 | The atlas and the 27.2 MiB preview                     |
| MeshModeler      | 18.25 |      1.37 | The atlas, the depth buffer, the geometry buffer       |
| GameInputDemo    |  4.00 |      1.48 | The atlas alone                                        |
| Veehiicuul       | 18.39 |      2.47 | The atlas, the depth buffer, the models, the constants |
| MinimalSceneTest | 17.86 |      1.00 | The atlas, the depth buffer                            |
| SpinningCarTest  | 17.93 |      1.00 | The atlas, the depth buffer                            |

All of it goes back when the program's objects do, which the device confirms: dropping each one
freed what it had taken, to the tenth of a mebibyte. The exception is UiShowcase's row, which was
measured first: it took 9.88 MiB and gave back 4.00, because about 6 MiB of it is state the driver
sets up the first time anything draws and keeps afterwards. GameInputDemo's row, measured fourth,
is the clean one: 4.00 MiB, which is the atlas exactly.

Three things worth knowing:

- **The glyph atlas is 4 MiB in every program, and almost all of it is empty.** It's
  2048 x 2048 one-byte pixels, 4.19 million of them, made once in `Renderer::new`. A glyph of
  body text is about 10 x 14 pixels, so even 2,000 distinct glyphs, over every style and size an
  interface uses, fill 7% of it. GameInputDemo's row above is the atlas and nothing else: 4 MiB
  for thirteen widgets' labels. Starting it at 512 x 512 and growing it when the shelves run out
  would give back about 3.9 MiB a program. The packer already starts over when it's full, and the
  layouts still in use rasterize their glyphs again on the next frame, so growing it is that same
  path with a bigger texture.
- **F11 makes a new depth buffer.** `DepthBuffer::view` notices the frame's size changed and
  creates the new texture before dropping the old, so a program holds 27 MiB of depth for a moment
  as it goes fullscreen. That's fine on a 24 GB GPU; it's the kind of thing to keep in mind on
  anything smaller.
- **SimplePaintDemo's preview holds the largest single resource in the repo**, and it's earned:
  the card shows the sphere at 1152 x 1239, and 4x multisampling of a target that size is 21.8
  MiB, with 5.4 MiB more for the resolved copy. It's drawn only when the material or the
  background changes, not every frame. Two samples instead of four would save 10.9 MiB, and the
  sphere's edge would show it.

## The heap at start

What each program holds on the Rust heap once it has started and drawn its first frame, through
`Headless`, from `memory/frames`. Kept is the footprint; peak is the most it held while starting.

| Program               | Allocations | Bytes allocated | Peak KiB | Kept KiB |
| --------------------- | ----------: | --------------: | -------: | -------: |
| Veehiicuul            |       5,489 |       4,763,305 |  1,429.9 |    982.8 |
| SimplePaintDemo       |       1,364 |       1,182,046 |    606.2 |    596.1 |
| MeshModeler, 4 levels |      20,840 |       2,715,128 |  1,021.2 |    531.2 |
| UiShowcase            |       3,052 |         906,667 |    496.6 |    495.7 |
| GameInputDemo         |       1,661 |         685,369 |    356.2 |    354.5 |
| MeshModeler           |       2,839 |         540,402 |    276.7 |    275.8 |
| SpinningCarTest       |         552 |         138,051 |     77.8 |     77.7 |
| MinimalSceneTest      |         506 |         134,315 |     75.2 |     75.0 |

Veehiicuul's 983 KiB is mostly its two models, 837 KiB of the 983 (see [Files](#files)).
MeshModeler at 4 levels holds its subdivided mesh and its triangles; the 20,840 allocations and
2.7 MB it churns through to get there are the subdivision's, and they're the same ones
[Meshes](#meshes) counts. UiShowcase's 496 KiB is the widest interface in the repo, 100 widgets
and 2,022 instances, and it's half a mebibyte.

The peak is barely above the kept in every row but MeshModeler's and Veehiicuul's, which means
starting allocates almost exactly what it keeps: no large temporary buffers, no parse-then-copy.

## What a frame churns

A steady frame's allocations, all of which it frees before the next one, from `memory/frames`.
The moving row counts two frames an iteration, as a pointer move runs two; a widget there is of
one frame.

| Program                    | Targets | Allocations |  Bytes | A widget |
| -------------------------- | ------: | ----------: | -----: | -------: |
| UiShowcase                 |     100 |       691.8 | 15,115 |      6.9 |
| UiShowcase, pointer moving |     100 |     1,393.8 | 31,026 |      7.0 |
| MeshModeler, 4 levels      |      68 |       508.0 | 13,159 |      7.5 |
| Veehiicuul                 |      16 |       278.0 | 11,903 |     17.4 |
| SimplePaintDemo            |      25 |       181.1 | 10,713 |      7.2 |
| GameInputDemo              |      13 |       113.0 |  6,605 |      8.7 |
| SpinningCarTest            |       2 |        31.2 |  1,759 |        - |
| MinimalSceneTest           |       2 |        25.2 |    968 |        - |

The cost is a widget, not a pixel or an instance: about 7 allocations and 150 bytes a widget a
frame, in five programs that have nothing else in common. Veehiicuul's 17 a widget is its track,
its terrain and its vehicles on top of its 16 widgets, and the two scene tests are nearly all
fixed cost. A moving pointer changes what's drawn but not what's allocated: 697 allocations a
frame against 692.

They're names. Every frame:

- `Ui::register` keeps each target's name as a `String`, so that scripts and tests can point at
  widgets by name, and `set_label` keeps another for what it shows. The frame's targets are
  cleared and built again, so that's one or two allocations a widget a frame.
- Compound widgets build their parts' names with `format!`: `"{name}/Draft"`, `"{name}/Up"`,
  `"{name}/Down"` in a number box, `"{name}/List"` and `"{name}/{index}"` in a combo box,
  `"{name}/Spectrum"` in a color picker. Each is an allocation whose only use is to be hashed
  into an `Id` and dropped.
- `Ui::cover` builds `format!("Cover:{x}:{y}:{width}:{height}")` for each card that covers what's
  under it, and the main loop asks `program.title()` for a fresh `String` a frame to see whether
  it changed.

None of this costs memory, since it's all freed in the same frame, and 15 KB a frame is nothing
for a 64 GB machine. What it costs is time, inside the frame times that
[Performance.md](Performance.md) measures, and it buys nothing: every one of those strings exists
to be hashed into an `Id`, or to be read by a script that usually isn't running. Two changes
would take most of them:

- An `Id` from a parent and a part, hashing the two without joining them, since `id` is already
  FNV over bytes. That removes every `format!` that exists only to make an id.
- Names in a per-frame arena: one `String` a frame that targets point into by range, instead of
  one allocation each. Or keep names only when something asks for them, since outside a scripted
  run or a test nothing reads them.

A real run adds what `Headless` doesn't do. `InputDevices::reading` clones the whole `Device`,
with its two `String`s, to read its `id`, once a reading: with a 1000 Hz mouse at 120 frames a
second that's about 8 readings and 16 allocations a frame. `InputReader::frame` clones each
`Reading`, including a keyboard's `Vec` of held keys, into its sorted list. Both are listed in
[Performance.md](Performance.md)'s small things, and the fix is the same: pass the index or the
id, and sort indices.

## What grows

Run UiShowcase for 1,500 frames, after 800 that fill its caches and cross the purges:

| Benchmark                        | Allocations a frame | Bytes a frame | Peak KiB | Kept KiB |
| -------------------------------- | ------------------: | ------------: | -------: | -------: |
| 1,500 frames after the first 800 |               690.0 |      14,854.5 |      5.0 |     -0.4 |

It ends 0.4 KiB below where it started, and never holds more than 5 KiB above it. The same
program left running in its window held the same memory for three minutes:

| Running time | Working set | Private commit | GPU dedicated |
| -----------: | ----------: | -------------: | ------------: |
|          0 s |    83.6 MiB |      118.6 MiB |      61.8 MiB |
|         30 s |    83.6 MiB |      118.6 MiB |      61.8 MiB |
|         60 s |    83.6 MiB |      118.5 MiB |      61.8 MiB |
|        120 s |    83.6 MiB |      118.6 MiB |      61.8 MiB |
|        180 s |    83.6 MiB |      118.6 MiB |      61.8 MiB |

That's with the PerfBar making new text ten times a second and PresentTiming following every
present, which are the two things in a window that produce new data forever. Both are bounded:

- The **text cache** lets go of layouts unused for 600 frames, on every 600th frame, so what it
  holds is what the last 600 frames laid out. At 120 frames a second that's five seconds of text.
- The **present tracker** follows presents in a ring of 2,048, and drops the oldest when it wraps.
  The PerfBar's own deques are trimmed by time, to a one-second window and a few seconds of
  slack, and the presents it has asked about but not heard back on are dropped after they're late.

The one thing that isn't bounded is `Ui::memory`, where widgets keep state between frames by id:
a text field's caret, a scroll offset, a menu's state. Nothing ever removes an entry. Today's
programs name their widgets from a fixed set, so the map settles at a few hundred entries and
there's nothing to see. A program that names a widget after what it shows, which is the natural
thing to do for a list of files or a vertex being created, would grow it forever. Measured with a
program that gives 20 widgets new names every frame:

| Benchmark               | Allocations a frame | Bytes a frame | Peak KiB | Kept KiB |
| ----------------------- | ------------------: | ------------: | -------: | -------: |
| `Ui::memory`, new names |                49.4 |       4,197.0 |  4,037.0 |  2,730.9 |

That's 93 bytes a name that never goes, so 20 new names a frame is 1.8 KiB a frame, 110 KiB a
second at 60 frames a second, and 400 MB in an hour. `Ui::memory` should be purged now and then,
like the text cache, on a frame of its own, keeping what the last frame's targets used.

## Text

The text cache is the largest thing the interface keeps, and most of it isn't on our heap.
`Fonts` holds the layouts; DirectWrite holds what's in them.

| Benchmark                   | Per      | Allocations | Bytes | Kept KiB |
| --------------------------- | -------- | ----------: | ----: | -------: |
| `Fonts::new`                | a cache  |        22.0 | 1,131 |      0.4 |
| 2,000 layouts, cached       | a layout |         8.5 | 1,267 |  1,539.4 |
| 400 glyphs, rasterized      | a glyph  |         2.6 |   334 |     74.3 |
| The purge, after 600 frames | a purge  |         0.0 |     0 | -1,430.0 |

A cached layout costs about 790 bytes of our heap, in 8.5 allocations: the text, the placed
glyphs, and the entry. DirectWrite's `IDWriteTextLayout` behind it costs much more, and the only
place it shows is the process. Laying out 1,500 more labels after the first 500, which leaves out
the font files that the first ones load, raised the private commit by 19.8 KB a layout; counting
all 2,000 instead gave 19.1 KB. Some of that is DirectWrite's heap growing in chunks rather than a
layout's own size, but the order is right: 2,000 cached layouts are about 38 MB of commit, against
1.5 MiB of ours.

The purge gives back 1,430 KiB of the 1,539 that 2,000 layouts held, and DirectWrite's share with
it. That's why the cache has a lifetime at all, and why a program that shows changing text, like
the PerfBar's numbers, stays flat.

A rasterized glyph's pixels sit on our heap, in `Atlas::uploads`, until the renderer copies them
into the atlas texture, which it does on the next frame it draws. A test or a benchmark that
never draws keeps them all: 74 KiB for that row's 400 glyphs, which are ten digits at forty
sizes, counting the forty layouts and the atlas's index of what it has along with the pixels.

## Meshes

Library/PolygonMesh, for MeshModeler, on a torus subdivided twice: 18,432 quads, 18,432 vertices,
36,864 edges.

| Benchmark                      | Allocations | Bytes allocated | Peak KiB | Kept KiB | A quad |
| ------------------------------ | ----------: | --------------: | -------: | -------: | -----: |
| The mesh                       |      59,867 |       5,105,552 |  3,269.0 |  1,080.0 |   60 B |
| Its topology                   |      95,541 |       9,964,712 |  6,740.0 |  6,452.0 |  358 B |
| Its triangles, flat            |      55,378 |      22,749,132 | 10,280.2 |  4,608.0 |  256 B |
| Subdividing to it, a run       |      59,867 |       5,105,552 |  3,269.0 |        0 |      - |
| Triangulating it smooth, a run |     150,902 |      22,391,940 |  7,964.0 |        0 |      - |

The shape of all of this is `Vec<Vec<_>>`:

- A `Face` is a `Vec<u32>` of its vertices and a material. A quad's four indices are 16 bytes;
  the `Vec` header is 24 more, and its own allocation at least 16, so a quad costs 60 bytes
  instead of about 20, and building a mesh does an allocation a face. Faces in one flat index
  array, with an offset a face, as a compressed sparse row, would be a third of the memory and
  one allocation.
- A `Topology` is four of them: `edge_faces`, `face_edges`, `vertex_edges` and `vertex_faces` are
  all `Vec<Vec<usize>>`, plus a `HashMap<Edge, usize>` of every edge. That's 358 bytes a quad and
  95,541 allocations, about 5 an edge. The same flattening applies, and it's the same change
  [Performance.md](Performance.md) asks for to make the topology faster.
- Smooth triangulation churns 22 MB and 150,902 allocations a call, keeps nothing, and builds the
  topology again inside itself after each subdivision level already built one.
- `Triangles` holds 4,608 KiB where its data is 2,736 KiB, because its three `Vec`s are filled by
  pushing and double when they're full: 73,728 vertices of 28 bytes sit in room for 131,072,
  110,592 indices in room for 131,072, and 36,864 face numbers in room for 65,536. The corner
  count is known before the loop, so `Vec::with_capacity` there, or a `shrink_to_fit` at the end,
  gives back 1.8 MiB on this mesh.

All of which matters because of the undo history. `Document::commit` keeps a `Snapshot` of the
whole mesh, the paints and the selection, up to `HISTORY = 200` steps. The snapshot is of the
control mesh, not the subdivided one, so today's cube is nothing. At 60 bytes a quad a model of
5,000 faces is 300 KiB a step and 60 MiB of history, and one of 50,000 faces is 600 MiB. Three
ways out, in the order they're worth doing: flatten `Face` first, which halves every snapshot;
bound the history by bytes as well as steps; and keep what changed instead of the whole mesh for
the edits that touch a few faces.

## Files

| Benchmark              | Allocations | Bytes allocated | Peak KiB | Kept KiB | On disk |
| ---------------------- | ----------: | --------------: | -------: | -------: | ------- |
| glTF, SlopeCar.glb     |         523 |         370,751 |    167.5 |    107.7 | 60 KB   |
| glTF, BigCone.glb      |         573 |       2,498,359 |  1,044.4 |    729.0 | 384 KB  |
| JSON, the Ribeye scene |         760 |          75,936 |     47.0 |     47.0 | 19 KB   |

A model takes about twice its bytes on disk in memory, which is what separate `Vec`s of positions,
normals, materials and indices come to, and it peaks at about three times while it's read, since
the file's bytes are still held. A parsed scene is 2.5 times its text, which is what
`Value::Object(Vec<(String, Value)>)` costs; it's parsed once at start and written back on save.
Both are small enough that nothing here is worth changing.

## Upload buffers, and one frame at a time

Every upload buffer in the repo is written by the CPU and read by the GPU with nothing between
them: the interface's instances, MeshModeler's geometry and frame data, Veehiicuul's constants,
SimplePaintDemo's and MinimalSceneTest's. That's safe only because the main loop keeps exactly
one frame in flight: the presenter's maximum frame latency is 1, and `poll_ready` waits for the
GPU to finish the last frame before the next one begins, so by the time a frame writes into a
buffer, the frame that read it is done.

[Performance.md](Performance.md)'s first recommendation is to pace frames to the refresh instead
of starting them as early as possible. That's a change to *when* a frame begins, not to how many
are in flight, and it has to stay that way. Allowing two frames in flight would make every one of
these buffers a race, and fixing them means a second of each: 1 MiB more for the interface, 0.8
MiB for Veehiicuul, and a rule to remember in every program that draws.

The buffers themselves grow by half again when a frame needs more, and never shrink. The
interface's starts at 1 MiB, which holds about 9,000 instances at 112 bytes each; the widest
program in the repo draws 2,640. MeshModeler's geometry buffer is the one that follows the data:
a cube at 4 levels is 1,536 quads, 6,144 vertices of 28 bytes and 9,216 indices, so about 0.3 MiB,
and it grows with the model, in system memory that the GPU reads every frame it changes.

## What to do, in order

1. **Purge `Ui::memory` every 600 frames, like the text cache**, keeping what the last frame's
   targets used. A dozen lines in `Ui::begin_frame`, and it closes the only unbounded thing in the
   interface before a program runs into it.
2. **Flatten `Face` and `Topology` into index arrays with offsets.** Two thirds of a mesh's
   memory, most of a topology's, and about 150,000 allocations an edit in MeshModeler, which is
   also where [Performance.md](Performance.md)'s second recommendation points. Do this before
   MeshModeler's history meets a real model.
3. **Bound MeshModeler's undo history by bytes**, not only by 200 steps, once meshes are
   flattened, and give `Triangles` its capacity up front, which is 1.8 MiB on an 18,432-quad
   mesh.
4. **Stop allocating names a frame**: an `Id` from a parent id and a part without `format!`, and
   targets' names in a per-frame arena or kept only when something reads them. About 700
   allocations a frame in UiShowcase, which is time more than memory.
5. **Start the glyph atlas small and grow it.** 3.9 MiB of the GPU's memory a program, for a
   path the packer already has.
6. Run `Benchmarks --filter memory` after changing any of this, and compare with
   `target\Benchmarks\Memory.json` from before, with the same `Lightly` in the environment line.
