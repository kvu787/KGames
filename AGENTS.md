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

# GameInput

GameInput 4.0.306 must be installed, once, from an elevated prompt, with:

```powershell
winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"
```

That's the user's to do. Nothing in the repo may install GameInput. KGames doesn't build without
it, and programs don't start without it, which `Library/GameInput` checks. If a build or a program
says that it isn't installed, stop and tell the user.

The build restores the `Microsoft.GameInput` NuGet package, 4.0.306, with `dotnet`, as
`Library/GameInput/NuGetPackages.proj` declares and its `packages.lock.json` pins, and links its
`GameInput.lib`. It needs the .NET SDK, 10 or later: if a build says that it's missing, stop and
tell the user. Programs start GameInput with `GameInputInitialize` from `GameInput.lib`, Microsoft's
loader, which loads the installed runtime unless a `GameInputRedist.dll` is beside the exe. Nothing
in the repo may put one there, nor copy or load the runtimes in the package, which are for
sideloading.

# PowerShell

All PowerShell scripts must:

- Run on Powershell 7 or higher
- Use `Set-StrictMode -Version Latest`
- Use `$ErrorActionPreference = 'Stop'`

# Correctness

Development here is heavy and session-based: a rule that only a machine check enforces is the only
kind that holds, because a later session never read the conversation that invented it.

- The law of the frame: given the state and the frame's input at its start, everything the input
  causes is in that frame's output - in the interface, and in game logic. An effect delayed any
  number of frames is a bug. Test a cause by asserting its whole consequence graph on the frame
  that caused it.
- Every bug fix lands with a test that pins it, in the same change. A fix without a test is a bug
  that will come back.
- A discipline that can't be a type must be a test. For messages and game events: they carry the
  facts of their moment - which coin, which row's item, the view the user was shown - and their
  consumers read everything else from the state as it is when they apply, never from a snapshot.
  The test for an intent-carrying message is two of the same intent in one frame: they cancel, or
  apply twice, and never collapse into one.
- Simulations move by each frame's own time, the time since the frame before, in one step a frame,
  like a Godot game's `_process` with its delta, never in fixed steps on a clock of their own: a
  frame rate that a fixed step doesn't divide, like 100 frames a second in steps of 1/240 of a
  second, fits 2 steps in some frames and 3 in others, and what moves judders. Cap a frame's time,
  so that a late one, like after the window was minimized, doesn't carry things far, and test
  frames at 100 frames a second and of uneven lengths: each moves things by its own time. Event
  queues drain to quiescence inside the frame; a cascade that doesn't settle stops the program
  instead of hiding.
- A comment is a promise: a comment or README that says the code does something the code doesn't
  do is a bug, and models believe documentation more than people do. Keep them true in the same
  change that would make them false.
- A rewrite of a library core, like `Library/UserInterface/src/core`, is a rare, deliberate
  event: new machinery re-encounters every edge case the old machinery had already fixed. It ends
  with an adversarial review of the new code and the findings pinned by tests, as
  `Documentation/LogicReview.md` records for the last one.

# License

KGames is licensed under GPL-3.0-or-later, the GNU General Public License, version 3 or any later
version. `LICENSE.txt` has its text, and it covers everything in the repository.

- Code or assets taken from elsewhere must be under a license that GPL-3.0 can take in, like MIT,
  BSD, Apache-2.0, zlib, ISC, Boost, LGPL, GPL-2.0-or-later or GPL-3.0, and keep their copyright
  and license notices. GPL-2.0-only, CDDL, EPL-1.0, licenses that forbid commercial use, and code
  with no license can't come in. The FSF's
  [license list](https://www.gnu.org/licenses/license-list.html) says which licenses are
  compatible.
- Closed-source SDKs and runtimes, like the GameInput runtime, which the user installs, stay out
  of the repository.
- `GameInput.lib`, which KGames links, comes from the `Microsoft.GameInput` NuGet package, which the
  build restores outside the repository. The package ships its source, `native/src/GameInput.cpp`,
  under the MIT license, and KGames treats the library as MIT-licensed too.
- Every Rust package takes its license from the root `Cargo.toml`, with `license.workspace = true`,
  except `Library/WindowsApi/BindingsGenerator`, a workspace of its own, which states it.

# Git

When implementing stuff, avoid difficult-to-review "mega-commits".
When it makes sense, split work into multiple commits to make it easier to review.
After you make changes, commit them. There's no need to ask the user for permission to commit.

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
- Build every GUI program on `Library/UserInterface`, the UI library: make the program a `Program`, run it with `user_interface::run`, which is the main loop, and make its controls with the library's widgets. A program is its state and messages: input on the tree the user saw becomes messages, `update` applies them, and one read-only `view` describes the frame's widgets from the state, so a frame's output always comes from that frame's input, never a frame late. When a program needs a widget or a kind of input that the library doesn't have, add it to `Library/UserInterface`, test it in `Test/UserInterface`, and show it in `Program/UiShowcase`, instead of making it in the program.
- `Program/WinUi3Sample` is the exception to these rules for GUI programs: a real WinUI 3 app, in C# and XAML, to compare the library with. Its own `AGENTS.md` says what applies to it.
- Follow the library's design system, Microsoft's Fluent 2: color with the theme's roles, from `Ui::theme`, like `text_secondary` or `accent`, instead of colors of your own, and size and space with the tokens in `Library/UserInterface/src/design/tokens.rs`, like its spacing, radii and type ramp, and WinUI 3's controls' metrics in `Library/UserInterface/src/design/controls.rs`.
- Don't animate what isn't integral to what the GUI does. A change of state, like a hover, a press, a selection, a switch that's flipped, or a menu, submenu or dialog that opens, shows whole on the frame that makes it, without a fade, a slide or an ease. Animate only what has to move to do its job, like an indeterminate progress ring, whose motion says that the program is working and isn't frozen. `Library/UserInterface` follows this, as its README's design system says, and `Test/UserInterface/tests/rendering.rs` checks it.
- `F9` toggles between light and dark mode.
- `F10` toggles between VSync on and off.
- `F11` toggles between windowed and borderless fullscreen.
- Optimize for minimal input latency. Draw through `Library/Graphics`' presenter, and read input right before drawing, once `Presenter::poll_ready` is true. `Library/UserInterface`'s main loop does both, and measures the input latency from GameInput's readings to the screen, which the PerfBar along the top of every window shows.

## Supported window setups

Programs only support the user's own setup, to keep development simple. They support exactly two window setups, both on a 2560 × 1440 monitor at 100% scaling (96 DPI), where logical and physical pixels are the same:

- **Windowed maximized**, with a client area of 2560 × 1369. The taskbar takes 48 pixels at the bottom of the monitor, which leaves a 2560 × 1392 work area. The title bar takes the top 23 pixels of that, so the client area runs from screen y = 23 down to the bottom of the work area: 1392 − 23 = 1369.
- **Borderless fullscreen**, with a client area of 2560 × 1440 that covers the whole monitor. `F11` switches between the two.

In any other setup, the window shows an error that says what's wrong, instead of the program's cards. `Library/UserInterface` does the check.

So:

- The top 17 pixels of the client area are `Library/UserInterface`'s PerfBar, in every program. Lay out cards in fixed pixels for what's left of these two client areas, from `ui.content_top()`: 2560 × 1352 and 2560 × 1423. The only difference between the two is 71 pixels of height.
- Don't add DPI scaling, other window sizes, a windowed mode, minimum sizes, or layouts that adapt to the window's size. Don't keep programs working in other setups beyond showing the error. A 3D view may fill the window at either height.

## Non-disruptive workflow

While working:

- The agent must not launch a GUI program on any display except for CVVNY2.
- The agent must not steal input focus away. For example, if the user is typing in another window, then those keypresses shouldn't be rerouted to processes or windows that the agent launches.
- It is fine if the agent launches things that show up on the Windows taskbar. This isn't considered disruptive.
- The agent must not make a game that the user is playing stutter. Keep the CPU and GPU work that it starts light, as below.

How:

- Run builds, tests and anything else that keeps the CPU busy with `Program\RunLightly\RunLightly.ps1 <command>`, like `Program\RunLightly\RunLightly.ps1 cargo test --workspace`. It runs the command, and every process that it starts, only on the CPU's efficiency cores, at idle priority, with at most a third of the CPU's time, at low I/O and memory priority, in efficiency mode, and with their GPU work after other processes', so that a game the user is playing keeps its cores and its frame rate. It's slower than running the command directly. Give it `cargo` as the command itself, not a script that runs cargo: rustup's cargo puts itself in a job of its own, which RunLightly can't take it out of afterwards.
- Run a GUI program with `Program\RunInBackground\RunInBackground.ps1 <program>`, like `Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo`, from the root of the repository. It builds the program, opens it on CVVNY2 without activating it, without GameInput, in the light theme and windowed maximized whatever the program's `Settings.json` says, and with vertical sync, so that it draws at most one frame a refresh and is light on a computer that someone's playing games on, saves a screenshot of what it draws to `target\RunInBackground\<program>.png`, and closes it. Read the screenshot to see the result.
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

1. **Tests**, in `Test/<program>`. Each GUI program is a library (`src/lib.rs`) and a small `src/main.rs`, so tests drive its parts without a window: what input does, where clicks land in the cards' real layout, which the library lays out without a GPU, and its motion and settings. `user_interface::Headless` runs a program's frames that way, with input that the test makes up, and can draw them offscreen. Put a program's logic in its library, and test it there. Run them with `Program\RunLightly\RunLightly.ps1 cargo test --workspace`.
2. **Scripts**, in `Test/<program>/Scripts`. `Program\RunInBackground\RunInBackground.ps1 <program> --script <path>` runs the real program with a script's input instead of the user's, on a fixed clock, and `--all-scripts` runs every script in that folder. Steps move, click, drag, scroll, press and hold keys in the window, move the gamepad, save frames and the program's state, and check the state. [Library/UserInterface's README](Library/UserInterface/README.md#testing) describes the format. A run fails if an expectation isn't met, or if the window takes focus. Read the saved PNGs to see the frames.

To write a script, run one that captures, and read the state it saves: `Interface.Targets` has every widget's name and area, like SimplePaintDemo's sliders. Point at widgets by name, like `{ "Frame": 1, "Press": "Volume" }`, instead of by pixels, so that scripts keep working when a layout changes. GameInputDemo's scripts also connect and press gamepads, keyboards and mice, which its README describes.

After changing a program, run `cargo test --workspace`, with RunLightly, and its scripts with `--all-scripts`. Neither covers real devices, Windows' input path, the user taking or giving focus, or input latency. Ask the user to try those.
