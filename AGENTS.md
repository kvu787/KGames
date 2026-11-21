# Key info sources

- MCP servers that should be connected:
  - https://learn.microsoft.com/api/mcp
  - https://docs.xbox.com/mcp
- Links:
  - https://github.com/Microsoft/DirectX-Specs
  - https://github.com/microsoft/DirectX-Graphics-Samples

# Where to find certain things on this machine

- Blender: C:\Users\K\Program\blender-5.2.3-windows-x64
- PresentMon: C:\Users\K\Program\PresentMon-2.6.0-x64.exe
- Local copies of some external repositories: C:\Users\K\RepositoryExternal

# Style

For single and double quotes, only use the ASCII forms: ', "
Never use these: “, ”, ‘, ’, etc.

# Application compatibility

Do not attempt to maintain any sort of application compatibility between different commits of the repo. This creates unwanted complexity.

# Development platform compatibility

Support Windows 11 x64 as the only development platform.

# Target platform compatibility

Support Windows 11 x64 as the only target platform.

# PowerShell

All PowerShell scripts must:

- Run on Powershell 7 or higher
- Use `Set-StrictMode -Version Latest`
- Use `$ErrorActionPreference = 'Stop'`

# Git

Never change stuff on GitHub.
When implementing stuff, avoid difficult-to-review "mega-commits".
When it makes sense, split work into multiple commits to make it easier to review.
After you make changes, commit them. There's no need to ask the user for permission to commit.
Run `git lfs checkout` after creating a new worktree.

# Correctness

The law of the frame: given the state and the frame's input at its start, everything the input
causes is in that frame's output - in the interface, and in game logic. An effect delayed any
number of frames is a bug. Test a cause by asserting its whole consequence graph on the frame
that caused it.

# Markdown

Tables in Markdown must be padded and aligned in a way to make them easy to read in a plaintext editor, not only in a Markdown viewer.
Any mathematical notation in Markdown files (LaTeX, KaTeX, MathJax, etc) must display properly in VSCode's Markdown previewer and GitHub.com's Markdown displayer.

# Repo layout

- `Program` holds runnable stuff.
- `Library` contains shared stuff that isn't runnable.
- `Test` contains all test stuff.
- Every Rust package is a member of the Cargo workspace in the root `Cargo.toml`.

# GUI program rules

- For 3D models, only use SimplePaint or unlit shader.
- `F9` toggles between light and dark mode.
- `F10` toggles between VSync on and off.
- These shortcuts are owned by `Library/user-interface` and never reach a program:
  - `Ctrl + Shift + Alt + F12`: Quits the program at once without asking
    - Closing the window from its taskbar button does the same.
  - `Ctrl + Shift + Alt + F11`: Toggles between GameInput cursor and Windows hardware cursor.

## Supported window setup

Programs only support the user's own setup, to keep development simple. They support exactly one window setup: **borderless fullscreen** on a 2560 × 1440 monitor at 100% scaling (96 DPI), where logical and physical pixels are the same, with a client area of 2560 × 1440 that covers the whole monitor.

In any other setup, the window shows an error that says what's wrong, instead of the program's cards. `Library/user-interface` does the check.

So:

- The top 17 pixels of the client area are `Library/user-interface`'s PerfBar, in every program. Lay out cards in fixed pixels for what's left of the client area, from `ui.content_top()`: 2560 × 1423.
- Don't add DPI scaling, other window sizes, a windowed mode, minimum sizes, or layouts that adapt to the window's size. Don't keep programs working in other setups beyond showing the error. A 3D view may fill the window.

## Non-disruptive workflow

While working:

- The agent must not launch a GUI program on any display except for CVVNY2.
- The agent must not steal input focus away. For example, if the user is typing in another window, then those keypresses shouldn't be rerouted to processes or windows that the agent launches.
- It is fine if the agent launches things that show up on the Windows taskbar. This isn't considered disruptive.
- The agent should avoid making a game that the user is playing stutter. Keep the CPU and GPU work that it starts light, as below.

How:

- Run builds, tests and anything else that keeps the CPU busy with `Program\RunLightly\RunLightly.ps1 <command>`, like `Program\RunLightly\RunLightly.ps1 cargo test --workspace`. It runs the command, and every process that it starts, only on the CPU's efficiency cores, at idle priority, with at most a third of the CPU's time, at low I/O and memory priority, in efficiency mode, and with their GPU work after other processes', so that a game the user is playing keeps its cores and its frame rate. It's slower than running the command directly. Give it `cargo` as the command itself, not a script that runs cargo: rustup's cargo puts itself in a job of its own, which RunLightly can't take it out of afterwards. Quote a `--` that's for the command, like `Program\RunLightly\RunLightly.ps1 cargo test -p present-timing-tests '--' --ignored`: PowerShell takes a bare `--` for itself.
- Run a GUI program with `Program\RunInBackground\RunInBackground.ps1 <program>`, like `Program\RunInBackground\RunInBackground.ps1 simple-paint-demo`, from the root of the repository. It builds the program, opens it on CVVNY2 without activating it, without GameInput, in the light theme whatever the program's `Settings.json` says, and with vertical sync, so that it draws at most one frame a refresh and is light on a computer that someone's playing games on, saves a screenshot of what it draws to `target\RunInBackground\<program>.png`, and closes it. Read the screenshot to see the result.
- With `--debug-build`, the D3D12 debug layer's warnings and errors go to `target\RunInBackground\<program>Errors.log`. Check it after changing how a program draws.
- Don't start a GUI program any other way, like with `cargo run`. The `HumanRun.cmd` and `HumanRun.ps1` in a program's folder are for the user to launch it and use it. Never run them. Without `--no-activate`, a program's window takes focus. Without `--display CVVNY2`, it opens on the display in its `Settings.json`, or on the primary display if that one isn't connected.
- If the script says CVVNY2 isn't connected, stop and tell the user. Don't pick another display.
- With `--keep-open`, the program keeps running after the script ends. Stop it with `Stop-Process` and the process ID the script prints, once you're done with it.
- Don't send keyboard or mouse input with `SendInput` or anything like it. It goes to the window that has focus, which is the user's, and programs ignore all input without focus anyway. Test input with tests and scripts instead, as below.
- Tests that use the GPU, like `cargo test -p simple-paint-tests`, draw offscreen, without a window, so run them freely, with RunLightly like any other build.

## Power and locking

Assume that Windows' "Turn my screen off after", "Make my device sleep after" and "Make my device hibernate after" are all set to "Never", both plugged in and on battery. The machine may be locked or unlocked, though, and may lock or unlock while the agent works.

- Work the same either way. Builds, `cargo test --workspace`, and RunInBackground's screenshots and scripts all work while the machine is locked, and draw the same frames.
- While it's locked, the lock screen has focus, so a program never does, as in any run.
- Don't lock or unlock the machine, wake or turn off its displays, or change its power settings.

## Testing input

Test what programs do with input in two layers, which never touch the user's input or focus:

1. **Tests**, in `Test/<program>`. Each GUI program is a library (`src/lib.rs`) and a small `src/main.rs`, so tests drive its parts without a window: what input does, where clicks land in the cards' real layout, which the library lays out without a GPU, and its motion and settings. `user_interface::Headless` runs a program's frames that way, with input that the test makes up, and can draw them offscreen. Put a program's logic in its library, and test it there. Run them with `Program\RunLightly\RunLightly.ps1 cargo test --workspace`.
2. **Scripts**, in `Test/<program>/Scripts`. `Program\RunInBackground\RunInBackground.ps1 <program> --script <path>` runs the real program with a script's input instead of the user's, on a fixed clock, and `--all-scripts` runs every script in that folder. Steps move, click, drag, scroll, press and hold keys in the window, move the gamepad, save frames and the program's state, and check the state. [Library/user-interface's README](Library/user-interface/README.md#testing) describes the format. A run fails if an expectation isn't met, or if the window takes focus. Read the saved PNGs to see the frames.

To write a script, run one that captures, and read the state it saves: `Interface.Targets` has every widget's name and area, like SimplePaintDemo's sliders. Point at widgets by name, like `{ "Frame": 1, "Press": "Volume" }`, instead of by pixels, so that scripts keep working when a layout changes. GameInputDemo's scripts also connect and press gamepads, keyboards and mice, which its README describes.

After changing a program, run `cargo test --workspace`, with RunLightly, and its scripts with `--all-scripts`. Neither covers real devices, Windows' input path, the user taking or giving focus, or input latency. Ask the user to try those.
