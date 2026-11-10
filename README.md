This repo will contain some of the games I make.

I plan to develop the latest versions of Veehiicuul and Automatou in here.

# TODO

KGames bugs that BUI showed up

BUI handles each Win32 key and character message as it arrives (BUI/kit/intern/kit_window_win32.c:656-703). KGames instead collects a whole frame's input first, and loses information doing so:

Keys and typed text lose their order within a frame. The text box handles all the frame's keys first, then all its typed text (Widgets/TextBox.cpp:260-340).
Typing "x" then Backspace in one frame deletes the old character and then inserts "x".
"a", Left, "b" gives "ab" instead of "ba".
Modifiers are read once, at the end of the frame, not per key.
Repeated presses collapse into one. key_pressed only asks whether the key was pressed at all this frame (Input/Events.cpp:159).
In Widgets/ListView.h:188-205, two Down presses in one frame move one row, and Down then Up always ends on Up, whatever the order.
That breaks AGENTS.md's rule that two of the same intent never collapse into one.
The survey says ComboBox, Menu and Navigation have the same problem. I didn't check those three myself.
Fix for bugs 1 and 2: keep one ordered list of key and character events, each key with its own modifiers, and have the widgets work through it in order.
When the glyph atlas fills, text is missing for a frame. When there's no room left, the glyph is simply not drawn (Drawing/Text.cpp:230-240, Drawing/Text.h:279-287), and the atlas only starts over on the next frame (Drawing/Text.cpp:411). A code comment even describes this. BUI never drops a glyph: it draws what it has, clears the atlas and carries on. For KGames, the fix is to start the atlas over and paint the frame again.

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

# Design

- Folders
  - Program: Contains things that are runnable.
  - Library: Contains things that aren't runnable and are shared.
  - Test: Contains all tests.
- GUI apps
  - If the window isn't focused, the app must ignore all input.
  - Use Win32, C++, Direct3D 12, DirectWrite
  - Use a light theme by default
  - F9 to toggle between light and dark mode
  - F10 to toggle VSync, on by default
  - F11 to toggle borderless fullscreen, windowed by default
  - Optimize for minimal input latency

GUI programs are built with [Library/UserInterface](Library/UserInterface/README.md), a user
interface library for Win32, Direct3D 12 and DirectWrite after Microsoft's Fluent 2 design system,
with WinUI 3's own colors and sizes but none of its animations: its main loop runs them, its widgets are their
controls, and they look and act like Windows 11's, in light or dark. [UiShowcase](Program/UiShowcase/README.md) shows every widget. GUI programs ignore
input without focus, including gamepads, and a click that gives a window focus does only that. The
library's README has the details, and the two window setups that programs support.
[AGENTS.md](AGENTS.md) has the rules for coding agents, like how to run programs without disrupting
you.

Programs read all input with GameInput 4.0.306, which you install once, from an elevated prompt:

```powershell
winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"
```

KGames doesn't build without it, and programs don't start without it. Building needs Visual Studio
with its Desktop development with C++ workload, for MSVC and MSBuild, and the .NET SDK, 10 or
later, whose `dotnet` restores GameInput's NuGet package, which Visual Studio installs with its
.NET desktop development workload. [Library/GameInput](Library/GameInput/README.md) says how
programs start it.

## Program

| Program                                                | What it is                                                               |
| ------------------------------------------------------ | ------------------------------------------------------------------------ |
| [Veehiicuul](Program/Veehiicuul/README.md)             | A racing game seen from above, and the editors of its levels             |
| [GameInputDemo](Program/GameInputDemo/README.md)       | Shows what gamepads, keyboards and mice send through GameInput           |
| [SimplePaintDemo](Program/SimplePaintDemo/README.md)   | Edits a SimplePaint material live on a UV sphere                         |
| [MeshModeler](Program/MeshModeler/README.md)           | Models polygon meshes, subdivides them, and paints them with SimplePaint |
| [UiShowcase](Program/UiShowcase/README.md)             | Shows every widget of the UI framework, and what input does to it        |
| [WinUi3Sample](Program/WinUi3Sample/README.md)         | A real WinUI 3 app, in C# and XAML, to compare the UI framework with     |
| [MinimalSceneTest](Program/MinimalSceneTest/README.md) | A sphere moving over a ground plane, for checking smooth motion          |
| [SpinningCarTest](Program/SpinningCarTest/README.md)   | A car moving and spinning past a sphere, painted with SimplePaint        |
| [IdentifyDisplays](Program/IdentifyDisplays/README.md) | Shows each display's identifier, for the programs' `--display`           |
| [RunInBackground](Program/RunInBackground/README.md)   | Runs a GUI program without taking focus, and screenshots it              |
| [FrameTimes](Program/FrameTimes/README.md)             | Measures another process's frame times, like a game's while a build runs |
| [Benchmarks](Program/Benchmarks/README.md)             | Times the hot paths: frames on the CPU and GPU, text, meshes, and more   |
| [RunLightly](Program/RunLightly/README.md)             | Runs a command without disturbing a game that someone's playing          |
| `Program\Build\Build.ps1`                              | Builds the C++ projects with Visual Studio's MSBuild                     |
| `Program\RunTests\RunTests.ps1`                        | Builds the tests in `Test` and runs them                                 |

## Library

| Library                                            | What it is                                                                                                  |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| [UserInterface](Library/UserInterface/README.md)   | The UI library, after Fluent 2: the window, input, text, widgets, the main loop, scripts and headless tests |
| [Graphics](Library/Graphics/README.md)             | Direct3D 12: the GPU, the low-latency swap chain, frames, and offscreen targets                             |
| [SimplePaint](Library/SimplePaint/README.md)       | The SimplePaint shader: its specification, C++ and HLSL                                                     |
| [GameInput](Library/GameInput/README.md)           | GameInput 4.0.306: its NuGet package's header, in `game_input`, and `GameInput.lib`, which starts it        |
| [Displays](Library/Displays/README.md)             | Finds the displays, and gives each a lasting identifier                                                     |
| [Math](Library/Math/README.md)                     | Vectors and matrices, in DirectXMath's conventions                                                          |
| [Meshes](Library/Meshes/README.md)                 | A UV sphere, a box, a plane and the car, for SimplePaint                                                    |
| [PolygonMesh](Library/PolygonMesh/README.md)       | Polygon meshes: their edits, selections, Catmull-Clark subdivision, and files                               |
| [PaintSettings](Library/PaintSettings/README.md)   | SimplePaint materials in JSON settings files                                                                |
| [Gltf](Library/Gltf/README.md)                     | Reads models from the .glb files that Blender exports                                                       |
| [Json](Library/Json/README.md)                     | A JSON parser                                                                                               |
| [ShaderCompiler](Library/ShaderCompiler/README.md) | Compiles HLSL with the Windows SDK's `dxc.exe`, as a test runs. Builds compile it with `KGames.targets`     |
| `Library/Base`                                     | What every project uses and C++'s standard library lacks: sum types, panics, UTF-8, numbers as text, files  |
| `Library/Testing`                                  | The test harness: `TEST`, `CHECK` and `CHECK_EQ`, and the `main` that runs a test project's tests           |

## Documentation

`Documentation` holds design notes for what isn't code yet, like
[VeehiicuulFly](Documentation/VeehiicuulFly.md), a flying game, and the analyses made with
[Benchmarks](Program/Benchmarks/README.md): [Performance](Documentation/Performance.md), where the
programs' time goes, and [Memory](Documentation/Memory.md), where their memory goes.
[BuiTakeaways](Documentation/BuiTakeaways.md) tracks what KGames can take from BUI, a UI toolkit
derived from Blender's, as tasks with their status.

KGames was written in Rust, and ported to C++ file for file.
[CppVersusRust](Documentation/CppVersusRust.md) compares the two: how fast each is, how much code
each takes, and what each language checks and doesn't.

## Test

Each folder in `Test` tests the library or program of the same name. A GUI program's folder
also has `Scripts`, which run the real program with input from a script instead of yours, on a
fixed clock, without taking focus (see [RunInBackground](Program/RunInBackground/README.md#scripts)).
Every C++ project is in `KGames.slnx`, which Visual Studio opens, and builds into the `target`
folder at the root. `KGames.props` and `KGames.targets` have what the projects share, and
[AGENTS.md](AGENTS.md) how the C++ is written.

```powershell
Program\Build\Build.ps1 --release
Program\RunTests\RunTests.ps1
Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo --all-scripts
```

The GPU tests need a GPU with Shader Model 6.0, and run on WARP too. Scripts open the program
on display CVVNY2 unless `--display` names another.

# License

KGames is free software under the GNU General Public License, version 3 or (at your option) any
later version: GPL-3.0-or-later. [LICENSE.txt](LICENSE.txt) has its text.
