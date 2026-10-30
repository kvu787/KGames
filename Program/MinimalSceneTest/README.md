# MinimalSceneTest

A port of VsyncStutterTest, a Godot scene for checking how smoothly frames reach the screen,
onto KGames' own stack: a sphere moves back and forth at 8 units per second, between the top
left and bottom right corners of the screen, above a ground plane and past a cube, seen by an
orthographic camera. Like every KGames GUI program, it's built from the
[Library](../../Library) crates on Win32 and Direct3D 12, ignores input without focus, and
supports the two window setups in
[Library/Application](../../Library/Application/README.md#supported-window-setups).

## From the Godot scene

| Godot                                              | Here                                                   |
| -------------------------------------------------- | ------------------------------------------------------ |
| `Main.tscn`'s camera, objects, sizes and positions | The same, in `Source/Scene.rs`                         |
| `Sphere.gd`: X moves by 8 units per second times   | The same, with the time since the last frame. It       |
| the frame's delta, and turns back past 7           | doesn't jump ahead after the window was minimized      |
| Fullscreen, without vertical sync                  | Borderless fullscreen by default, frames presented     |
|                                                    | without waiting for vertical sync, tearing allowed     |
| No antialiasing                                    | No antialiasing                                        |
| Lit materials, and an unshaded sphere              | [SimplePaint](../../Library/SimplePaint/README.md)     |
|                                                    | materials: lit ones paint their color on faces that    |
|                                                    | face up, and the sphere's has equal Dark and Light     |
|                                                    | Points, a constant color                               |
| A gray background                                  | The light theme's background                           |

Frames go through [Library/Graphics](../../Library/Graphics/README.md)' renderer, which reads
the time right before drawing and presents as soon as the GPU and DXGI can take a frame, so the
sphere's position is as fresh as it can be.

## Use

| Key   | Action                                               |
| ----- | ---------------------------------------------------- |
| `F1`  | Shows or hides the text at the bottom left           |
| `F11` | Switches between borderless fullscreen and maximized |

The text says the frame rate, the GPU, the display, and whether the window has focus. It's at
the bottom left, which the sphere never crosses.

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which opens the test on the primary display. `--display` and
`--no-activate` work as in every program.

## Settings

`Settings.json` has the settings every program has, with `"WindowMode": "BorderlessFullscreen"`
by default, and the SimplePaint material of the `Sphere`, the `Ground` and the `Cube`, in the
format of [Library/PaintSettings](../../Library/PaintSettings/README.md). Tune a material in
[SimplePaintDemo](../SimplePaintDemo/README.md), save it, and copy it from its `Paint.json`.

## Testing

```powershell
cargo test -p minimal_scene_test_tests
Program\RunInBackground\RunInBackground.ps1 MinimalSceneTest --all-scripts
```

The tests in `Test/MinimalSceneTest` check the sphere's motion against `Sphere.gd`'s, the keys,
and the settings file, without a window. The scripts in `Test/MinimalSceneTest/Scripts` run the
real test on a fixed clock, as [Library/Application](../../Library/Application/README.md#scripts)
describes, and check where the sphere is frame by frame, and what F1 and F11 do. The state that
scripts save and check has `SpherePosition`, `SphereDirection` and `TextShown`.

## Layout

| Path                 | Contents                                                                   |
| -------------------- | -------------------------------------------------------------------------- |
| `Source/Main.rs`     | The main loop: settings, window, time, input, and drawing each frame       |
| `Source/Library.rs`  | The test's parts, as a library that `Test/MinimalSceneTest` tests          |
| `Source/Controls.rs` | What each key does                                                         |
| `Source/Scene.rs`    | The camera, the objects, and the sphere's motion, drawn with SimplePaint   |
| `Source/Settings.rs` | The objects' materials, and their defaults from the Godot materials        |
| `Build.rs`           | Compiles SimplePaint's shaders for three materials                         |
| `HumanRun.ps1`       | Builds the test and runs it on the primary display, for people to use it   |
| `HumanRun.cmd`       | Runs `HumanRun.ps1`, for double-clicking in File Explorer                  |
| `Settings.json`      | The settings, which the test writes if it's missing                        |
