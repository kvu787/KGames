# SpinningCarTest

A car moves back and forth at 8 units per second while it turns at 90 degrees per second, past a
sphere, over a baked image of the backdrop, the green ground and the red cube, seen by an
orthographic camera. The car and the sphere are painted with
[SimplePaint](../../Library/SimplePaint/README.md). Like every KGames GUI program, it's built
from the [Library](../../Library) crates on Win32 and Direct3D 12, ignores input without focus,
and supports the two window setups in
[Library/Application](../../Library/Application/README.md#supported-window-setups).

## Scene

| Part       | What it is                                                                  |
| ---------- | --------------------------------------------------------------------------- |
| Camera     | Orthographic, 5 units tall, from 1 to 20 units in front of it. It looks at  |
|            | the origin from 5 units away, 45 degrees above the horizon and 45 degrees   |
|            | from +Z toward +X                                                           |
| Car        | The mesh in `Source/CarMesh.rs`, at a quarter of its size. It goes from the |
|            | middle to X = 7, back to -7, and again, at 8 units per second, and turns    |
|            | at 90 degrees per second. `S` keeps it in the middle, facing +X             |
| Sphere     | Radius 0.4, on the ground at X = 1.5 and Z = -1.5                           |
| Background | `Assets/SceneBackground.png`, a baked image of the backdrop, the ground and |
|            | the cube, drawn by `Shaders/Background.hlsl`                                |
| Frames     | Through Library/Graphics' renderer, paced for the lowest latency, without   |
|            | waiting for vertical sync                                                   |

The background is a 32:9 image, center-cropped to the window, so the camera's vertical scale
stays the same at any width. It's drawn first, without depth, and the car and the sphere go
over it, with a depth buffer between them.

## Use

| Key   | Action                                                    |
| ----- | --------------------------------------------------------- |
| `F1`  | Shows or hides the text at the bottom left                |
| `F11` | Switches between maximized and borderless fullscreen      |
| `S`   | Keeps the car in the middle, facing +X, or lets it move   |

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which opens the test on the primary display. `--display` and
`--no-activate` work as in every program.

## Settings

`Settings.json` has the settings every program has, then a SimplePaint material for each part
of the car and for the sphere, in the format of
[Library/PaintSettings](../../Library/PaintSettings/README.md), and the sphere's resolution:

| Setting                        | Value                                                        |
| ------------------------------ | ------------------------------------------------------------ |
| `SimplePaintShader_Axles`      | The axles' material                                          |
| `SimplePaintShader_Body`       | The body's material                                          |
| `SimplePaintShader_Cabin`      | The cabin's material                                         |
| `SimplePaintShader_Headlights` | The headlights' material                                     |
| `SimplePaintShader_Wheels`     | The wheels' material                                         |
| `SimplePaintShader_Sphere`     | The sphere's material                                        |
| `Sphere`                       | `UResolution`, from 3 to 512 vertices around each ring, and  |
|                                | `VResolution`, from 2 to 512 bands from pole to pole         |

Tune a material in [SimplePaintDemo](../SimplePaintDemo/README.md), save it, and copy it from
its `Paint.json`.

## Testing

```powershell
cargo test -p spinning_car_test_tests
Program\RunInBackground\RunInBackground.ps1 SpinningCarTest --all-scripts
```

The tests in `Test/SpinningCarTest` check the car's path and turn, the stationary car, the keys,
the settings file, and that the car mesh is what SimplePaint accepts, without a window. The scripts in `Test/SpinningCarTest/Scripts` run the
real test on a fixed clock, as [Library/Application](../../Library/Application/README.md#scripts)
describes, and check where the car is frame by frame, and what F1, F11 and S do. The state
that scripts save and check has `CarPosition`, `CarTurn`, `Stationary` and `TextShown`.

## Layout

| Path                         | Contents                                                               |
| ---------------------------- | ---------------------------------------------------------------------- |
| `Source/Main.rs`             | The main loop: settings, window, input, and drawing each frame         |
| `Source/Library.rs`          | The test's parts, as a library that `Test/SpinningCarTest` tests       |
| `Source/Controls.rs`         | What each key does                                                     |
| `Source/Scene.rs`            | The background, the car and the sphere, the camera and the animation   |
| `Source/CarMesh.rs`          | The car mesh                                                           |
| `Source/Settings.rs`         | The materials and the sphere's resolution, and their defaults          |
| `Shaders/Background.hlsl`    | Draws the background image, center-cropped, with its root signature    |
| `Assets/SceneBackground.png` | The baked background                                                   |
| `Build.rs`                   | Compiles SimplePaint's shaders for six materials, and the background's |
| `HumanRun.ps1`               | Builds the test and runs it on the primary display, for people to use  |
| `HumanRun.cmd`               | Runs `HumanRun.ps1`, for double-clicking in File Explorer              |
| `Settings.json`              | The settings, which the test writes if it's missing                    |
