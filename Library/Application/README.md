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
  input from them as `Event`s, in a `Frame`: presses, releases and the wheel, keys, and the
  characters that the keys type, for text fields. The `Frame` also has the keys held down, for
  what goes on while a key is held, like steering, and the gamepad, if the program has the
  window read it with `read_gamepads`.
- `Gamepads` reads the gamepad with GameInput, for programs that a gamepad controls. Of the
  gamepads connected, the one used last counts: the first that GameInput has a reading of,
  and then any other whose button goes down, or whose stick or trigger moves past halfway, so
  that a second gamepad that lies still doesn't take over. A frame's `Gamepad` has where the
  sticks and triggers are, without a dead zone, the buttons held, and the ones that went down
  since the frame before.
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
`Window::begin_frame` returns has no cursor then, no keys held, and no events but a `Release`
that ends a press it got with focus, so that each `Press` has its `Release`. A click in the
window that gives it focus does only that, and a key that's still down when it gets focus only
counts as held once it's pressed again.

GameInput gives gamepad input to windows without focus, so `Gamepads` drops it: without focus,
the gamepad is at rest, and a button that's still down when the window gets focus, or when the
program starts, only counts once it's pressed again. Its sticks and triggers count right away.
Programs that read GameInput themselves, like GameInputDemo, drop its readings themselves while
the window doesn't have focus.

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
        { "Frame": 5, "Text": "Hello" },
        { "Frame": 5, "Hold": ["Up", "Space"] },
        { "Frame": 5, "Gamepad": { "RightStick": [0, 1], "Buttons": ["A"] } },
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
| `Key`     | Presses a key by name, like `F11`, `Left`, `Home` or `A`, with `Control` and   |
|           | `Shift`. `Repeat` makes it a repeat of a key that's held, which toggles ignore |
| `Hold`    | Holds down the keys it lists until the next `Hold`. Each that wasn't held      |
|           | already is pressed then, and the keys it leaves out let go                     |
| `Gamepad` | Puts the gamepad where it says until the next `Gamepad`: `LeftStick` and       |
|           | `RightStick` as `[x, y]`, `LeftTrigger`, `RightTrigger`, and the `Buttons`     |
|           | held, like `A`, `LB` or `Up`. What it leaves out is at rest, and `null`        |
|           | disconnects the gamepad, which is disconnected until the first `Gamepad`       |
| `Text`    | Types characters, as `Character` events, without the keys that type them       |
| `Capture` | Saves the frame as `<name>.png` and the program's state as `<name>.json`, in   |
|           | the output folder. A frame has one at most, and each has its own name          |
| `Expect`  | Checks the program's state: the members it lists, and only those. Numbers are  |
|           | equal within 1e-9, relative to the expected number if it's above 1, and an     |
|           | infinite or NaN number never is                                                |
| `Program` | Goes to the program as it is, for input that doesn't come from the window      |

Frames count from 0, and clicks hit what the frame before them drew, so they start at frame 1.
After the last step, the program exits, with an error that lists each expectation that wasn't
met. If it exits before the last step, like when its window is closed, that's an error too.
Programs give their state to `Window::handle_checkpoints`, which adds the window's own as
`Window`: `Fullscreen`, `Problems` and `Frame`. On the frame that toggles fullscreen, they're
the new setup's already, but a capture shows that frame as drawn for the old one.
`Source/Script.rs` reads scripts, and `Test/Application` tests it.

| Path                      | Contents                                                               |
| ------------------------- | ---------------------------------------------------------------------- |
| `Source/Process.rs`       | DPI awareness, and opting out of power throttling                      |
| `Source/Settings.rs`      | Reads the settings file, and writes it with the defaults if missing    |
| `Source/Arguments.rs`     | Reads the command line                                                 |
| `Source/Window.rs`        | The window, its messages, input events, and borderless fullscreen      |
| `Source/Gamepad.rs`       | Reads the gamepad with GameInput, and drops its input without focus    |
| `Source/Script.rs`        | Reads scripts of input, and compares a program's state with a script's |
| `Source/WindowCheck.rs`   | Checks that the window is in one of the two supported setups           |
| `Source/FrameCounter.rs`  | Counts frames per second                                               |
