# ShadowTesting

Tries ways of drawing a box's omnidirectional shadow, side by side, in the light theme or the dark
one: shadows that spread from the box the same way on every side, as if lit from straight above,
without an offset or a direction, like Material's and Fluent's ambient shadows. The page has a tile
for each way, three to a row: a card with sliders for the way's settings and a Reset button, beside
a stage with the same blue box, 200 × 120 with square corners, in its middle. Every stage is the
same size, and every way starts at defaults matched to the others', so the ways can be compared at a
glance, and `F9` compares them in the other theme.

Each way draws its box and shadow with a shader of its own, in the program's `render`, before
[Library/user-interface](../../Library/user-interface/README.md) draws the cards over the rest of the
frame, and only inside its stage. Like every KGames GUI program, it ignores input without focus, and
supports the one window setup in
[Library/user-interface](../../Library/user-interface/README.md#window-setup).

## The ways

| Way         | What it draws                                                                      |
| ----------- | ---------------------------------------------------------------------------------- |
| Linear      | Pure black all around the box, as far as its length. Its opacity goes in a         |
|             | straight line from the start opacity at the box's edge to the end opacity at the   |
|             | length, by each pixel's distance from the box, so it's round around square corners |
| Curved      | Like Linear, but fading to nothing along a curve: the opacity times the fraction   |
|             | of the length that's left, to the power. A power of 1 is Linear fading to nothing  |
| Smoothstep  | Like Linear, but fading to nothing along an S curve, smoothstep: soft both where   |
|             | it leaves the box and where it ends                                                |
| Exponential | Falls by a factor of e every decay length out from the box: steeply at first,      |
|             | with a long faint tail                                                             |
| Bell        | Falls along a bell curve, a Gaussian of the distance from the box, whose standard  |
|             | deviation is the width: flat where it leaves the box, then faster and faster       |
| Gaussian    | A copy of the box, grown by the spread on every side, and blurred by a Gaussian    |
|             | whose deviation is half the blur, like CSS's `box-shadow` with no offset           |

| Way         | Setting       | Range        | Default |
| ----------- | ------------- | ------------ | ------- |
| Linear      | Length        | 0 to 128 px  | 20 px   |
|             | Start opacity | 0 to 1       | 0.25    |
|             | End opacity   | 0 to 1       | 0       |
| Curved      | Length        | 0 to 128 px  | 32 px   |
|             | Opacity       | 0 to 1       | 0.25    |
|             | Power         | 0.25 to 4    | 2       |
| Smoothstep  | Length        | 0 to 128 px  | 20 px   |
|             | Opacity       | 0 to 1       | 0.25    |
| Exponential | Decay length  | 1 to 20 px   | 14 px   |
|             | Opacity       | 0 to 1       | 0.25    |
| Bell        | Width         | 0 to 36 px   | 8 px    |
|             | Opacity       | 0 to 1       | 0.25    |
| Gaussian    | Blur          | 0 to 64 px   | 28 px   |
|             | Spread        | -16 to 16 px | 0 px    |
|             | Opacity       | 0 to 1       | 0.5     |

The defaults are matched, so that the ways differ only in the shape of their falloff: out from the
middle of a side, each starts at about 0.25 at the box's edge, a subtle shadow on the light theme's
background, and is half that about 10 px out. Gaussian's opacity is 0.5 for that, since a blurred
edge is half as dark as the rest of the blur. A test holds every way to the match.

All but Gaussian's shadow go by each pixel's distance from the box alone, so they're as dark
diagonally out from a corner as straight out from a side. Gaussian's is the same on all four sides,
and softer around the corners. Exponential's and Bell's tails never end, so they stop where they're
fainter than half an 8-bit step, which changes no pixel. Each pixel's distance from the box is
measured at the pixel's center, so the box's edges, on whole pixels, leave the first pixel out from
them half a pixel away, and the shadows with a length end exactly that far from the box. Every
shadow blends in sRGB, as the interface's colors do: a pixel of the background is its color times
one less the shadow's opacity there. The ranges are as far as each way's shadow can go and still
fade out on its stage.

## Input

| Input          | Action                                                                  |
| -------------- | ----------------------------------------------------------------------- |
| A press        | Sets a slider to where it is on its rail, and drags it                  |
| Drag a slider  | Moves it, past its ends too, until the button is let go of              |
| The wheel      | Moves a slider a step a notch                                           |
| A way's Reset  | Sets that way's settings back to their defaults; off while they are     |
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
`user_interface::Headless`, and check that each press sets its setting on its own frame, that each
way's Reset button resets that way alone on its frame, and is off at the defaults, that `Ctrl+R`
resets every way on its frame, that every widget fits the window, and that the tiles are alike,
with each way's sliders in its own tile. They check each way's formula, that each way's shadow is
the same on every side, and that every way's defaults match, and draw frames offscreen with the GPU
to check that every pixel around each box, and in it, is what its way's formula says, and the same
as the pixel mirrored across the box, in both themes, that each way's strongest shadow fades out
before the edge of its stage, and that a press on a slider draws the new shadow on its own frame.

The scripts in `Test/shadow-testing/Scripts` drag and scroll sliders and press a Reset button in
the real window and check the settings, and capture the start, the dark theme, and a linear shadow
that's darkest at its far end. The state that scripts save and check has each way's settings, like
`Linear.Length` and `Gaussian.Blur`, and the interface's, as `Interface`.

## Adding a way

1. Write `Shaders/<Way>.hlsl`, which includes `Common.hlsli`, works out its shadow's opacity at a
   point, the same on every side of the box, and has a `PSMain` that puts the box over it, and add
   it to `build.rs`.
2. Write `src/<way>.rs`, with its description, its settings, its settings as the shader takes them,
   the area that its shadow can reach, and its opacity at a point as the shader works it out. Give
   it defaults that match the others', which a test in `Test/shadow-testing` checks.
3. Add it to `Kind` and `KINDS` in `src/approaches.rs`, and its shader to
   `Test/simple-paint/tests/model_shaders.rs`' list. The page lays out three tiles to a row, and as
   many rows as the ways need.
4. Give it settings for checking pixels in `test_values` in `Test/shadow-testing`, and a test of its
   formula.

## Layout

| Path                    | Contents                                                                |
| ----------------------- | ----------------------------------------------------------------------- |
| `src/main.rs`           | Runs the program with Library/user-interface's main loop                |
| `src/lib.rs`            | The program's parts, as a library that `Test/shadow-testing` tests      |
| `src/shadow_testing.rs` | The ways' settings, what the input does, `render`, and the state        |
| `src/cards.rs`          | Lays out the tiles in fixed pixels, with each way's card                |
| `src/approaches.rs`     | The ways, their settings, the defaults' match, and the distance         |
| `src/pipeline.rs`       | Draws a way's box and shadow with its shaders, inside its stage         |
| `src/linear.rs`         | The linear shadow: its settings, its area, and its formula              |
| `src/curved.rs`         | The curved shadow                                                       |
| `src/smoothstep.rs`     | The smoothstep shadow                                                   |
| `src/exponential.rs`    | The exponential shadow, and how far its tail shows                      |
| `src/bell.rs`           | The bell shadow, and how far its tail shows                             |
| `src/gaussian.rs`       | The Gaussian shadow, and the error function                             |
| `Shaders/Common.hlsli`  | What the ways' shaders share: the constants, the quad, the box          |
| `Shaders/<Way>.hlsl`    | Each way's shadow                                                       |
| `build.rs`              | Compiles the shaders at build time                                      |
| `HumanRun.ps1`          | Builds the program and runs it on the primary display, for people       |
| `HumanRun.cmd`          | Runs `HumanRun.ps1`, for double-clicking in File Explorer               |
| `Settings.json`         | The settings that every program has                                     |
