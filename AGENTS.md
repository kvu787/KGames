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

# C++

KGames is C++, as the newest MSVC compiles it with `/std:c++latest`, built with MSBuild. It uses
the C++ standard library, the Windows SDK and GameInput, and no other library.

## Building and testing

- `Program\Build\Build.ps1` builds every project in `KGames.slnx`, in Debug, or the projects named,
  like `Program\Build\Build.ps1 --release Platformer`. It finds Visual Studio's MSBuild, which isn't
  on the PATH. Programs and libraries go to `target\Debug` and `target\Release`.
- `Program\RunTests\RunTests.ps1` builds the tests in `Test` and runs them, or the test projects
  named, like `Program\RunTests\RunTests.ps1 JsonTests --filter writes`. Tests build in Debug, where
  the C runtime and the standard library check what Release doesn't, like the bounds of an index.
- Run both through RunLightly, as below.
- `KGames.props` says what each `.vcxproj` builds from where it is, and `KGames.targets` what it
  compiles: every `.cpp` in its folder, so a new file needs no line in a project. A new project
  goes in `KGames.slnx`, and names the projects it uses with `ProjectReference`.
- Warnings are errors, at `/W4`. Fix what a warning points at instead of turning it off.
- Format C++ with `.clang-format`, with Visual Studio's `clang-format.exe`, before committing.

## Files

- A library is `Library/<Name>`: `<Name>.vcxproj`, and its headers and sources beside it, in
  folders for its parts, like `Library/UserInterface/Core/Interface.h` and `Interface.cpp`.
  `<Name>.h` includes every header that the library's users need.
- A program is `Program/<Name>`: `<Name>Library.vcxproj` builds everything but `Main.cpp` into a
  static library, which its tests link, and `<Name>.vcxproj` builds `Main.cpp` into `<Name>.exe`.
- A project's tests are `Test/<Name>/<Name>Tests.vcxproj`, with a `.cpp` for each thing they test.
- Name files and folders in PascalCase. Headers are `.h`, start with a comment that says what the
  file is, and then `#pragma once`.
- Include the repository's headers with quotes, from `Library` or `Program`, like
  `"Json/Json.h"` or `"Platformer/Game.h"`, and everything else with angle brackets.
- Include `Windows.h` only through `"Base/Windows.h"`, before any other header of the Windows SDK.
- Link a system library where it's used, with `#pragma comment(lib, "d3d12.lib")` in the `.cpp`.
- A project that compiles shaders names them with `Shader` items, as `KGames.targets` says.

## Names

- Types are `PascalCase`. Functions, variables, parameters, members and namespaces are
  `snake_case`. Constants and enumerators are `PascalCase`, like `LineLength` and `Layer::Popup`.
- Nothing is `ALL_CAPS` but a macro, since the Windows SDK defines thousands of those, like
  `DELETE`, `TRANSPARENT`, `NEAR`, `ERROR` and `TEXT`, which would replace a constant of the name.
- A library's namespace is its name in snake_case, like `user_interface` for
  `Library/UserInterface`. A program's is the program's name, like `platformer`.
- A data member that a member function shares its name with ends in an underscore: `rect_` for
  `rect()`.
- These are macros or keywords, so a name that would be one ends in an underscore, like `near_`:
  `near`, `far`, `small`, `interface`, `min`, `max`, `leave`, `except`, `try`, `module`,
  `requires`, `default`, `delete`, `new`, `this`, `template`, `class`, `union`, `export`,
  `register`, `and`, `or`, `not`, `xor`.
- A test's name is a sentence in snake_case that says what holds, like
  `a_jump_press_rises_on_its_own_frame`.

## Types

- A struct that's only data is a `struct` with public members, default member initializers, and
  `bool operator==(const T&) const = default;` if anything compares it. A type with something to
  keep true is a `class` with private members.
- A type that code never copies is move-only: delete its copy constructor and copy assignment, so
  that a copy that's meant to be a move doesn't compile. Types that own what's costly to copy, like
  a tree of widgets or a mesh, are move-only unless copying one is part of what they're for.
- Take what a function only reads as `const T&`, or by value if it's as small as two pointers, and
  as `std::string_view` or `std::span<const T>` for text and arrays. Take what it keeps by value,
  and `std::move` it in. Take what it changes as `T&`.
- Never keep a reference, pointer, iterator, `string_view` or `span` into a container across a
  call that can grow or change the container.
- Something that may be absent is a `std::optional<T>`, or a pointer, null for absent, when it's
  something that lives elsewhere.
- A value that's one of several alternatives with their own data is a sum type, as
  `Library/Base/Sum.h` says: a struct with its alternatives nested, and `base::match`, `base::is`
  and `base::as`. Without data, it's an `enum class`.
- Something with several implementations, chosen as the program runs, is a class with virtual
  functions and a virtual destructor, held in a `std::unique_ptr`. Chosen as it compiles, it's a
  template, with a concept that says what it needs.
- COM objects are held in `Microsoft::WRL::ComPtr`.
- Text is UTF-8 in `std::string`, and UTF-16 only on the way into and out of the Windows API, with
  `base::widen` and `base::narrow`. A character is a `char32_t`, from `base::chars`.
- Sizes and indices are `size_t`. A cast between number types is a `static_cast`, and a float that
  may be out of an integer's range goes through `base::saturating_cast`. Float literals of `float`s
  end in `f`.

## Errors

Three kinds, and each has one way:

- What can go wrong in what the program reads, like a settings file or a model: the function
  returns `std::expected<T, std::string>`, or an error type of its own, and its caller decides.
- A call of the Windows API that fails: `base::check` throws a `base::WindowsError`, which
  nothing catches but the code that can go on without what failed.
- What can't happen: `base::panic`, which throws a `base::Panic` that only tests catch, so it
  stops the program. `optional.value()` and `expected.value()` throw too, where the code relies
  on there being a value.

A destructor never throws. Nothing else throws on purpose.

## Tests

- A test is `TEST(name) { ... }` with `CHECK`, `CHECK_EQ`, `CHECK_NE` and `CHECK_THROWS`, as
  `Library/Testing/Testing.h` says. A failed check ends its test.
- A type that tests compare says how a failed check shows it, with a `describe_value` function
  beside it, in its namespace, that returns a `std::string`.
- A test reads its files from `folders::Project` or `folders::Repository`, from `"Folders.h"`, and
  writes only under `std::filesystem::temp_directory_path()`, in a folder of its own.

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
- Simulations run in fixed steps on their own clock, never the frame's, and a test drives the same
  inputs at 60 and 240 frames a second and asserts identical results, events included. Event
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
- Use the light theme by default.
- Follow the library's design system, Microsoft's Fluent 2: color with the theme's roles, from `Ui::theme`, like `text_secondary` or `accent`, instead of colors of your own, and size and space with the tokens in `Library/UserInterface/src/design/tokens.rs`, like its spacing, radii and type ramp, and WinUI 3's controls' metrics in `Library/UserInterface/src/design/controls.rs`.
- Don't animate what isn't integral to what the GUI does. A change of state, like a hover, a press, a selection, a switch that's flipped, or a menu, submenu or dialog that opens, shows whole on the frame that makes it, without a fade, a slide or an ease. Animate only what has to move to do its job, like an indeterminate progress ring, whose motion says that the program is working and isn't frozen. `Library/UserInterface` follows this, as its README's design system says, and `Test/UserInterface/tests/rendering.rs` checks it.
- `F9` toggles between light and dark mode. Default is light mode.
- `F10` toggles between VSync on and off. Default is VSync on.
- `F11` toggles between windowed and borderless fullscreen. Default is windowed.
- Optimize for minimal input latency. Draw through `Library/Graphics`' presenter, and read input right before drawing, once `Presenter::poll_ready` is true. `Library/UserInterface`'s main loop does both, and measures the input latency from GameInput's readings to the screen, which the PerfBar along the top of every window shows.

## Supported window setups

Programs only support the user's own setup, to keep development simple. They support exactly two window setups, both on a 2560 × 1440 monitor at 100% scaling (96 DPI), where logical and physical pixels are the same:

- **Maximized**, with a client area of 2560 × 1369. The taskbar takes 48 pixels at the bottom of the monitor, which leaves a 2560 × 1392 work area. The title bar takes the top 23 pixels of that, so the client area runs from screen y = 23 down to the bottom of the work area: 1392 − 23 = 1369.
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
- Run a GUI program with `Program\RunInBackground\RunInBackground.ps1 <program>`, like `Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo`, from the root of the repository. It builds the program, opens it on CVVNY2 without activating it, without GameInput, with vertical sync, so that it draws at most one frame a refresh and is light on a computer that someone's playing games on, saves a screenshot of what it draws to `target\RunInBackground\<program>.png`, and closes it. Read the screenshot to see the result.
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
