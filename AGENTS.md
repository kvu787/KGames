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

- Ignore all input while the window isn't focused, gamepads too. `Library/Application`'s `Window` does this for input from window messages, and programs that read GameInput drop its readings themselves.
- Use Win32, Rust, Direct3D 12 and DirectWrite, through the crates in `Library`.
- Use the light theme in `Library/Canvas` by default.
- `F11` toggles borderless fullscreen.
- Optimize for minimal input latency. Draw through `Library/Graphics`' renderer, and read input right before drawing, once `Renderer::is_ready` is true.

## Supported window setups

Programs only support the user's own setup, to keep development simple. They support exactly two window setups, both on a 2560 × 1440 monitor at 100% scaling (96 DPI), where logical and physical pixels are the same:

- **Maximized**, with a client area of 2560 × 1369. The taskbar takes 48 pixels at the bottom of the monitor, which leaves a 2560 × 1392 work area. The title bar takes the top 23 pixels of that, so the client area runs from screen y = 23 down to the bottom of the work area: 1392 − 23 = 1369.
- **Borderless fullscreen**, with a client area of 2560 × 1440 that covers the whole monitor. `F11` switches between the two.

In any other setup, the window shows an error that says what's wrong, instead of the program's panels. `Library/Application` does the check.

So:

- Lay out panels in fixed pixels for these two client areas. The only difference between the two is 71 pixels of height.
- Don't add DPI scaling, other window sizes, a windowed mode, minimum sizes, or layouts that adapt to the window's size. Don't keep programs working in other setups beyond showing the error. A 3D view may fill the window at either height.

## Non-disruptive workflow

While working:

- The agent must not launch a GUI program on any display except for CVVNY2.
- The agent must not steal input focus away. For example, if the user is typing in another window, then those keypresses shouldn't be rerouted to processes or windows that the agent launches.
- It is fine if the agent launches things that show up on the Windows taskbar. This isn't considered disruptive.

How:

- Run a GUI program with `Program\RunInBackground\RunInBackground.ps1 <program>`, like `Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo`, from the root of the repository. It builds the program, opens it on CVVNY2 without activating it, saves a screenshot of what it draws to `target\RunInBackground\<program>.png`, and closes it. Read the screenshot to see the result.
- With `--debug-build`, the D3D12 debug layer's warnings and errors go to `target\RunInBackground\<program>Errors.log`. Check it after changing how a program draws.
- Don't start a GUI program any other way, like with `cargo run`. The `HumanRun.cmd` and `HumanRun.ps1` in a program's folder are for the user to launch it and use it. Never run them. Without `--no-activate`, a program's window takes focus. Without `--display CVVNY2`, it opens on the display in its `Settings.json`, or on the primary display if that one isn't connected.
- If the script says CVVNY2 isn't connected, stop and tell the user. Don't pick another display.
- With `--keep-open`, the program keeps running after the script ends. Stop it with `Stop-Process` and the process ID the script prints, once you're done with it.
- Run `IdentifyDisplays` only with `--list`. Without it, it puts a box on every display.
- Don't send keyboard or mouse input with `SendInput` or anything like it. It goes to the window that has focus, which is the user's, and programs ignore all input without focus anyway. Test input with tests and scripts instead, as below.
- Tests that use the GPU, like `cargo test -p simple_paint_tests`, draw offscreen, without a window, so run them freely.

## Testing input

Test what programs do with input in two layers, which never touch the user's input or focus:

1. **Tests**, in `Test/<program>`. Each GUI program is a library (`Source/Library.rs`) and a small `Source/Main.rs`, so tests drive its parts without a window: what input does, where clicks land in the panels' real layout, which a canvas builds without a GPU, and its motion and settings. Put a program's logic in its library, and test it there. Run them with `cargo test --workspace`.
2. **Scripts**, in `Test/<program>/Scripts`. `Program\RunInBackground\RunInBackground.ps1 <program> --script <path>` runs the real program with a script's input instead of the user's, on a fixed clock, and `--all-scripts` runs every script in that folder. Steps move, click, drag, scroll and press keys in the window, save frames and the program's state, and check the state. `Library/Application/Source/Script.rs` describes the format. A run fails if an expectation isn't met, or if the window takes focus. Read the saved PNGs to see the frames.

To write a script, run one that captures, and read the state it saves: programs with click targets save where they are, like SimplePaintDemo's sliders. GameInputDemo's scripts also connect and press gamepads, keyboards and mice, which its README describes.

After changing a program, run `cargo test --workspace` and its scripts with `--all-scripts`. Neither covers real devices, Windows' input path, the user taking or giving focus, or input latency. Ask the user to try those.
