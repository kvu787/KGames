# SpinningCarTest

A port of Veehiicuul's `3dTestScene_CppDx12`, a C++ Direct3D 12 scene, onto KGames' own stack:
a car moves back and forth at 8 units per second while it turns at 90 degrees per second, past
a sphere, over a baked image of the backdrop, the green ground and the red cube, seen by an
orthographic camera. The car and the sphere are painted with
[SimplePaint](../../Library/SimplePaint/README.md), whose canonical home moved from that scene
to KGames. Like every KGames GUI program, it's built from the [Library](../../Library) crates on
Win32 and Direct3D 12, ignores input without focus, and supports the two window setups in
[Library/Application](../../Library/Application/README.md#supported-window-setups).

## From the C++ scene

| C++ scene                                          | Here                                                   |
| -------------------------------------------------- | ------------------------------------------------------ |
| Camera, car and sphere placement, and timing       | The same, in `Source/Scene.rs`                         |
| `Source/Generated/CarMesh.generated.h`             | `Source/CarMesh.rs`, converted with every number as    |
|                                                    | written                                                |
| `Assets/SceneBackground.png`, drawn by             | The same image and shader, with the root signature     |
| `Source/Shaders/Background.hlsl`                   | embedded in the shader                                 |
| The paint and sphere settings in `Settings.json`   | The same names and values, in this folder's            |
|                                                    | `Settings.json`                                        |
| `RenderPipeline` settings, and `V` for VSync       | Gone: frames go through Library/Graphics' renderer,    |
|                                                    | paced for the lowest latency, without waiting for      |
|                                                    | vertical sync                                          |
| A 1280 x 720 window                                | Maximized, or borderless fullscreen with `F11`         |
| `S` for a stationary car                           | The same                                               |
| `Esc` to quit                                      | Gone, as in the other programs                         |

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

## Layout

| Path                         | Contents                                                               |
| ---------------------------- | ---------------------------------------------------------------------- |
| `Source/Main.rs`             | The main loop: settings, window, input, and drawing each frame         |
| `Source/Scene.rs`            | The background, the car and the sphere, the camera and the animation   |
| `Source/CarMesh.rs`          | The car mesh, converted from the C++ scene's generated header          |
| `Source/Settings.rs`         | The materials and the sphere's resolution, with the C++ defaults       |
| `Shaders/Background.hlsl`    | Draws the background image, center-cropped, with its root signature    |
| `Assets/SceneBackground.png` | The baked background, from the C++ scene                               |
| `Build.rs`                   | Compiles SimplePaint's shaders for six materials, and the background's |
| `HumanRun.ps1`               | Builds the test and runs it on the primary display, for people to use  |
| `HumanRun.cmd`               | Runs `HumanRun.ps1`, for double-clicking in File Explorer              |
| `Settings.json`              | The settings, which the test writes if it's missing                    |
