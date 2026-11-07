# WinUi3Sample

A sample of a real WinUI 3 app, in C# and XAML, for comparing with the programs built on
Library/UserInterface. It's the exception to the root AGENTS.md's rules for GUI programs: keep it a
plain WinUI 3 app, with WinUI's own controls, styles and input, and don't move it onto
Library/UserInterface, add the PerfBar, GameInput or a fixed layout, or add its widgets to the library.

Follow the non-disruptive workflow, though:

- Run it with `Program\RunInBackground\RunInBackground.ps1 WinUi3Sample`, which builds it with
  `dotnet`, lightly, and opens it with `--no-activate`. Pass `--page`, `--theme` and `--open-menu`
  with RunInBackground's `--arguments` to screenshot the rest, as the [README](README.md) shows.
- With `--no-activate`, a CBT hook keeps every window of the app from activating, including WinUI's
  menus and flyouts, which are windows of their own and activate when they open. Keep it, and open
  anything else that makes a window only in a run that has it.
- Build it with `Program\RunLightly\RunLightly.ps1 dotnet build Program\WinUi3Sample\WinUi3Sample.csproj --configuration Release`.
  Give options like `--verbosity minimal` in their long form: PowerShell splits `-v:minimal` in two.
- The repository's `.editorconfig` makes analyzer warnings errors, in C# too.
