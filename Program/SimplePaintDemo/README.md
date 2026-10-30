# SimplePaintDemo

Edits a [SimplePaint](../../Library/SimplePaint/README.md) material, and shows it live on a UV
sphere. Like every KGames GUI program, it's built from the [Library](../../Library) crates on
Win32 and Direct3D 12, in a light theme, ignores input without focus, and supports the two
window setups in [Library/Application](../../Library/Application/README.md#supported-window-setups).

## What it shows

- **Preview**: a UV sphere painted with the material, in an orthographic view, with 4x
  multisampling. SimplePaint colors a surface by its normal alone, and any view of a sphere
  shows every normal that faces the camera once, so the sphere shows the whole material.
- **Settings**: a slider for each of SimplePaint's parameters, the base color as a swatch, in
  hex and in linear, and the tone it appears at.
- **Tone curve**: each channel's displayed value over the tone, the tones from Dark Point to
  Light Point that the sphere shows, the tone where the base color appears, and the colors
  along the curve.
- **GPU constants**: the 80 bytes of `GpuMaterial` that `Material::compile` makes, which the
  shader reads.

## Editing

| Input              | Action                                                                  |
| ------------------ | ----------------------------------------------------------------------- |
| Drag a slider      | Sets the setting                                                        |
| Scroll over one    | Moves it by its smallest step                                           |
| `Up`, `Down`       | Selects a setting                                                       |
| `Left`, `Right`    | Moves the selected setting by its smallest step, or 10 with `Shift`     |
| `Ctrl+S`, **Save** | Saves the material to `Paint.json` in this folder                       |
| **Reset**          | Goes back to SimplePaint's default material                             |
| `F11`              | Switches between maximized and borderless fullscreen                    |

Settings step by 0.001, and Rotation by 0.1 degrees, and stop at the last step inside the range
that SimplePaint accepts, so the material is always valid. The first click on the window while
it doesn't have focus only gives it focus.

When it starts, the demo loads `Paint.json`, if it's there. It's in the format of
[Library/PaintSettings](../../Library/PaintSettings/README.md), so a material can be copied
from it into another program's settings, like MinimalSceneTest's. The demo only writes it when
you save.

## Build and run

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which you can also double-click in File Explorer. It opens the demo on
the primary display. `--display` and `--no-activate` work as in every program, and
[RunInBackground](../RunInBackground/README.md) runs it for coding agents. `Settings.json` has
the settings that every program has.

## Testing

```powershell
cargo test -p simple_paint_demo_tests
Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo --all-scripts
```

The tests in `Test/SimplePaintDemo` drive the editor through the panels' real layout, which a
canvas builds without a window or a GPU: clicks, drags past a track's ends, the wheel, the arrow
keys, Ctrl+S, **Save**, **Reset**, losing focus mid-drag, a bad `Paint.json`, and thousands of
random inputs that must leave a valid material. They also check that the panels fit both
window heights.

The scripts in `Test/SimplePaintDemo/Scripts` run the same kinds of input through the real
window and renderer, as [Library/Application](../../Library/Application/README.md#scripts)
describes, including F11 there and back. A scripted run starts from SimplePaint's default
material, and saves `Paint.json` in its output folder, not in the demo's. The state that scripts
save and check has the material's `Parameters`, the `Selected` slider, whether one is
`Dragging`, whether it's `Saved`, the shader's `Constants`, and where each slider's track and
each button are, as `Targets`.

## How it draws

Each frame, after the renderer is ready, the demo reads the window's input, builds the
panels on a canvas, and draws them in one draw call. It then draws the sphere into its own
4x multisampled texture, through an sRGB view, so the shader's linear output is encoded once,
and resolves it, which averages the samples in linear light. It copies the result into the
back buffer over the preview panel. The copy keeps the bytes, which are sRGB, as the back
buffer's are. The sphere has 256 vertices around and 128 bands from pole to pole, so its edge
looks round and its normals interpolate smoothly.

## Layout

| Path                | Contents                                                                    |
| ------------------- | --------------------------------------------------------------------------- |
| `Source/Main.rs`    | The main loop: settings, window, input, and drawing each frame              |
| `Source/Library.rs` | The demo's parts, as a library that `Test/SimplePaintDemo` tests            |
| `Source/Editor.rs`  | The material, its sliders, what input does to them, and `Paint.json`        |
| `Source/Panels.rs`  | Lays out the panels in fixed pixels, and draws them                         |
| `Source/Preview.rs` | Draws the sphere with SimplePaint, and copies it into the frame             |
| `Build.rs`          | Compiles SimplePaint's shaders for one material                             |
| `HumanRun.ps1`      | Builds the demo and runs it on the primary display, for people to use it    |
| `HumanRun.cmd`      | Runs `HumanRun.ps1`, for double-clicking in File Explorer                   |
| `Settings.json`     | The settings that every program has, which the demo writes if it's missing  |
| `Paint.json`        | The material, once it's saved; not in Git until you add it                  |
