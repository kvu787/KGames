# MinimalSceneTest

A scene for checking how smoothly frames reach the screen: a sphere moves back and forth at 8
units per second, between the top left and bottom right corners of the screen, above a ground
plane and past a cube, seen by an orthographic camera. Like every KGames GUI program, it's built
on [Library/UserInterface](../../Library/UserInterface/README.md), with Win32 and Direct3D 12,
ignores input without focus, and supports the two
[window setups](../../Library/UserInterface/README.md#window-setups).

## Scene

| Part       | What it is                                                                  |
| ---------- | --------------------------------------------------------------------------- |
| Camera     | Orthographic, 5 units tall, from 1 to 20 units in front of it. It looks at  |
|            | the origin from 5 units away, 47.5 degrees above the horizon and 30 degrees |
|            | from +Z toward +X                                                           |
| Sphere     | Radius 0.5, on the ground. X moves by 8 units per second times the time     |
|            | since the last frame, and bounces back from 7 by as far as it went past. Of |
|            | a frame's time, a tenth of a second at most counts, so it doesn't jump      |
|            | ahead after the window was minimized, or showed what's wrong with its setup |
| Ground     | A plane 10 units along X and 5 along Z                                      |
| Cube       | 1 unit on each side, on the ground at Z = -1.5                              |
| Materials  | [SimplePaint](../../Library/SimplePaint/README.md) materials: the ground's  |
|            | and the cube's paint their color on faces that face up, and the sphere's    |
|            | has equal Dark and Light Points, a constant color                           |
| Background | #4c4c4c                                                                     |
| Frames     | Borderless fullscreen by default, presented without waiting for vertical    |
|            | sync, tearing allowed, and without antialiasing                             |

The scene fills the window at either height, with its own depth buffer, and the card goes over
it. Library/UserInterface's main loop reads the input and the time right before it draws a
frame, and presents as soon as the GPU and DXGI can take one, so the sphere's position is as
fresh as it can be.

## Use

| Key   | Action                                               |
| ----- | ---------------------------------------------------- |
| `F1`  | Shows or hides the card at the bottom left           |
| `F11` | Switches between borderless fullscreen and maximized |

The card floats over the scene at the bottom left, which the sphere never crosses, and is as tall
as what it says: the test's name and what the scene shows, the frame rate, the GPU, the display,
whether the window has focus, with an info bar's status icon, the keys on keycaps, and what's
wrong with `Settings.json`, if anything, as warning info bars.

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which opens the test on the primary display. `--display` and
`--no-activate` work as in every program.

## Settings

`Settings.json` has the settings every program has, with `"WindowMode": "BorderlessFullscreen"`
by default, and the SimplePaint material of the `Sphere`, the `Ground` and the `Cube`, in the
format of [Library/PaintSettings](../../Library/PaintSettings/README.md). A material that's
wrong, like one that SimplePaint rejects, keeps its default, and the card says what's wrong
with it. Tune a material in [SimplePaintDemo](../SimplePaintDemo/README.md), save it, and copy
it from its `Paint.json`.

## Testing

```powershell
cargo test -p minimal_scene_test_tests
Program\RunInBackground\RunInBackground.ps1 MinimalSceneTest --all-scripts
```

The tests in `Test/MinimalSceneTest` check, without a window, the sphere's motion, what F1, F11
and the window's focus do, that the card stays out of the sphere's path at both heights, the
settings file, and frames drawn offscreen with the GPU. The scripts in
`Test/MinimalSceneTest/Scripts` run the real test on a fixed clock, as
[Library/UserInterface](../../Library/UserInterface/README.md#testing) describes, and check where
the sphere is frame by frame, that a long frame counts a tenth of a second, and what F1, F11 and
the window's focus do. The state that scripts save and check has `SpherePosition`,
`SphereDirection` and `TextShown`, and the interface's, whose targets have the card as
`InfoCard`, of kind `Card`, and what's wrong with `Settings.json` as `Notice/0`, `Notice/1` and
so on.

## Layout

| Path                     | Contents                                                                 |
| ------------------------ | ------------------------------------------------------------------------ |
| `Source/Main.rs`         | Runs the test with Library/UserInterface                                 |
| `Source/Library.rs`      | The test's parts, as a library that `Test/MinimalSceneTest` tests        |
| `Source/MovingSphere.rs` | The test as a `Program`: the sphere's motion, the keys, and the state    |
| `Source/Scene.rs`        | The camera, the objects, and the sphere's motion, drawn with SimplePaint |
| `Source/Settings.rs`     | The objects' materials, and their defaults                               |
| `Build.rs`               | Compiles SimplePaint's shaders for three materials                       |
| `HumanRun.ps1`           | Builds the test and runs it on the primary display, for people to use it |
| `HumanRun.cmd`           | Runs `HumanRun.ps1`, for double-clicking in File Explorer                |
| `Settings.json`          | The settings, which the test writes if it's missing                      |
