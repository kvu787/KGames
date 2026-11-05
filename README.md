This repo will contain some of the games I make.

I plan to develop the latest versions of Veehiicuul and Automatou in here.

# TODO

- Implement in-app and logged measurement for these
  - Input latency
  - Frame time
  - Jitters
- Implement in-app frame rate limiter
- Implement vsync option
- Implement a scene that displays nothing or almost nothing to measure base fps
- Figure out how to design or constrain UI

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
  - Use Win32, Rust, Direct3D 12, DirectWrite
  - Use a light theme by default
  - F11 to toggle borderless fullscreen
  - Optimize for minimal input latency

GUI programs are built with [Library/UserInterface](Library/UserInterface/README.md), whose main
loop runs them, and whose widgets are their controls. [UiShowcase](Program/UiShowcase/README.md)
shows every widget. GUI programs ignore input without focus, including gamepads, and a click that
gives a window focus does only that. [Library/Application](Library/Application/README.md) has the
details, and the two window setups that programs support. [AGENTS.md](AGENTS.md) has the rules for
coding agents, like how to run programs without disrupting you.

## Program

| Program                                                       | What it is                                                        |
| ------------------------------------------------------------- | ----------------------------------------------------------------- |
| [Veehiicuul](Program/Veehiicuul/README.md)                    | A racing game seen from above, driven with a gamepad or the keys  |
| [GameInputDemo](Program/GameInputDemo/README.md)              | Shows what gamepads, keyboards and mice send through GameInput    |
| [SimplePaintDemo](Program/SimplePaintDemo/README.md)          | Edits a SimplePaint material live on a UV sphere                  |
| [UiShowcase](Program/UiShowcase/README.md)                    | Shows every widget of the UI framework, and what input does to it |
| [MinimalSceneTest](Program/MinimalSceneTest/README.md)        | A sphere moving over a ground plane, for checking smooth motion   |
| [SpinningCarTest](Program/SpinningCarTest/README.md)          | A car moving and spinning past a sphere, painted with SimplePaint |
| [IdentifyDisplays](Program/IdentifyDisplays/README.md)        | Shows each display's identifier, for the programs' `--display`    |
| [RunInBackground](Program/RunInBackground/README.md)          | Runs a GUI program without taking focus, and screenshots it       |

## Library

| Library                                                       | What it is                                                        |
| ------------------------------------------------------------- | ----------------------------------------------------------------- |
| [Application](Library/Application/README.md)                  | The window, its input and the gamepad's, settings, and the setups |
| [UserInterface](Library/UserInterface/README.md)              | The UI framework: widgets, input, and the main loop of programs   |
| [Graphics](Library/Graphics/README.md)                        | The Direct3D 12 renderer and its low-latency frame pacing         |
| [Canvas](Library/Canvas/README.md)                            | 2D text and shapes, the light theme, and rectangles               |
| [SimplePaint](Library/SimplePaint/README.md)                  | The SimplePaint shader: its specification, Rust and HLSL          |
| [GameInput](Library/GameInput/README.md)                      | GameInput bindings, and the runtime next to each exe              |
| [Displays](Library/Displays/README.md)                        | Finds the displays, and gives each a lasting identifier           |
| [Math](Library/Math/README.md)                                | Vectors and matrices, in DirectXMath's conventions                |
| [Meshes](Library/Meshes/README.md)                            | A UV sphere, a box, a plane and the car, for SimplePaint          |
| [PaintSettings](Library/PaintSettings/README.md)              | SimplePaint materials in JSON settings files                      |
| [SceneTest](Library/SceneTest/README.md)                      | The loop and text box of MinimalSceneTest and SpinningCarTest     |
| [Json](Library/Json/README.md)                                | A JSON parser                                                     |
| [ShaderCompiler](Library/ShaderCompiler/README.md)            | Compiles HLSL with the Windows SDK's `dxc.exe`, at build time     |
| [WindowsApi](Library/WindowsApi/README.md)                    | The generated Windows API bindings that every crate shares        |

## Test

Each folder in `Test` tests the library or program of the same name. A GUI program's folder
also has `Scripts`, which run the real program with input from a script instead of yours, on a
fixed clock, without taking focus (see [RunInBackground](Program/RunInBackground/README.md#scripts)).
Every Rust package is in one Cargo workspace, whose `Cargo.lock` and `target` folder are at the
root.

```powershell
cargo build --release --workspace
cargo test --workspace
Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo --all-scripts
```

The GPU tests need a GPU with Shader Model 6.0, and run on WARP too. Scripts open the program
on display CVVNY2 unless `--display` names another.
