# RunInBackground

Builds one of the GUI programs in `Program`, opens it on one display without taking focus,
saves a screenshot of its window, and closes it. It's how coding agents run the programs
while you work in other windows (see the [AGENTS.md](../../AGENTS.md) at the root).

```powershell
Program\RunInBackground\RunInBackground.ps1 GameInputDemo
Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo --display KXGR16 --keep-open
```

The program opens with `--display` and `--no-activate`, which every program built on
[Library/Application](../../Library/Application/README.md) takes, so it opens behind the window
that has focus, and typing there stays there. Without focus, programs ignore all input.

The script only runs `RunInBackground.cs`, a .NET file-based app, with `dotnet run`, which
builds it into its own cache first. It needs the .NET 10 SDK.

| Argument              | Value                                                                      |
| --------------------- | -------------------------------------------------------------------------- |
| `<program>`           | The program to run, like `GameInputDemo`: its folder in `Program`, and its |
|                       | exe's name                                                                 |
| `--display <id>`      | The identifier of the display to open on. `CVVNY2` by default              |
| `--seconds <seconds>` | How long the program runs before the screenshot. 3 by default              |
| `--screenshot <path>` | Where to save the screenshot, as a PNG.                                    |
|                       | `target\RunInBackground\<program>.png` by default                          |
| `--keep-open`         | Leaves the program running after the screenshot                            |
| `--debug-build`       | Builds with Cargo's dev profile, which turns on the D3D12 debug layer      |

The screenshot is of what the program draws in its window, without the title bar. It comes
from `PrintWindow`, so it doesn't matter if other windows are in front of the program's. What
the program writes to standard output and standard error, like why it couldn't start, goes to
`target\RunInBackground\<program>Output.log` and `<program>Errors.log`.
