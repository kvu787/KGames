# SpinningCarTest

A car moves back and forth at 8 units per second while it turns at 90 degrees per second, past a
sphere and a red cube, on green ground, seen by an orthographic camera. The car and the sphere
are painted with [SimplePaint](../../Library/SimplePaint/README.md), and the ground and the cube
are lit by one light, like in the C++ scene that the test is a port of. Every frame draws all of
it. Like every KGames GUI program, it's built on
[Library/UserInterface](../../Library/UserInterface/README.md), with Win32 and Direct3D 12,
ignores input without focus, and supports the two
[window setups](../../Library/UserInterface/README.md#window-setups).

## Scene

| Part       | What it is                                                                 |
| ---------- | -------------------------------------------------------------------------- |
| Camera     | Orthographic, 5 units tall, from 1 to 20 units in front of it. It looks at |
|            | the origin from 5 units away, 45 degrees above the horizon and 45 degrees  |
|            | from +Z toward +X                                                          |
| Car        | The car of Library/Meshes, at a quarter of its size. It goes from the      |
|            | middle to X = 7, back to -7, and again, at 8 units per second, and turns   |
|            | at 90 degrees per second, by the time since the test started, so it keeps  |
|            | going while the window is minimized. `S` keeps it in the middle, facing +X |
| Sphere     | Radius 0.4, on the ground at X = 1.5 and Z = -1.5                          |
| Ground     | A plane 10 units along X and 5 along Z, at Y = 0                           |
| Cube       | 1 unit on each side, on the ground at Z = -1.5                             |
| Lighting   | Each face of the ground and the cube is a flat color: its sRGB color times |
|            | 0.28, plus 0.72 times how much it faces the light, which is above, between |
|            | +X and +Z. `Shaders/FlatShading.hlsl` draws them                           |
| Background | #4c4c4c                                                                    |
| Frames     | Through Library/UserInterface's main loop, paced for the lowest latency,   |
|            | without waiting for vertical sync                                          |

Every frame draws the whole scene, with one depth buffer as big as the window: the ground and the
cube, in colors that are stored as they are, then the car and the sphere, whose linear colors are
encoded to sRGB. The view is as wide as the window, at either supported height, and the card
goes over it.

## Use

| Key   | Action                                                  |
| ----- | ------------------------------------------------------- |
| `F1`  | Shows or hides the card at the bottom left              |
| `F11` | Switches between maximized and borderless fullscreen    |
| `S`   | Keeps the car in the middle, facing +X, or lets it move |

The card floats over the scene at the bottom left, which the car never crosses, and is as tall as
what it says: the test's name and what the scene shows, the frame rate, the GPU, the display,
whether the car moves, whether the window has focus, with an info bar's status icon, the keys on
keycaps, and what's wrong with `Settings.json`, if anything, as warning info bars.

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which opens the test on the primary display. `--display` and
`--no-activate` work as in every program.

## Settings

`Settings.json` has the settings every program has, then a SimplePaint material for each part
of the car and for the sphere, in the format of
[Library/PaintSettings](../../Library/PaintSettings/README.md), and the sphere's resolution:

| Setting                        | Value                                                       |
| ------------------------------ | ----------------------------------------------------------- |
| `SimplePaintShader_Axles`      | The axles' material                                         |
| `SimplePaintShader_Body`       | The body's material                                         |
| `SimplePaintShader_Cabin`      | The cabin's material                                        |
| `SimplePaintShader_Headlights` | The headlights' material                                    |
| `SimplePaintShader_Wheels`     | The wheels' material                                        |
| `SimplePaintShader_Sphere`     | The sphere's material                                       |
| `Sphere`                       | `UResolution`, from 4 to 512 vertices around each ring, and |
|                                | `VResolution`, from 2 to 512 bands from pole to pole        |

A setting that's wrong, like a material that SimplePaint rejects, keeps its default, and the
card says what's wrong with it. Tune a material in [SimplePaintDemo](../SimplePaintDemo/README.md),
save it, and copy it from its `Paint.json`.

## Testing

```powershell
cargo test -p spinning_car_test_tests
Program\RunInBackground\RunInBackground.ps1 SpinningCarTest --all-scripts
```

The tests in `Test/SpinningCarTest` check, without a window, the car's path and turn, the
stationary car, what S, F1, F11 and the window's focus do, that the card stays out of the car's
path at both heights, the settings file, and frames drawn offscreen with the GPU. `Test/Meshes`
checks that the car mesh is what SimplePaint accepts. The scripts in
`Test/SpinningCarTest/Scripts` run the real test on a fixed clock, as
[Library/UserInterface](../../Library/UserInterface/README.md#testing) describes, and check where
the car is frame by frame, and what S, F1, F11 and the window's focus do. The state that scripts
save and check has `CarPosition`, `CarTurn`, `Stationary` and `TextShown`, and the interface's,
whose targets have the card as `InfoCard`, of kind `Card`, and what's wrong with `Settings.json`
as `Notice/0`, `Notice/1` and so on.

## Layout

| Path                       | Contents                                                                 |
| -------------------------- | ------------------------------------------------------------------------ |
| `Source/Main.rs`           | Runs the test with Library/UserInterface                                 |
| `Source/Library.rs`        | The test's parts, as a library that `Test/SpinningCarTest` tests         |
| `Source/SpinningCar.rs`    | The test as a `Program`: where the car is, the keys, and the state       |
| `Source/Controls.rs`       | What S does                                                              |
| `Source/Scene.rs`          | The objects, the camera, the light and the animation                     |
| `Source/InfoCard.rs`       | The card, and F1, the same as MinimalSceneTest's                         |
| `Source/Depth.rs`          | The scene's depth buffer, the same as MinimalSceneTest's                 |
| `Source/Settings.rs`       | The materials and the sphere's resolution, and their defaults            |
| `Shaders/FlatShading.hlsl` | Draws the ground and the cube, with its root signature, which is         |
|                            | SimplePaint's, so SimplePaint's pipeline and meshes draw with it         |
| `Build.rs`                 | Compiles SimplePaint's shaders for six materials, and `FlatShading.hlsl` |
| `HumanRun.ps1`             | Builds the test and runs it on the primary display, for people to use    |
| `HumanRun.cmd`             | Runs `HumanRun.ps1`, for double-clicking in File Explorer                |
| `Settings.json`            | The settings, which the test writes if it's missing                      |
