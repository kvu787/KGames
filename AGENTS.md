# Style

For single and double quotes, only use the ASCII forms: ', "
Never use these: “, ”, ‘, ’, etc.

# Application compatibility

Do not attempt to maintain any sort of application compatibility between different commits of the repo. This creates unwanted complexity.

# Development platform compatibility

Support Windows 11 x64 as the only development platform.

# Target platform compatibility

Support Windows 11 x64 as the only target platform.

# Folder and file naming

This only applies to things that we have the freedom to name as wanted.
Use CamelCase.
Use complete proper words. Don't use typical shortenings. Good: Source, Documentation. Bad: src, docs.

Exceptions: README.md, LICENSE.md, TODO.md, README.txt, LICENSE.txt, TODO.txt, AGENTS.md

# PowerShell

All PowerShell scripts must:

- Run on Powershell 7 or higher
- Use `Set-StrictMode -Version Latest`
- Use `$ErrorActionPreference = 'Stop'`

# Git

When implementing stuff, avoid difficult-to-review "mega-commits".
When it makes sense, split work into multiple commits to make it easier to review.

# Markdown

Tables in Markdown must be padded and aligned in a way to make them easy to read in a plaintext editor, not only in a Markdown viewer.
Any mathematical notation in Markdown files (LaTeX, KaTeX, MathJax, etc) must display properly in VSCode's Markdown previewer and GitHub.com's Markdown displayer.

# Repo layout

- `Program` holds what's runnable, `Library` what isn't and is shared, and `Test` every test.
- Put code that more than one program can use in `Library`, not in a program.
- Every Rust package is a member of the Cargo workspace in the root `Cargo.toml`.
- Add Windows APIs to `Library/WindowsApi`, as its README says, instead of generating bindings
  in another crate. Every crate must share its types.

# GUI programs

- Read all input with GameInput, through `Library/UserInterface`, which reads keyboards, mice and gamepads with it, and not from window messages.
- Ignore all input while the window isn't focused, gamepads too. `Library/UserInterface` does this for the events it makes from GameInput's readings, and programs that read those readings themselves, like GameInputDemo, drop them themselves.
- Use Win32, Rust, Direct3D 12 and DirectWrite, through the crates in `Library`.
- Build every GUI program on `Library/UserInterface`, the UI library: make the program a `Program`, run it with `user_interface::run`, which is the main loop, and make its controls with the library's widgets. When a program needs a widget or a kind of input that the library doesn't have, add it to `Library/UserInterface`, test it in `Test/UserInterface`, and show it in `Program/UiShowcase`, instead of making it in the program.
- Use the light theme by default.
- Follow the library's design system, Microsoft's Fluent 2: color with the theme's roles, from `Ui::theme`, like `text_secondary` or `accent`, instead of colors of your own, and size and space with the tokens in `Library/UserInterface/Source/Design/Tokens.rs`, like its spacing, radii and type ramp, and WinUI 3's controls' metrics in `Library/UserInterface/Source/Design/Controls.rs`.
- `F11` toggles borderless fullscreen.
- Optimize for minimal input latency. Draw through `Library/Graphics`' presenter, and read input right before drawing, once `Presenter::poll_ready` is true. `Library/UserInterface`'s main loop does both, and measures the input latency from GameInput's readings to the screen, which the performance stats along the top of every window show.

## Supported window setups

Programs only support the user's own setup, to keep development simple. They support exactly two window setups, both on a 2560 × 1440 monitor at 100% scaling (96 DPI), where logical and physical pixels are the same:

- **Maximized**, with a client area of 2560 × 1369. The taskbar takes 48 pixels at the bottom of the monitor, which leaves a 2560 × 1392 work area. The title bar takes the top 23 pixels of that, so the client area runs from screen y = 23 down to the bottom of the work area: 1392 − 23 = 1369.
- **Borderless fullscreen**, with a client area of 2560 × 1440 that covers the whole monitor. `F11` switches between the two.

In any other setup, the window shows an error that says what's wrong, instead of the program's cards. `Library/UserInterface` does the check.

So:

- The top 30 pixels of the client area are `Library/UserInterface`'s performance stats, in every program. Lay out cards in fixed pixels for what's left of these two client areas, from `ui.content_top()`: 2560 × 1339 and 2560 × 1410. The only difference between the two is 71 pixels of height.
- Don't add DPI scaling, other window sizes, a windowed mode, minimum sizes, or layouts that adapt to the window's size. Don't keep programs working in other setups beyond showing the error. A 3D view may fill the window at either height.

## Non-disruptive workflow

While working:

- The agent must not launch a GUI program on any display except for CVVNY2.
- The agent must not steal input focus away. For example, if the user is typing in another window, then those keypresses shouldn't be rerouted to processes or windows that the agent launches.
- It is fine if the agent launches things that show up on the Windows taskbar. This isn't considered disruptive.
- The agent must not make a game that the user is playing stutter. Keep the CPU and GPU work that it starts light, as below.

How:

- Run builds, tests and anything else that keeps the CPU busy with `Program\RunLightly\RunLightly.ps1 <command>`, like `Program\RunLightly\RunLightly.ps1 cargo test --workspace`. It runs the command, and every process that it starts, only on the CPU's efficiency cores, at idle priority, with at most a third of the CPU's time, so that a game the user is playing keeps its cores and its frame rate. It's slower than running the command directly.
- Run a GUI program with `Program\RunInBackground\RunInBackground.ps1 <program>`, like `Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo`, from the root of the repository. It builds the program, opens it on CVVNY2 without activating it, without GameInput, with vertical sync and at most 100 frames a second, so that it's light on a computer that someone's playing games on, saves a screenshot of what it draws to `target\RunInBackground\<program>.png`, and closes it. Read the screenshot to see the result.
- With `--debug-build`, the D3D12 debug layer's warnings and errors go to `target\RunInBackground\<program>Errors.log`. Check it after changing how a program draws.
- Don't start a GUI program any other way, like with `cargo run`. The `HumanRun.cmd` and `HumanRun.ps1` in a program's folder are for the user to launch it and use it. Never run them. Without `--no-activate`, a program's window takes focus. Without `--display CVVNY2`, it opens on the display in its `Settings.json`, or on the primary display if that one isn't connected.
- If the script says CVVNY2 isn't connected, stop and tell the user. Don't pick another display.
- With `--keep-open`, the program keeps running after the script ends. Stop it with `Stop-Process` and the process ID the script prints, once you're done with it.
- Run `IdentifyDisplays` only with `--list`. Without it, it puts a box on every display.
- Don't send keyboard or mouse input with `SendInput` or anything like it. It goes to the window that has focus, which is the user's, and programs ignore all input without focus anyway. Test input with tests and scripts instead, as below.
- Tests that use the GPU, like `cargo test -p simple_paint_tests`, draw offscreen, without a window, so run them freely, with RunLightly like any other build.

## Power and locking

Assume that Windows' "Turn my screen off after", "Make my device sleep after" and "Make my device hibernate after" are all set to "Never", both plugged in and on battery. The machine may be locked or unlocked, though, and may lock or unlock while the agent works.

- Work the same either way. Builds, `cargo test --workspace`, and RunInBackground's screenshots and scripts all work while the machine is locked, and draw the same frames.
- While it's locked, the lock screen has focus, so a program never does, as in any run.
- Don't lock or unlock the machine, wake or turn off its displays, or change its power settings.

## Testing input

Test what programs do with input in two layers, which never touch the user's input or focus:

1. **Tests**, in `Test/<program>`. Each GUI program is a library (`Source/Library.rs`) and a small `Source/Main.rs`, so tests drive its parts without a window: what input does, where clicks land in the cards' real layout, which the library lays out without a GPU, and its motion and settings. `user_interface::Headless` runs a program's frames that way, with input that the test makes up, and can draw them offscreen. Put a program's logic in its library, and test it there. Run them with `Program\RunLightly\RunLightly.ps1 cargo test --workspace`.
2. **Scripts**, in `Test/<program>/Scripts`. `Program\RunInBackground\RunInBackground.ps1 <program> --script <path>` runs the real program with a script's input instead of the user's, on a fixed clock, and `--all-scripts` runs every script in that folder. Steps move, click, drag, scroll, press and hold keys in the window, move the gamepad, save frames and the program's state, and check the state. [Library/UserInterface's README](Library/UserInterface/README.md#testing) describes the format. A run fails if an expectation isn't met, or if the window takes focus. Read the saved PNGs to see the frames.

To write a script, run one that captures, and read the state it saves: `Interface.Targets` has every widget's name and area, like SimplePaintDemo's sliders. Point at widgets by name, like `{ "Frame": 1, "Press": "Volume" }`, instead of by pixels, so that scripts keep working when a layout changes. GameInputDemo's scripts also connect and press gamepads, keyboards and mice, which its README describes.

After changing a program, run `cargo test --workspace`, with RunLightly, and its scripts with `--all-scripts`. Neither covers real devices, Windows' input path, the user taking or giving focus, or input latency. Ask the user to try those.
