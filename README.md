This repo will contain some of the games I make.

I plan to develop the latest versions of Veehiicuul and Automatou in here.

# Motivation

This repo aims to minimize dependencies on things like game engines and UI frameworks like Unity,
Godot, WPF, Dear ImGui, Electron, etc.

This decision was primarily motivated by my experience working with those dependencies, but here are a summary of the reasons:

- Agents are now good enough to handle the significantly increased development burden of using low-level APIs.
- Maintain precise control over window behavior, input latency, vsync behavior.
- Avoid memory performance issues and frametime spikes caused by garbage collection.
- Avoid annoying bugs and quirks caused by dependencies.
- Avoid opinionated workflows and fundamental design decisions that I don't like that are imposed by dependencies.
- Avoid maintenance burden, bugs, churn, etc introduced by new versions of dependencies that don't benefit my specific usage.
- Make it easier to configure a non-disruptive agent workflow.

# References

- SDL 3
- Godot Engine
- Blender
- GLFW
- winit
- Bevy Engine
- iced
- DirectXTK12
- Mindustry

# Design

- Folders
  - Program: Contains things that are runnable.
  - Library: Contains things that aren't runnable and are shared.
  - Test: Contains all tests.
- GUI apps
  - If the window isn't focused, the app must ignore all input.
  - Use Win32, Rust, Direct3D 12, DirectWrite
  - Borderless fullscreen on a 2560 x 1440 display, the one window setup
  - F9 to toggle between light and dark mode
  - F10 to toggle VSync
  - Each program's Settings.json says which of those it starts in
  - Optimize for minimal input latency

GUI programs are built with [Library/user-interface](Library/user-interface/README.md), a user
interface library for Win32, Direct3D 12 and DirectWrite after Microsoft's Fluent 2 design system,
with WinUI 3's own colors and sizes but none of its animations: its main loop runs them, its widgets are their
controls, and they look and act like Windows 11's, in light or dark. [UiShowcase](Program/ui-showcase/README.md) shows every widget. GUI programs ignore
input without focus, including gamepads, and a click that gives a window focus does only that. The
library's README has the details, and the one window setup that programs support.
[AGENTS.md](AGENTS.md) has the rules for coding agents, like how to run programs without disrupting
you.

KGames was once rewritten in C++, to compare it with the Rust in performance and in the structure
of the code. KGames stayed in Rust, and the C++ is kept on the `cpp-rewrite` branch.

Programs read all input with GameInput 4.0.306, which you install once, from an elevated prompt:

```powershell
winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"
```

KGames doesn't build without it, and programs don't start without it. Building also needs the .NET
SDK, 10 or later, whose `dotnet` restores GameInput's NuGet package. Visual Studio installs it with
its .NET desktop development workload. [Library/game-input](Library/game-input/README.md) says how
programs start it.

# misc notes

SetCursor(NULL)
WM_SETCURSOR
ClipCursor
GetForegroundWindow
GetClipCursor
That's the argument for keeping it in one pixel at the middle, as SDL does.
WM_ACTIVATE
GetAsyncKeyState
IsIconic
exactly one window
deal with SetCursorPos and ClipCursor unreliability
it seems like clipcursor is a requirement even despite its unreliability

## Program

| Program                                                  | What it is                                                               |
| -------------------------------------------------------- | ------------------------------------------------------------------------ |
| [Veehiicuul](Program/veehiicuul/README.md)               | A racing game seen from above, and the editors of its levels             |
| [Platformer](Program/platformer/README.md)               | A 2D platformer that runs on events, with nothing a frame late           |
| [GameInputDemo](Program/game-input-demo/README.md)       | Shows what gamepads, keyboards and mice send through GameInput           |
| [SimplePaintDemo](Program/simple-paint-demo/README.md)   | Edits a SimplePaint material live on a UV sphere                         |
| [MeshModeler](Program/mesh-modeler/README.md)            | Models polygon meshes, subdivides them, and paints them with SimplePaint |
| [UiShowcase](Program/ui-showcase/README.md)              | Shows every widget of the UI framework, and what input does to it        |
| [ShadowTesting](Program/shadow-testing/README.md)        | Tries ways of drawing a box's shadow, with sliders for their settings    |
| [WinUi3Sample](Program/WinUi3Sample/README.md)           | A real WinUI 3 app, in C# and XAML, to compare the UI framework with     |
| [MinimalSceneTest](Program/minimal-scene-test/README.md) | A sphere moving over a ground plane, for checking smooth motion          |
| [SpinningCarTest](Program/spinning-car-test/README.md)   | A car moving and spinning past a sphere, painted with SimplePaint        |
| [IdentifyDisplays](Program/identify-displays/README.md)  | Prints each display's identifier, for the programs' `--display`          |
| [RunInBackground](Program/RunInBackground/README.md)     | Runs a GUI program without taking focus, and screenshots it              |
| [RunLightly](Program/RunLightly/README.md)               | Runs a command so that a game you're playing doesn't stutter             |
| [FrameTimes](Program/frame-times/README.md)              | Measures another process's frame times, like a game's while a build runs |
| [Benchmarks](Program/benchmarks/README.md)               | Times the hot paths: frames on the CPU and GPU, text, meshes, and more   |

## Library

| Library                                             | What it is                                                                                                  |
| --------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| [UserInterface](Library/user-interface/README.md)   | The UI library, after Fluent 2: the window, input, text, widgets, the main loop, scripts and headless tests |
| [Graphics](Library/graphics/README.md)              | Direct3D 12: the GPU, the low-latency swap chain, frames, and offscreen targets                             |
| [SimplePaint](Library/simple-paint/README.md)       | The SimplePaint shader: its specification, Rust and HLSL                                                    |
| [GameInputRaw](Library/game-input-raw/README.md)    | GameInput 4.0.306's bindings for keyboards, mice and gamepads, and `GameInput.lib` from its NuGet package   |
| [GameInput](Library/game-input/README.md)           | Starts GameInput, once it has checked that GameInput 4.0.306's runtime is installed                         |
| [Displays](Library/displays/README.md)              | Finds the displays, and gives each a lasting identifier                                                     |
| [PresentTiming](Library/present-timing/README.md)   | When each of a process's presents reached the screen, by PresentMon's rules                                 |
| [Math](Library/math/README.md)                      | Vectors and matrices, in DirectXMath's conventions                                                          |
| [Meshes](Library/meshes/README.md)                  | A UV sphere, a box, a plane and the car, for SimplePaint                                                    |
| [PolygonMesh](Library/polygon-mesh/README.md)       | Polygon meshes: their edits, selections, Catmull-Clark subdivision, and files                               |
| [PaintSettings](Library/paint-settings/README.md)   | SimplePaint materials in JSON settings files                                                                |
| [Gltf](Library/gltf/README.md)                      | Reads models from the .glb files that Blender exports                                                       |
| [Json](Library/json/README.md)                      | A JSON parser and writer                                                                                    |
| [ShaderCompiler](Library/shader-compiler/README.md) | Compiles HLSL with the Windows SDK's `dxc.exe`, at build time                                               |
| [WindowsApiRaw](Library/windows-api-raw/README.md)  | The generated Windows API bindings that every crate shares, and nothing else                                |
| [WindowsApi](Library/windows-api/README.md)         | What KGames adds to the Windows API bindings, like `check_handle`                                           |

## Documentation

`Documentation` holds design notes for what isn't code yet, like
[VeehiicuulFly](Documentation/VeehiicuulFly.md), a flying game.

## Test

Each folder in `Test` tests the library or program of the same name. A GUI program's folder
also has `Scripts`, which run the real program with input from a script instead of yours, on a
fixed clock, without taking focus (see [RunInBackground](Program/RunInBackground/README.md#scripts)).
Every Rust package is in one Cargo workspace, whose `Cargo.lock` and `target` folder are at the
root.

```powershell
cargo build --release --workspace
cargo test --workspace
Program\RunInBackground\RunInBackground.ps1 simple-paint-demo --all-scripts
```

The GPU tests need a GPU with Shader Model 6.0, and run on WARP too. Scripts open the program
on display CVVNY2 unless `--display` names another.

## Editors

VS Code with rust-analyzer and RustRover can have the repository open at once. Each builds what
its analysis needs, build scripts, proc macros and `cargo check`, in a target folder of its own,
so that neither waits on the other's build, or on a `cargo test --workspace` in `target`, nor
rebuilds over it:

| Editor                  | Its target folder  | Where it's set                        |
| ----------------------- | ------------------ | ------------------------------------- |
| VS Code (rust-analyzer) | `target/VsCode`    | `.vscode/settings.json`               |
| RustRover               | `target/RustRover` | Settings, Rust, Environment variables |

RustRover keeps its settings in `.idea`, which Git leaves out, so its folder is yours to set, once:
`CARGO_TARGET_DIR=target\RustRover`.

# License

KGames is free software under the GNU General Public License, version 3 or (at your option) any
later version: GPL-3.0-or-later. [LICENSE.txt](LICENSE.txt) has its text.
