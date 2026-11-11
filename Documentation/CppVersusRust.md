# C++ versus Rust

KGames was written in Rust. It was then rewritten in C++, compiled by MSVC, to see how the two
compare in performance and in the structure of the code: all 14 libraries, all 11 programs and
all of their tests, file for file and test for test, about 48,000 lines of Rust into about 59,000
of C++. Both pass the same tests, run the same input scripts and draw the same pixels, so this
compares two working implementations of the same programs, not ideals, as
[UiArchitectures.md](UiArchitectures.md) did for the library's architectures. The Rust is `main`,
as it was at commit `e171759`. The C++ is the `cpp-rewrite` branch, where this was written and
measured, and the C++ files that this names, like `Library/Base/Sum.h`, are there. KGames went on
in Rust, as [What was decided](#what-was-decided) says.

The short of it:

1. **They do the same.** All 525 of the Rust's tests pass in C++, and the 52 input scripts of the
   eight programs that have them save the same frames and states, byte for byte. See
   [They behave the same](#they-behave-the-same).
2. **A frame costs about the same, and the C++'s is a little cheaper.** A frame's update is 6% to
   16% faster in C++ for six of the seven programs and 5% slower for the seventh. The GPU's work
   is identical. See [Performance](#performance).
3. **Run as a person runs them, nothing tells them apart.** In their windows, with vertical sync,
   both draw 120 frames a second without dropping one, answer the mouse in the same 9
   milliseconds, and use the same CPU time, memory and GPU. Without vertical sync the Rust draws
   0% to 5% more frames a second: in a window the C++ takes up to 9% longer to build a frame, the
   reverse of the benchmarks. See [As a person runs them](#as-a-person-runs-them).
4. **Code that builds data structures was up to twice as slow as ported.** Meshes, glTF and JSON
   took 1.4 to 2.1 times the Rust's time when the C++ was the Rust written line for line. Five
   changes of a few lines each brought most of the mesh code within 16% and made JSON faster than
   the Rust. Reading glTF is still 1.4 to 1.7 times as slow, and one mesh function twice. The
   causes were the standard library's containers and MSVC's inlining more than the language.
5. **Memory is the same** for the process and for the GPU. On its own heap the C++ allocates
   half to nine tenths as often in a frame, and 1.1 to 1.9 times the bytes. See
   [Memory](#memory).
6. **The C++ is a quarter more code**: 1.24 times the lines and 1.30 times the tokens, in 419
   files where the Rust had 255, not counting the 1,294 lines of `Library/Base`, which has what
   C++'s standard library lacks. Sum types and headers are most of it. What calls Windows got
   shorter. See [The code](#the-code).
7. **C++ let through what Rust refuses or stops at.** Thirteen kinds of mistake compiled without a
   warning, and tests, a comparison with the Rust's output and three reviews found them. Two
   could have read or written memory that wasn't theirs. Turning the standard library's checks
   on costs nothing that shows, but in the glTF reader, where it's 2.3 times. See
   [What C++ lets through](#what-c-lets-through).
8. **The C++ builds ten times slower**: 335 seconds for everything from nothing, where the Rust
   took 31, and 171 seconds after one source file changed, where the Rust took 20. Unthrottled
   it's 117 seconds against 13, and 63 against 7. See
   [Building and testing](#building-and-testing).

So for these programs the two languages give the same frames at the same cost. What C++ adds is a
few percent on a frame in the benchmarks, which doesn't show in a window, and Windows' own
headers. What it takes is a quarter more code to read, a
build that's ten times as long, and the compiler's checks, which tests, reviews and a hardened
build have to stand in for.

KGames stayed in Rust, and the Rust took what the comparison showed about it, which made its
frames a quarter to a third cheaper. See [What was decided](#what-was-decided).

## What was compared

| What                | Rust                                             | C++                                                     |
| ------------------- | ------------------------------------------------ | ------------------------------------------------------- |
| Compiler            | rustc 1.99.1, which generates code with LLVM     | MSVC 19.51, of Visual Studio 2026 (18.11), toolset v145 |
| Optimized build     | Cargo's release profile: `opt-level = 3`, no LTO | `/O2`, with `/GL` and `/LTCG`: the whole program        |
| Standard library    | Rust's                                           | MSVC's, at `/std:c++latest`, and `Library/Base`         |
| Checks when it runs | Every index, every slice, every `unwrap`         | None. `--hardened` turns the standard library's on      |
| Windows' API        | Bindings that `windows-bindgen` generates        | The Windows SDK's headers                               |
| Build               | Cargo, a `Cargo.toml` a package, `build.rs`      | MSBuild, a `.vcxproj` a project, `KGames.props`         |
| Tests               | `#[test]` and `cargo test`                       | `Library/Testing`'s `TEST` and `Program\RunTests`       |

Two more builds were measured, to tell the language from the settings: the C++ hardened, with
`_MSVC_STL_HARDENING=1`, where the standard library checks every index into a vector, a span, a
string or an array and every use of an empty optional, and stops the program at a wrong one, as
Rust does; and the Rust with link-time optimization (`lto = "fat"`, one codegen unit), which is
what `/LTCG` is to the C++. The C++ wasn't built with clang-cl, which would have told MSVC's code
generation from LLVM's with the same source: Visual Studio's ClangCL toolset isn't installed here.

| What    | Value                                                                       |
| ------- | --------------------------------------------------------------------------- |
| CPU     | Intel Core Ultra 9 275HX, 8 performance and 16 efficiency cores, 24 threads |
| GPU     | NVIDIA GeForce RTX 5090 Laptop GPU                                          |
| Memory  | 64 GB                                                                       |
| Windows | 11 Pro 10.0.26300                                                           |
| Date    | 2026-11-10                                                                  |

Everything ran through [RunLightly](../Program/RunLightly/README.md), as coding agents must: on
the efficiency cores, at idle priority, with a third of the CPU's time. So every time here is a
few times what a performance core would take, for both languages alike. The ratios are what to
read. [As a person runs them](#as-a-person-runs-them) is the exception: what's there ran
unthrottled, on the performance cores and in real windows.

## They behave the same

- **Tests.** The Rust had 525 tests, and SimplePaint's four test programs. Each was ported, with
  its name and its checks, and passes. The C++ has 577 tests: the other 48 are `Library/Base`'s
  own, and the ones that pin what the port and its review found.
- **Scripts.** The eight programs that have input scripts run all 52 of them, and the 212 frames
  and states that the scripts save were compared with the Rust's by their bytes. 200 are
  identical. The other 12 show a path that has the run's own folder in it.
- **Numbers as text.** The programs write numbers into their states, their settings files and
  their status lines, so the C++ has to write every `float` and `double` as Rust does.
  `base::display` and `base::debug` do, for all of 1,248,746 numbers that were tried.

What it took to get there says something about the two languages, and is in
[What C++ lets through](#what-c-lets-through). What no test or script covers is in
[What wasn't measured](#what-wasnt-measured).

## Performance

[Benchmarks](../Program/Benchmarks/README.md) times the same hot paths in both languages: the
programs' frames through `Headless`, the GPU's work for a frame, text, the input reader, meshes,
glTF, JSON and matrices. The two programs ran one after the other, a group at a time, each in a
process of its own, for five rounds. A time below is a program's best minimum of the five rounds,
in microseconds. A ratio is the middle one of the five rounds' own ratios: the computer's load
moves both programs by more than they differ, so only runs taken together compare.

The C++ was measured twice. **As ported** is the port as it first passed its tests, written line
for line from the Rust. The other columns are the C++ now, after
[five small changes](#the-five-changes) where the first measurement showed it behind.

| Benchmark                                |    C++ |   Rust | C++ / Rust | As ported | Hardened / C++ | Rust LTO / Rust |
| ---------------------------------------- | -----: | -----: | ---------: | --------: | -------------: | --------------: |
| `ui/UiShowcase/frame/steady`             |    176 |    187 |       0.94 |      0.94 |           0.99 |            0.98 |
| `ui/UiShowcase/frame/moving`             |    181 |    201 |       0.91 |      0.93 |           1.00 |            0.96 |
| `ui/UiShowcase/start`                    |  2,587 |  2,567 |       1.01 |      1.02 |           0.99 |            1.00 |
| `ui/SimplePaintDemo/frame/steady`        |   70.5 |   83.7 |       0.84 |      0.83 |           1.00 |            0.98 |
| `ui/MeshModeler/frame/steady`            |    108 |    114 |       0.94 |      0.94 |           1.00 |            0.95 |
| `ui/MeshModeler/subdivided/frame/steady` |    107 |    115 |       0.93 |      0.93 |           1.00 |            0.95 |
| `ui/MeshModeler/subdivided/frame/moving` |    108 |    116 |       0.93 |      0.92 |           1.00 |            0.95 |
| `ui/GameInputDemo/frame/steady`          |   65.2 |   75.3 |       0.87 |      0.86 |           0.99 |            0.98 |
| `ui/Veehiicuul/frame/steady`             |   78.1 |   74.3 |       1.05 |      1.05 |           1.00 |            0.97 |
| `ui/Veehiicuul/start`                    |  3,831 |  3,095 |       1.25 |      1.44 |           1.18 |            0.99 |
| `ui/MinimalSceneTest/frame/steady`       |   15.9 |   18.2 |       0.87 |      0.84 |           0.99 |            0.98 |
| `ui/SpinningCarTest/frame/steady`        |   18.2 |   21.1 |       0.85 |      0.84 |           1.00 |            0.98 |
| `gpu/UiShowcase/draw`                    |   96.8 |   97.1 |       0.99 |      1.00 |           1.01 |            1.00 |
| `cpu/UiShowcase/record`                  |   9.50 |   9.70 |       0.97 |      0.97 |           1.00 |            0.99 |
| `gpu/SimplePaintDemo/draw`               |    124 |    123 |       1.00 |      1.00 |           1.00 |            1.00 |
| `cpu/SimplePaintDemo/record`             |   10.6 |   10.9 |       0.97 |      0.97 |           0.99 |            0.98 |
| `gpu/MeshModeler/subdivided/draw`        |    136 |    136 |       1.00 |      1.00 |           1.00 |            1.00 |
| `cpu/MeshModeler/subdivided/record`      |   8.10 |   8.40 |       0.98 |      0.96 |           1.00 |            0.99 |
| `gpu/GameInputDemo/draw`                 |   98.0 |   98.1 |       1.00 |      1.00 |           1.00 |            1.00 |
| `cpu/GameInputDemo/record`               |   5.90 |   6.10 |       0.95 |      0.95 |           1.02 |            0.98 |
| `gpu/Veehiicuul/draw`                    |   41.2 |   37.3 |       1.00 |      1.00 |           1.00 |            1.00 |
| `cpu/Veehiicuul/record`                  |   13.3 |   10.7 |       1.22 |      1.21 |           1.02 |            0.98 |
| `gpu/MinimalSceneTest/draw`              |   18.6 |   18.6 |       1.00 |      1.00 |           1.00 |            1.00 |
| `cpu/MinimalSceneTest/record`            |   4.80 |   4.80 |       0.96 |      0.98 |           1.00 |            0.94 |
| `gpu/SpinningCarTest/draw`               |   19.6 |   19.9 |       1.00 |      1.02 |           1.00 |            1.00 |
| `cpu/SpinningCarTest/record`             |   5.60 |   5.50 |       1.00 |      0.95 |           1.02 |            0.98 |
| `text/layout/cached`                     |  0.050 |  0.048 |       1.03 |      1.03 |           1.06 |            0.94 |
| `text/layout/new`                        |   8.80 |   8.70 |       1.00 |      1.01 |           1.00 |            0.98 |
| `text/layout/wrapped-new`                |   26.2 |   26.0 |       1.01 |      1.00 |           1.01 |            0.99 |
| `text/quads/cached`                      |  0.017 |  0.024 |       0.69 |      0.70 |           1.00 |            0.91 |
| `text/rasterize/new`                     |  0.640 |  0.580 |       1.02 |      1.02 |           0.98 |            1.05 |
| `text/layout/caption-cached`             |  0.050 |  0.047 |       1.03 |      0.97 |           1.06 |            0.91 |
| `input/reader/frame`                     |  0.600 |  0.500 |       1.40 |      1.60 |           0.86 |            1.00 |
| `perfbar/frame` (mean)                   |  0.248 |  0.182 |       1.38 |      1.39 |           1.00 |            0.97 |
| `mesh/catmull-clark/cube/1`              |   4.50 |   4.00 |       1.12 |      1.63 |           1.02 |            0.90 |
| `mesh/catmull-clark/cube/2`              |   20.0 |   18.2 |       1.10 |      1.72 |           1.03 |            0.89 |
| `mesh/catmull-clark/cube/3`              |   79.9 |   73.0 |       1.11 |      1.79 |           1.04 |            0.90 |
| `mesh/catmull-clark/cube/4`              |    336 |    294 |       1.14 |      1.88 |           1.04 |            0.90 |
| `mesh/catmull-clark/torus/1`             |    766 |    661 |       1.16 |      1.83 |           1.04 |            0.89 |
| `mesh/catmull-clark/torus/2`             |  3,862 |  3,541 |       1.09 |      1.77 |           1.03 |            0.94 |
| `mesh/triangulate/smooth`                | 20,927 | 18,598 |       1.12 |      1.62 |           1.06 |            0.88 |
| `mesh/triangulate/flat`                  | 12,781 |  8,808 |       1.45 |      1.51 |           1.03 |            0.88 |
| `mesh/topology`                          |  9,223 |  9,216 |       1.01 |      1.93 |           1.05 |            0.87 |
| `mesh/ray-cast/triangles`                |    252 |    514 |       0.48 |      0.48 |           1.00 |            0.24 |
| `mesh/ray-cast/mesh`                     |  2,773 |  2,724 |       1.03 |      1.03 |           1.02 |            0.79 |
| `mesh/primitive/uv-sphere`               |   92.5 |   72.5 |       1.27 |      1.26 |           1.01 |            0.93 |
| `mesh/edges`                             |  3,477 |  1,690 |       2.05 |      2.07 |           1.04 |            0.86 |
| `gltf/parse/SlopeCar`                    |   95.8 |   67.6 |       1.42 |      1.62 |           2.33 |            0.98 |
| `gltf/parse/BigCone`                     |    516 |    310 |       1.66 |      1.85 |           2.31 |            0.99 |
| `json/parse/scene`                       |   43.9 |   58.2 |       0.75 |      1.89 |           1.02 |            0.98 |
| `json/write/scene`                       |   60.8 |    102 |       0.60 |      1.38 |           1.00 |            1.01 |
| `math/matrix/multiply`                   |  0.011 | 0.0096 |       1.03 |      1.02 |           1.02 |            0.69 |
| `math/matrix/transform-points`           | 0.0013 | 0.0016 |       0.79 |      0.79 |           1.01 |            0.55 |
| `present-timing/tracker/present`         |  0.900 |  0.900 |       1.00 |      1.11 |           1.00 |            0.89 |

`cpu/.../record` is the CPU's time to record a frame's drawing, and `gpu/.../draw` the GPU's to
draw it. A row marked (mean) takes less than a tick of the counter, so its minimum is zero.

### What it says

- **Frames.** A frame's update, which is the program's `update` and `view`, the layout and the
  input, is 6% to 16% faster in C++ for UiShowcase, SimplePaintDemo, MeshModeler, GameInputDemo
  and the two scene tests, and 5% slower for Veehiicuul. Recording a frame's drawing is within 5%,
  but for Veehiicuul's again, at 1.22. Why the C++ leads wasn't profiled. It allocates less
  often in a frame, as [Memory](#memory) says, and it's optimized as a whole program, which is
  worth 2% to 5% of a frame to the Rust when the Rust gets it too.
- **The GPU.** Drawing a frame takes the GPU the same time to the percent. Both send it the same
  bytes: the same shaders, compiled by the same `dxc.exe`, and the same instances.
- **Text.** Laying out and rasterizing text is DirectWrite's time in both. Finding a cached
  layout's quads is a hash table's, and faster in C++.
- **Starting.** UiShowcase's first frame takes the same. Veehiicuul's is 1.25 times the Rust's,
  down from 1.44: it reads its models through glTF and JSON, and checks its track.
- **Meshes.** Subdividing, triangulating with smooth normals and building the topology were 1.6
  to 1.9 times the Rust's as ported, and are 1.01 to 1.16 now. Three rows weren't touched and
  are as they were: `mesh/edges` at 2.05, which returns a `std::set`, flat triangulation at 1.45
  and the sphere at 1.27.
- **JSON** was 1.9 and 1.4 times the Rust's, and is 0.75 and 0.60 of it.
- **glTF** was 1.6 and 1.9 times, and is 1.4 and 1.7.
- **The small rows.** `input/reader/frame` is 6 ticks of the counter against 5, and
  `perfbar/frame` less than one, so their ratios are coarse. Their profiles point at scans of
  256 keys that MSVC left as plain loops, and at `std::format`, without accounting for all of it.

### Where the differences came from

Each of these is from a sampling profile of both programs and their disassembly, on the port as
it was first measured.

**The port's own.** Three rows were slow because of how the port wrote a line, not because of
C++:

- JSON. `Json.cpp` took a string one character at a time, as the Rust does, but through calls
  that the Rust inlines and MSVC didn't: appending a byte to a `std::string` was a call and a
  `memmove` of one byte. Two thirds of parsing and of writing was there.
- glTF. `indices.insert(indices.end(), {a, b, c})` for each triangle, which is the natural way to
  write Rust's `extend`, was 15% of reading a model.
- The input reader reserved room for a frame's changes of focus, and then took its 18 readings one
  at a time.

**The standard library's.**

- `std::vector` grows from nothing to 1, 2, 3, 4, 6, 9 items, where Rust's `Vec` goes 4, 8, 16.
  The mesh code keeps thousands of lists of 2 to 4 items, the faces around a vertex and the like,
  and each of those is three or four allocations in C++ and one in Rust. An allocation costs both
  the same, about 31 ns for the pair with its free, and the C++ made 2.1 to 3.3 times as many.
  Close to two thirds of the time of `topology` and `catmull-clark` was inside the heap.
- `std::set`, `std::map` and `std::unordered_map` allocate a node for each item. Rust's `BTreeMap`
  keeps several items a node, and its `HashMap` none. The edges of a torus were 36,864 nodes.
- It goes the other way too: Rust's `HashMap` hashes with SipHash, which is safe against chosen
  keys and slow for short ones, and was about 30% of `text/quads/cached` in the Rust. MSVC's hash
  is cheaper, and the C++ is faster there.

**The compiler's.**

- MSVC kept a 64-bit division for `(index + 1) % size` in a loop over pairs of a track's edges,
  and followed the references that `std::min` and `std::max` return through the stack. LLVM had
  neither. That loop was a quarter of Veehiicuul's start in C++ and nearly nothing in Rust.
- MSVC left small functions out of line that LLVM inlines: a cast, a `push_back`. In the glTF
  reader, which reads every number through them, they were an eighth of its time, and they're
  part of what's left of its gap. What MSVC inlines also changes with the program that the code
  is linked into: the same glTF source took a quarter longer in another exe.
- Cargo's release profile doesn't optimize across packages, and `/GL` optimizes across the whole
  program. `mesh/ray-cast/triangles` is twice as fast in C++ for that alone: the Rust calls a
  small function in another package, and copies each triangle through the stack to do it. With
  link-time optimization the Rust inlines it, and is then twice as fast as the C++.

### The five changes

| Where                                    | What changed                                              | C++ / Rust, before |        After |
| ---------------------------------------- | --------------------------------------------------------- | -----------------: | -----------: |
| `Library/Json/Json.cpp`                  | Runs of plain ASCII are appended at once, not a character |         1.89, 1.38 |   0.75, 0.60 |
|                                          | at a time, in parsing and in writing                      |                    |              |
| `Library/Gltf/Gltf.cpp`                  | Three `push_back`s for a triangle, not an `insert`        |         1.62, 1.85 |   1.42, 1.66 |
| `Library/UserInterface/Input/Reader.cpp` | Reserves for the readings too                             |               1.60 |         1.40 |
| `Library/PolygonMesh/Topology.cpp`       | Edges by sorting, not through a `std::set`, and a list's  |       1.62 to 1.93 | 1.01 to 1.16 |
|                                          | first item reserves for 2 or 4                            |                    |              |
| `Program/Veehiicuul/Track.cpp`           | Wraps an index with a compare, not `%`, and takes the     |               1.44 |         1.25 |
|                                          | least and the most by value                               |                    |              |

None changes what the code does, and none is something that the Rust needed. They're what
writing C++ rather than transcribing Rust looks like: knowing what the standard library's
containers cost, and that the compiler takes less on trust.

### Hardened

`Program\Build\Build.ps1 --hardened` builds the same code with the standard library checking
every index into a vector, a span, a string or an array, and every use of an empty optional, and
stopping the program at a wrong one. That's most of what Rust checks as it runs. It costs between
nothing and 6% on every row but three:

- The glTF reader takes 2.3 times as long. It reads every number of a model through an index into
  a span of bytes. The Rust has the same checks in the same places, and LLVM proves most of them
  away. MSVC keeps them, and stops inlining the function that reads four bytes.
- Veehiicuul's start takes 1.18 times as long, since it reads models.
- The input reader's row is coarse, as above.

The heap's numbers are the same as without the checks, and Benchmarks' exe is 16 KB larger. So
the checks could stay on in the programs that people run, which is what Rust does, at the cost of
slower loading of models.

### Rust with link-time optimization

Cargo's release profile optimizes each package by itself. The C++'s Release build optimizes the
whole program, so some of the C++'s lead is the setting's and not the language's. With
`lto = "fat"` and one codegen unit, the Rust is 2% to 14% faster than without on most rows that
are the CPU's, and far more where a small function was called across packages:
`mesh/ray-cast/triangles` takes a quarter of the time, and `math/matrix/transform-points` half.
Loading, JSON and the GPU's rows don't change.

Against that Rust, the C++'s frames still lead, by 1% to 14% for the same six programs, the mesh
rows that are now within 16% are 12% to 30% behind, and the ray cast that was twice as fast is
twice as slow. The table's own Rust column is the one to read for what KGames was, since KGames
built with Cargo's release profile as it comes.

## As a person runs them

Everything above ran throttled, on the efficiency cores. For this section the computer was left
alone, and nothing was throttled: the benchmarks ran on the performance cores, and the eight
programs with windows ran as [HumanRun](../Program/UiShowcase/HumanRun.ps1) starts them, the
optimized exes, maximized on the primary display, 2560 x 1440 at 120 Hz, with the focus, reading
their input from GameInput. The two languages took turns, and which went first changed by round.

The short of it: with vertical sync, which is how the programs start, nothing tells the two
apart. Both draw 120 frames a second without dropping one, answer the mouse in the same 9
milliseconds, and use the same CPU time, memory and GPU. Without it, where a frame is as short as
the program and Windows can make it, the Rust draws 0% to 5% more frames a second.

### On the performance cores

Benchmarks, as in [Performance](#performance), each process held to the eight performance cores.
**Throttled** is the ratio from the table above, to compare.

| Benchmark                                |    C++ |   Rust | C++ / Rust | Throttled | Hardened / C++ | Rust LTO / Rust |
| ---------------------------------------- | -----: | -----: | ---------: | --------: | -------------: | --------------: |
| `ui/UiShowcase/frame/steady`             |    127 |    134 |       0.95 |      0.94 |           0.97 |            1.05 |
| `ui/UiShowcase/frame/moving`             |    132 |    140 |       0.95 |      0.91 |           1.02 |            1.04 |
| `ui/UiShowcase/start`                    |  1,938 |  1,880 |       1.04 |      1.01 |           1.00 |            1.00 |
| `ui/SimplePaintDemo/frame/steady`        |   48.9 |   58.8 |       0.83 |      0.84 |           1.00 |            0.99 |
| `ui/MeshModeler/frame/steady`            |   77.0 |   80.0 |       0.97 |      0.94 |           1.00 |            1.03 |
| `ui/MeshModeler/subdivided/frame/steady` |   77.3 |   80.8 |       0.95 |      0.93 |           0.99 |            1.04 |
| `ui/MeshModeler/subdivided/frame/moving` |   77.6 |   80.9 |       0.94 |      0.93 |           0.99 |            1.04 |
| `ui/GameInputDemo/frame/steady`          |   41.3 |   50.9 |       0.82 |      0.87 |           1.00 |            1.00 |
| `ui/Veehiicuul/frame/steady`             |   52.6 |   51.7 |       1.02 |      1.05 |           1.01 |            0.98 |
| `ui/Veehiicuul/start`                    |  3,050 |  2,378 |       1.29 |      1.25 |           1.24 |            1.00 |
| `ui/MinimalSceneTest/frame/steady`       |   10.1 |   12.3 |       0.83 |      0.87 |           0.99 |            0.98 |
| `ui/SpinningCarTest/frame/steady`        |   11.7 |   14.4 |       0.81 |      0.85 |           0.99 |            0.98 |
| `gpu/UiShowcase/draw`                    |   97.2 |   97.2 |       1.00 |      0.99 |           1.00 |            1.00 |
| `cpu/UiShowcase/record`                  |   4.00 |   4.10 |       0.98 |      0.97 |           1.00 |            1.00 |
| `gpu/SimplePaintDemo/draw`               |    124 |    123 |       1.00 |      1.00 |           1.00 |            1.00 |
| `cpu/SimplePaintDemo/record`             |   4.80 |   4.90 |       0.98 |      0.97 |           1.00 |            1.02 |
| `gpu/MeshModeler/subdivided/draw`        |    136 |    137 |       1.00 |      1.00 |           1.00 |            1.00 |
| `cpu/MeshModeler/subdivided/record`      |   3.90 |   4.10 |       0.98 |      0.98 |           0.97 |            1.00 |
| `gpu/GameInputDemo/draw`                 |   99.1 |   98.2 |       1.00 |      1.00 |           1.00 |            1.00 |
| `cpu/GameInputDemo/record`               |   2.50 |   2.60 |       0.96 |      0.95 |           1.00 |            1.00 |
| `gpu/Veehiicuul/draw`                    |   41.3 |   41.2 |       1.00 |      1.00 |           1.00 |            0.99 |
| `cpu/Veehiicuul/record`                  |   7.90 |   6.20 |       1.29 |      1.22 |           1.04 |            0.95 |
| `gpu/MinimalSceneTest/draw`              |   19.2 |   18.7 |       1.00 |      1.00 |           1.00 |            1.00 |
| `cpu/MinimalSceneTest/record`            |   2.40 |   2.60 |       0.93 |      0.96 |           1.00 |            0.96 |
| `gpu/SpinningCarTest/draw`               |   20.6 |   19.8 |       1.00 |      1.00 |           1.00 |            1.00 |
| `cpu/SpinningCarTest/record`             |   2.90 |   3.00 |       0.94 |      1.00 |           1.00 |            0.97 |
| `text/layout/cached`                     |  0.022 |  0.030 |       0.79 |      1.03 |           1.00 |            0.95 |
| `text/layout/new`                        |   6.30 |   6.20 |       1.00 |      1.00 |           1.00 |            1.00 |
| `text/layout/wrapped-new`                |   19.3 |   19.5 |       1.00 |      1.01 |           1.00 |            1.00 |
| `text/quads/cached`                      |  0.010 |  0.018 |       0.59 |      0.69 |           1.00 |            0.82 |
| `text/rasterize/new`                     |  0.440 |  0.440 |       0.98 |      1.02 |           0.98 |            0.98 |
| `text/layout/caption-cached`             |  0.020 |  0.030 |       0.68 |      1.03 |           1.00 |            0.95 |
| `input/reader/frame`                     |  0.400 |  0.400 |       1.25 |      1.40 |           1.00 |            0.75 |
| `perfbar/frame` (mean)                   |  0.213 |  0.156 |       1.36 |      1.38 |           1.01 |            0.92 |
| `mesh/catmull-clark/cube/1`              |   3.50 |   3.20 |       1.09 |      1.12 |           1.06 |            0.91 |
| `mesh/catmull-clark/cube/2`              |   15.9 |   14.7 |       1.07 |      1.10 |           1.07 |            0.89 |
| `mesh/catmull-clark/cube/3`              |   64.8 |   59.8 |       1.08 |      1.11 |           1.06 |            0.90 |
| `mesh/catmull-clark/cube/4`              |    270 |    236 |       1.12 |      1.14 |           1.05 |            0.91 |
| `mesh/catmull-clark/torus/1`             |    605 |    541 |       1.14 |      1.16 |           1.04 |            0.92 |
| `mesh/catmull-clark/torus/2`             |  3,119 |  2,923 |       0.97 |      1.09 |           1.05 |            0.84 |
| `mesh/triangulate/smooth`                | 14,782 | 13,494 |       1.14 |      1.12 |           1.07 |            0.91 |
| `mesh/triangulate/flat`                  | 10,540 |  7,188 |       1.45 |      1.45 |           1.03 |            0.93 |
| `mesh/topology`                          |  7,416 |  7,022 |       1.05 |      1.01 |           1.05 |            0.90 |
| `mesh/ray-cast/triangles`                |    210 |    512 |       0.42 |      0.48 |           1.04 |            0.23 |
| `mesh/ray-cast/mesh`                     |  2,285 |  2,171 |       1.04 |      1.03 |           1.04 |            0.82 |
| `mesh/primitive/uv-sphere`               |   75.6 |   57.5 |       1.31 |      1.27 |           1.00 |            0.97 |
| `mesh/edges`                             |  2,860 |  1,584 |       1.80 |      2.05 |           1.08 |            0.87 |
| `gltf/parse/SlopeCar`                    |   88.5 |   53.1 |       1.67 |      1.42 |           2.51 |            0.99 |
| `gltf/parse/BigCone`                     |    474 |    236 |       2.04 |      1.66 |           2.37 |            1.06 |
| `json/parse/scene`                       |   35.1 |   46.5 |       0.77 |      0.75 |           1.01 |            0.98 |
| `json/write/scene`                       |   47.6 |   75.4 |       0.63 |      0.60 |           0.99 |            1.00 |
| `math/matrix/multiply`                   |  0.012 | 0.0091 |       1.25 |      1.03 |           1.01 |            0.87 |
| `math/matrix/transform-points`           | 0.0012 | 0.0016 |       0.73 |      0.79 |           1.03 |            0.54 |
| `present-timing/tracker/present`         |  0.500 |  0.600 |       0.83 |      1.00 |           1.00 |            1.00 |

- **The ratios held.** A performance core takes about seven tenths of the time for both, and
  most rows' ratios moved by a few hundredths. A frame's update is 3% to 19% faster in C++ for
  the same six programs, and 2% slower for Veehiicuul. Leaving Windows to choose the cores gave
  the same ratios as holding the processes to the performance cores.
- **glTF is further behind**: 1.67 and 2.04 times the Rust's time, where throttled it was 1.42
  and 1.66. The Rust gained more from the faster core than the C++ did.
- **Looking up cached text is further ahead**, at 0.59 to 0.79.
- **Link-time optimization doesn't help the Rust's frames here.** UiShowcase's and MeshModeler's
  take 3% to 5% longer with it, where throttled they took 2% to 5% less. It still helps the mesh
  rows by about a tenth.

### In real windows

Each program ran for 3 seconds, and then for 8 that were measured, in four ways: with vertical
sync and without, each left alone and with the mouse moving over the window. The mouse was moved
by injected input, a thousand moves a second on a path that crosses the window, which reaches
GameInput as a mouse's own does. There were two sets of runs:

- With PresentMon recording every frame's present, twice for each program, way and language.
- Without it, four times, with only the program running. The CPU's time is the process's cycle
  count, which is every thread's, the driver's and GameInput's included. Windows' own count of a
  process's CPU time is sampled at the timer's tick, and for a program that works for a fraction
  of a millisecond every 8 it gave anything from 0.02 to 1.1 seconds for the same idle 8 seconds.

A number is the middle one of its runs. With vertical sync, which is what a program starts with:

| Program          | Frames a second | 99th percentile frame, ms, C++ | Rust | GPU busy, ms, C++ |  Rust | Core %, idle, C++ | Rust | Core %, mouse, C++ |  Rust |
| ---------------- | --------------: | -----------------------------: | ---: | ----------------: | ----: | ----------------: | ---: | -----------------: | ----: |
| UiShowcase       |           120.0 |                           8.50 | 8.51 |             0.271 | 0.274 |              8.08 | 8.01 |              11.24 | 10.90 |
| SimplePaintDemo  |           120.0 |                           8.48 | 8.47 |             0.305 | 0.307 |              6.90 | 6.84 |               9.39 |  9.37 |
| MeshModeler      |           120.0 |                           8.46 | 8.48 |             0.319 | 0.317 |              7.43 | 7.32 |               9.86 |  9.81 |
| GameInputDemo    |           120.0 |                           8.47 | 8.46 |             0.279 | 0.276 |              7.01 | 6.85 |              11.32 | 11.27 |
| Veehiicuul       |           120.0 |                           8.44 | 8.45 |             0.227 | 0.229 |              7.36 | 6.98 |              10.08 |  9.66 |
| MinimalSceneTest |           120.0 |                           8.45 | 8.46 |             0.262 | 0.263 |              6.65 | 6.63 |               8.81 |  8.88 |
| SpinningCarTest  |           120.0 |                           8.50 | 8.52 |             0.257 | 0.274 |              6.47 | 6.58 |               9.00 |  9.02 |
| Platformer       |           120.0 |                           8.47 | 8.47 |             0.281 | 0.285 |              6.91 | 6.88 |               8.97 |  8.85 |

- **Frames.** Both languages presented 120.0 frames a second in every run, a frame every 8.33
  milliseconds, with the 99th percentile within a quarter of a millisecond of that, and PresentMon
  saw every frame reach the screen. Every present was an independent flip on a hardware plane.
- **The GPU** is busy for 0.2 to 0.3 milliseconds of a frame in both.
- **The CPU.** A program uses 6.5% to 8% of one core when it's left alone and 9% to 11% while
  the mouse moves. The C++ uses between 2% less and 5% more than the Rust, and a language's own
  runs of a program differ by about as much.
- **Memory.** The working set is 98 to 103 MiB in both, within a MiB of each other, the
  private commit 126 to 156 MiB, within 2%, and the GPU's dedicated memory 61.8 to 91.6 MiB, the
  same to the tenth. Both run 33 or 34 threads. GameInputDemo, which keeps what the mouse sent,
  grows to 140 MiB in C++ and 144 in Rust while it moves.

Without vertical sync a program draws as often as it can. PresentMon records at most 2,048
presents a second, so these are the programs' own counts, which the PerfBar shows:

| Program          | Frames a second, idle, C++ |  Rust |    x | Frames a second, mouse, C++ |  Rust |    x |
| ---------------- | -------------------------: | ----: | ---: | --------------------------: | ----: | ---: |
| UiShowcase       |                      1,761 | 1,803 | 0.98 |                       1,515 | 1,581 | 0.96 |
| SimplePaintDemo  |                      2,080 | 2,078 | 1.00 |                       1,896 | 1,896 | 1.00 |
| MeshModeler      |                      1,846 | 1,909 | 0.97 |                       1,680 | 1,722 | 0.98 |
| GameInputDemo    |                      2,173 | 2,214 | 0.98 |                       1,602 | 1,620 | 0.99 |
| Veehiicuul       |                      2,439 | 2,556 | 0.95 |                       2,095 | 2,181 | 0.96 |
| MinimalSceneTest |                      3,026 | 3,130 | 0.97 |                       2,605 | 2,581 | 1.01 |
| SpinningCarTest  |                      3,055 | 3,073 | 0.99 |                       2,578 | 2,546 | 1.01 |
| Platformer       |                      2,700 | 2,740 | 0.99 |                       2,372 | 2,367 | 1.00 |

The Rust draws up to 5% more frames a second, and never fewer by more than 1%. A frame here is a
third to two thirds of a millisecond, and the program's own code is a third of it or less: the
rest is the present, the driver, Windows' messages and waiting for the GPU's queue, which are the
same code in both. Either way the process uses about four fifths of one core.

### Input latency

The PerfBar measures the time from when GameInput stamped a reading to when the frame that used
it reached the screen, in three parts: waiting to be read, building the frame, and presenting it.
To record it, both languages were built with the same few temporary lines, which write the
PerfBar's summary to a file as often as the bar refreshes it, 10 times a second. They were never
committed. The mouse moved as above. Milliseconds, the middle of four runs without PresentMon.
Three more with it gave the same to within a tenth of a millisecond.

| Program          | Latency, vertical sync, C++ | Rust | Building, C++ |  Rust |    x | Latency, no sync, C++ | Rust | Building, C++ |  Rust |    x |
| ---------------- | --------------------------: | ---: | ------------: | ----: | ---: | --------------------: | ---: | ------------: | ----: | ---: |
| UiShowcase       |                        8.96 | 8.95 |         0.379 | 0.360 | 1.05 |                  1.00 | 0.98 |         0.319 | 0.308 | 1.03 |
| SimplePaintDemo  |                        9.00 | 8.99 |         0.218 | 0.224 | 0.97 |                  0.84 | 0.84 |         0.182 | 0.180 | 1.01 |
| MeshModeler      |                        8.96 | 8.95 |         0.262 | 0.255 | 1.03 |                  0.92 | 0.92 |         0.228 | 0.217 | 1.05 |
| GameInputDemo    |                        8.99 | 8.97 |         0.403 | 0.374 | 1.08 |                  0.98 | 0.97 |         0.328 | 0.321 | 1.02 |
| Veehiicuul       |                        8.97 | 8.87 |         0.229 | 0.215 | 1.06 |                  0.73 | 0.73 |         0.180 | 0.165 | 1.09 |
| MinimalSceneTest |                        8.92 | 9.04 |         0.145 | 0.144 | 1.01 |                  0.64 | 0.65 |         0.108 | 0.107 | 1.01 |
| SpinningCarTest  |                        8.93 | 8.92 |         0.153 | 0.155 | 0.99 |                  0.64 | 0.65 |         0.110 | 0.108 | 1.02 |
| Platformer       |                        8.92 | 8.94 |         0.185 | 0.173 | 1.06 |                  0.68 | 0.68 |         0.119 | 0.110 | 1.08 |

- **The latency is the same**: 8.9 to 9.0 milliseconds with vertical sync, of which 7.9 to 8.2
  is the present waiting for the display and 0.6 to 0.7 the reading waiting to be read, and 0.6
  to 1.0 without it. The 99th percentile is 10.3 to 10.7 with, and 1.0 to 1.5 without, in both.
- **Building the frame**, the only part that's the program's, takes up to 9% longer in C++,
  which is at most 29 microseconds, and up to 3% less.
- This is from the reading's stamp to the screen. What a mouse and its cable take before, and a
  display after, are a person's to measure, and injected input isn't a device.

That the C++ builds a frame a little slower in its window is the reverse of the benchmarks,
where its frames are faster. Building includes the present's call, the same in both. For the
rest, the main thread of UiShowcase, MeshModeler and Veehiicuul was sampled in their windows,
without vertical sync, a few thousand samples each, so this is rough:

- The program's `view` takes about 1.4 times the Rust's share of a frame in UiShowcase and
  MeshModeler, while layout and drawing take about 0.8 of it in UiShowcase and Veehiicuul. In
  MeshModeler drawing too took 1.2 times.
- In UiShowcase, more than half of `view`'s difference is one function. The list's search
  lowercases every item's text each frame, which the C++ does through UTF-16 and Windows'
  `LCMapStringEx`, with up to three allocations, and Rust's `to_lowercase` does in UTF-8: 2.9% of
  the thread's time against 0.5%.
- In Veehiicuul it's `update`, at 1.8 times the Rust's share, which compares the document with
  the saved one each frame. The terrain in it is a `std::map`, with a node for each cell, where
  Rust's `BTreeMap` keeps several cells in a node.

Both are the kind that [Where the differences came from](#where-the-differences-came-from)
found in the mesh code. Why they weigh more in a window than in the benchmark, which runs the
same `view`, wasn't found. A window's frame runs after the driver and Windows have had the
core's caches for a few hundred microseconds, where the benchmark's runs right after the last.

### Starting

Each program was started ten times in each language, after a first start that isn't counted,
with PresentMon recording its presents. Milliseconds from the call that creates the process, the
middle of the ten:

| Program          | Window shows, C++ | Rust | First frame presented, C++ | Rust |    x | First frame on the screen, C++ | Rust |
| ---------------- | ----------------: | ---: | -------------------------: | ---: | ---: | -----------------------------: | ---: |
| UiShowcase       |               218 |  216 |                        232 |  231 | 1.00 |                            251 |  245 |
| SimplePaintDemo  |               219 |  219 |                        233 |  235 | 0.99 |                            251 |  251 |
| MeshModeler      |               215 |  214 |                        234 |  231 | 1.01 |                            251 |  249 |
| GameInputDemo    |               214 |  216 |                        231 |  232 | 1.00 |                            250 |  251 |
| Veehiicuul       |               222 |  223 |                        238 |  238 | 1.00 |                            257 |  258 |
| MinimalSceneTest |               217 |  218 |                        232 |  231 | 1.01 |                            249 |  249 |
| SpinningCarTest  |               224 |  221 |                        234 |  234 | 1.00 |                            253 |  250 |
| Platformer       |               219 |  220 |                        231 |  232 | 0.99 |                            246 |  247 |

A program is on the screen a quarter of a second after it's started, in either language, and
a language's own starts of a program differ by 8 to 44 milliseconds. What the program does to
start, which the benchmarks' `start` rows time at 2 to 3 milliseconds, is a hundredth of that.
The rest is Windows making the process, the Direct3D device, the swap chain and the window, and
GameInput starting. So the 0.7 milliseconds that Veehiicuul's start takes longer in C++ doesn't
show.

## Memory

A program's memory is the same in both. The process that measures the GPU's memory, which makes a
device, a render target and each program in turn, ends with a working set of 58 MiB and 113 MiB
of private commit in either language, and the GPU's memory is identical to the hundredth of a MiB
on every row. [Memory.md](Memory.md) says where that goes: nearly all of it is Windows',
Direct3D's and the driver's, which no language changes.

The program's own heap is where they differ. A counting allocator in Benchmarks counts each
program's allocations, with a frame, a start or one call as the unit:

- **In a frame, the C++ allocates half to nine tenths as often, and 1.1 to 1.9 times the bytes.**
  Neither was profiled. The count is likely `std::string`, which holds up to 15 bytes without
  allocating, where Rust's `String` always allocates, and a frame makes hundreds of short texts.
  The bytes are likely `std::vector`'s growth: a list that grows to a hundred items asks for about
  1.7 times the bytes in steps of a half that it does by doubling.
- **Starting, most programs allocate up to a third more often in C++**, and up to 1.5 times the
  bytes.
- **What's kept is close.** After a start it's within a fifth either way. The caches keep less in
  C++: 951 KiB of text layouts against 1,539, and 2,145 KiB of what the interface remembers of
  widgets whose names keep changing against 2,731.
- **The mesh code made 2.1 to 3.3 times as many allocations as ported**, which was its time, as
  above. It makes 1.2 to 1.35 times as many now. `memory/mesh/triangles` wasn't touched, and is
  still 2.3 times.
- **JSON** makes 0.56 times the allocations now, where it made 1.07 times.

The hardened build's rows are the same as the plain one's.

| Benchmark                                    | Allocations, C++ |    Rust |    x | KiB allocated, C++ |     Rust |    x | KiB kept, C++ |   Rust |
| -------------------------------------------- | ---------------: | ------: | ---: | -----------------: | -------: | ---: | ------------: | -----: |
| `memory/UiShowcase/start`                    |            3,926 |   3,699 | 1.06 |            1,410.7 |    967.8 | 1.46 |           578 |    554 |
| `memory/UiShowcase/frame/steady`             |              645 |   1,352 | 0.48 |              182.8 |    107.7 | 1.70 |            18 |     42 |
| `memory/UiShowcase/frame/moving`             |              657 |   1,366 | 0.48 |              183.7 |    109.2 | 1.68 |            78 |    172 |
| `memory/SimplePaintDemo/start`               |            1,918 |   1,436 | 1.34 |              823.2 |    689.7 | 1.19 |           325 |    371 |
| `memory/SimplePaintDemo/frame/steady`        |              249 |     344 | 0.72 |               74.6 |     42.2 | 1.77 |             1 |      1 |
| `memory/MeshModeler/start`                   |            3,199 |   4,277 | 0.75 |              891.1 |    660.3 | 1.35 |           289 |    301 |
| `memory/MeshModeler/frame/steady`            |              438 |     992 | 0.44 |              103.8 |     68.3 | 1.52 |             0 |      0 |
| `memory/MeshModeler/subdivided/start`        |           26,543 |  22,278 | 1.19 |            3,112.9 |  2,784.0 | 1.12 |           512 |    556 |
| `memory/MeshModeler/subdivided/frame/steady` |              439 |     994 | 0.44 |              103.9 |     68.4 | 1.52 |             0 |      0 |
| `memory/MeshModeler/subdivided/frame/moving` |              442 |     997 | 0.44 |              104.1 |     68.9 | 1.51 |             0 |      0 |
| `memory/GameInputDemo/start`                 |            2,402 |   2,021 | 1.19 |              867.4 |    739.9 | 1.17 |           314 |    372 |
| `memory/GameInputDemo/frame/steady`          |              170 |     232 | 0.73 |               43.3 |     30.6 | 1.42 |             0 |      0 |
| `memory/Veehiicuul/start`                    |            7,383 |   6,545 | 1.13 |            6,264.8 |  4,791.3 | 1.31 |         1,012 |    999 |
| `memory/Veehiicuul/frame/steady`             |              374 |     542 | 0.69 |               58.6 |     48.0 | 1.22 |             0 |      0 |
| `memory/MinimalSceneTest/start`              |              775 |     577 | 1.34 |              197.6 |    140.0 | 1.41 |            87 |     80 |
| `memory/MinimalSceneTest/frame/steady`       |               77 |      87 | 0.88 |               16.8 |      8.8 | 1.92 |             3 |      3 |
| `memory/SpinningCarTest/start`               |              852 |     638 | 1.34 |              202.8 |    206.1 | 0.98 |            90 |    114 |
| `memory/SpinningCarTest/frame/steady`        |               89 |     106 | 0.83 |               17.4 |     15.7 | 1.11 |             3 |      3 |
| `memory/UiShowcase/frames/long`              |              642 |   1,350 | 0.48 |              182.7 |    107.5 | 1.70 |            -0 |     -0 |
| `memory/text/fonts`                          |               24 |      22 | 1.09 |                1.5 |      1.1 | 1.34 |             1 |      0 |
| `memory/text/layouts`                        |               12 |       9 | 1.40 |                1.1 |      1.2 | 0.91 |           951 |  1,539 |
| `memory/text/glyphs`                         |                3 |       3 | 1.26 |                0.3 |      0.3 | 1.04 |            76 |     74 |
| `memory/text/purge`                          |                0 |       0 |    - |                0.0 |      0.0 |    - |          -932 | -1,430 |
| `memory/ui/memory/new-names`                 |              106 |      98 | 1.09 |               24.0 |     16.1 | 1.48 |         2,145 |  2,731 |
| `memory/mesh/torus/2`                        |           70,364 |  59,867 | 1.18 |            4,969.4 |  4,985.9 | 1.00 |         1,080 |  1,080 |
| `memory/mesh/catmull-clark/torus/2`          |           70,363 |  59,867 | 1.18 |            4,969.3 |  4,985.9 | 1.00 |             0 |      0 |
| `memory/mesh/topology`                       |          129,064 |  95,541 | 1.35 |            9,381.8 |  9,731.2 | 0.96 |         7,365 |  6,452 |
| `memory/mesh/triangulate/smooth`             |          202,891 | 150,902 | 1.34 |           22,637.8 | 21,867.1 | 1.04 |             0 |      0 |
| `memory/mesh/triangles`                      |          129,150 |  55,378 | 2.33 |           23,744.0 | 22,215.9 | 1.07 |         3,380 |  4,608 |
| `memory/gltf/SlopeCar`                       |              375 |     523 | 0.72 |              495.3 |    362.1 | 1.37 |           104 |    108 |
| `memory/gltf/BigCone`                        |              422 |     573 | 0.74 |            3,575.5 |  2,439.8 | 1.47 |           732 |    729 |
| `memory/json/scene`                          |              429 |     760 | 0.56 |               64.3 |     74.2 | 0.87 |            36 |     47 |
| `memory/json/parse/scene`                    |              429 |     760 | 0.56 |               64.3 |     74.2 | 0.87 |             0 |      0 |

## The code

### How much

Lines that aren't blank or comments, of what was written by hand, in the packages that both have:

| Part      | Rust files | Rust lines | C++ files | C++ lines | C++ / Rust | Of the C++, in headers |
| --------- | ---------: | ---------: | --------: | --------: | ---------: | ---------------------: |
| `Library` |         92 |     21,347 |       180 |    26,187 |       1.23 |                    41% |
| `Program` |         92 |     15,223 |       163 |    19,687 |       1.29 |                    21% |
| `Test`    |         71 |     11,083 |        76 |    13,296 |       1.20 |                     1% |
| All       |        255 |     47,653 |       419 |    59,170 |       1.24 |                    25% |

The C++ is a quarter longer by lines, and the lines are longer too: clang-format wraps them at 180
columns here, where rustfmt wraps at 100. By tokens it's 1.30 times the Rust, and by characters
that aren't spaces, 1.40 times. Tests grew least, since a test is calls and comparisons, which
read the same in both. Programs grew most, since they're made of messages, which are sum types.

Two things aren't in those rows, one on each side:

- `Library/Base` and `Library/Testing`, 1,550 lines, and Base's tests, 624: what Rust's standard
  library and `cargo test` have and C++'s lacks. See [What C++ needed](#what-c-needed).
- `Library/WindowsApi`, the Rust's bindings for Windows: 9,377 generated lines, and the generator's
  configuration. The C++ includes the Windows SDK's headers.

And one row has the same thing in it: the Rust's `Library/GameInput` was 761 lines, mostly
Microsoft's `GameInput.h` written again as Rust by hand. The C++ is 276, since it includes the
header.

### Where the extra lines are

**Sum types.** A Rust `enum` whose variants carry data, and the `match` that takes it apart, are
the shape of every program here: its messages, its events, its prompts. C++ has `std::variant`,
which holds one of several types but doesn't name them, so each alternative is a struct of its
own. The Platformer's events, in Rust:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    RunStarted,
    CoinCollected { coin: usize },
    Died { cause: Cause },
    RecordSet { seconds: f64, previous: Option<f64> },
    // ... 12 more
}

impl Event {
    pub fn label(&self) -> String {
        match self {
            Event::RunStarted => "RunStarted".to_string(),
            Event::CoinCollected { coin } => format!("CoinCollected {coin}"),
            // ...
        }
    }
}
```

and in C++, as `Library/Base/Sum.h` has it:

```cpp
struct Event {
    struct RunStarted {
        bool operator==(const RunStarted&) const = default;
    };
    struct CoinCollected {
        size_t coin;
        bool operator==(const CoinCollected&) const = default;
    };
    struct Died {
        Cause cause;
        bool operator==(const Died&) const = default;
    };
    struct RecordSet {
        double seconds;
        std::optional<double> previous;
        bool operator==(const RecordSet&) const = default;
    };
    // ... 12 more
    std::variant<RunStarted, CoinCollected, Died, RecordSet /* ... 12 more */> value;

    template <base::AlternativeOf<Event> T> Event(T&& alternative) : value(std::forward<T>(alternative)) {}

    std::string label() const;
    bool operator==(const Event&) const = default;
};

std::string Event::label() const {
    return base::match(
        *this,
        [](const RunStarted&) { return std::string("RunStarted"); },
        [](const CoinCollected& event) { return std::format("CoinCollected {}", event.coin); },
        // ...
    );
}
```

The enum's 33 lines are 86 in C++, and each alternative is named twice. What carries over is what
matters: `Event::CoinCollected{2}` is a value of its own type, the `match` takes a function for
each alternative, and one that's missing doesn't compile. The Rust had 83 enums and 423 `match`es.
The C++ has 48 `std::variant`s, 42 `enum class`es for the enums without data, 65 `base::match`es,
218 `base::is` and `base::as`, which are `if let`, and 113 `switch`es.

**Declarations, twice.** A quarter of the C++ is in headers. A function is declared in its header
and defined in its `.cpp`, a class lists its members' functions before it defines them, and what
a file uses is named in `#include`s, 4,245 of them, where the Rust had 1,217 `use`s. In
`Library/UserInterface` more than half is in headers, since a widget is a template over the
program's message type, and a template is defined where it's declared.

**Errors that go up.** Rust's `?` returns an error to the caller in one character, 615 times. C++
has `std::expected` and nothing like `?`, so each is an `if` that returns `std::unexpected`, 454
times, over three lines:

```rust
let rows = member(value, "Rows", "Terrain.")?;
```

```cpp
std::expected<const Value*, std::string> rows = files::member(value, "Rows", "Terrain.");
if (!rows) {
    return std::unexpected(rows.error());
}
```

**What a `derive` writes.** `#[derive(Clone, Debug, PartialEq)]`, 271 times, became a defaulted
`operator==` in each struct that's compared, 399 of them, a `describe_value` function for each
type that a test's failed check prints, and for a type that mustn't be copied, a deleted copy
constructor and assignment, 215 lines of `= delete`: in Rust a type can't be copied unless it
says so, and in C++ it can unless it says not.

### What got shorter

- **Windows.** A call of the Windows API is a call. In Rust each was `unsafe`, which the code
  said 383 times, with its types converted by hand at the edges, through bindings that had to be
  generated for every function and interface used. COM objects are `ComPtr`s.
- **Build scripts.** The 10 `build.rs` programs, 323 lines, which compiled shaders, generated the
  car's mesh and found GameInput's package, are one rule in `KGames.targets`, a `Shader` item in a
  project, and a small tool that a project runs.
- **Project files.** A `.vcxproj` is the projects it uses and two imports, 8 lines for most.
  `KGames.props` and `KGames.targets`, 254 lines, have the rest: a project's kind follows from the
  folder it's in, and it compiles every `.cpp` beside it. All the build files are 1,092 lines,
  where the `Cargo.toml`s and `build.rs`s were 1,430.

### How Rust's constructs came across

| Rust                                | Times | C++                                                      | Times |
| ----------------------------------- | ----: | -------------------------------------------------------- | ----: |
| `enum`                              |    83 | A struct around a `std::variant`, or an `enum class`     |    90 |
| `match`                             |   423 | `base::match`, `switch`                                  |   178 |
| `if let`, `let ... else`            |   517 | `if` on an optional or a pointer, `base::is`, `base::as` |       |
| `?` on a `Result`                   |   615 | `if`, and `return std::unexpected(...)`                  |   454 |
| `Option<T>`                         |       | `std::optional<T>`, or a pointer to what lives elsewhere | 1,161 |
| `#[derive(...)]`                    |   271 | `operator==` and `= delete`, written out                 |   614 |
| `trait`, `Box<dyn Trait>`           |       | A class with virtual functions, in a `std::unique_ptr`   |       |
| Generics, `impl Trait`              |       | Templates, with concepts                                 |   369 |
| `&str`, `&[T]`, with lifetimes      |    45 | `std::string_view`, `std::span<const T>`                 | 1,783 |
| A move, which is what `=` does      |       | `std::move`, which has to be written                     | 1,086 |
| `.clone()`, which has to be written |   337 | A copy, which is what `=` does                           |       |
| `as` between number types           | 1,133 | `static_cast`, `base::saturating_cast`                   | 1,173 |
| `unwrap`, `expect`, `panic!`        |   503 | `.value()`, `base::expect`, `base::panic`                |       |
| `unsafe`                            |   383 | Everything                                               |       |
| `mod`                               |   162 | `namespace`                                              |   626 |
| `use`                               | 1,217 | `#include`                                               | 4,245 |
| `#[test]`, `assert!`                | 2,679 | `TEST`, `CHECK`                                          | 3,167 |

The counts are of the words in the code, found with regular expressions, so they're near enough
and no nearer. Some rows are the same thing with another name. Others moved a decision:

- **Moves and copies swap places.** In Rust a value moves unless the code asks for a copy with
  `.clone()`, and the compiler refuses a use of what moved. In C++ it copies unless the code asks
  for a move with `std::move`, and a use of what moved compiles. So the C++ says `std::move` 1,086
  times, and a missing one is a copy that nobody sees: a tree of widgets copied each frame would
  show only in a benchmark. The types that own something costly are move-only for that reason.
- **A reference that's kept needs a comment instead of a lifetime.** The Rust had 45 lifetime
  parameters, on the types that borrow. The C++ has `std::string_view` and `std::span` in 1,783
  places, nearly all parameters, and where one is a member, a comment says what must outlive it.
- **A program is a template argument.** Rust's `trait Program` with its `type Message` is a base
  class `Program<Message>`, and `user_interface::run` and `Headless` are templates over the
  program, so its `update` and `view` are called directly, as in Rust. Widgets are the one place
  with virtual functions: a tree holds `std::unique_ptr<Widget<Message>>`, as the Rust held
  `Box<dyn Widget<Message>>`.
- **Privacy is a class's, not a module's.** A Rust module's functions see each other's private
  parts. In C++ what a file keeps to itself goes in an unnamed namespace in its `.cpp`, and what a
  type keeps to itself is `private`, so some helpers that were a module's became members, and some
  fields that a neighbor read became public.
- **Names.** Functions, variables and namespaces are `snake_case` and types `PascalCase` in both.
  Constants are `PascalCase` in the C++, where Rust shouts them, because the Windows SDK defines
  thousands of macros in capitals, like `DELETE`, `TRANSPARENT` and `NEAR`, and a constant of the
  same name would silently become its number. `near`, `far`, `small` and `interface` are macros
  too, so those names end in an underscore.

### What C++ needed

`Library/Base` is what the port had to write before it could write anything else, 1,294 lines:

| In Base                                   | For Rust's                                              |
| ----------------------------------------- | ------------------------------------------------------- |
| `Sum.h`: `match`, `is`, `as`, `make`      | `enum`, `match`, `if let`                               |
| `Panic.h`: `panic`, `unreachable`         | `panic!`, with its message on standard error            |
| `Windows.h`: `check`, `WindowsError`      | `windows-core`'s `Result` and `?`                       |
| `Text.h`: `chars`, `trim`, `split`, ...   | `str`: UTF-8 by character, which `std::string` isn't    |
| `Text.h`: `display`, `debug`, `parse_f64` | `{}`, `{:?}` and `parse` of numbers                     |
| `Numbers.h`: `saturating_cast`, ...       | `as`, `saturating_sub`, `wrapping_add`, `rem_euclid`    |
| `VecDeque.h`                              | `VecDeque`: MSVC's `std::deque` allocates for each item |
| `Files.h`                                 | `std::fs` with errors as values                         |
| `Library/Testing`                         | `#[test]`, `assert_eq!` and `cargo test`'s filters      |

C++ has pieces of most of these. `std::variant` and `std::visit` are a sum type without names.
`std::format` writes numbers, but not as Rust does, and the programs' files and states are
compared by their bytes. `std::string` is bytes, with nothing that steps by character. A cast of
a `float` that's out of an integer's range is undefined, where Rust's `as` stops at the ends.

## What C++ lets through

The Rust compiler checks what a reference may outlive, what may be used after it moved, and what
two threads may share, and its programs check every index as they run. C++ checks none of that.
The port kept the Rust's structure, which had already passed those checks, so most of what the
Rust compiler would have refused never got written. What did get through was found three ways:
by the ported tests and the byte comparison with the Rust's frames; by the standard library's
checks in Debug, where the tests run; and by three adversarial reviews of the finished C++, which
[LogicReview.md](LogicReview.md#the-review-of-the-port-to-c) records.

What compiled without a warning at `/W4` and was wrong:

| What                                                                | In Rust                           | Found by                |
| ------------------------------------------------------------------- | --------------------------------- | ----------------------- |
| A table of `std::initializer_list`s, each pointing at a dead array  | Doesn't compile                   | A test                  |
| An index kept across frames, past a list that shrank                | Stops the program                 | A test, and the review  |
| `std::clamp` with its bounds crossed                                | Stops the program                 | The review              |
| Signed sums past the type's ends                                    | Stops in debug, wraps in release  | The review              |
| `std::min` and `std::max` giving a NaN back                         | `f32::min` gives the other number | The review              |
| A move-assignment of a value from itself, through a builder         | Doesn't compile                   | The review              |
| An exception that nothing catches, which destroys nothing           | A panic unwinds                   | The review              |
| An exception that leaves a function that Windows called             | A panic there stops the program   | The review              |
| A terminate handler that's only the first thread's                  | The panic hook is the process's   | The review              |
| The debug runtime's report of a corrupt heap going to a file        | -                                 | The review              |
| `std::to_chars` and `std::format` writing numbers another way       | -                                 | Comparing with the Rust |
| A sum of floats that starts from 0.0, where Rust's starts from -0.0 | -                                 | Comparing with the Rust |
| `std::deque` allocating for each item                               | -                                 | A benchmark             |

The build had its own: two files named `Text.cpp` in one project compiled into one object file,
the default `/LTCG:incremental` failed to link after some edits, and a constant named like one of
the Windows SDK's macros became a number.

None of them was a crash that a user would have seen in what the tests and scripts reach, and the
reviewers' runs under AddressSanitizer and with random input found no read or write outside what
the code owns there. Two could have been one in what they don't reach: the stale paint in
MeshModeler, which read and wrote past a vector on a frame with two messages, and the kept index
into a menu. In Rust both were a clean stop with a message. That's the difference in one line:
the same mistakes get made in both, and Rust turns each into an error when it compiles or a stop
when it runs, where C++ turns some into anything at all.

What the C++ does about it, as [AGENTS.md](../AGENTS.md) now says:

- Tests build in Debug, where MSVC's standard library checks every index and iterator.
- `Program\Build\Build.ps1 --hardened` builds the optimized programs with the standard library's
  checks on. [Hardened](#hardened) has what that costs: nothing that shows, but in the glTF reader.
- What input, a file or a program using a library can make wrong is checked by hand, and stops the
  program with `base::panic`, which says why.
- A rule that the compiler doesn't keep is a rule that a review has to: a lambda that's kept
  captures by value, a view that's a member says what must outlive it, and a function that
  Windows calls back is `noexcept`.

The reverse is worth a line. The Rust said `unsafe` 383 times, nearly all around calls into
Windows, where the compiler takes the code's word. Those places were short, and marked. In C++
nothing marks the code that matters most to read twice.

## Building and testing

This is where the two differ most. Seconds, through RunLightly, which gives either build 8 jobs.
Everything is every library, program and test: 62 projects and 36 exes in C++, and 47 packages
with `--all-targets` in Rust.

| What                                                             |   C++ | Rust | C++ / Rust |
| ---------------------------------------------------------------- | ----: | ---: | ---------: |
| Everything, optimized, from nothing                              |   335 | 30.7 |       10.9 |
| Everything, optimized, with nothing changed                      |   3.1 |  1.0 |        3.1 |
| UiShowcase, optimized, after touching one widget's source file   |  17.3 |  5.4 |        3.2 |
| Everything, optimized, after touching one widget's source file   |   171 | 20.3 |        8.4 |
| UiShowcase, optimized, after touching the UI core's `Interface`  |  44.8 |  5.5 |        8.1 |
| Everything, optimized, after touching the UI core's `Interface`  |   212 | 20.3 |       10.5 |
| Everything, in debug, from nothing                               |   132 | 24.2 |        5.4 |
| Everything, in debug, with nothing changed                       |   3.3 |  1.1 |        3.0 |
| The UI library's tests, in debug, after one widget's source file |   6.3 |  7.6 |       0.83 |
| The UI library's tests, in debug, after the core's `Interface`   |  31.8 |  7.6 |        4.2 |
| Every test, run, in debug                                        |  61.9 | 28.5 |        2.2 |
| What an optimized build of everything leaves on the disk, in MB  | 6,488 |  589 |       11.0 |

Without RunLightly, with every core and nothing else running, both build about three times as
fast, and the gap between them stays:

| What                                                             |  C++ | Rust | C++ / Rust |
| ---------------------------------------------------------------- | ---: | ---: | ---------: |
| Everything, optimized, from nothing                              |  117 | 12.6 |        9.3 |
| Everything, optimized, with nothing changed                      |  1.3 |  0.2 |        6.5 |
| UiShowcase, optimized, after touching one widget's source file   |  8.1 |  3.2 |        2.5 |
| Everything, optimized, after touching one widget's source file   | 62.6 |  6.8 |        9.2 |
| UiShowcase, optimized, after touching the UI core's `Interface`  | 25.4 |  3.1 |        8.2 |
| Everything, optimized, after touching the UI core's `Interface`  | 79.3 |  6.7 |       11.8 |
| Everything, in debug, from nothing                               | 54.7 | 10.6 |        5.2 |
| Everything, in debug, with nothing changed                       |  1.3 |  0.2 |        6.5 |
| The UI library's tests, in debug, after one widget's source file |  3.3 |  4.2 |       0.79 |
| Every test, run, in debug                                        | 48.3 | 21.4 |        2.3 |

The Rust builds in a tenth of the time, and it isn't the compilers' speed that does it:

- **Headers.** The C++ is 253 source files, and each compiles what it includes again: the
  repository's own headers, half of whose UI library is templates, and the standard library's,
  which are large at `/std:c++latest`. `<format>` alone is included by 119 of them. Rust compiles
  each of its 47 packages once, and what one exports is read from what was compiled. That's why
  touching `Interface.h` costs the C++ up to five times what touching a `.cpp` does, and costs
  the Rust the same as any file: it recompiles the package either way.
- **Optimizing as it links.** With `/GL`, the compiler leaves the code to be generated when a
  program links, for the whole program at once. It's worth a few percent of a frame, to judge by
  what it gives the Rust, and it's done again for each of the 36 exes, 23 of which are tests that
  link the same libraries. After one widget's source file changed, nearly all of the
  171 seconds was linking. The debug build, which doesn't do it, is 132 seconds.
- **Tests run one at a time.** A test project's tests run one after another, in Debug, where the
  standard library checks every iterator. Rust's ran 8 at once.
- **Debug information.** The C++ keeps it in every object file and beside every exe, in both
  configurations, which is most of what's on the disk. Cargo's release profile leaves it out.

Nothing was tried to shorten it. Precompiled headers or the standard library as a module would go
at the first, and building the tests without `/GL` at the second. As it is, a coding agent that
changes the UI library waits minutes for what took seconds, at every change.

The exes themselves are about the same size, the C++'s a little smaller. They load the C++
standard library from `msvcp140.dll`, which Visual Studio installs, where each Rust exe had what
it used of Rust's inside it:

| Program          | C++, KB | Rust, KB | C++ / Rust |
| ---------------- | ------: | -------: | ---------: |
| Benchmarks       |   2,913 |    4,219 |       0.69 |
| FrameTimes       |     408 |      398 |       1.03 |
| GameInputDemo    |   1,146 |    1,298 |       0.88 |
| IdentifyDisplays |     517 |      484 |       1.07 |
| MeshModeler      |   1,519 |    2,038 |       0.75 |
| MinimalSceneTest |     905 |    1,017 |       0.89 |
| Platformer       |     971 |    1,072 |       0.91 |
| SimplePaintDemo  |     973 |    1,087 |       0.90 |
| SpinningCarTest  |     996 |    1,109 |       0.90 |
| UiShowcase       |   1,215 |    1,366 |       0.89 |
| Veehiicuul       |   1,528 |    1,722 |       0.89 |

## What was decided

KGames stays in Rust. The C++ is kept on the `cpp-rewrite` branch, as it was when this was
measured, and isn't developed.

- **The C++ bought nothing that a person at the computer would notice.** In its window a program
  draws the same frames at the same rate, answers input as fast, and uses the same CPU time,
  memory and GPU. The few percent that the C++'s frames lead by in the benchmarks don't show
  there.
- **It costs the most where KGames is worked on.** Coding agents write KGames, in many sessions,
  and build after every change. A build that takes nine times as long is paid at each of them.
- **The compiler's checks are what the repository leans on.** [AGENTS.md](../AGENTS.md) says that
  only a rule that a machine checks holds, since a later session never read the conversation
  that made it. Rust checks lifetimes, moves, indexes and overflow in debug builds. In C++ those
  became thirteen rules for the code and its reviews to keep, and the port broke each of them at
  least once before a test or a review found it.
- **It's a quarter more code** to read and to keep true.

What speaks for the C++ is Windows' own headers and Microsoft's own `GameInput.h` in place of
generated bindings and `unsafe`, and smaller exes. Neither changes what the programs do.

Nothing else was measured. A language with a garbage collector would work against the input
latency that the programs are built for, and one without Rust's checks gives up what cost the
C++ the most.

### What the Rust took from it

Everything above is of the Rust as it was at commit `e171759`. After it, the Rust took what the
comparison showed about it.

**The defects that were the Rust's too.** The reviews of the C++ read it against the Rust, so
what they found in the logic was in both.
[LogicReview.md](LogicReview.md#what-the-rewrite-in-c-found) lists them: two in MeshModeler, one
in `ListView`, Veehiicuul's terrain at the ends of an `i32`, and two sums past an `i64`'s ends.
Each has a test now.

**The places where the Rust was behind, or slower than it had to be.** The profiles that
explained the C++'s numbers explained the Rust's too:

- **SipHash.** The interface's own tables, of glyphs, layouts and widgets' memory, hashed their
  keys with `HashMap`'s default, which was a seventh of a frame's time. They hash with a multiply
  and a fold now, in `Library/UserInterface/src/drawing/hashing.rs`.
- **Calls across packages.** The vector and matrix functions of `Library/Math`, and the ray's
  test of a triangle, are `#[inline]`, which is what link-time optimization had done for them.
- **JSON** reads and writes a run of plain characters at once, as the C++ came to.

Each is a few lines, and none changes what the programs do: the 52 scripts save the frames and
states that they saved before, byte for byte. Through RunLightly, as in
[Performance](#performance), the Rust after against the Rust before:

| Benchmark                          | Before, µs | After, µs | After / Before |
| ---------------------------------- | ---------: | --------: | -------------: |
| `ui/UiShowcase/frame/steady`       |        192 |       141 |           0.74 |
| `ui/UiShowcase/frame/moving`       |        198 |       147 |           0.74 |
| `ui/SimplePaintDemo/frame/steady`  |       84.0 |      54.6 |           0.65 |
| `ui/MeshModeler/frame/steady`      |        115 |      90.9 |           0.79 |
| `ui/GameInputDemo/frame/steady`    |       75.7 |      49.0 |           0.65 |
| `ui/Veehiicuul/frame/steady`       |       74.8 |      57.5 |           0.77 |
| `ui/MinimalSceneTest/frame/steady` |       18.5 |      12.0 |           0.65 |
| `ui/SpinningCarTest/frame/steady`  |       21.4 |      14.1 |           0.66 |
| `ui/UiShowcase/start`              |      2,600 |     2,481 |           0.95 |
| `ui/Veehiicuul/start`              |      3,168 |     3,067 |           0.97 |
| `text/layout/cached`               |      0.047 |     0.030 |           0.61 |
| `text/quads/cached`                |      0.024 |     0.013 |           0.52 |
| `mesh/ray-cast/triangles`          |        513 |       124 |           0.24 |
| `mesh/ray-cast/mesh`               |      2,744 |     2,562 |           0.93 |
| `math/matrix/multiply`             |      0.010 |     0.007 |           0.66 |
| `math/matrix/transform-points`     |      0.002 |     0.001 |           0.60 |
| `json/parse/scene`                 |       58.9 |      34.2 |           0.58 |
| `json/write/scene`                 |        102 |      58.6 |           0.57 |

The other rows didn't move. A frame of the programs takes 0.65 to 0.79 of what it did. The C++'s
frames took 0.84 to 1.05 of the same Rust's, so by the two sets of ratios the Rust's frames are
now cheaper than the C++'s were by a fifth to a quarter, and its JSON, which the C++ had passed,
is level with it or ahead again. The two weren't run against each other after this, and the C++
wasn't given the same look: its tables hash with MSVC's cheaper hash already. Nor was this
measured in a window, where, as [In real windows](#in-real-windows) says, a frame's time is the
display's with vertical sync and mostly the present's without.

## What wasn't measured

- **Another C++ compiler.** With clang-cl, the C++ would get LLVM's code generation, and what's
  MSVC's here would show apart from what's C++'s. The toolset isn't installed.
- **Profile-guided optimization**, for either.
- **A person's devices.** In the real windows the mouse was moved by injected input, and the
  latency is from GameInput's stamp to the screen. A real mouse, keyboard or gamepad, what they
  take before GameInput has a reading, and what the display takes after, weren't measured, nor
  was anything while a person used a program.
- **Borderless fullscreen**, which `F11` switches to, and displays at other rates than 120 Hz.
  The real windows were maximized.
- **Why a frame costs the C++ more in a window than in the benchmark.** A rough profile points at
  two functions, as [Input latency](#input-latency) says, and doesn't account for the rest.
- **`GameInputSource` with real devices, and IdentifyDisplays' boxes**, which no test or script
  runs in either language.

## To reproduce

The C++ is the `cpp-rewrite` branch, in a folder of its own:

```powershell
Program\RunLightly\RunLightly.ps1 Program\Build\Build.ps1 --release Benchmarks
Program\RunLightly\RunLightly.ps1 Program\Build\Build.ps1 --release --hardened Benchmarks
Program\RunLightly\RunLightly.ps1 target\Release\Benchmarks.exe --filter mesh --json target\Benchmarks\Mesh.json
```

and the Rust is this repository as it was at commit `e171759`, in another:

```powershell
Program\RunLightly\RunLightly.ps1 cargo build --release -p benchmarks
Program\RunLightly\RunLightly.ps1 target\release\benchmarks.exe --filter mesh --json target\Benchmarks\Mesh.json
```

[As a person runs them](#as-a-person-runs-them) ran the same exes without RunLightly. For the
performance cores, each Benchmarks process was started with its affinity set to them, which on
this CPU is the mask `0xC03C03`. For the real windows, a script started each program with
`--display Primary`, and `--no-vsync` for the runs without vertical sync, gave its window the
focus, moved the mouse with `SendInput`, and recorded it with
`PresentMon --process_id <id> --timed 8 --qpc_time --output_file <csv>` or, without PresentMon,
with `QueryProcessCycleTime`. Those scripts aren't in the repository: they take the focus and
inject input, which [AGENTS.md](../AGENTS.md) forbids an agent that hasn't been given the
computer, as this one was for these measurements.

Both programs take the same arguments and run the same benchmarks, as
[Benchmarks' README](../Program/Benchmarks/README.md) says. Run the two one after the other, a
group at a time, for several rounds, and compare within a round: the computer's load moves both by
more than they differ. The Rust with link-time optimization is
`cargo build --release -p benchmarks --config "profile.release.lto='fat'" --config profile.release.codegen-units=1`.
