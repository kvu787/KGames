# ShadowTesting

Tries ways of drawing a box's shadow, side by side, in the light theme or the dark one. The page
has a tile for each way, three to a row: a card with sliders for the way's settings, beside a stage
with the same blue box, 200 × 120 with square corners, in its middle. Every stage is the same size,
so the ways can be compared at a glance, and `F9` compares them in the other theme.

Each way draws its box and shadow with a shader of its own, in the program's `render`, before
[Library/user-interface](../../Library/user-interface/README.md) draws the cards over the rest of the
frame, and only inside its stage. Fluent's is the library's own, for reference, which the interface
draws. Like every KGames GUI program, it ignores input without focus, and supports the one window
setup in [Library/user-interface](../../Library/user-interface/README.md#window-setup).

## The ways

| Way      | What it draws                                                                         |
| -------- | ------------------------------------------------------------------------------------- |
| Linear   | Pure black all around the box, as far as its length. Its opacity goes in a straight   |
|          | line from the start opacity at the box's edge to the end opacity at the length, by    |
|          | each pixel's distance from the box, so it's round around square corners               |
| Curved   | Like Linear, but fading to nothing along a curve: the opacity times the fraction of   |
|          | the length that's left, to the power. A power of 1 is Linear fading to nothing        |
| Gaussian | A copy of the box, grown by the spread, moved by the offset, and blurred by a         |
|          | Gaussian whose deviation is half the blur, like CSS's `box-shadow`                    |
| Offset   | A copy of the box in black, moved by the offset, with hard edges                      |
| Long     | The box dragged along a direction as far as the length, as flat designs' long         |
|          | shadows are, from the start opacity at the box to the end opacity at the length       |
| Fluent   | Library/user-interface's own shadow: WinUI's ThemeShadow for a surface raised by Z,   |
|          | a directional Gaussian, and above 32 an ambient one too, stronger in the dark theme   |

| Way      | Setting       | Range            | Default |
| -------- | ------------- | ---------------- | ------- |
| Linear   | Length        | 0 to 128 px      | 32 px   |
|          | Start opacity | 0 to 1           | 0.5     |
|          | End opacity   | 0 to 1           | 0       |
| Curved   | Length        | 0 to 128 px      | 48 px   |
|          | Opacity       | 0 to 1           | 0.5     |
|          | Power         | 0.25 to 4        | 2       |
| Gaussian | Blur          | 0 to 48 px       | 24 px   |
|          | Offset across | -32 to 32 px     | 0 px    |
|          | Offset down   | -32 to 32 px     | 8 px    |
|          | Spread        | -16 to 16 px     | 0 px    |
|          | Opacity       | 0 to 1           | 0.4     |
| Offset   | Offset across | -48 to 48 px     | 12 px   |
|          | Offset down   | -48 to 48 px     | 12 px   |
|          | Opacity       | 0 to 1           | 0.3     |
| Long     | Length        | 0 to 128 px      | 64 px   |
|          | Angle         | 0° to 359°       | 45°     |
|          | Start opacity | 0 to 1           | 0.3     |
|          | End opacity   | 0 to 1           | 0       |
| Fluent   | Z             | 0 to 128         | 32      |

Each pixel's distance from the box is measured at the pixel's center, so the box's edges, on whole
pixels, leave the first pixel out from them half a pixel away, and Linear's and Curved's shadows
end exactly their length from the box. Long's edges run along its angle, so each of its pixels is
the mean of 16 samples in a 4 × 4 grid. Every shadow blends in sRGB, as the interface's colors do:
a pixel of the background is its color times one less the shadow's opacity there. The ranges are as
far as each way's shadow can go and still fade out on its stage.

## Input

| Input          | Action                                                                  |
| -------------- | ----------------------------------------------------------------------- |
| A press        | Sets a slider to where it is on its rail, and drags it                  |
| Drag a slider  | Moves it, past its ends too, until the button is let go of              |
| The wheel      | Moves a slider a step a notch                                           |
| `Ctrl+R`       | Sets every way's settings back to their defaults                        |
| `F9`           | Switches between the light and dark themes                              |
| `F10`          | Switches vertical sync on or off                                        |

The first press on the window while it doesn't have focus only gives it focus.

## Build and run

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which you can also double-click in File Explorer. It opens the program on
the primary display. `--display` and `--no-activate` work as in every program, and
[RunInBackground](../RunInBackground/README.md) runs it for coding agents. `Settings.json` has the
settings that every program has.

## Testing

```powershell
cargo test -p shadow-testing-tests
Program\RunInBackground\RunInBackground.ps1 shadow-testing --all-scripts
```

The tests in `Test/shadow-testing` press and drag every slider in the tiles' real layout, with
`user_interface::Headless`, and check that each press sets its setting on its own frame, that
`Ctrl+R` resets every way on its frame, that every widget fits the window, and that the tiles are
alike, with each way's sliders in its own tile. They check each way's formula, and draw frames
offscreen with the GPU to check that every pixel around each box, and in it, is what its way's
formula says, in both themes, that each way's strongest shadow fades out before the edge of its
stage, that Fluent's falls below its box and grows with Z, and that a press on a slider draws the
new shadow on its own frame.

The scripts in `Test/shadow-testing/Scripts` drag sliders in the real window and check the
settings, and capture the start, the dark theme, and a linear shadow that's darkest at its far end.
The state that scripts save and check has each way's settings, like `Linear.Length` and
`Gaussian.Blur`, and the interface's, as `Interface`.

## Adding a way

1. Write `Shaders/<Way>.hlsl`, which includes `Common.hlsli`, works out its shadow's opacity at a
   point, and has a `PSMain` that puts the box over it, and add it to `build.rs`.
2. Write `src/<way>.rs`, with its description, its settings, its settings as the shader takes them,
   the area that its shadow can reach, and its opacity at a point as the shader works it out.
3. Add it to `Kind` and `KINDS` in `src/approaches.rs`, and its shader to
   `Test/simple-paint/tests/model_shaders.rs`' list. The page has room for six tiles; a seventh
   needs another row or column in `src/cards.rs`.
4. Give it settings for checking pixels in `test_values` in `Test/shadow-testing`, and a test of its
   formula.

## Layout

| Path                    | Contents                                                                |
| ----------------------- | ----------------------------------------------------------------------- |
| `src/main.rs`           | Runs the program with Library/user-interface's main loop                |
| `src/lib.rs`            | The program's parts, as a library that `Test/shadow-testing` tests      |
| `src/shadow_testing.rs` | The ways' settings, what the input does, `render`, and the state        |
| `src/cards.rs`          | Lays out the tiles in fixed pixels, with each way's card of sliders     |
| `src/approaches.rs`     | The ways, their settings, and the distance from the box                 |
| `src/pipeline.rs`       | Draws a way's box and shadow with its shaders, inside its stage         |
| `src/linear.rs`         | The linear shadow: its settings, its area, and its formula              |
| `src/curved.rs`         | The curved shadow                                                       |
| `src/gaussian.rs`       | The Gaussian shadow, and the error function                             |
| `src/offset.rs`         | The offset shadow                                                       |
| `src/long.rs`           | The long shadow                                                         |
| `src/fluent.rs`         | Fluent's shadow, which the interface draws                              |
| `Shaders/Common.hlsli`  | What the ways' shaders share: the constants, the quad, the box          |
| `Shaders/<Way>.hlsl`    | Each way's shadow                                                       |
| `build.rs`              | Compiles the shaders at build time                                      |
| `HumanRun.ps1`          | Builds the program and runs it on the primary display, for people       |
| `HumanRun.cmd`          | Runs `HumanRun.ps1`, for double-clicking in File Explorer               |
| `Settings.json`         | The settings that every program has                                     |
