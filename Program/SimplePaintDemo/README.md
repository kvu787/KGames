# SimplePaintDemo

Edits a [SimplePaint](../../Library/SimplePaint/README.md) material, and shows it live on a UV
sphere. Like every KGames GUI program, it's built on
[Library/UserInterface](../../Library/UserInterface/README.md), after Fluent 2, in its light
theme, ignores input without focus, and supports the two
[window setups](../../Library/UserInterface/README.md#window-setups).

## What it shows

- **Preview**: a UV sphere painted with the material, in an orthographic view, with 4x
  multisampling. SimplePaint colors a surface by its normal alone, and any view of a sphere
  shows every normal that faces the camera once, so the sphere shows the whole material.
- **Settings**: a slider for each of SimplePaint's parameters, in rows like a list's, with
  its value and range, and the base color as a swatch, in hex and in linear, with the tone
  it appears at. **Reset** and **Save** are in the card's header, with what the last save did,
  whose whole message shows as a tooltip when it's cut short.
- **Tone curve**: each channel's displayed value over the tone, the tones from Dark Point to
  Light Point that the sphere shows, the tone where the base color appears, and the colors
  along the curve.
- **GPU constants**: the 80 bytes of `GpuMaterial` that `Material::compile` makes, which the
  shader reads.

## Editing

| Input                  | Action                                                            |
| ---------------------- | ----------------------------------------------------------------- |
| Drag a slider          | Sets the setting, with its value over the thumb, and selects it   |
| Scroll over one        | Moves it by its smallest step for each notch, and selects it      |
| Press a setting's name | Selects it                                                        |
| `Up`, `Down`           | Selects a setting                                                 |
| `Left`, `Right`        | Moves the selected setting by its smallest step, or 10 with Shift |
| `Ctrl+S`, **Save**     | Saves the material to `Paint.json` in this folder                 |
| **Reset**              | Goes back to SimplePaint's default material                       |
| `F10`                  | Switches vertical sync on or off                                  |
| `F11`                  | Switches between maximized and borderless fullscreen              |

Settings step by 0.001, and Rotation by 0.1 degrees, and stop at the last step inside the range
that SimplePaint accepts, so the material is always valid. Parts of a notch, like a touchpad's,
add up over the same slider. Holding an arrow key repeats it, but holding `Ctrl+S` saves once.

When it starts, the demo loads `Paint.json`, if it's there, and a notice under the app bar says
what's wrong with it if it can't. It's in the format of
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
the settings that every program has; the demo has none of its own.

## Testing

```powershell
cargo test -p simple_paint_demo_tests
Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo --all-scripts
```

The tests in `Test/SimplePaintDemo` drive the demo through the cards' real layout, which
`user_interface::Headless` builds without a window or a GPU: presses, drags past a rail's ends,
the wheel and its parts of a notch, the arrow keys, Ctrl+S, **Save**, **Reset**, losing focus
mid-drag, a bad `Paint.json`, and thousands of random inputs that must leave a valid material.
They also check that the cards and their text fit both window heights, and draw frames offscreen
with the GPU, where the sphere must fill the preview's view.

The scripts in `Test/SimplePaintDemo/Scripts` run the same kinds of input through the real
window and GPU, as [Library/UserInterface](../../Library/UserInterface/README.md#testing)
describes, including F11 there and back. A scripted run starts from SimplePaint's default
material, and saves `Paint.json` in its output folder, not in the demo's. The state that scripts
save and check has the material's `Parameters`, the `Selected` setting, the one whose slider is
`Dragging`, whether it's `Saved`, and the shader's `Constants`. The interface's state has where
each slider, row and button is, as `Interface.Targets`, with each slider's value as its `Label`,
and what the last save did as `SaveStatus`, with its message as its `Label`. Sliders are named
after their settings, like `Light Point`, and their rows like `Setting/Light Point`. A slider's
rail runs from 9 pixels past its area's left edge to 9 before its right, so a press at its
middle sets half its rail, like 180 degrees for Rotation.

## How it draws

Each frame, Library/UserInterface's main loop reads the window's input once the GPU can take
another frame, the demo does what it says and builds its cards, and the frame is drawn: the
demo's `render` copies the sphere into the preview card's view, and then the interface draws
over the rest in one draw call. The sliders don't take keyboard focus, as the arrow keys move
the selected one, which stands out like a list's selected row, with the accent's pill sliding
to it. The library's slider handles dragging, inside the range that SimplePaint accepts, but the
demo turns the wheel over a slider itself, by whole steps of the setting, so that a value always
lands on the decimal that it shows, and a value from `Paint.json` between steps or past the
range never moves the other way. Parts of a notch add up over the same slider.

The demo draws the sphere into its own 4x multisampled texture, through an sRGB view, so the
shader's linear output is encoded once, and resolves it, which averages the samples in linear
light. The texture is as big as the view in borderless fullscreen, around the card's color, and
the view shows its middle, 71 pixels shorter, maximized. The demo keeps the result, and draws
the sphere again only when the material changes. Each frame, it copies the result into the
frame at the view, where the card leaves a hole. The copy keeps the bytes, which are sRGB, as
the frame's are. The sphere has 256 vertices around and 128 bands from pole to pole, so its edge
looks round and its normals interpolate smoothly.

## Layout

| Path                | Contents                                                                     |
| ------------------- | ---------------------------------------------------------------------------- |
| `Source/Main.rs`    | Runs the demo with Library/UserInterface's main loop                         |
| `Source/Library.rs` | The demo's parts, as a library that `Test/SimplePaintDemo` tests             |
| `Source/Demo.rs`    | The demo as a program: the editor, its cards, and the sphere's preview       |
| `Source/Editor.rs`  | The material, its settings, what the keys and the wheel do, and `Paint.json` |
| `Source/Cards.rs`   | Lays out the cards in fixed pixels, and draws them and their widgets         |
| `Source/Preview.rs` | Draws the sphere with SimplePaint, and copies it into the frame              |
| `Build.rs`          | Compiles SimplePaint's shaders for one material                              |
| `HumanRun.ps1`      | Builds the demo and runs it on the primary display, for people to use it     |
| `HumanRun.cmd`      | Runs `HumanRun.ps1`, for double-clicking in File Explorer                    |
| `Settings.json`     | The settings that every program has, which the demo writes if it's missing   |
| `Paint.json`        | The material, once it's saved; not in Git until you add it                   |
