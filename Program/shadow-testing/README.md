# ShadowTesting

Tries ways of drawing a shadow, on a blue box with square corners in the middle of the page, with
sliders for each way's settings, in the light theme or the dark one. Each way draws the box and its
shadow with a shader of its own, in the program's `render`, before
[Library/user-interface](../../Library/user-interface/README.md) draws the cards over the rest of the
frame. Like every KGames GUI program, it ignores input without focus, and supports the one window
setup in [Library/user-interface](../../Library/user-interface/README.md#window-setup).

## The ways

| Way    | What it draws                                                                         |
| ------ | ------------------------------------------------------------------------------------- |
| Linear | Pure black all around the box, as far as its length from the box's edge. Its opacity  |
|        | goes in a straight line from the start opacity at the edge to the end opacity at the  |
|        | length, by each pixel's distance from the box, so it's round around square corners    |

Each pixel's distance from the box is measured at the pixel's center, so the box's edges, on whole
pixels, leave the first pixel out from them half a pixel away, and the shadow ends exactly its
length from the box. The shadow blends in sRGB, as the interface's colors do: a pixel of the
background is its color times one less the shadow's opacity.

| Setting       | Range        | Default |
| ------------- | ------------ | ------- |
| Length        | 0 to 256 px  | 32 px   |
| Start opacity | 0 to 1       | 0.5     |
| End opacity   | 0 to 1       | 0       |

## Input

| Input          | Action                                                                  |
| -------------- | ----------------------------------------------------------------------- |
| A press        | Sets a slider to where it is on its rail, and drags it                  |
| Drag a slider  | Moves it, past its ends too, until the button is let go of              |
| The wheel      | Moves a slider a step a notch                                           |
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

The tests in `Test/shadow-testing` press and drag the sliders in the card's real layout, with
`user_interface::Headless`, and check that each press sets its setting on its own frame, that every
widget fits the window, and that the box and its longest shadow fit on the stage, clear of the
cards. They check the linear shadow's formula, and draw frames offscreen with the GPU to check that
every pixel around the box, and in it, is what the formula says, in both themes, and that a press
on a slider draws the new shadow on its own frame.

The scripts in `Test/shadow-testing/Scripts` drag the sliders in the real window and check the
settings, and capture the start, the dark theme, and a shadow that's darkest at its far end. The
state that scripts save and check has each way's settings, like `Linear.Length`, and the
interface's, as `Interface`.

## Layout

| Path                    | Contents                                                                |
| ----------------------- | ----------------------------------------------------------------------- |
| `src/main.rs`           | Runs the program with Library/user-interface's main loop                |
| `src/lib.rs`            | The program's parts, as a library that `Test/shadow-testing` tests      |
| `src/shadow_testing.rs` | The settings, what the sliders do, `render`, and the state for scripts  |
| `src/cards.rs`          | Lays out the cards and the stage in fixed pixels                        |
| `src/linear.rs`         | The linear shadow: its settings, its formula, and its pipeline          |
| `Shaders/Linear.hlsl`   | Draws the box and its linear shadow                                     |
| `build.rs`              | Compiles the shaders at build time                                      |
| `HumanRun.ps1`          | Builds the program and runs it on the primary display, for people       |
| `HumanRun.cmd`          | Runs `HumanRun.ps1`, for double-clicking in File Explorer               |
| `Settings.json`         | The settings that every program has                                     |
