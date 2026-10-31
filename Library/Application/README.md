# Application

What every KGames GUI program shares besides drawing:

- `prepare_process` makes the process aware of each monitor's DPI, and opts it out of power
  throttling.
- `Settings::load` reads the program's `Settings.json`, with the settings every program has
  (`Display`, `WindowMode` and `AllowTearing`), and any of its own, through `ProgramSettings`.
  It writes the file with the defaults if it's missing, and says what's wrong with it in
  notices, which programs show. `Settings::for_program` gives a scripted run the defaults
  instead, so that what the file says doesn't change the run.
- `Arguments::parse` reads `--display <identifier>`, `--no-activate`, and `--script <path>`
  with `--output <folder>`. `Arguments::file` puts a file that the program writes, like
  SimplePaintDemo's `Paint.json`, in the output folder in a scripted run.
- `Window` opens the program's window on its display, maximized or in borderless fullscreen,
  and switches between the two with `toggle_fullscreen`, for F11. It records the window's
  messages, and `begin_frame` gives the program each frame's time and the mouse and keyboard
  input from them as `Event`s, in a `Frame`.
- `window_check` says what's wrong with the window's setup, if it isn't one of the two that
  programs support.
- `FrameCounter` counts frames per second.

## Supported window setups

Programs support exactly two window setups, both on a 2560 × 1440 monitor at 100% scaling
(96 DPI), so that they can lay out in fixed pixels:

| Setup                 | Client area | What makes it                                                              |
| --------------------- | ----------- | -------------------------------------------------------------------------- |
| Maximized             | 2560 × 1369 | A 48-pixel taskbar at the bottom leaves a 2560 × 1392 work area, and the   |
|                       |             | title bar takes its top 23 pixels                                          |
| Borderless fullscreen | 2560 × 1440 | A window without a frame that covers the whole monitor                     |

In any other setup, like a restored window, another monitor, another scale, or a moved
taskbar, `Window::problems` says what's wrong, and programs show that instead of their panels.
It's checked again whenever the window moves or changes size, or the displays, the scale or the
taskbar change.

## Input without focus

Programs ignore all input while their window doesn't have focus. The `Frame` that
`Window::begin_frame` returns has no events then, and no cursor. A click in the window that
gives it focus does only that. Programs that read input with GameInput drop it themselves
while the window doesn't have focus.

## Scripts

With `--script <path> --output <folder>`, which
[RunInBackground](../../Program/RunInBackground/README.md) passes, the program's input comes
from a script instead of the user, so that coding agents can test it without the user's mouse
and keyboard, or focus. Input from the window's messages is dropped, and the window acts as if
it had focus. Time is a fixed clock, `FrameSeconds` a frame, 1/60 of a second by default, so
every run draws the same frames. Each step happens at a frame:

```json
{
    "FrameSeconds": 0.01,
    "Steps": [
        { "Frame": 1, "Move": [1400, 300] },
        { "Frame": 2, "Press": [1400, 300] },
        { "Frame": 3, "Release": [1800, 300] },
        { "Frame": 4, "Wheel": [1400, 300], "Notches": -2 },
        { "Frame": 5, "Key": "S", "Control": true },
        { "Frame": 6, "Capture": "AfterSave" },
        { "Frame": 6, "Expect": { "Window": { "Fullscreen": false } } },
        { "Frame": 7, "Program": { "Anything": "the program reads" } }
    ]
}
```

| Step      | What it does                                                                   |
| --------- | ------------------------------------------------------------------------------ |
| `Move`    | Moves the cursor to a point in the client area, in pixels                      |
| `Press`   | Moves there, and presses the left button                                       |
| `Release` | Moves there, and lets go of it                                                 |
| `Wheel`   | Turns the wheel there by `Notches`, positive away from the user                |
| `Key`     | Presses a key by name, like `F11`, `Left` or `A`, with `Control` and `Shift`.  |
|           | `Repeat` makes it a repeat of a key that's held, which toggles ignore          |
| `Capture` | Saves the frame as `<name>.png` and the program's state as `<name>.json`, in   |
|           | the output folder                                                              |
| `Expect`  | Checks the program's state: the members it lists, and only those. Numbers are  |
|           | equal within a relative 1e-9, and an infinite or NaN number never is           |
| `Program` | Goes to the program as it is, for input that doesn't come from the window      |

Frames count from 0, and clicks hit what the frame before them drew, so they start at frame 1.
After the last step, the program exits, with an error that lists each expectation that wasn't
met. Programs give their state to `Window::handle_checkpoints`, which adds the window's own as
`Window`: `Fullscreen`, `Problems` and `Frame`. `Source/Script.rs` reads scripts, and
`Test/Application` tests it.

| Path                      | Contents                                                               |
| ------------------------- | ---------------------------------------------------------------------- |
| `Source/Process.rs`       | DPI awareness, and opting out of power throttling                      |
| `Source/Settings.rs`      | Reads the settings file, and writes it with the defaults if missing    |
| `Source/Arguments.rs`     | Reads the command line                                                 |
| `Source/Window.rs`        | The window, its messages, input events, and borderless fullscreen      |
| `Source/Script.rs`        | Reads scripts of input, and compares a program's state with a script's |
| `Source/WindowCheck.rs`   | Checks that the window is in one of the two supported setups           |
| `Source/FrameCounter.rs`  | Counts frames per second                                               |
