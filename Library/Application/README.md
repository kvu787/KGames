# Application

What every KGames GUI program shares besides drawing:

- `prepare_process` makes the process aware of each monitor's DPI, and opts it out of power
  throttling.
- `Settings::load` reads the program's `Settings.json`, with the settings every program has
  (`Display`, `WindowMode` and `AllowTearing`), and any of its own, through `ProgramSettings`.
  It writes the file with the defaults if it's missing, and says what's wrong with it in
  notices, which programs show.
- `Arguments::parse` reads `--display <identifier>` and `--no-activate`.
- `Window` opens the program's window on its display, maximized or in borderless fullscreen,
  and switches between the two with `toggle_fullscreen`, for F11. It records the window's
  messages, and gives the mouse and keyboard input from them to the program as `Event`s.
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

Programs ignore all input while their window doesn't have focus. `Window::take_events`
returns nothing then, and `Window::cursor` returns `None`. A click in the window that gives it
focus does only that. Programs that read input with GameInput drop it themselves while the
window doesn't have focus.

| Path                      | Contents                                                               |
| ------------------------- | ---------------------------------------------------------------------- |
| `Source/Process.rs`       | DPI awareness, and opting out of power throttling                      |
| `Source/Settings.rs`      | Reads the settings file, and writes it with the defaults if missing    |
| `Source/Arguments.rs`     | Reads the command line                                                 |
| `Source/Window.rs`        | The window, its messages, input events, and borderless fullscreen      |
| `Source/WindowCheck.rs`   | Checks that the window is in one of the two supported setups           |
| `Source/FrameCounter.rs`  | Counts frames per second                                               |
