# RunInBackground

Builds one of the GUI programs in `Program`, opens it on one display without taking focus,
saves a screenshot of its window, and closes it. Or runs it with scripts of input, which test
it without your mouse and keyboard. It's how coding agents run the programs while you work in
other windows (see the [AGENTS.md](../../AGENTS.md) at the root), or while your machine is
locked, which works the same.

```powershell
Program\RunInBackground\RunInBackground.ps1 GameInputDemo
Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo --display KXGR16 --keep-open
Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo --script Test\SimplePaintDemo\Scripts\DragSlider.json
Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo --all-scripts
```

The program opens with `--display` and `--no-activate`, which every program built on
[Library/Application](../../Library/Application/README.md) takes, so it opens on top of the
other windows on that display, but without taking focus, and typing in the window that has
focus stays there. Without focus, programs ignore all input.

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
| `--script <path>`     | Runs the program with a script of input instead, and saves what it         |
|                       | captures in `target\RunInBackground\<program>\<script>`                    |
| `--all-scripts`       | Runs every script in `Test\<program>\Scripts`, one at a time               |
| `--timeout <seconds>` | How long a script may run before it fails. 60 by default                   |

The screenshot is of what the program draws in its window, without the title bar. It comes
from `PrintWindow`, so it doesn't matter if other windows are in front of the program's. What
the program writes to standard output and standard error, like why it couldn't start, goes to
`target\RunInBackground\<program>Output.log` and `<program>Errors.log`.

## Scripts

A script run passes the program `--script` and `--output`, and the program takes its input
from the script instead of you, on a fixed clock, as
[Library/Application](../../Library/Application/README.md#scripts) describes. It opens on the
display without taking focus, like any run, and exits after the script's last step. The run
fails if the program exits with an error, which includes expectations in the script that
weren't met, if it runs past `--timeout`, or if its window ever becomes the foreground window,
which would mean that it took focus from you. RunInBackground checks that every 5 ms while
the program runs.

RunInBackground prints whether each script passed, and the frames it captured. Each script's
folder in `target\RunInBackground\<program>` has its captures, as `<name>.png`, the program's
state at each, as `<name>.json`, and `Output.log` and `Errors.log`. Files that the program
writes, like SimplePaintDemo's `Paint.json`, go there too, not in the program's folder.
