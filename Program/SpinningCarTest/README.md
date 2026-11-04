# SpinningCarTest

A car moves back and forth at 8 units per second while it turns at 90 degrees per second, past a
sphere and a red cube, on green ground, seen by an orthographic camera. The car and the sphere
are painted with [SimplePaint](../../Library/SimplePaint/README.md), and the ground and the cube
are lit by one light, like in the C++ scene that the test is a port of. Every frame draws all of
it. Like every KGames GUI program, it's built from the [Library](../../Library) crates on Win32
and Direct3D 12, ignores input without focus, and supports the two window setups in
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
| Ground     | A plane 10 units along X and 5 along Z, at Y = 0                            |
| Cube       | 1 unit on each side, on the ground at Z = -1.5                              |
| Lighting   | Each face of the ground and the cube is a flat color: its sRGB color times  |
|            | 0.28, plus 0.72 times how much it faces the light, which is above, between  |
|            | +X and +Z. `Shaders/FlatShading.hlsl` draws them                            |
| Background | #4c4c4c                                                                     |
| Frames     | Through Library/Graphics' renderer, paced for the lowest latency, without   |
|            | waiting for vertical sync                                                   |

Every frame draws the whole scene, with one depth buffer: the ground and the cube, in colors that
are stored as they are, then the car and the sphere, whose linear colors are encoded to sRGB. The
view is as wide as the window, at either supported height.

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
| `Sphere`                       | `UResolution`, from 4 to 512 vertices around each ring, and  |
|                                | `VResolution`, from 2 to 512 bands from pole to pole         |

Tune a material in [SimplePaintDemo](../SimplePaintDemo/README.md), save it, and copy it from
its `Paint.json`.

## Testing

```powershell
cargo test -p spinning_car_test_tests
Program\RunInBackground\RunInBackground.ps1 SpinningCarTest --all-scripts
```

The tests in `Test/SpinningCarTest` check the car's path and turn, the stationary car, the S key,
the settings file, and that the car mesh is what SimplePaint accepts, without a window, and
`Test/SceneTest` checks F1 and F11. The scripts in `Test/SpinningCarTest/Scripts` run the real
test on a fixed clock, as [Library/Application](../../Library/Application/README.md#scripts)
describes, and check where the car is frame by frame, and what F1, F11 and S do. The state
that scripts save and check has `CarPosition`, `CarTurn`, `Stationary` and `TextShown`.

## Layout

| Path                       | Contents                                                                 |
| -------------------------- | ------------------------------------------------------------------------ |
| `Source/Main.rs`           | The scene, with the car's state, for Library/SceneTest's main loop       |
| `Source/Library.rs`        | The test's parts, as a library that `Test/SpinningCarTest` tests         |
| `Source/Controls.rs`       | What S does                                                              |
| `Source/Scene.rs`          | The objects, the camera, the light and the animation                     |
| `Source/CarMesh.rs`        | The car mesh                                                             |
| `Source/Settings.rs`       | The materials and the sphere's resolution, and their defaults            |
| `Shaders/FlatShading.hlsl` | Draws the ground and the cube, with its root signature, which is         |
|                            | SimplePaint's, so SimplePaint's pipeline and meshes draw with it         |
| `Build.rs`                 | Compiles SimplePaint's shaders for six materials, and `FlatShading.hlsl` |
| `HumanRun.ps1`             | Builds the test and runs it on the primary display, for people to use    |
| `HumanRun.cmd`             | Runs `HumanRun.ps1`, for double-clicking in File Explorer                |
| `Settings.json`            | The settings, which the test writes if it's missing                      |
